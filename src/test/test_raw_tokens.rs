use crate::kokoro::{load_voice_style, KokoroTTS, TTSConfig};
use std::error::Error;

pub fn test_raw_tokens() -> Result<(), Box<dyn Error>> {
    println!("=== Testing Raw Token Input to ONNX Model ===");

    // Initialize TTS
    let tts_config = TTSConfig::new(
            "models/kokoro/kokoro.onnx", 
            "models/kokoro/tokenizer.json"
        )
        .with_graph_optimization_level(ort::GraphOptimizationLevel::Disable)
        .with_max_tokens_length(512)
        .with_sample_rate(24000);

    let tts = KokoroTTS::with_config(tts_config)?;

    // Example tokens in the format you provided
    let tokens: Vec<i64> = vec![
        50, 157, 43, 135, 16, 53, 135, 46, 16, 43, 102, 16, 56, 156, 57, 135, 6, 16, 102, 62, 61, 16, 70, 56, 16, 138, 56, 156, 72, 56, 61, 85, 123, 83, 44, 83, 54, 16, 53, 65, 156, 86, 61, 62, 131, 83, 56, 4, 16, 54, 156, 43, 102, 53, 16, 156, 72, 61, 53, 102, 112, 16, 70, 56, 16, 138, 56, 44, 156, 76, 158, 123, 56, 16, 62, 131, 156, 43, 102, 54, 46, 16, 102, 48, 16, 81, 47, 102, 54, 16, 54, 156, 51, 158, 46, 16, 70, 16, 92, 156, 135, 46, 16, 54, 156, 43, 102, 48, 4, 16, 81, 47, 102, 16, 50, 156, 72, 64, 83, 56, 62, 16, 156, 51, 158, 64, 83, 56, 16, 44, 157, 102, 56, 16, 44, 156, 76, 158, 123, 56, 4
    ];

    println!("tokens = {:?}", tokens);

    // Create voice style
    let voice_style = load_voice_style("models/kokoro/af.bin")?;
    let speed = 1.0;

    // Generate audio from raw tokens
    let audio = tts.generate_from_tokens(
        &tokens,
        &voice_style,
        speed
    )?;

    println!("Audio generated successfully!");

    // Save the audio file
    audio.save_to_wav("raw_tokens_test.wav")?;
    println!("Output saved to: raw_tokens_test.wav");

    Ok(())
}
