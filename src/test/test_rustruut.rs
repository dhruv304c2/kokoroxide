use phonetisaurus_g2p::PhonetisaurusModel;

pub fn test_rustruut() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing phonetisaurus text-to-IPA conversion");

    // Initialize phonetisaurus with model file
    let model_path = std::path::Path::new("models/american-english.fst");
    let phonemizer = PhonetisaurusModel::try_from(model_path)?;

    // Test single letters
    println!("\nTesting single letters:");
    for letter in ["a", "i", "I", "A", "b", "c", "e", "o", "u"] {
        match phonemizer.phonemize_word(letter) {
            Ok(phonemes) => println!("  '{}' -> '{}'", letter, phonemes.phonemes),
            Err(e) => println!("  '{}' -> ERROR: {}", letter, e),
        }
    }

    // Test full sentence vs word by word
    println!("\nTesting full sentence processing:");
    let sentence = "i want to be a cat";
    match phonemizer.phonemize_word(sentence) {
        Ok(phonemes) => println!("  Full sentence: '{}' -> '{}'", sentence, phonemes.phonemes),
        Err(e) => println!("  Full sentence ERROR: {}", e),
    }

    let test_text = "fast racing car";
    println!("\nText: {}", test_text);

    // Get phonemes for each word
    for word in test_text.split_whitespace() {
        let phonemes = phonemizer.phonemize_word(&word.to_lowercase())?;
        println!("  {} -> {}", word, phonemes.phonemes);
    }

    // Test another example
    let test_text2 = "Hello world, this is a test";
    println!("\nText: {}", test_text2);

    for word in test_text2.split_whitespace() {
        // Remove punctuation for phonemization
        let clean_word = word.trim_matches(|c: char| !c.is_alphabetic());
        if !clean_word.is_empty() {
            let phonemes = phonemizer.phonemize_word(&clean_word.to_lowercase())?;
            println!("  {} -> {}", clean_word, phonemes.phonemes);
        }
    }

    Ok(())
}
