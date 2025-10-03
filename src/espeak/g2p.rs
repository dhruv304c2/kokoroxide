use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::sync::Once;

// FFI bindings for espeak-ng
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

    #[allow(dead_code)]
    fn espeak_Terminate() -> c_int;
}

// Constants
const AUDIO_OUTPUT_RETRIEVAL: c_int = 0x02;
const ESPEAK_PHONEMES_IPA: c_int = 0x02;
const ESPEAK_PHONEMES_SHOW_STRESS: c_int = 0x04;
const ESPEAK_PHONEMES_TIE: c_int = 0x08;
const ESPEAK_CHARS_UTF8: c_int = 1;

static INIT: Once = Once::new();
static mut INITIALIZED: bool = false;

pub struct EspeakG2P;

impl EspeakG2P {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize espeak-ng only once
        unsafe {
            INIT.call_once(|| {
                let result = espeak_Initialize(AUDIO_OUTPUT_RETRIEVAL, 0, std::ptr::null(), 0);
                INITIALIZED = result >= 0;

                if INITIALIZED {
                    // Set voice to American English
                    let voice_name = CString::new("en-us").unwrap();
                    let result = espeak_SetVoiceByName(voice_name.as_ptr());
                    if result != 0 {
                        eprintln!("Warning: Failed to set voice to en-us, result: {}", result);
                    }
                }
            });

            if !INITIALIZED {
                return Err("Failed to initialize espeak-ng".into());
            }
        }

        Ok(EspeakG2P)
    }

    pub fn text_to_ipa(&self, text: &str) -> Result<String, Box<dyn std::error::Error>> {
        unsafe {
            let c_text = CString::new(text)?;
            let mut text_ptr = c_text.as_ptr() as *const c_void;
            let mut all_phonemes = String::new();

            // espeak_TextToPhonemes processes one sentence at a time,
            // so we need to call it repeatedly until all text is processed
            loop {
                // Get IPA phonemes with stress markers and tie markers (as per Misaki docs)
                let phoneme_mode =
                    ESPEAK_PHONEMES_IPA | ESPEAK_PHONEMES_SHOW_STRESS | ESPEAK_PHONEMES_TIE;
                let phonemes_ptr =
                    espeak_TextToPhonemes(&mut text_ptr, ESPEAK_CHARS_UTF8, phoneme_mode);

                if phonemes_ptr.is_null() {
                    break;
                }

                let phonemes = CStr::from_ptr(phonemes_ptr).to_string_lossy().to_string();
                if !phonemes.is_empty() {
                    if !all_phonemes.is_empty() {
                        all_phonemes.push(' ');
                    }
                    all_phonemes.push_str(&phonemes);
                }

                // Check if we've processed all text
                if text_ptr.is_null() {
                    break;
                }

                // Check if the remaining text is empty or only whitespace
                let remaining = CStr::from_ptr(text_ptr as *const c_char).to_string_lossy();
                if remaining.trim().is_empty() {
                    break;
                }
            }

            if all_phonemes.is_empty() {
                return Err("Failed to get phonemes from espeak-ng".into());
            }

            Ok(all_phonemes)
        }
    }
}

impl Drop for EspeakG2P {
    fn drop(&mut self) {
        // Note: We don't terminate espeak-ng here because it's a global resource
        // and other instances might still be using it
    }
}
