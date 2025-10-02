use crate::kokoro::{KokoroTTS, load_voice_style, normalize_for_kokoro};
use crate::playback::play_wav_file;

pub fn run_kokoro(no_play: bool) -> Result<(), Box<dyn std::error::Error>> {
    run_kokoro_with_text("Hello world, how are you today?", no_play)
}

pub fn run_kokoro_with_text(text: &str, no_play: bool) -> Result<(), Box<dyn std::error::Error>> {
    let tts = KokoroTTS::new("models/kokoro/kokoro.onnx", "models/kokoro/tokenizer.json")?;

    let voice = load_voice_style("models/kokoro/af.bin", "Nicole")?;

    let normalized_text = normalize_for_kokoro(text.to_string());

    println!("Generating speech for: \"{}\"", text);
    let output_path = "kokoro_test_output.wav";
    let audio = tts.generate_speech(&normalized_text, &voice, 1.0)?;

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

