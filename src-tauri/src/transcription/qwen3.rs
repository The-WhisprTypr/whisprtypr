use qwen3_asr::{best_device, AsrInference, TranscribeOptions};
use std::path::Path;

pub struct Qwen3AsrTranscriber {
    pub(crate) engine: AsrInference,
    pub(crate) language: String,
    pub(crate) model_id: String,
}

pub fn qwen3_language_name(code: &str) -> &'static str {
    match code {
        "zh" => "chinese",
        "en" => "english",
        "yue" => "cantonese",
        "ar" => "arabic",
        "de" => "german",
        "fr" => "french",
        "es" => "spanish",
        "pt" => "portuguese",
        "id" => "indonesian",
        "it" => "italian",
        "ko" => "korean",
        "ru" => "russian",
        "th" => "thai",
        "vi" => "vietnamese",
        "ja" => "japanese",
        "tr" => "turkish",
        "hi" => "hindi",
        "ms" => "malay",
        "nl" => "dutch",
        "sv" => "swedish",
        "da" => "danish",
        "fi" => "finnish",
        "pl" => "polish",
        "cs" => "czech",
        "fil" => "filipino",
        "fa" => "persian",
        "el" => "greek",
        "hu" => "hungarian",
        "mk" => "macedonian",
        "ro" => "romanian",
        _ => "english",
    }
}

impl Qwen3AsrTranscriber {
    pub fn new(model_id: &str, model_path: &str, language: &str) -> Result<Self, String> {
        let model_dir = Path::new(model_path);
        if !model_dir.is_dir() {
            return Err(format!(
                "Qwen3-ASR model directory not found: {}",
                model_path
            ));
        }

        let device = best_device();
        let engine = AsrInference::load(model_dir, device)
            .map_err(|e| format!("Failed to load Qwen3-ASR model: {}", e))?;

        Ok(Self {
            engine,
            language: language.to_string(),
            model_id: model_id.to_string(),
        })
    }

    pub fn transcribe(&self, audio_samples: &[f32]) -> Result<String, String> {
        if audio_samples.is_empty() {
            return Err("No audio samples to transcribe".to_string());
        }

        let mut options = TranscribeOptions::default();
        if !self.language.is_empty() && self.language != "auto" {
            options = options.with_language(qwen3_language_name(&self.language));
        }

        let result = self
            .engine
            .transcribe_samples(audio_samples, options)
            .map_err(|e| format!("Qwen3-ASR transcription failed: {}", e))?;

        Ok(result.text)
    }

    pub fn set_language(&mut self, language: &str) {
        self.language = language.to_string();
    }
}
