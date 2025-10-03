//! # kokoroxide
//!
//! A high-performance Rust implementation of Kokoro TTS (Text-to-Speech) synthesis,
//! leveraging ONNX Runtime for efficient neural speech generation.
//!
//! ## Example
//!
//! ```no_run
//! use kokoroxide::{KokoroTTS, TTSConfig, load_voice_style};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Configure and initialize Kokoro TTS
//! let config = TTSConfig::new("path/to/model.onnx", "path/to/tokenizer.json");
//! let tts = KokoroTTS::with_config(config)?;
//!
//! // Load a voice style
//! let voice = load_voice_style("path/to/voice.bin")?;
//!
//! // Generate speech
//! let audio = tts.speak("Hello, world!", &voice)?;
//!
//! // Save to file
//! audio.save_to_wav("output.wav")?;
//! # Ok(())
//! # }
//! ```

// Internal modules - not exposed to library users
mod espeak;
#[allow(dead_code)]
mod interactive;
#[allow(dead_code)]
mod playback;
#[allow(dead_code)]
mod test;

// Public API modules
/// Kokoro TTS synthesis engine and related types
pub mod kokoro;

// Re-export main types for convenience
pub use kokoro::{load_voice_style, GeneratedAudio, KokoroTTS, TTSConfig, VoiceStyle};

// Re-export ONNX GraphOptimizationLevel for configuration
pub use ort::GraphOptimizationLevel;
