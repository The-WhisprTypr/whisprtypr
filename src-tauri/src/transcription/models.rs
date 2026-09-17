use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParakeetFile {
    pub filename: &'static str,
    pub url: &'static str,
}

pub fn get_model_url(model_id: &str) -> Option<String> {
    let base = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main";
    let distil_base = "https://huggingface.co/distil-whisper";

    match model_id {
        "tiny" => Some(format!("{}/ggml-tiny.bin", base)),
        "base" => Some(format!("{}/ggml-base.bin", base)),
        "small" => Some(format!("{}/ggml-small.bin", base)),
        "medium" => Some(format!("{}/ggml-medium.bin", base)),
        "large-v2" => Some(format!("{}/ggml-large-v2.bin", base)),
        "large-v3" => Some(format!("{}/ggml-large-v3.bin", base)),
        "large-v3-turbo" => Some(format!("{}/ggml-large-v3-turbo.bin", base)),

        "tiny.en" => Some(format!("{}/ggml-tiny.en.bin", base)),
        "base.en" => Some(format!("{}/ggml-base.en.bin", base)),
        "small.en" => Some(format!("{}/ggml-small.en.bin", base)),
        "medium.en" => Some(format!("{}/ggml-medium.en.bin", base)),

        "distil-small.en" => Some(format!(
            "{}/distil-small.en/resolve/main/ggml-distil-small.en.bin",
            distil_base
        )),

        "large" => Some(format!("{}/ggml-large-v3.bin", base)),

        _ => None,
    }
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
        "distil-small.en" => "ggml-distil-small.en.bin".to_string(),
        "tiny.en" => "ggml-tiny.en.bin".to_string(),
        "base.en" => "ggml-base.en.bin".to_string(),
        "small.en" => "ggml-small.en.bin".to_string(),
        "medium.en" => "ggml-medium.en.bin".to_string(),
        _ => format!("ggml-{}.bin", model_id),
    }
}
