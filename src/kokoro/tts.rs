use std::sync::Arc;
use std::path::Path;
use std::error::Error;
use std::time::Instant;
use ndarray::{Array1, Array2, CowArray, IxDyn};
use ort::{Environment, GraphOptimizationLevel, Session, SessionBuilder, Value};
use crate::ipa_tokenizer::IpaTokenizer;
use super::voice::VoiceStyle;

pub struct TTSConfig {
    pub model_path: String,
    pub tokenizer_path: String,
    pub max_length: usize,
    pub sample_rate: u32,
}

impl Default for TTSConfig {
    fn default() -> Self {
        TTSConfig {
            model_path: "models/kokoro/kokoro.onnx".to_string(),
            tokenizer_path: "models/kokoro/tokenizer.json".to_string(),
            max_length: 64,
            sample_rate: 22050,
        }
    }
}

pub struct GeneratedAudio {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub duration_seconds: f32,
}

impl GeneratedAudio {
    pub fn save_to_wav<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: self.sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = hound::WavWriter::create(path, spec)?;

        // Add 0.1 seconds of silence at the beginning
        let silence_samples = (self.sample_rate as f32 * 0.1) as usize;
        for _ in 0..silence_samples {
            writer.write_sample(0i16)?;
        }

        // Write the actual audio
        for &sample in &self.samples {
            // Clamp to prevent overflow
            let clamped = sample.max(-1.0).min(1.0);
            let amplitude = (clamped * i16::MAX as f32) as i16;
            writer.write_sample(amplitude)?;
        }

        // Add 0.1 seconds of silence at the end
        for _ in 0..silence_samples {
            writer.write_sample(0i16)?;
        }

        writer.finalize()?;
        Ok(())
    }
}

pub struct KokoroTTS {
    session: Session,
    tokenizer: IpaTokenizer,
    config: TTSConfig,
}

