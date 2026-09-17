use std::path::Path;
use transcribe_rs::onnx::parakeet::{ParakeetModel, ParakeetParams, TimestampGranularity};
use transcribe_rs::onnx::Quantization;
use transcribe_rs::TranscriptionResult;
use transcribe_rs::{set_ort_accelerator, OrtAccelerator};

pub fn configure_ort_acceleration() {
    let accelerator = std::env::var("WHISPRTYPR_ORT_ACCELERATOR")
        .ok()
        .and_then(|value| value.parse::<OrtAccelerator>().ok())
        .unwrap_or_else(default_ort_accelerator);

    set_ort_accelerator(accelerator);
    log::info!(
        "Using ONNX Runtime accelerator preference: {} (compiled: {:?})",
        accelerator,
        OrtAccelerator::available()
    );
}

fn default_ort_accelerator() -> OrtAccelerator {
    #[cfg(target_os = "windows")]
    {
        OrtAccelerator::DirectMl
    }

    #[cfg(target_os = "macos")]
    {
        OrtAccelerator::CoreMl
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        OrtAccelerator::Auto
    }
}

pub struct ParakeetTranscriber {
    pub(crate) model: ParakeetModel,
    pub(crate) language: String,
    pub(crate) model_id: String,
}

impl ParakeetTranscriber {
    pub fn new(model_id: &str, model_path: &str, language: &str) -> Result<Self, String> {
        configure_ort_acceleration();

        let model_dir = Path::new(model_path);
        if !model_dir.is_dir() {
            return Err(format!(
                "Parakeet model directory not found: {}",
                model_path
            ));
        }

        let model = ParakeetModel::load(model_dir, &Quantization::Int8)
            .map_err(|e| format!("Failed to load Parakeet transcription model: {}", e))?;

        Ok(Self {
            model,
            language: language.to_string(),
            model_id: model_id.to_string(),
        })
    }

    pub fn transcribe(&mut self, audio_samples: &[f32]) -> Result<String, String> {
        if audio_samples.is_empty() {
            return Err("No audio samples to transcribe".to_string());
        }

        let result: TranscriptionResult = self
            .model
            .transcribe_with(
                audio_samples,
                &ParakeetParams {
                    language: if self.language == "auto" {
                        None
                    } else {
                        Some(self.language.clone())
                    },
                    timestamp_granularity: Some(TimestampGranularity::Segment),
                },
            )
            .map_err(|e| format!("Parakeet transcription failed: {}", e))?;

        Ok(result.text)
    }

    pub fn set_language(&mut self, language: &str) {
        self.language = language.to_string();
    }
}
