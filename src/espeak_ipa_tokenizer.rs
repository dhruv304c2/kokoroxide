use std::collections::HashMap;
use std::error::Error;
use std::time::Instant;
use crate::espeak_g2p::EspeakG2P;

pub struct EspeakIpaTokenizer {
    vocab: HashMap<String, i64>,
    bos_id: i64,
    eos_id: i64,
    model_max_length: usize,
    g2p: EspeakG2P,
    max_token_chars: usize, // NEW
}


impl EspeakIpaTokenizer {
    pub fn new(vocab: HashMap<String, i64>) -> Result<Self, Box<dyn std::error::Error>> {
        let bos_id = *vocab.get("$").ok_or("BOS token '$' not found")?;
        // otherwise fetch the explicit EOS and assert
        let eos_id = bos_id;

        let g2p = EspeakG2P::new()?;
        let max_token_chars = Self::max_token_chars(&vocab);

        Ok(Self {
            vocab,
            bos_id,
            eos_id,
            model_max_length: 512,
            g2p,
            max_token_chars,
        })
    }

    /// Convert espeak IPA to Misaki phonemes to match kokoro Python output
    fn espeak_ipa_to_misaki(&self, ipa: &str) -> String {
        // First, replace the Unicode tie bar (U+0361) with caret (^) to match Misaki docs format
        let mut result = ipa.replace('\u{0361}', "^");

        // Apply the exact transformations from Misaki docs (sorted by length descending)
        // Note: Using American English (british = false)
        // Important: Order matters - longer patterns must come first
        let from_espeaks = vec![
            // Longest patterns first
            ("ʔˌn̩", "tᵊn"),
            ("ʔˌn\u{0329}", "tᵊn"), // Alternative encoding
            // Three-character patterns
            ("ɜːɹ", "ɜɹ"), // American English
            ("a^ɪ", "I"),
            ("a^ʊ", "W"),
            ("d^ʒ", "ʤ"),
            ("e^ɪ", "A"),
            ("e^ə", "ɛ"), // Additional pattern for British (but we skip for American)
            ("o^ʊ", "O"),  // American English
            ("ə^ʊ", "O"),  // Also map to O for American (as in "hello")
            ("t^ʃ", "ʧ"),
            ("ɔ^ɪ", "Y"),
            ("ə^l", "ᵊl"),
            ("ʲO", "jO"),  // Special cases from docs
            ("ʲQ", "jQ"),  // Special cases from docs
            // Two-character patterns
            ("ɜː", "ɜɹ"),  // American English
            ("ɪə", "iə"),  // American English
            ("iə", "iə"),  // Keep as is for American
            ("aɪ", "I"),   // Without tie marker
            ("aʊ", "W"),   // Without tie marker
            ("dʒ", "ʤ"),   // Without tie marker
            ("eɪ", "A"),   // Without tie marker
            ("oʊ", "O"),   // Without tie marker (American English)
            ("tʃ", "ʧ"),   // Without tie marker
            ("ɔɪ", "Y"),   // Without tie marker
            ("ʔn", "tᵊn"),
            ("ɚ", "əɹ"),
            ("ɬ", "l"),
            // Single character patterns
            ("\u{0303}", ""), // U+0303 combining tilde
            // ("ɐ", "ə"),  // REMOVED - kokoro uses ɐ directly
            ("ʔ", "t"),
            ("ʲ", ""),
            ("e", "A"),
            ("r", "ɹ"),
            ("x", "k"),
            ("ç", "k"),
            // Additional British to American conversions
            ("ɒ", "ɑ"), // British 'o' sound to American 'a' sound
        ];

        // Apply replacements
        for (old, new) in from_espeaks {
            result = result.replace(old, new);
        }

        // Handle syllabic consonants: (\S)̩ -> ᵊ\1
        // This is regex: r'(\S)\u0329' -> r'ᵊ\1' in Python
        let mut chars: Vec<char> = result.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if i + 1 < chars.len() && chars[i + 1] == '\u{0329}' {
                // Found syllabic marker after a character
                let consonant = chars[i];
                chars[i] = 'ᵊ';
                chars[i + 1] = consonant;
                i += 2;
            } else {
                i += 1;
            }
        }
        result = chars.into_iter().collect();

        // Remove any remaining syllabic markers
        result = result.replace('\u{0329}', "");

        // For American English, remove length marks
        result = result.replace("ː", "");

        // Remove tie markers
        result = result.replace("^", "");

        // IMPORTANT: Map regular 'g' (U+0067) to IPA 'ɡ' (U+0261) if present
        result = result.replace('g', "ɡ");