impl KokoroTTS {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Self::with_config(TTSConfig::default())
    }

    pub fn with_config(config: TTSConfig) -> Result<Self, Box<dyn Error>> {
        let env = Arc::new(
            Environment::builder()
                .with_name("kokoro_tts")
                .build()?
        );

        let session = SessionBuilder::new(&env)?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_model_from_file(&config.model_path)?;

        let tokenizer_content = std::fs::read_to_string(&config.tokenizer_path)?;
        let tokenizer_json: serde_json::Value = serde_json::from_str(&tokenizer_content)?;
        let vocab_obj = tokenizer_json["model"]["vocab"].as_object()
            .ok_or("No vocab found in tokenizer.json")?;

        let mut vocab = std::collections::HashMap::new();
        for (token, id) in vocab_obj {
            vocab.insert(token.clone(), id.as_i64().unwrap_or(0));
        }

        let tokenizer = IpaTokenizer::new(vocab)?;

        Ok(KokoroTTS {
            session,
            tokenizer,
            config,
        })
    }

    pub fn generate_speech(
        &self,
        text: &str,
        voice_style: &VoiceStyle,
        speed: f32,
        output_path: Option<&str>,
    ) -> Result<GeneratedAudio, Box<dyn Error>> {
        let tokens = self.tokenizer.encode(text, None)?;

        let input_ids = Array2::<i64>::from_shape_vec((1, tokens.len()), tokens)?;
        let style_vector = voice_style.get_style_vector(256);
        let style = Array2::<f32>::from_shape_vec((1, 256), style_vector)?;
        let speed_array = Array1::<f32>::from_vec(vec![speed]);

        let input_ids_cow: CowArray<i64, IxDyn> = CowArray::from(input_ids.into_dyn());
        let style_cow: CowArray<f32, IxDyn> = CowArray::from(style.into_dyn());
        let speed_cow: CowArray<f32, IxDyn> = CowArray::from(speed_array.into_dyn());

        let input_ids_tensor = Value::from_array(self.session.allocator(), &input_ids_cow)?;
        let style_tensor = Value::from_array(self.session.allocator(), &style_cow)?;
        let speed_tensor = Value::from_array(self.session.allocator(), &speed_cow)?;

        let generation_start = Instant::now();
        let outputs = self.session.run(vec![input_ids_tensor, style_tensor, speed_tensor])?;
        let generation_duration = generation_start.elapsed();
        println!("Model output generation took: {:?}", generation_duration);

        if let Ok(output) = outputs[0].try_extract::<f32>() {
            let view = output.view();
            let samples = view.as_slice().unwrap().to_vec();
            let duration_seconds = samples.len() as f32 / self.config.sample_rate as f32;

            let audio = GeneratedAudio {
                samples,
                sample_rate: self.config.sample_rate,
                duration_seconds,
            };

            if let Some(path) = output_path {
                audio.save_to_wav(path)?;
            } else {
                let safe_filename = text.chars()
                    .filter(|c| c.is_alphanumeric() || *c == ' ')
                    .collect::<String>()
                    .replace(" ", "_")
                    .to_lowercase();
                let filename = format!("kokoro_{}.wav", &safe_filename[..safe_filename.len().min(20)]);
                audio.save_to_wav(&filename)?;
            }

            Ok(audio)
        } else {
            Err("Failed to extract audio output".into())
        }
    }

    pub fn speak(&self, text: &str, voice_style: &VoiceStyle) -> Result<GeneratedAudio, Box<dyn Error>> {
        self.generate_speech(text, voice_style, 1.0, None)
    }

    pub fn get_sample_rate(&self) -> u32 {
        self.config.sample_rate
    }

    pub fn get_max_length(&self) -> usize {
        self.config.max_length
    }

    pub fn generate_speech_raw(
        &self,
        text: &str,
        voice_style: &VoiceStyle,
        speed: f32,
        output_path: Option<&str>,
    ) -> Result<GeneratedAudio, Box<dyn Error>> {
        let tokens = self.tokenizer.encode_raw(text)?;

        let input_ids = Array2::<i64>::from_shape_vec((1, tokens.len()), tokens)?;
        let style_vector = voice_style.get_style_vector(256);
        let style = Array2::<f32>::from_shape_vec((1, 256), style_vector)?;
        let speed_array = Array1::<f32>::from_vec(vec![speed]);

        let input_ids_cow: CowArray<i64, IxDyn> = CowArray::from(input_ids.into_dyn());
        let style_cow: CowArray<f32, IxDyn> = CowArray::from(style.into_dyn());
        let speed_cow: CowArray<f32, IxDyn> = CowArray::from(speed_array.into_dyn());

        let input_ids_tensor = Value::from_array(self.session.allocator(), &input_ids_cow)?;
        let style_tensor = Value::from_array(self.session.allocator(), &style_cow)?;
        let speed_tensor = Value::from_array(self.session.allocator(), &speed_cow)?;

        let generation_start = Instant::now();
        let outputs = self.session.run(vec![input_ids_tensor, style_tensor, speed_tensor])?;
        let generation_duration = generation_start.elapsed();
        println!("Model output generation took: {:?}", generation_duration);

        if let Ok(output) = outputs[0].try_extract::<f32>() {
            let view = output.view();
            let samples = view.as_slice().unwrap().to_vec();
            let duration_seconds = samples.len() as f32 / self.config.sample_rate as f32;

            let audio = GeneratedAudio {
                samples,
                sample_rate: self.config.sample_rate,
                duration_seconds,
            };

            if let Some(path) = output_path {
                audio.save_to_wav(path)?;
            } else {
                let safe_filename = text.chars()
                    .filter(|c| c.is_alphanumeric() || *c == ' ')
                    .collect::<String>()
                    .replace(" ", "_")
                    .to_lowercase();
                let filename = format!("kokoro_raw_{}.wav", &safe_filename[..safe_filename.len().min(20)]);
                audio.save_to_wav(&filename)?;
            }

            Ok(audio)
        } else {
            Err("Failed to extract audio output".into())
        }
    }
}
