use crate::kokoro::{KokoroTTS, load_voice_style, normalize_for_kokoro};
use crate::playback::play_wav_file;

pub fn run_kokoro(no_play: bool) -> Result<(), Box<dyn std::error::Error>> {
    run_kokoro_with_text("Hello world, how are you today?", no_play)
}

pub fn run_kokoro_with_text(text: &str, no_play: bool) -> Result<(), Box<dyn std::error::Error>> {
    let tts = KokoroTTS::new()?;

    let voice = load_voice_style("models/kokoro/af.bin", "Nicole")?;

    let normalized_text = normalize_for_kokoro(text.to_string());

    println!("Generating speech for: \"{}\"", text);
    let output_path = "kokoro_test_output.wav";
    let audio = tts.generate_speech(&normalized_text, &voice, 1.0, Some(output_path))?;

    println!("Duration: {:.2} seconds", audio.duration_seconds);

    if !no_play {
        println!("\nPlaying generated speech...");
        play_wav_file(output_path)?;
    } else {
        println!("\nPlayback skipped (--no-play flag set)");
    }

    Ok(())
}

pub fn test_kokoro_comparison(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Testing Kokoro with different tokenization methods ===\n");

    let tts = KokoroTTS::new()?;
    let voice = load_voice_style("models/kokoro/af.bin", "Nicole")?;

    println!("Test 1: Standard encoding (with start/end tokens)");
    let filename1 = "kokoro_test_with_delimiters.wav";
    let audio1 = tts.generate_speech(text, &voice, 1.0, Some(filename1))?;
    println!("Duration: {:.2}s\n", audio1.duration_seconds);

    println!("Test 2: Raw encoding (no start/end tokens)");
    let filename2 = "kokoro_test_raw.wav";
    let audio2 = tts.generate_speech_raw(text, &voice, 1.0, Some(filename2))?;
    println!("Duration: {:.2}s\n", audio2.duration_seconds);

    println!("Compare the two audio files to see which has better pronunciation at the end.");

    Ok(())
}

