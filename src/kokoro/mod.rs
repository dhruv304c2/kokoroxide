mod tts;
mod voice;

pub use tts::{KokoroTTS, TTSConfig};
#[allow(unused_imports)]
pub use tts::GeneratedAudio;
pub use voice::{VoiceStyle, load_voice_style};
