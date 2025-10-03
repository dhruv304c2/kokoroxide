use std::collections::HashMap;

pub fn analyze_ipa_in_kokoro() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Analyzing IPA symbols in Kokoro vocabulary ===\n");

    // Read tokenizer.json
    let tokenizer_content = std::fs::read_to_string("models/kokoro/tokenizer.json")?;
    let tokenizer_json: serde_json::Value = serde_json::from_str(&tokenizer_content)?;

    let vocab = tokenizer_json["model"]["vocab"]
        .as_object()
        .ok_or("No vocab found")?;

    // Common IPA symbols to look for (from rustruut output)
    let ipa_symbols = vec![
        "æ", "ɹ", "eɪ", "ɪ", "ŋ", "ɑ", "ə", "ɛ", "ɔ", "ʊ", "ʌ", "i", "u", "o", "a",
        "e", // basic vowels
        "θ", "ð", "ʃ", "ʒ", "tʃ", "dʒ", // fricatives/affricates
        "ˈ", "ˌ", // stress markers
    ];

    println!("Checking for common IPA symbols in vocabulary:");
    for symbol in &ipa_symbols {
        if let Some(id) = vocab.get(*symbol) {
            println!("  '{}' -> {}", symbol, id);
        } else {
            // Check if it's part of a Unicode escape
            let mut found = false;
            for (key, id) in vocab {
                if key.contains(symbol) {
                    println!("  '{}' found in '{}' -> {}", symbol, key, id);
                    found = true;
                    break;
                }
            }
            if !found {
                println!("  '{}' -> NOT FOUND", symbol);
            }
        }
    }

    // Print all IPA-looking symbols (non-ASCII)
    println!("\nAll IPA/phonetic symbols in vocabulary:");
    let mut ipa_map: HashMap<String, i64> = HashMap::new();

    for (key, id) in vocab {
        if key.chars().any(|c| !c.is_ascii() && c != '$') {
            ipa_map.insert(key.clone(), id.as_i64().unwrap_or(0));
        }
    }

    // Sort by Unicode value for easier reading
    let mut sorted: Vec<_> = ipa_map.iter().collect();
    sorted.sort_by_key(|(k, _)| k.as_str());

    for (symbol, id) in sorted {
        let unicode: String = symbol
            .chars()
            .map(|c| format!("U+{:04X}", c as u32))
            .collect::<Vec<_>>()
            .join(" ");
        println!("  '{}' ({}) -> {}", symbol, unicode, id);
    }

    Ok(())
}
