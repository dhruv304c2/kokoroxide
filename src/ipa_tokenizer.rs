use std::collections::HashMap;
use std::error::Error;
use std::time::Instant;
use std::path::Path;
use phonetisaurus_g2p::PhonetisaurusModel;

pub struct IpaTokenizer {
    vocab: HashMap<String, i64>,
    bos_id: i64,
    eos_id: i64,
    model_max_length: usize,
    phonemizer: PhonetisaurusModel,
}

impl IpaTokenizer {
    pub fn new(vocab: HashMap<String, i64>) -> Result<Self, Box<dyn Error>> {
        // Get special tokens
        let bos_id = *vocab.get("$").ok_or("BOS token '$' not found")?;
        let eos_id = bos_id; // Same as BOS in Kokoro

        // Load phonetisaurus model from file
        let model_path = Path::new("models/american-english.fst");
        let phonemizer = PhonetisaurusModel::try_from(model_path)?;

        Ok(IpaTokenizer {
            vocab,
            bos_id,
            eos_id,
            model_max_length: 512,
            phonemizer,
        })
    }



    /// Convert IPA to Misaki phonemes
    fn ipa_to_misaki(&self, ipa: &str) -> String {
        let mut result = ipa.to_string();

        // Remove word boundaries and clean up
        result = result
            .trim_start_matches('#')
            .trim_end_matches('#')
            .replace('#', "");

        // Keep stress marks (Misaki uses them)
        // Primary stress ˈ and secondary stress ˌ are kept

        // Convert diphthongs to Misaki single characters
        result = result
            .replace("eɪ", "A")  // "ay" sound
            .replace("ei", "A")  // Alternative spelling
            .replace("aɪ", "I")  // "eye" sound
            .replace("ai", "I")  // Alternative spelling
            .replace("aʊ", "W")  // "ow" sound
            .replace("au", "W")  // Alternative spelling
            .replace("oʊ", "O")  // "oh" sound (American)
            .replace("ou", "O")  // Alternative spelling
            .replace("ɔɪ", "Y")  // "oy" sound
            .replace("oi", "Y"); // Alternative spelling

        // Convert affricates
        result = result
            .replace("tʃ", "ʧ")  // "ch" sound
            .replace("dʒ", "ʤ"); // "j" sound

        // Map regular g to IPA ɡ (U+0261)
        result = result.replace('g', "ɡ");

        // Handle syllabic consonants
        result = result
            .replace("l̩", "əl")  // syllabic l
            .replace("n̩", "ən")  // syllabic n
            .replace("m̩", "əm")  // syllabic m
            .replace("̩", "");    // Remove any remaining syllabic markers

        // Remove syllable boundaries
        result = result.replace(".", "");

        // Handle other IPA to Misaki mappings
        result = result
            .replace("ɜː", "ɜ")  // British long vowel to American
            .replace("ɜ˞", "ɝ")  // r-colored vowel
            .replace("ər", "ɚ")  // schwa + r
            .replace("ɹ̩", "ɚ")  // syllabic r
            .replace("ɵ", "ə");  // close-mid central rounded to schwa

        result
    }

    /// Convert text to IPA using phonetisaurus
    fn text_to_ipa(&self, text: &str) -> Result<String, Box<dyn Error>> {
        // Process each word separately and join with spaces
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut ipa_words = Vec::new();

        for word in words {
            // Remove punctuation and convert to lowercase
            let clean_word = word.chars()
                .filter(|c| c.is_alphabetic() || c.is_whitespace())
                .collect::<String>()
                .to_lowercase();

            // Skip empty words
            if clean_word.is_empty() {
                continue;
            }

            // Convert word to phonemes
            let phonemes = self.phonemizer.phonemize_word(&clean_word)?;

            println!("DEBUG: Word '{}' -> Raw phonemes: '{}'", clean_word, phonemes.phonemes);

            // Handle special cases where phonetisaurus returns empty
            let ipa_phonemes = if phonemes.phonemes.trim() == "#" || phonemes.phonemes.trim().is_empty() {
                // Common single-letter words
                match clean_word.as_str() {
                    "i" => "#ˈaɪ#",      // pronoun "I"
                    "a" => "#ə#",        // article "a" (schwa)
                    _ => phonemes.phonemes.trim(),
                }
            } else {
                phonemes.phonemes.trim()
            };

            // Convert IPA to Misaki phonemes
            let misaki_phonemes = self.ipa_to_misaki(ipa_phonemes);

            println!("DEBUG: Cleaned IPA for '{}': '{}'", clean_word, misaki_phonemes);
            ipa_words.push(misaki_phonemes);
        }

        let result = ipa_words.join(" ");
        println!("DEBUG: Final IPA text: '{}'", result);

        // Join words with spaces
        Ok(result)
    }

