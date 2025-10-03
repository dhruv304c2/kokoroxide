use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::error::Error;

#[derive(Clone)]
pub struct VoiceStyle {
    pub data: Vec<f32>,
    pub vector_size: usize,
}

impl VoiceStyle {
    pub fn new(data: Vec<f32>, vector_size: usize) -> Self {
        VoiceStyle { data, vector_size }
    }

    pub fn get_style_vector(&self, size: usize) -> Vec<f32> {
        // This returns a fixed style vector of the requested size
        // In the Python implementation, the style is selected based on token length
        // but this method doesn't have access to token length
        let mut result = self.data.iter().take(size).cloned().collect::<Vec<f32>>();
        while result.len() < size {
            result.push(0.0);
        }
        result
    }

    pub fn get_style_vector_for_token_length(&self, token_length: usize, vector_size: usize) -> Vec<f32> {
        // Select style vector based on token length, matching Python implementation
        // voices[len(tokens)] where voices has shape (-1, 1, 256)
        let offset = token_length * self.vector_size;

        if offset + vector_size <= self.data.len() {
            self.data[offset..offset + vector_size].to_vec()
        } else {
            // If the requested offset is out of bounds, use the last available vector
            let last_vector_start = (self.data.len() / self.vector_size) * self.vector_size;
            if last_vector_start + vector_size <= self.data.len() {
                self.data[last_vector_start..last_vector_start + vector_size].to_vec()
            } else {
                // Fallback to the first vector
                self.get_style_vector(vector_size)
            }
        }
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

    // Each style vector is 256 floats
    Ok(VoiceStyle::new(style_data, 256))
}