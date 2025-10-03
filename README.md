# kokoroxide [WIP]

A high-performance Rust implementation of Kokoro TTS (Text-to-Speech) synthesis, leveraging ONNX Runtime for efficient neural speech generation. Uses espeak-ng for text-to-phoneme conversion, with built-in conversion logic into Misaki phoneme notation expected by Kokoro models.

> **Note:** Currently only supports and has been tested with American English. Contributions for different languages are very welcome! 

## Features

- 🎨 **Voice Style Control** - Customize voice characteristics with style vectors
- 🔤 **Phoneme Support** - Direct phoneme input for precise pronunciation control
- ⚡ **Speed Control** - Adjust speech rate dynamically
- 🔧 **Flexible API** - Multiple generation methods for different use cases

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
kokoroxide = "0.1.0"
```

## Quick Start

```rust
use kokoroxide::{KokoroTTS, VoiceStyle, load_voice_style};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize TTS with model and tokenizer
    let tts = KokoroTTS::new("path/to/model.onnx", "path/to/tokenizer.json")?;

    // Load a voice style
    let voice = load_voice_style("path/to/voice.bin")?;

    // Generate speech
    let audio = tts.speak("Hello, world!", &voice)?;

    // Save to file
    audio.save_to_wav("output.wav")?;

    Ok(())
}
```

## API Overview

### Core Types

#### `KokoroTTS`
The main TTS engine that handles text-to-speech conversion.

```rust
// Create with default config
let tts = KokoroTTS::new(model_path, tokenizer_path)?;

// Create with custom config
let config = TTSConfig::new(model_path, tokenizer_path)
    .with_max_tokens_length(128)
    .with_sample_rate(24000);
let tts = KokoroTTS::with_config(config)?;
```

#### `VoiceStyle`
Represents voice characteristics as a style vector.

```rust
// Load from binary file
let voice = load_voice_style("voice.bin")?;

// Create custom voice
let custom_voice = VoiceStyle::new(vec![0.1, 0.2, ...]);
```

#### `GeneratedAudio`
Contains the generated audio samples and metadata.

```rust
let audio = tts.speak("Hello!", &voice)?;
println!("Duration: {} seconds", audio.duration_seconds);
println!("Sample rate: {} Hz", audio.sample_rate);
audio.save_to_wav("output.wav")?;
```

### Generation Methods

#### 1. Simple Text-to-Speech
```rust
let audio = tts.speak("Hello, world!", &voice)?;
```

#### 2. With Speed Control
```rust
let audio = tts.generate_speech("Speak faster!", &voice, 1.5)?; // 1.5x speed
```

#### 3. From Phonemes
```rust
let audio = tts.generate_speech_from_phonemes("həˈloʊ wɜːld", &voice, 1.0)?;
```

#### 4. From Token IDs
```rust
let tokens = vec![101, 2234, 1567, 102]; // Pre-tokenized input
let audio = tts.generate_from_tokens(&tokens, &voice, 1.0)?;

## Configuration

### TTSConfig Options

```rust
use ort::GraphOptimizationLevel;

let config = TTSConfig::new(model_path, tokenizer_path)
    .with_max_tokens_length(64)     // Maximum token sequence length
    .with_sample_rate(22050)        // Audio sample rate in Hz
    .with_graph_optimization_level(GraphOptimizationLevel::Level3); // ONNX graph optimization
```

#### Graph Optimization Levels

The `with_graph_optimization_level()` method allows you to control ONNX Runtime's graph optimization:

- `GraphOptimizationLevel::Disable` - No optimizations
- `GraphOptimizationLevel::Level1` - Basic optimizations
- `GraphOptimizationLevel::Level2` - Extended optimizations
- `GraphOptimizationLevel::Level3` - Maximum optimizations (default)

## Requirements

- Rust 1.70+
- ONNX Runtime (automatically downloaded via `ort` crate)
- Kokoro model files (.onnx model and tokenizer.json)

## Model Files

You'll need:
1. A Kokoro ONNX model file (e.g., `kokoro-v0_19.onnx`)
2. A tokenizer configuration file (`tokenizer.json`)
3. Voice style files (`.bin` format)

## Examples

### Basic TTS Application

```rust
use kokoroxide::{KokoroTTS, load_voice_style, normalize_for_kokoro};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tts = KokoroTTS::new("model.onnx", "tokenizer.json")?;
    let voice = load_voice_style("voice.bin")?;

    let text = "Welcome to kokoroxide TTS!";
    let normalized = normalize_for_kokoro(text.to_string());

    let audio = tts.generate_speech(&normalized, &voice, 1.0)?;
    audio.save_to_wav("welcome.wav")?;

    println!("Generated {} seconds of audio", audio.duration_seconds);
    Ok(())
}
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

This project implements the Kokoro TTS model in Rust, providing a high-performance alternative to Python implementations.
