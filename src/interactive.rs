use std::io::{self, Write};
use std::time::Instant;
use crate::kokoro::{KokoroTTS, load_voice_style, normalize_for_kokoro};
use crate::playback::play_wav_file;

pub struct InteractiveTTS {
    tts: KokoroTTS,
    voice_style: crate::kokoro::VoiceStyle,
    output_counter: usize,
}

impl InteractiveTTS {
    /// Create a new interactive TTS session
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let tts = KokoroTTS::new()?;
        let voice_style = load_voice_style("models/kokoro/af.bin", "Nicole")?;

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
            // Prompt for input
            print!("> ");
            io::stdout().flush()?;

            // Read user input
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            // Check for exit commands
            if input.is_empty() {
                continue;
            }

            if matches!(input.to_lowercase().as_str(), "quit" | "exit" | "q") {
                println!("Goodbye!");
                break;
            }

            // Process the input
            if let Err(e) = self.process_text(input) {
                eprintln!("Error: {}", e);
                println!("Please try again.");
            }
        }

        Ok(())
    }

    /// Process a single text input
    fn process_text(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Normalize text
        let normalized = normalize_for_kokoro(text.to_string());

        // Start timing
        let start_time = Instant::now();

        // Generate filename
        self.output_counter += 1;
        let filename = "interactive_tts.wav";

        // Generate speech
        let audio = self.tts.generate_speech(&normalized, &self.voice_style, 1.0, Some(&filename))?;

        // Calculate generation time
        let generation_time = start_time.elapsed();

        // Play the audio
        play_wav_file(&filename)?;

        // Print concise stats
        println!("Generated in {:.2}s ({:.1}x realtime)",
                 generation_time.as_secs_f32(),
                 audio.duration_seconds / generation_time.as_secs_f32());

        Ok(())
    }

    /// Get statistics about the session
    pub fn get_stats(&self) -> String {
        format!("Total generations: {}", self.output_counter)
    }
}

/// Run interactive TTS with custom settings
pub fn run_interactive_tts_with_options(
    speed: f32,
    voice_path: Option<&str>,
    voice_name: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing Interactive TTS with custom options...");

    let tts = KokoroTTS::new()?;
    let voice_path = voice_path.unwrap_or("models/kokoro/af.bin");
    let voice_name = voice_name.unwrap_or("Nicole");
    let voice_style = load_voice_style(voice_path, voice_name)?;

    println!("TTS engine initialized!");
    println!("Voice: {} ({})", voice_name, voice_path);
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
        let normalized = normalize_for_kokoro(input.to_string());
        let start_time = Instant::now();

        output_counter += 1;
        let filename = format!("interactive_tts_{:04}.wav", output_counter);

        println!("\nGenerating speech for: \"{}\"", input);

        match tts.generate_speech(&normalized, &voice_style, speed, Some(&filename)) {
            Ok(audio) => {
                let generation_time = start_time.elapsed();

                println!("Generation completed in: {:.2}s", generation_time.as_secs_f32());
                println!("Audio duration: {:.2}s", audio.duration_seconds);
                println!("Generation speed: {:.2}x realtime",
                         audio.duration_seconds / generation_time.as_secs_f32());

                println!("Playing audio...");
                let play_start = Instant::now();

                if let Err(e) = play_wav_file(&filename) {
                    eprintln!("Playback error: {}", e);
                } else {
                    let play_time = play_start.elapsed();
                    println!("Playback completed in: {:.2}s", play_time.as_secs_f32());
                }

                println!("Total time: {:.2}s\n", start_time.elapsed().as_secs_f32());
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
