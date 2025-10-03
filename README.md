# kokoroxide [WIP]

A high-performance Rust implementation of Kokoro TTS (Text-to-Speech) synthesis, leveraging ONNX Runtime for efficient neural speech generation. Uses espeak-ng for text-to-phoneme conversion, with built-in conversion logic into Misaki phoneme notation expected by Kokoro models. Distributed under a dual MIT/Apache-2.0 license to match the broader Rust ecosystem.

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
kokoroxide = "0.1.3"
```

## Quick Start

```rust
use kokoroxide::{load_voice_style, KokoroTTS, TTSConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure the ONNX model + tokenizer that Kokoro requires.
    // These files live outside the crate; download them from Kokoro's distribution (https://huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX).
    let config = TTSConfig::new("path/to/kokoro.onnx", "path/to/tokenizer.json")
        .with_sample_rate(24000)
        .with_max_tokens_length(512)
        .with_graph_optimization_level(kokoroxide::GraphOptimizationLevel::Disable);

    // Build the speech engine with the explicit configuration so advanced knobs are available.
    let tts_service = KokoroTTS::with_config(config)?;

    // Load a voice style vector (.bin) that controls prosody and speaker identity.
    let voice = load_voice_style("path/to/voice.bin")?;

    // Generate speech at 1.0x speed for the requested text.
    let text = "Hello, this is a text-to-speech synthesis example.";
    let audio = tts_service.generate_speech(text, &voice, 1.0)?;

    // Persist the synthesized waveform to a WAV file for playback.
    audio.save_to_wav("path/to/output.wav")?;

    Ok(())
}
```

For a complete runnable example pointing at real assets, see the `kokoroxide-demo` sample project in this workspace (`kokoroxide-demo/src/main.rs`).

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
Represents voice characteristics as a style vector. Voice files contain multiple style vectors indexed by token length.

```rust
// Load from binary file
let voice = load_voice_style("voice.bin")?;

// Create custom voice with vector size
let custom_voice = VoiceStyle::new(vec![0.1, 0.2, ...], 256);
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
```

## Configuration

### TTSConfig Options

```rust
use ort::{execution_providers::CoreMLExecutionProviderOptions, ExecutionProvider, GraphOptimizationLevel};

let config = TTSConfig::new(model_path, tokenizer_path)
    .with_max_tokens_length(512)    // Maximum token sequence length
    .with_sample_rate(24000)        // Audio sample rate in Hz
    .with_graph_optimization_level(GraphOptimizationLevel::Level3)
    .with_execution_providers(vec![
        ExecutionProvider::CoreML(CoreMLExecutionProviderOptions::default()),
    ]); // Optional hardware acceleration
```

If you don't need custom providers, you can skip the call to `with_execution_providers` and the default CPU provider will be used.

#### Graph Optimization Levels

The `with_graph_optimization_level()` method allows you to control ONNX Runtime's graph optimization:

- `GraphOptimizationLevel::Disable` - No optimizations
- `GraphOptimizationLevel::Level1` - Basic optimizations
- `GraphOptimizationLevel::Level2` - Extended optimizations
- `GraphOptimizationLevel::Level3` - Maximum optimizations (default)

## System Requirements

### Prerequisites

1. **Rust 1.70+**

2. **espeak-ng** (required for text-to-phoneme conversion):
   - **Ubuntu/Debian**: `sudo apt-get install espeak-ng libespeak-ng-dev`
   - **macOS**: `brew install espeak-ng`
   - **Windows**: Download from [espeak-ng releases](https://github.com/espeak-ng/espeak-ng/releases)
   - **Arch Linux**: `sudo pacman -S espeak-ng`

3. **ONNX Runtime** (automatically downloaded via `ort` crate)

4. **Kokoro model files**:
   - Model file (e.g., `kokoro-v0_19.onnx`)
   - Tokenizer configuration (`tokenizer.json`)
   - Voice style files (`.bin` format)
   - Downloaded at runtime or managed outside the crate package to keep the published crate lightweight

### Build Configuration

The crate automatically links to espeak-ng based on your platform:
- **macOS**: Looks for espeak-ng in `/opt/homebrew/lib` (Homebrew default)
- **Linux**: Uses system library paths

If espeak-ng is installed in a non-standard location, you may need to set:
```bash
export LD_LIBRARY_PATH=/path/to/espeak-ng/lib:$LD_LIBRARY_PATH  # Linux
export DYLD_LIBRARY_PATH=/path/to/espeak-ng/lib:$DYLD_LIBRARY_PATH  # macOS
```

### Environment Variables

- **`DEBUG_PHONEMES`** - Enable phoneme debugging output:
  ```bash
  DEBUG_PHONEMES=1 cargo run
  ```
  This will print:
  - Input text
  - Espeak IPA output
  - Converted Misaki phonemes

- **`DEBUG_TOKENS`** - Enable token debugging output:
  ```bash
  DEBUG_TOKENS=1 cargo run
  ```
  This will print:
  - Generated token IDs array

- **`DEBUG_TIMING`** - Enable performance timing logs:
  ```bash
  DEBUG_TIMING=1 cargo run
  ```
  This will print:
  - Phoneme tokenization time
  - Espeak IPA conversion time
  - Total tokenization time

- **All debug modes**:
  ```bash
  DEBUG_PHONEMES=1 DEBUG_TOKENS=1 DEBUG_TIMING=1 cargo run
  ```

## Model Files

Download the Kokoro model files from the official repository:
- Model: [Kokoro-82M ONNX](https://huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX)
- Required files:
  - `*.onnx` - The model file
  - `tokenizer.json` - Tokenizer configuration
  - Voice files (`*.bin`) - Style vectors for different voices
  - Provide these assets at runtime (they are not packaged with the crate to keep the published tarball lightweight)

## Examples

### Basic TTS Application

```rust
use kokoroxide::{KokoroTTS, load_voice_style};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tts = KokoroTTS::new("model.onnx", "tokenizer.json")?;
    let voice = load_voice_style("voice.bin")?;

    let text = "Welcome to kokoroxide TTS!";
    let audio = tts.generate_speech(text, &voice, 1.0)?;
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
