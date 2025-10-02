fn main() {
    // Link espeak-ng library
    if cfg!(target_os = "macos") {
        // For macOS with Homebrew
        println!("cargo:rustc-link-search=/opt/homebrew/lib");
        println!("cargo:rustc-link-lib=espeak-ng");
    } else if cfg!(target_os = "linux") {
        // For Linux
        println!("cargo:rustc-link-lib=espeak-ng");
    }
}