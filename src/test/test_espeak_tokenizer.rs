use crate::espeak::EspeakIpaTokenizer;

pub fn test_espeak_tokenizer() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Testing Espeak IPA Tokenizer ===\n");

    // Get vocabulary from tokenizer.json
    let tokenizer_content = std::fs::read_to_string("models/kokoro/tokenizer.json")?;
    let tokenizer_json: serde_json::Value = serde_json::from_str(&tokenizer_content)?;
    let vocab_obj = tokenizer_json["model"]["vocab"]
        .as_object()
        .ok_or("No vocab found")?;

    let mut vocab = std::collections::HashMap::new();
    for (token, id) in vocab_obj {
        vocab.insert(token.clone(), id.as_i64().unwrap_or(0));
    }

    // Create Espeak IPA tokenizer
    let tokenizer = EspeakIpaTokenizer::new(vocab)?;

    // Test sentences
    let test_sentences = vec![
        "I want to be a cat",
        "Hello world",
        "The quick brown fox",
        "Testing pronunciation",
        "a i u e o", // Single letters
    ];

    for sentence in test_sentences {
        println!("\nText: '{}'", sentence);

        // Espeak tokenizer
        match tokenizer.encode(sentence, None) {
            Ok(tokens) => {
                println!("Espeak tokens: {:?}", tokens);
                println!("Token count: {}", tokens.len());
            }
            Err(e) => {
                println!("Espeak tokenization error: {}", e);
            }
        }
    }

    Ok(())
}
