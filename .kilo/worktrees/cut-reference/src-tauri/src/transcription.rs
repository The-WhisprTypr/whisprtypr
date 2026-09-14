use qwen3_asr::{best_device, AsrInference, TranscribeOptions};
use std::path::Path;
use transcribe_rs::onnx::parakeet::{ParakeetModel, ParakeetParams, TimestampGranularity};
use transcribe_rs::onnx::Quantization;
use transcribe_rs::TranscriptionResult;
use transcribe_rs::{set_ort_accelerator, OrtAccelerator};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub enum Transcriber {
    Whisper(WhisperTranscriber),
    Parakeet(ParakeetTranscriber),
    Qwen3Asr(Box<Qwen3AsrTranscriber>),
}

impl Transcriber {
    pub fn new(model_id: &str, model_path: &str, language: &str) -> Result<Self, String> {
        if model_id.starts_with("qwen3-asr-") {
            Ok(Self::Qwen3Asr(Box::new(Qwen3AsrTranscriber::new(
                model_path, language,
            )?)))
        } else if model_id.starts_with("parakeet-") {
            Ok(Self::Parakeet(ParakeetTranscriber::new(
                model_path, language,
            )?))
        } else {
            Ok(Self::Whisper(WhisperTranscriber::new(
                model_path, language,
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

    pub fn set_language(&mut self, language: &str) {
        match self {
            Self::Whisper(transcriber) => transcriber.set_language(language),
            Self::Parakeet(transcriber) => transcriber.set_language(language),
            Self::Qwen3Asr(transcriber) => transcriber.set_language(language),
        }
    }
}

pub struct Qwen3AsrTranscriber {
    engine: AsrInference,
    language: String,
}

impl Qwen3AsrTranscriber {
    pub fn new(model_path: &str, language: &str) -> Result<Self, String> {
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

pub struct WhisperTranscriber {
    ctx: WhisperContext,
    language: String,
}

impl WhisperTranscriber {
    pub fn new(model_path: &str, language: &str) -> Result<Self, String> {
        if !Path::new(model_path).exists() {
            return Err(format!("Model file not found: {}", model_path));
        }

        // Configure context parameters for maximum speed
        let mut ctx_params = WhisperContextParameters::default();

        // Enable flash attention for faster inference (CPU-only optimization)
        // Flash attention reduces memory bandwidth and speeds up attention computation
        ctx_params.flash_attn(true);

        // GPU is handled via compile-time features (metal on macOS, cuda/vulkan on others)
        // The default will use GPU if the feature is enabled

        let ctx = WhisperContext::new_with_params(model_path, ctx_params)
            .map_err(|e| format!("Failed to load Whisper model: {}", e))?;

        Ok(Self {
            ctx,
            language: language.to_string(),
        })
    }

    pub fn transcribe(&self, audio_samples: &[f32]) -> Result<String, String> {
        if audio_samples.is_empty() {
            return Err("No audio samples to transcribe".to_string());
        }

        // Use Greedy decoding for fastest results
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        // Set language (empty string = auto-detect)
        if !self.language.is_empty() && self.language != "auto" {
            params.set_language(Some(&self.language));
        }

        // Disable translation, we want transcription
        params.set_translate(false);

        // ========== AGGRESSIVE SPEED OPTIMIZATIONS ==========

        // Single segment mode - fastest for voice input (< 30 seconds)
        params.set_single_segment(true);

        // Disable ALL output printing for maximum speed
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_print_special(false);

        // Disable ALL timestamps - not needed for text output
        params.set_token_timestamps(false);
        params.set_no_timestamps(true);

        // Disable context - each utterance is independent
        params.set_no_context(true);

        // Suppress non-speech tokens for cleaner, faster output
        params.set_suppress_blank(true);
        params.set_suppress_nst(true);

        // Reduced max tokens - voice input is typically short
        params.set_max_tokens(64);

        // Audio context 0 = use default from model (fastest)
        params.set_audio_ctx(0);

        // Use all available CPU cores for parallel inference
        let num_threads = std::thread::available_parallelism()
            .map(|p| p.get() as i32)
            .unwrap_or(4);
        params.set_n_threads(num_threads);

        // Higher entropy threshold = faster decoding, slightly less accuracy
        params.set_entropy_thold(2.8);

        // Temperature 0 = greedy decoding (fastest, deterministic)
        params.set_temperature(0.0);

        // Disable beam search fallback - stick with greedy for speed
        params.set_temperature_inc(0.0);

        // Speed penalty - prefer shorter sequences (faster decoding)
        params.set_length_penalty(1.0);

        // Create state for this transcription
        let mut state = self
            .ctx
            .create_state()
            .map_err(|e| format!("Failed to create Whisper state: {}", e))?;

        // Run inference
        state
            .full(params, audio_samples)
            .map_err(|e| format!("Transcription failed: {}", e))?;

        // Collect all segments efficiently
        let num_segments = state.full_n_segments();

        // Pre-allocate string capacity for typical transcription length
        // Average word is ~5 chars, so 128 chars is a reasonable estimate
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

pub struct ParakeetTranscriber {
    model: ParakeetModel,
    language: String,
}

impl ParakeetTranscriber {
    pub fn new(model_path: &str, language: &str) -> Result<Self, String> {
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

fn configure_ort_acceleration() {
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
        // DirectML is fast when it works, but on some Windows GPU/driver
        // combinations it can hard-fail during ONNX MemcpyToHost nodes. CPU is
        // the safer production default; advanced users can opt into DirectML
        // with WHISPRTYPR_ORT_ACCELERATOR=directml.
        OrtAccelerator::CpuOnly
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

fn qwen3_language_name(code: &str) -> &'static str {
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

// Model download URLs (Hugging Face)
pub fn get_model_url(model_id: &str) -> Option<String> {
    let base = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main";
    let distil_base = "https://huggingface.co/distil-whisper";

    match model_id {
        // Standard Whisper models (multilingual)
        "tiny" => Some(format!("{}/ggml-tiny.bin", base)),
        "base" => Some(format!("{}/ggml-base.bin", base)),
        "small" => Some(format!("{}/ggml-small.bin", base)),
        "medium" => Some(format!("{}/ggml-medium.bin", base)),
        "large-v2" => Some(format!("{}/ggml-large-v2.bin", base)),
        "large-v3" => Some(format!("{}/ggml-large-v3.bin", base)),
        "large-v3-turbo" => Some(format!("{}/ggml-large-v3-turbo.bin", base)),

        // English-only Whisper models (faster, optimized for English)
        "tiny.en" => Some(format!("{}/ggml-tiny.en.bin", base)),
        "base.en" => Some(format!("{}/ggml-base.en.bin", base)),
        "small.en" => Some(format!("{}/ggml-small.en.bin", base)),
        "medium.en" => Some(format!("{}/ggml-medium.en.bin", base)),

        // Distil-Whisper models (6x faster, similar accuracy)
        "distil-small.en" => Some(format!(
            "{}/distil-small.en/resolve/main/ggml-distil-small.en.bin",
            distil_base
        )),

        // Legacy (for backwards compatibility)
        "large" => Some(format!("{}/ggml-large-v3.bin", base)),

        _ => None,
    }
}

pub struct ParakeetFile {
    pub filename: &'static str,
    pub url: &'static str,
}

pub fn get_parakeet_files(model_id: &str) -> Option<&'static [ParakeetFile]> {
    const V3_FILES: &[ParakeetFile] = &[
        ParakeetFile {
            filename: "encoder-model.int8.onnx",
            url: concat!(
                "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main",
                "/encoder-model.int8.onnx"
            ),
        },
        ParakeetFile {
            filename: "decoder_joint-model.int8.onnx",
            url: concat!(
                "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main",
                "/decoder_joint-model.int8.onnx"
            ),
        },
        ParakeetFile {
            filename: "nemo128.onnx",
            url: concat!(
                "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main",
                "/nemo128.onnx"
            ),
        },
        ParakeetFile {
            filename: "vocab.txt",
            url: concat!(
                "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main",
                "/vocab.txt"
            ),
        },
    ];

    const V2_FILES: &[ParakeetFile] = &[
        ParakeetFile {
            filename: "encoder-model.int8.onnx",
            url: concat!(
                "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v2-onnx/resolve/main",
                "/encoder-model.int8.onnx"
            ),
        },
        ParakeetFile {
            filename: "decoder_joint-model.int8.onnx",
            url: concat!(
                "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v2-onnx/resolve/main",
                "/decoder_joint-model.int8.onnx"
            ),
        },
        ParakeetFile {
            filename: "nemo128.onnx",
            url: concat!(
                "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v2-onnx/resolve/main",
                "/nemo128.onnx"
            ),
        },
        ParakeetFile {
            filename: "vocab.txt",
            url: concat!(
                "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v2-onnx/resolve/main",
                "/vocab.txt"
            ),
        },
    ];

    match model_id {
        "parakeet-v3" => Some(V3_FILES),
        "parakeet-v2" => Some(V2_FILES),
        _ => None,
    }
}

pub fn get_qwen3_asr_files(model_id: &str) -> Option<&'static [ParakeetFile]> {
    const QWEN3_ASR_06B_FILES: &[ParakeetFile] = &[
        ParakeetFile {
            filename: "chat_template.json",
            url: "https://huggingface.co/Qwen/Qwen3-ASR-0.6B/resolve/main/chat_template.json",
        },
        ParakeetFile {
            filename: "config.json",
            url: "https://huggingface.co/Qwen/Qwen3-ASR-0.6B/resolve/main/config.json",
        },
        ParakeetFile {
            filename: "generation_config.json",
            url: "https://huggingface.co/Qwen/Qwen3-ASR-0.6B/resolve/main/generation_config.json",
        },
        ParakeetFile {
            filename: "merges.txt",
            url: "https://huggingface.co/Qwen/Qwen3-ASR-0.6B/resolve/main/merges.txt",
        },
        ParakeetFile {
            filename: "model.safetensors",
            url: "https://huggingface.co/Qwen/Qwen3-ASR-0.6B/resolve/main/model.safetensors",
        },
        ParakeetFile {
            filename: "preprocessor_config.json",
            url: "https://huggingface.co/Qwen/Qwen3-ASR-0.6B/resolve/main/preprocessor_config.json",
        },
        ParakeetFile {
            filename: "tokenizer_config.json",
            url: "https://huggingface.co/Qwen/Qwen3-ASR-0.6B/resolve/main/tokenizer_config.json",
        },
        ParakeetFile {
            filename: "vocab.json",
            url: "https://huggingface.co/Qwen/Qwen3-ASR-0.6B/resolve/main/vocab.json",
        },
    ];

    match model_id {
        "qwen3-asr-0.6b" => Some(QWEN3_ASR_06B_FILES),
        _ => None,
    }
}

pub fn get_model_filename(model_id: &str) -> String {
    match model_id {
        "qwen3-asr-0.6b" => "qwen3-asr-0.6b".to_string(),
        "parakeet-v3" => "parakeet-tdt-0.6b-v3-int8".to_string(),
        "parakeet-v2" => "parakeet-tdt-0.6b-v2-int8".to_string(),
        // Distil models have different naming
        "distil-small.en" => "ggml-distil-small.en.bin".to_string(),
        // English-only models
        "tiny.en" => "ggml-tiny.en.bin".to_string(),
        "base.en" => "ggml-base.en.bin".to_string(),
        "small.en" => "ggml-small.en.bin".to_string(),
        "medium.en" => "ggml-medium.en.bin".to_string(),
        // Standard models
        _ => format!("ggml-{}.bin", model_id),
    }
}