    /// Map IPA diphthongs and special cases to available tokens
    fn map_ipa_to_tokens(&self, ipa_char: &str) -> Vec<String> {
        match ipa_char {
            // Diphthongs - split into components
            "eɪ" => vec!["e".to_string(), "ɪ".to_string()],
            "aɪ" => vec!["a".to_string(), "ɪ".to_string()],
            "ɔɪ" => vec!["ɔ".to_string(), "ɪ".to_string()],
            "aʊ" => vec!["a".to_string(), "ʊ".to_string()],
            "oʊ" => vec!["o".to_string(), "ʊ".to_string()],

            // Affricates - use alternatives or split
            "tʃ" => vec!["ʧ".to_string()], // Kokoro has ʧ at index 133
            "dʒ" => vec!["ʤ".to_string()], // Kokoro has ʤ at index 82

            // Regular IPA symbols - pass through
            _ => vec![ipa_char.to_string()],
        }
    }

    /// Encode text to token IDs with IPA conversion
    pub fn encode(&self, text: &str, max_length: Option<usize>) -> Result<Vec<i64>, Box<dyn Error>> {
        let start_time = Instant::now();
        let max_len = max_length.unwrap_or(self.model_max_length);
        let mut tokens = Vec::with_capacity(text.len() + 2);

        // BOS token
        tokens.push(self.bos_id);

        // Convert text to IPA
        let ipa_start = Instant::now();
        let ipa_text = self.text_to_ipa(text)?;
        let ipa_duration = ipa_start.elapsed();
        println!("Phoneme tokenization (IPA conversion) took: {:?}", ipa_duration);

        // Process IPA text character by character
        let mut chars = ipa_text.chars().peekable();
        while let Some(ch) = chars.next() {
            let mut symbol = ch.to_string();

            // Check for multi-character IPA symbols (diphthongs)
            if chars.peek() == Some(&'ɪ') || chars.peek() == Some(&'ʊ') {
                if matches!(ch, 'e' | 'a' | 'ɔ' | 'o') {
                    symbol.push(chars.next().unwrap());
                }
            } else if ch == 't' && chars.peek() == Some(&'ʃ') {
                symbol.push(chars.next().unwrap());
            } else if ch == 'd' && chars.peek() == Some(&'ʒ') {
                symbol.push(chars.next().unwrap());
            }

            // Map IPA to tokens
            let token_strings = self.map_ipa_to_tokens(&symbol);

            for token_str in token_strings {
                if let Some(&id) = self.vocab.get(&token_str) {
                    tokens.push(id);
                    println!("DEBUG: Token '{}' -> ID {}", token_str, id);
                } else if token_str == " " {
                    // Space might not be in vocab, skip it
                    if let Some(&space_id) = self.vocab.get(" ") {
                        tokens.push(space_id);
                        println!("DEBUG: Space token -> ID {}", space_id);
                    }
                } else {
                    println!("DEBUG: Token '{}' NOT FOUND in vocabulary!", token_str);
                }
            }
        }

        // EOS token
        tokens.push(self.eos_id);

        // Truncate if needed
        if tokens.len() > max_len {
            let mut truncated = Vec::with_capacity(max_len);
            truncated.push(self.bos_id);
            let keep_inner = max_len.saturating_sub(2);
            truncated.extend_from_slice(&tokens[1..1+keep_inner]);
            truncated.push(self.eos_id);
            let total_duration = start_time.elapsed();
            println!("Total tokenization time: {:?}", total_duration);
            return Ok(truncated);
        }

        let total_duration = start_time.elapsed();
        println!("Total tokenization time: {:?}", total_duration);
        Ok(tokens)
    }

    /// Encode without BOS/EOS tokens (for raw generation)
    pub fn encode_raw(&self, text: &str) -> Result<Vec<i64>, Box<dyn Error>> {
        let start_time = Instant::now();
        let mut tokens = Vec::new();

        // Convert text to IPA
        let ipa_start = Instant::now();
        let ipa_text = self.text_to_ipa(text)?;
        let ipa_duration = ipa_start.elapsed();
        println!("Phoneme tokenization (IPA conversion) took: {:?}", ipa_duration);

        // Process IPA text character by character
        let mut chars = ipa_text.chars().peekable();
        while let Some(ch) = chars.next() {
            let mut symbol = ch.to_string();

            // Check for multi-character IPA symbols (diphthongs)
            if chars.peek() == Some(&'ɪ') || chars.peek() == Some(&'ʊ') {
                if matches!(ch, 'e' | 'a' | 'ɔ' | 'o') {
                    symbol.push(chars.next().unwrap());
                }
            } else if ch == 't' && chars.peek() == Some(&'ʃ') {
                symbol.push(chars.next().unwrap());
            } else if ch == 'd' && chars.peek() == Some(&'ʒ') {
                symbol.push(chars.next().unwrap());
            }

            // Map IPA to tokens
            let token_strings = self.map_ipa_to_tokens(&symbol);

            for token_str in token_strings {
                if let Some(&id) = self.vocab.get(&token_str) {
                    tokens.push(id);
                } else if token_str == " " {
                    // Space might not be in vocab, skip it
                    if let Some(&space_id) = self.vocab.get(" ") {
                        tokens.push(space_id);
                    }
                }
            }
        }

        let total_duration = start_time.elapsed();
        println!("Total tokenization time: {:?}", total_duration);
        Ok(tokens)
    }
}
