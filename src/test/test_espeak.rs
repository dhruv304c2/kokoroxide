use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int, c_void};

// Simple FFI bindings for espeak-ng
#[link(name = "espeak-ng")]
extern "C" {
    fn espeak_Initialize(
        output: c_int,
        buflength: c_int,
        path: *const c_char,
        options: c_int,
    ) -> c_int;

    fn espeak_SetVoiceByName(name: *const c_char) -> c_int;

    fn espeak_TextToPhonemes(
        textptr: *const *const c_void,
        textmode: c_int,
        phonememode: c_int,
    ) -> *const c_char;

    fn espeak_Terminate() -> c_int;
}

// Constants
#[allow(dead_code)]
const AUDIO_OUTPUT_SYNCHRONOUS: c_int = 0x01;
const AUDIO_OUTPUT_RETRIEVAL: c_int = 0x02;
const ESPEAK_PHONEMES_IPA: c_int = 0x02;
const ESPEAK_CHARS_UTF8: c_int = 1;

pub fn test_espeak() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing espeak-ng FFI for G2P conversion");

    unsafe {
        // Initialize espeak-ng (no audio output needed for phoneme conversion)
        let init_result = espeak_Initialize(
            AUDIO_OUTPUT_RETRIEVAL,
            0,
            std::ptr::null(),
            0,
        );

        if init_result < 0 {
            return Err("Failed to initialize espeak-ng".into());
        }

        // Set voice to English
        let voice_name = CString::new("en")?;
        espeak_SetVoiceByName(voice_name.as_ptr());

        // Test words
        let test_words = vec!["hello", "world", "a", "i", "I", "the", "cat", "apple"];

        for word in test_words {
            println!("\nTesting word: '{}'", word);

            let c_text = CString::new(word)?;
            let mut text_ptr = c_text.as_ptr() as *const c_void;

            // Get IPA phonemes
            let phonemes_ptr = espeak_TextToPhonemes(
                &mut text_ptr,
                ESPEAK_CHARS_UTF8,
                ESPEAK_PHONEMES_IPA,
            );

            if !phonemes_ptr.is_null() {
                let phonemes = CStr::from_ptr(phonemes_ptr).to_string_lossy();
                println!("  IPA: {}", phonemes);
            } else {
                println!("  Failed to get phonemes");
            }
        }

        // Cleanup
        espeak_Terminate();
    }

    Ok(())
}