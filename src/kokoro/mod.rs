mod tts;
mod voice;

#[allow(unused_imports)]
pub use tts::GeneratedAudio;
pub use tts::{KokoroTTS, TTSConfig};
pub use voice::{load_voice_style, VoiceStyle};
