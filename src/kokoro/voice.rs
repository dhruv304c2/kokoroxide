use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::error::Error;

#[derive(Clone)]
pub struct VoiceStyle {
    pub data: Vec<f32>,
}

impl VoiceStyle {
    pub fn new(data: Vec<f32>) -> Self {
        VoiceStyle { data }
    }

    pub fn get_style_vector(&self, size: usize) -> Vec<f32> {
        // Take the first 'size' values, or pad with zeros if needed
        let mut result = self.data.iter().take(size).cloned().collect::<Vec<f32>>();
        while result.len() < size {
            result.push(0.0);
        }
        result
    }
}

pub fn load_voice_style<P: AsRef<Path>>(path: P) -> Result<VoiceStyle, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Convert bytes to f32 array (assuming little-endian)
    let style_data: Vec<f32> = buffer.chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();

    Ok(VoiceStyle::new(style_data))
}