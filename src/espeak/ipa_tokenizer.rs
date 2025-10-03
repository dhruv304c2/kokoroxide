use super::EspeakG2P;
use std::collections::HashMap;
use std::error::Error;
use std::time::Instant;

pub struct EspeakIpaTokenizer {
    vocab: HashMap<String, i64>,
    bos_id: i64,
    eos_id: i64,
    model_max_length: usize,
    g2p: EspeakG2P,
    max_token_chars: usize,
}

impl EspeakIpaTokenizer {
    pub fn new(vocab: HashMap<String, i64>) -> Result<Self, Box<dyn std::error::Error>> {
        let bos_id = *vocab.get("$").ok_or("BOS token '$' not found")?;
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
        // First, replace the Unicode tie bar (U+0361) with caret (^) to match Python
        let mut result = ipa.replace('\u{0361}', "^");

        // FROM_ESPEAKS = sorted({...}.items(), key=lambda kv: -len(kv[0]))
        let from_espeaks = vec![
            // Sorted by length descending (longest first)
            ("ʔˌn\u{0329}", "tᵊn"), // 5 chars
            ("a^ɪ", "I"),           // 3 chars
            ("a^ʊ", "W"),           // 3 chars
            ("d^ʒ", "ʤ"),           // 3 chars
            ("e^ɪ", "A"),           // 3 chars
            ("t^ʃ", "ʧ"),           // 3 chars
            ("ɔ^ɪ", "Y"),           // 3 chars
            ("ə^l", "ᵊl"),          // 3 chars
            ("ʔn", "tᵊn"),          // 2 chars
            ("ɚ", "əɹ"),            // 2 chars (even though it's 1 char, it maps to 2)
            ("ʲO", "jO"),           // 2 chars
            ("ʲQ", "jQ"),           // 2 chars
            ("\u{0303}", ""),       // 1 char (U+0303 combining tilde)
            ("e", "A"),             // 1 char
            ("r", "ɹ"),             // 1 char
            ("x", "k"),             // 1 char
            ("ç", "k"),             // 1 char
            ("ɐ", "ə"),             // 1 char
            ("ɬ", "l"),             // 1 char
            ("ʔ", "t"),             // 1 char
            ("ʲ", ""),              // 1 char
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

        // Remove any remaining syllabic markers (chr(809) in Python)
        result = result.replace('\u{0329}', "");

        // Apply American English specific transformations (british = false)
        result = result.replace("o^ʊ", "O");
        result = result.replace("ɜːɹ", "ɜɹ");
        result = result.replace("ɜː", "ɜɹ");
        result = result.replace("ɪə", "iə");
        result = result.replace("ː", ""); // Remove all length marks

        // Finally remove tie markers
        result = result.replace("^", "");

        result
    }

    fn text_to_ipa(&self, text: &str) -> Result<String, Box<dyn Error>> {
        let ipa = self.g2p.text_to_ipa(text)?;

        let misaki_phonemes = self.espeak_ipa_to_misaki(&ipa);

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

    pub fn encode_phonemes(
        &self,
        phonemes: &str,
        max_length: Option<usize>,
    ) -> Result<Vec<i64>, Box<dyn std::error::Error>> {
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
            if std::env::var("DEBUG_TIMING").is_ok() {
                println!(
                    "Direct phoneme tokenization time: {:?}",
                    start_time.elapsed()
                );
            }
            return Ok(truncated);
        }

        if std::env::var("DEBUG_TIMING").is_ok() {
            println!(
                "Direct phoneme tokenization time: {:?}",
                start_time.elapsed()
            );
        }
        if std::env::var("DEBUG_TOKENS").is_ok() {
            println!("tokens = {:?}", tokens);
        }
        Ok(tokens)
    }

    pub fn encode(
        &self,
        text: &str,
        max_length: Option<usize>,
    ) -> Result<Vec<i64>, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        let max_len = max_length.unwrap_or(self.model_max_length);

        // BOS
        let mut tokens = Vec::with_capacity(text.len() + 2);
        tokens.push(self.bos_id);

        let ipa_start = Instant::now();
        let ipa_text = self.text_to_ipa(text)?;
        if std::env::var("DEBUG_TIMING").is_ok() {
            println!(
                "Phoneme tokenization (espeak IPA conversion) took: {:?}",
                ipa_start.elapsed()
            );
        }

        let mut inner = self.tokenize_longest(&ipa_text);
        tokens.append(&mut inner);

        // EOS
        tokens.push(self.eos_id);

        if tokens.len() > max_len {
            let keep_inner = max_len.saturating_sub(2);
            let mut truncated = Vec::with_capacity(max_len);
            truncated.push(self.bos_id);
            truncated.extend_from_slice(&tokens[1..1 + keep_inner]);
            truncated.push(self.eos_id);
            if std::env::var("DEBUG_TIMING").is_ok() {
                println!("Total tokenization time: {:?}", start_time.elapsed());
            }
            return Ok(truncated);
        }

        if std::env::var("DEBUG_TIMING").is_ok() {
            println!("Total tokenization time: {:?}", start_time.elapsed());
        }
        if std::env::var("DEBUG_TOKENS").is_ok() {
            println!("tokens = {:?}", tokens);
        }
        Ok(tokens)
    }
}
