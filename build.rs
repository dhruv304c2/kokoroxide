fn main() {
    // Link espeak-ng library
    if cfg!(target_os = "macos") {
        // For macOS with Homebrew
        println!("cargo:rustc-link-search=/opt/homebrew/lib");
        println!("cargo:rustc-link-lib=espeak-ng");
    } else if cfg!(any(target_os = "linux", target_os = "android")) {
        // For Linux and Android
        println!("cargo:rustc-link-lib=espeak-ng");
    }
}
