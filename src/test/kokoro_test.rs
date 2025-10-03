use crate::kokoro::{load_voice_style, KokoroTTS, TTSConfig};
use crate::playback::play_wav_file;

pub fn run_kokoro(no_play: bool) -> Result<(), Box<dyn std::error::Error>> {
    run_kokoro_with_text("Hello world, how are you today?", no_play)
}

pub fn run_kokoro_with_text(text: &str, no_play: bool) -> Result<(), Box<dyn std::error::Error>> {
    let tts_config = TTSConfig::new("models/kokoro/kokoro.onnx", "models/kokoro/tokenizer.json")
        .with_graph_optimization_level(ort::GraphOptimizationLevel::Disable)
        .with_max_tokens_length(512)
        .with_sample_rate(24000);

    let tts = KokoroTTS::with_config(tts_config)?;

    let voice = load_voice_style("models/kokoro/af.bin")?;

    println!("Generating speech for: \"{}\"", text);
    let output_path = "kokoro_test_output.wav";
    let audio = tts.generate_speech(text, &voice, 1.0)?;

    println!("Duration: {:.2} seconds", audio.duration_seconds);

    // Save the audio file
    audio.save_to_wav(output_path)?;

    if !no_play {
        println!("\nPlaying generated speech...");
        play_wav_file(output_path)?;
    } else {
        println!("\nPlayback skipped (--no-play flag set)");
    }

    Ok(())
}
