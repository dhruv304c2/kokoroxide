use clap::{Parser, Subcommand};

// Internal modules for the binary
mod test;
mod kokoro;
mod playback;
mod interactive;
mod espeak_g2p;
mod espeak_ipa_tokenizer;

#[derive(Parser)]
#[command(name = "ronnex")]
#[command(about = "ONNX Runtime Rust example application", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run tests
    Test {
        /// Name of the test to run
        #[arg(value_enum)]
        name: TestName,
        /// Text to synthesize (for Kokoro test)
        #[arg(short, long)]
        text: Option<String>,
        /// Skip audio playback
        #[arg(long)]
        no_play: bool,
    },
}

#[derive(clap::ValueEnum, Clone)]
enum TestName {
    /// Run the identity test
    Identity,
    /// Run the Kokoro model test
    Kokoro,
    /// Analyze IPA symbols in Kokoro vocabulary
    AnalyzeIpa,
    /// Interactive Kokoro chat mode
    KokoroChat,
    /// Test espeak-ng FFI
    Espeak,
    /// Test espeak tokenizer
    EspeakTokenizer,
    /// Test direct phoneme input
    DirectPhonemes,
    /// Test raw token input
    RawTokens,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Test { name, text, no_play }) => {
            match name {
                TestName::Identity => {
                    test::identity_test::run_identity()?;
                }
                TestName::Kokoro => {
                    if let Some(custom_text) = text {
                        test::kokoro_test::run_kokoro_with_text(&custom_text, no_play)?;
                    } else {
                        test::kokoro_test::run_kokoro(no_play)?;
                    }
                }
                TestName::AnalyzeIpa => {
                    test::analyze_ipa::analyze_ipa_in_kokoro()?;
                }
                TestName::KokoroChat => {
                    interactive::run_interactive()?;
                }
                TestName::Espeak => {
                    test::test_espeak::test_espeak()?;
                }
                TestName::EspeakTokenizer => {
                    test::test_espeak_tokenizer::test_espeak_tokenizer()?;
                }
                TestName::DirectPhonemes => {
                    test::test_direct_phonemes::test_direct_phonemes()?;
                }
                TestName::RawTokens => {
                    test::test_raw_tokens::test_raw_tokens()?;
                }
            }
        }
        None => {
            println!("No command specified. Use --help for usage information.");
        }
    }

    Ok(())
}
