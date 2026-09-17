pub mod models;
pub mod parakeet;
pub mod qwen3;
pub mod whisper;

pub use models::{
    get_model_filename, get_model_url, get_parakeet_files, get_qwen3_asr_files, ParakeetFile,
};
pub use parakeet::configure_ort_acceleration;
pub use parakeet::ParakeetTranscriber;
pub use qwen3::qwen3_language_name;
pub use whisper::WhisperTranscriber;

use qwen3::Qwen3AsrTranscriber;

pub enum Transcriber {
    Whisper(WhisperTranscriber),
    Parakeet(ParakeetTranscriber),
    Qwen3Asr(Box<Qwen3AsrTranscriber>),
}

impl Transcriber {
    pub fn new(model_id: &str, model_path: &str, language: &str) -> Result<Self, String> {
        if model_id.starts_with("qwen3-asr-") {
            Ok(Self::Qwen3Asr(Box::new(Qwen3AsrTranscriber::new(
                model_id, model_path, language,
            )?)))
        } else if model_id.starts_with("parakeet-") {
            Ok(Self::Parakeet(ParakeetTranscriber::new(
                model_id, model_path, language,
            )?))
        } else {
            Ok(Self::Whisper(WhisperTranscriber::new(
                model_id, model_path, language,
            )?))
        }
    }

    pub fn transcribe(&mut self, audio_samples: &[f32]) -> Result<String, String> {
        match self {
            Self::Whisper(transcriber) => transcriber.transcribe(audio_samples),
            Self::Parakeet(transcriber) => transcriber.transcribe(audio_samples),
            Self::Qwen3Asr(transcriber) => transcriber.transcribe(audio_samples),
        }
    }

    pub fn warm_up(&mut self) -> Result<(), String> {
        let dummy: Vec<f32> = vec![0.0; 16_000];
        match self {
            Self::Whisper(transcriber) => transcriber.transcribe(&dummy).map(|_| ()),
            Self::Parakeet(transcriber) => transcriber.transcribe(&dummy).map(|_| ()),
            Self::Qwen3Asr(transcriber) => transcriber.transcribe(&dummy).map(|_| ()),
        }
    }

    pub fn set_language(&mut self, language: &str) {
        match self {
            Self::Whisper(transcriber) => transcriber.set_language(language),
            Self::Parakeet(transcriber) => transcriber.set_language(language),
            Self::Qwen3Asr(transcriber) => transcriber.set_language(language),
        }
    }

    pub fn language(&self) -> &str {
        match self {
            Self::Whisper(t) => &t.language,
            Self::Parakeet(t) => &t.language,
            Self::Qwen3Asr(t) => &t.language,
        }
    }

    pub fn model_id(&self) -> &str {
        match self {
            Self::Whisper(t) => &t.model_id,
            Self::Parakeet(t) => &t.model_id,
            Self::Qwen3Asr(t) => &t.model_id,
        }
    }
}
