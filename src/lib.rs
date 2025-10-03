//! # kokoroxide
//!
//! A high-performance Rust implementation of Kokoro TTS (Text-to-Speech) synthesis,
//! leveraging ONNX Runtime for efficient neural speech generation.
//!
//! ## Example
//!
//! ```no_run
//! use kokoroxide::{KokoroTTS, load_voice_style};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize TTS with model and tokenizer
//! let tts = KokoroTTS::new("path/to/model.onnx", "path/to/tokenizer.json")?;
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
mod espeak_g2p;
mod espeak_ipa_tokenizer;
mod playback;
mod interactive;
mod test;

// Public API modules
/// Kokoro TTS synthesis engine and related types
pub mod kokoro;

// Re-export main types for convenience
pub use kokoro::{
    KokoroTTS,
    TTSConfig,
    GeneratedAudio,
    VoiceStyle,
    load_voice_style,
};

// Re-export ONNX GraphOptimizationLevel for configuration
pub use ort::GraphOptimizationLevel;