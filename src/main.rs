use clap::{Parser, Subcommand};

pub mod test;
pub mod kokoro;
pub mod ipa_tokenizer;
pub mod playback;
pub mod interactive;

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
    /// Compare Kokoro tokenization methods
    KokoroCompare,
    /// Test rustruut IPA conversion
    Rustruut,
    /// Analyze IPA symbols in Kokoro vocabulary
    AnalyzeIpa,
    /// Test IPA tokenizer
    IpaTokenizer,
    /// Interactive Kokoro chat mode
    KokoroChat,
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
                TestName::KokoroCompare => {
                    let test_text = text.unwrap_or_else(|| "Testing the last word pronunciation".to_string());
                    test::kokoro_test::test_kokoro_comparison(&test_text)?;
                }
                TestName::Rustruut => {
                    test::test_rustruut::test_rustruut()?;
                }
                TestName::AnalyzeIpa => {
                    test::analyze_ipa::analyze_ipa_in_kokoro()?;
                }
                TestName::IpaTokenizer => {
                    test::test_ipa_tokenizer::test_ipa_tokenizer()?;
                }
                TestName::KokoroChat => {
                    interactive::run_interactive()?;
                }
            }
        }
        None => {
            println!("No command specified. Use --help for usage information.");
        }
    }

    Ok(())
}
