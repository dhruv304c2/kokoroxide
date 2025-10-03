use crate::espeak_g2p::EspeakG2P;
use crate::espeak_ipa_tokenizer::EspeakIpaTokenizer;
use std::collections::HashMap;

#[allow(dead_code)]
pub fn test_misaki_conversion() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Misaki phoneme conversion according to spec\n");

    // Initialize the G2P
    let g2p = EspeakG2P::new()?;

    // Test cases from the Misaki documentation
    let test_cases = vec![
        // From the doc example
        ("merchantship", "mˈɜːt^ʃəntʃˌɪp", "mˈɜɹʧəntʃˌɪp"), // American expected

        // Basic words to test common conversions
        ("yes", "", "jˈɛs"),      // j represents "y" sound
        ("get", "", "ɡɛt"),       // hard g sound
        ("sung", "", "sˈʌŋ"),     // ng sound
        ("red", "", "ɹˈɛd"),      // r sound
        ("shin", "", "ʃˈɪn"),     // sh sound
        ("Asia", "", "ˈAʒə"),     // zh sound
        ("than", "", "ðən"),      // soft th
        ("thin", "", "θˈɪn"),     // hard th
        ("jump", "", "ʤˈʌmp"),    // j/dg sound
        ("chump", "", "ʧˈʌmp"),   // ch sound
        ("easy", "", "ˈizi"),     // i vowel
        ("flu", "", "flˈu"),      // u vowel
        ("spa", "", "spˈɑ"),      // ɑ vowel
        ("all", "", "ˈɔl"),       // ɔ vowel
        ("bed", "", "bˈɛd"),      // ɛ vowel
        ("brick", "", "bɹˈɪk"),   // ɪ vowel
        ("wood", "", "wˈʊd"),     // ʊ vowel
        ("sun", "", "sˈʌn"),      // ʌ vowel
        ("hey", "", "hˈA"),       // A diphthong (eɪ)
        ("high", "", "hˈI"),      // I diphthong (aɪ)
        ("how", "", "hˌW"),       // W diphthong (aʊ)
        ("soy", "", "sˈY"),       // Y diphthong (ɔɪ)
        ("ash", "", "ˈæʃ"),       // æ vowel (American)
        ("butter", "", "bˈʌɾəɹ"), // ɾ sound (American)
        ("boxes", "", "bˈɑksᵻz"), // ᵻ sound (American)
        ("pixel", "", "pˈɪksᵊl"), // small schwa
    ];

    // Create a minimal vocab for testing
    let mut vocab = HashMap::new();
    vocab.insert("$".to_string(), 0);

    // Add all Misaki phonemes to vocab
    let phonemes = "AIWYbdfhijklmnpstuvwzðŋɑɔəɛɜɡɪɹʃʊʌʒʤʧˈˌθᵊOæɾᵻ";
    for (i, ch) in phonemes.chars().enumerate() {
        vocab.insert(ch.to_string(), (i + 1) as i64);
    }
    vocab.insert(" ".to_string(), 100);

    let tokenizer = EspeakIpaTokenizer::new(vocab)?;

    println!("Testing individual word conversions:");
    println!("{:<15} {:<25} {:<25} {:<25}", "Word", "Espeak IPA", "Expected", "Actual");
    println!("{}", "-".repeat(90));

    for (word, espeak_raw,_) in test_cases {
        if espeak_raw.is_empty() {
            g2p.text_to_ipa(word)?
        } else {
            espeak_raw.to_string()
        };

        // Set DEBUG_PHONEMES env var to see the conversion
        std::env::set_var("DEBUG_PHONEMES", "1");

        let _ = tokenizer.encode(word, None)?;

        std::env::remove_var("DEBUG_PHONEMES");
    }

    Ok(())
}
