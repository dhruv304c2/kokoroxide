use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use rodio::{Decoder, OutputStream, Sink};

pub struct AudioPlayer {
    _stream: OutputStream,
    sink: Sink,
}

impl AudioPlayer {
    /// Create a new audio player
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;

        Ok(AudioPlayer {
            _stream: stream,
            sink,
        })
    }

    /// Play an audio file from path
    pub fn play_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(path.as_ref())?;
        let source = Decoder::new(BufReader::new(file))?;

        self.sink.append(source);

        Ok(())
    }

    /// Play and wait for completion
    pub fn play_file_blocking<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        self.play_file(path)?;
        self.sink.sleep_until_end();
        Ok(())
    }

    /// Check if audio is still playing
    #[allow(dead_code)]
    pub fn is_playing(&self) -> bool {
        !self.sink.empty()
    }

    /// Pause playback
    #[allow(dead_code)]
    pub fn pause(&self) {
        self.sink.pause();
    }

    /// Resume playback
    #[allow(dead_code)]
    pub fn resume(&self) {
        self.sink.play();
    }

    /// Stop playback
    #[allow(dead_code)]
    pub fn stop(&self) {
        self.sink.stop();
    }

    /// Set volume (0.0 to 1.0)
    #[allow(dead_code)]
    pub fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume);
    }

    /// Get current volume
    #[allow(dead_code)]
    pub fn volume(&self) -> f32 {
        self.sink.volume()
    }
}

/// Simple function to play a WAV file and wait for completion
pub fn play_wav_file<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn std::error::Error>> {
    let player = AudioPlayer::new()?;
    player.play_file_blocking(&path)?;
    Ok(())
}

/// Play a WAV file with a custom message
#[allow(dead_code)]
pub fn play_wav_with_message<P: AsRef<Path>>(path: P, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", message);
    play_wav_file(path)
}