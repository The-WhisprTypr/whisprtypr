use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

#[derive(Debug)]
pub struct WhisperTranscriber {
    pub(crate) ctx: WhisperContext,
    pub(crate) language: String,
    pub(crate) model_id: String,
}

impl WhisperTranscriber {
    pub fn new(model_id: &str, model_path: &str, language: &str) -> Result<Self, String> {
        if !Path::new(model_path).exists() {
            return Err(format!("Model file not found: {}", model_path));
        }

        let mut ctx_params = WhisperContextParameters::default();
        ctx_params.flash_attn(true);

        let ctx = WhisperContext::new_with_params(model_path, ctx_params)
            .map_err(|e| format!("Failed to load Whisper model: {}", e))?;

        Ok(Self {
            ctx,
            language: language.to_string(),
            model_id: model_id.to_string(),
        })
    }

    pub fn transcribe(&self, audio_samples: &[f32]) -> Result<String, String> {
        if audio_samples.is_empty() {
            return Err("No audio samples to transcribe".to_string());
        }

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        if !self.language.is_empty() && self.language != "auto" {
            params.set_language(Some(&self.language));
        }

        params.set_translate(false);
        params.set_single_segment(true);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_print_special(false);
        params.set_token_timestamps(false);
        params.set_no_timestamps(true);
        params.set_no_context(true);
        params.set_suppress_blank(true);
        params.set_suppress_nst(true);
        params.set_max_tokens(64);
        params.set_audio_ctx(0);

        let num_threads = std::thread::available_parallelism()
            .map(|p| p.get() as i32)
            .unwrap_or(4);
        params.set_n_threads(num_threads);
        params.set_entropy_thold(2.8);
        params.set_temperature(0.0);
        params.set_temperature_inc(0.0);
        params.set_length_penalty(1.0);

        let mut state = self
            .ctx
            .create_state()
            .map_err(|e| format!("Failed to create Whisper state: {}", e))?;

        state
            .full(params, audio_samples)
            .map_err(|e| format!("Transcription failed: {}", e))?;

        let num_segments = state.full_n_segments();
        let mut result = String::with_capacity((num_segments as usize).saturating_mul(128));
        for i in 0..num_segments {
            if let Some(segment) = state.get_segment(i) {
                let text = segment
                    .to_str_lossy()
                    .map_err(|e| format!("Failed to get segment text: {}", e))?;
                let text = text.trim();
                if !text.is_empty() {
                    if !result.is_empty() {
                        result.push(' ');
                    }
                    result.push_str(text);
                }
            }
        }

        Ok(result)
    }

    pub fn set_language(&mut self, language: &str) {
        self.language = language.to_string();
    }
}
