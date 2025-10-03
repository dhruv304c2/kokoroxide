use std::sync::Arc;
use std::path::Path;
use std::error::Error;
use ndarray::{Array1, Array2, CowArray, IxDyn};
use ort::{Environment, GraphOptimizationLevel, Session, SessionBuilder, Value};
use crate::espeak_ipa_tokenizer::EspeakIpaTokenizer;
use super::voice::VoiceStyle;

pub struct TTSConfig {
    pub model_path: String,
    pub tokenizer_path: String,
    pub max_length: usize,
    pub sample_rate: u32,
    pub graph_level: GraphOptimizationLevel
}

impl TTSConfig {
    pub fn new(model_path: &str, tokenizer_path: &str) -> Self {
        TTSConfig {
            model_path: model_path.to_string(),
            tokenizer_path: tokenizer_path.to_string(),
            max_length: 512,
            sample_rate: 22050,
            graph_level: GraphOptimizationLevel::Level3
        }
    }

    pub fn with_max_tokens_length(mut self, max_length: usize) -> Self {
        self.max_length = max_length;
        self
    }

    pub fn with_sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = sample_rate;
        self
    }

    pub fn with_graph_optimization_level(mut self, level: GraphOptimizationLevel) -> Self {
        self.graph_level = level;
        self
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
    tokenizer: EspeakIpaTokenizer,
    config: TTSConfig,
}

impl KokoroTTS {

    pub fn with_config(config: TTSConfig) -> Result<Self, Box<dyn Error>> {
        let env = Arc::new(
            Environment::builder()
                .with_name("kokoro_tts")
                .build()?
        );

        let optimization = match config.graph_level{
            GraphOptimizationLevel::Disable => GraphOptimizationLevel::Disable,
            GraphOptimizationLevel::Level1 => GraphOptimizationLevel::Level1,
            GraphOptimizationLevel::Level2 => GraphOptimizationLevel::Level2,
            GraphOptimizationLevel::Level3 => GraphOptimizationLevel::Level3,
        };

        let session = SessionBuilder::new(&env)?
            .with_optimization_level(optimization)?
            .with_model_from_file(&config.model_path)?;

        let tokenizer_content = std::fs::read_to_string(&config.tokenizer_path)?;
        let tokenizer_json: serde_json::Value = serde_json::from_str(&tokenizer_content)?;
        let vocab_obj = tokenizer_json["model"]["vocab"].as_object()
            .ok_or("No vocab found in tokenizer.json")?;

        let mut vocab = std::collections::HashMap::new();
        for (token, id) in vocab_obj {
            vocab.insert(token.clone(), id.as_i64().unwrap_or(0));
        }

        let tokenizer = EspeakIpaTokenizer::new(vocab)?;

        Ok(KokoroTTS {
            session,
            tokenizer,
            config,
        })
    }

    pub fn generate_speech_from_phonemes(
        &self,
        phonemes: &str,
        voice_style: &VoiceStyle,
        speed: f32,
    ) -> Result<GeneratedAudio, Box<dyn Error>> {
        let tokens = self.tokenizer.encode_phonemes(phonemes, None)?;

        self.generate_from_tokens(&tokens, voice_style, speed)
    }

    pub fn generate_speech(
        &self,
        text: &str,
        voice_style: &VoiceStyle,
        speed: f32,
    ) -> Result<GeneratedAudio, Box<dyn Error>> {
        let tokens = self.tokenizer.encode(text, None)?;

        self.generate_from_tokens(&tokens, voice_style, speed)
    }

    pub fn generate_from_tokens(
        &self,
        tokens: &[i64],
        voice_style: &VoiceStyle,
        speed: f32,
    ) -> Result<GeneratedAudio, Box<dyn Error>> {

        let input_ids = Array2::<i64>::from_shape_vec((1, tokens.len()), tokens.to_vec())?;
        // Use token length to select the appropriate style vector, matching Python implementation
        let style_vector = voice_style.get_style_vector_for_token_length(tokens.len(), 256);
        let style = Array2::<f32>::from_shape_vec((1, 256), style_vector)?;
        let speed_array = Array1::<f32>::from_vec(vec![speed]);

        let input_ids_cow: CowArray<i64, IxDyn> = CowArray::from(input_ids.into_dyn());
        let style_cow: CowArray<f32, IxDyn> = CowArray::from(style.into_dyn());
        let speed_cow: CowArray<f32, IxDyn> = CowArray::from(speed_array.into_dyn());

        let input_ids_tensor = Value::from_array(self.session.allocator(), &input_ids_cow)?;
        let style_tensor = Value::from_array(self.session.allocator(), &style_cow)?;
        let speed_tensor = Value::from_array(self.session.allocator(), &speed_cow)?;

        let outputs = self.session.run(vec![input_ids_tensor, style_tensor, speed_tensor])?;

        if let Ok(output) = outputs[0].try_extract::<f32>() {
            let view = output.view();
            let samples = view.as_slice().unwrap().to_vec();
            let duration_seconds = samples.len() as f32 / self.config.sample_rate as f32;

            let audio = GeneratedAudio {
                samples,
                sample_rate: self.config.sample_rate,
                duration_seconds,
            };

            Ok(audio)
        } else {
            Err("Failed to extract audio output".into())
        }
    }

    #[allow(dead_code)]
    pub fn speak(&self, text: &str, voice_style: &VoiceStyle) -> Result<GeneratedAudio, Box<dyn Error>> {
        self.generate_speech(text, voice_style, 1.0)
    }
}
