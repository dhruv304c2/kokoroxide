use crate::ipa_tokenizer::IpaTokenizer;

pub fn test_ipa_tokenizer() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Testing IPA Tokenizer ===\n");

    // Get vocabulary from tokenizer.json
    let tokenizer_content = std::fs::read_to_string("models/kokoro/tokenizer.json")?;
    let tokenizer_json: serde_json::Value = serde_json::from_str(&tokenizer_content)?;
    let vocab_obj = tokenizer_json["model"]["vocab"].as_object()
        .ok_or("No vocab found")?;

    let mut vocab = std::collections::HashMap::new();
    for (token, id) in vocab_obj {
        vocab.insert(token.clone(), id.as_i64().unwrap_or(0));
    }

    // Create IPA tokenizer
    let ipa_tokenizer = IpaTokenizer::new(vocab)?;

    // Test sentences
    let test_sentences = vec![
        "Hello world",
        "cat dog",
        "The quick brown fox",
        "Testing pronunciation",
    ];

    for sentence in test_sentences {
        println!("\nText: '{}'", sentence);


        // IPA tokenizer
        match ipa_tokenizer.encode(sentence, None) {
            Ok(ipa_tokens) => {
                println!("IPA tokens: {:?}", ipa_tokens);
                println!("Token count: {}", ipa_tokens.len());
            }
            Err(e) => {
                println!("IPA tokenization error: {}", e);
            }
        }
    }

    Ok(())
}