        result
    }

    /// Convert text to IPA using espeak-ng
    fn text_to_ipa(&self, text: &str) -> Result<String, Box<dyn Error>> {
        // Get IPA from espeak
        let ipa = self.g2p.text_to_ipa(text)?;

        // Convert to Misaki phonemes
        let mut misaki_phonemes = self.espeak_ipa_to_misaki(&ipa);

        // Preserve punctuation from original text
        // This is a simple approach - just append punctuation if the text ends with it
        if text.ends_with('.') && !misaki_phonemes.ends_with('.') {
            misaki_phonemes.push('.');
        } else if text.ends_with(';') && !misaki_phonemes.ends_with(';') {
            misaki_phonemes.push(';');
        } else if text.ends_with('!') && !misaki_phonemes.ends_with('!') {
            misaki_phonemes.push('!');
        } else if text.ends_with('?') && !misaki_phonemes.ends_with('?') {
            misaki_phonemes.push('?');
        }

        if std::env::var("DEBUG_PHONEMES").is_ok() {
            println!("Input text: '{}'", text);
            println!("Espeak IPA: '{}'", ipa);
            println!("Misaki phonemes: '{}'", misaki_phonemes);
        }
        Ok(misaki_phonemes)
    }

     fn max_token_chars(vocab: &HashMap<String, i64>) -> usize {
        vocab.keys().map(|k| k.chars().count()).max().unwrap_or(1)
    }

    pub fn tokenize_longest(&self, ipa: &str) -> Vec<i64> {
        let mut ids = Vec::with_capacity(ipa.len());
        let chars: Vec<char> = ipa.chars().collect();
        let mut i = 0;
        let max_len = self.max_token_chars;

        while i < chars.len() {
            let mut matched = false;
            let limit = max_len.min(chars.len() - i);

            // try the longest possible token first
            for l in (1..=limit).rev() {
                let cand: String = chars[i..i + l].iter().collect();
                if let Some(&id) = self.vocab.get(&cand) {
                    ids.push(id);
                    i += l;
                    matched = true;
                    break;
                }
            }

            if !matched {
                // Optional: ignore pure whitespace if not in vocab
                if !chars[i].is_whitespace() {
                    eprintln!("Warning: unknown token {:?}", chars[i]);
                }
                i += 1;
            }
        }
        ids
    }

    pub fn encode_phonemes(&self, phonemes: &str, max_length: Option<usize>) -> Result<Vec<i64>, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        let max_len = max_length.unwrap_or(self.model_max_length);

        // BOS
        let mut tokens = Vec::with_capacity(phonemes.len() + 2);
        tokens.push(self.bos_id);

        // Tokenize the phonemes directly
        let mut inner = self.tokenize_longest(phonemes);
        tokens.append(&mut inner);

        // EOS
        tokens.push(self.eos_id);

        // Truncate if needed (keep BOS/EOS)
        if tokens.len() > max_len {
            let keep_inner = max_len.saturating_sub(2);
            let mut truncated = Vec::with_capacity(max_len);
            truncated.push(self.bos_id);
            truncated.extend_from_slice(&tokens[1..1 + keep_inner]);
            truncated.push(self.eos_id);
            println!("Direct phoneme tokenization time: {:?}", start_time.elapsed());
            return Ok(truncated);
        }

        println!("Direct phoneme tokenization time: {:?}", start_time.elapsed());
        println!("tokens = {:?}", tokens);
        Ok(tokens)
    }

    pub fn encode(&self, text: &str, max_length: Option<usize>) -> Result<Vec<i64>, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        let max_len = max_length.unwrap_or(self.model_max_length);

        // BOS
        let mut tokens = Vec::with_capacity(text.len() + 2);
        tokens.push(self.bos_id);

        // Text → IPA (via espeak) → normalize/misaki map (your existing logic)
        let ipa_start = Instant::now();
        let ipa_text = self.text_to_ipa(text)?; // already logs Espeak IPA + Misaki when DEBUG_PHONEMES=1
        println!("Phoneme tokenization (espeak IPA conversion) took: {:?}", ipa_start.elapsed());

        // Longest-match tokenize against vocab (handles multi-char phones)
        let mut inner = self.tokenize_longest(&ipa_text);
        tokens.append(&mut inner);

        // EOS
        tokens.push(self.eos_id);

        // Truncate if needed (keep BOS/EOS)
        if tokens.len() > max_len {
            let keep_inner = max_len.saturating_sub(2);
            let mut truncated = Vec::with_capacity(max_len);
            truncated.push(self.bos_id);
            truncated.extend_from_slice(&tokens[1..1 + keep_inner]);
            truncated.push(self.eos_id);
            println!("Total tokenization time: {:?}", start_time.elapsed());
            return Ok(truncated);
        }

        println!("Total tokenization time: {:?}", start_time.elapsed());
        println!("tokens = {:?}", tokens);
        Ok(tokens)
    }
}
