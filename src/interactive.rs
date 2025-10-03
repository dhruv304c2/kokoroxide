use std::io::{self, Write};
use std::time::Instant;
use crate::kokoro::{load_voice_style, KokoroTTS, TTSConfig};
use crate::playback::play_wav_file;

pub struct InteractiveTTS {
    tts: KokoroTTS,
    voice_style: crate::kokoro::VoiceStyle,
    output_counter: usize,
}

impl InteractiveTTS {
    /// Create a new interactive TTS session
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let tts_config = TTSConfig::new(
            "models/kokoro/kokoro.onnx", 
            "models/kokoro/tokenizer.json"
        )
        .with_graph_optimization_level(ort::GraphOptimizationLevel::Disable)
        .with_max_tokens_length(512)
        .with_sample_rate(24000);

        let tts = KokoroTTS::with_config(tts_config)?;
        let voice_style = load_voice_style("models/kokoro/af.bin")?;

        Ok(InteractiveTTS {
            tts,
            voice_style,
            output_counter: 0,
        })
    }

    /// Run the interactive TTS loop
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n=== Interactive Kokoro TTS ===");
        println!("Type your text and press Enter to generate speech.");
        println!("Type 'quit', 'exit', or 'q' to stop.\n");

        loop {
            print!("> ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            if matches!(input.to_lowercase().as_str(), "quit" | "exit" | "q") {
                println!("Goodbye!");
                break;
            }

            if let Err(e) = self.process_text(input) {
                eprintln!("Error: {}", e);
                println!("Please try again.");
            }
        }

        Ok(())
    }

    /// Process a single text input
    fn process_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        self.output_counter += 1;
        let filename = "interactive_tts.wav";
        let audio = self.tts.generate_speech(text, &self.voice_style, 1.0)?;
        audio.save_to_wav(&filename)?;
        let generation_time = start_time.elapsed();

        play_wav_file(&filename)?;

        if std::env::var("DEBUG_TIMING").is_ok() {
            println!("Generated in {:.2}s ({:.1}x realtime)",
                     generation_time.as_secs_f32(),
                     audio.duration_seconds / generation_time.as_secs_f32());
        }

        Ok(())
    }

    pub fn get_stats(&self) -> String {
        format!("Total generations: {}", self.output_counter)
    }
}

/// Run interactive TTS with custom settings
#[allow(dead_code)]
pub fn run_interactive_tts_with_options(
    speed: f32,
    voice_path: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing Interactive TTS with custom options...");

    let tts_config = TTSConfig::new(
            "models/kokoro/kokoro.onnx", 
            "models/kokoro/tokenizer.json"
        )
        .with_graph_optimization_level(ort::GraphOptimizationLevel::Disable)
        .with_max_tokens_length(512)
        .with_sample_rate(24000);

    let tts = KokoroTTS::with_config(tts_config)?;

    let voice_path = voice_path.unwrap_or("models/kokoro/af.bin");
    let voice_style = load_voice_style(voice_path)?;

    println!("TTS engine initialized!");
    println!("Voice: {}", voice_path);
    println!("Speed: {:.1}x", speed);
    println!("\n=== Interactive Kokoro TTS ===");
    println!("Type your text and press Enter to generate speech.");
    println!("Type 'quit', 'exit', or 'q' to stop.\n");

    let mut output_counter = 0;

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if matches!(input.to_lowercase().as_str(), "quit" | "exit" | "q") {
            println!("Goodbye! Generated {} audio files.", output_counter);
            break;
        }

        // Process with timing
        let start_time = Instant::now();

        output_counter += 1;
        let filename = format!("interactive_tts_{:04}.wav", output_counter);

        println!("\nGenerating speech for: \"{}\"", input);

        match tts.generate_speech(input, &voice_style, speed) {
            Ok(audio) => {
                // Save the audio file
                audio.save_to_wav(&filename)?;

                let generation_time = start_time.elapsed();

                if std::env::var("DEBUG_TIMING").is_ok() {
                    println!("Generation completed in: {:.2}s", generation_time.as_secs_f32());
                    println!("Audio duration: {:.2}s", audio.duration_seconds);
                    println!("Generation speed: {:.2}x realtime",
                             audio.duration_seconds / generation_time.as_secs_f32());
                }

                println!("Playing audio...");
                let play_start = Instant::now();

                if let Err(e) = play_wav_file(&filename) {
                    eprintln!("Playback error: {}", e);
                } else {
                    let play_time = play_start.elapsed();
                    if std::env::var("DEBUG_TIMING").is_ok() {
                        println!("Playback completed in: {:.2}s", play_time.as_secs_f32());
                    }
                }

                if std::env::var("DEBUG_TIMING").is_ok() {
                    println!("Total time: {:.2}s\n", start_time.elapsed().as_secs_f32());
                }
            }
            Err(e) => {
                eprintln!("Generation error: {}", e);
                println!("Please try again.");
            }
        }
    }

    Ok(())
}

/// Simple entry point for interactive TTS
pub fn run_interactive() -> Result<(), Box<dyn std::error::Error>> {
    let mut session = InteractiveTTS::new()?;
    session.run()?;
    println!("\nSession stats: {}", session.get_stats());
    Ok(())
}
