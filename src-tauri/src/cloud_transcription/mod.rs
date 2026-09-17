pub mod models;
pub mod providers;
pub mod util;

pub use models::{
    CloudProviderInfo, DeepgramWord, DEFAULT_DEEPGRAM_BASE_URL, DEFAULT_GROQ_BASE_URL,
    DEFAULT_MISTRAL_BASE_URL, DEFAULT_OPENAI_BASE_URL,
};
pub use util::{
    build_http_client, decrypt_api_key, encode_samples_to_wav, encrypt_api_key, mask_key,
    parse_cloud_model_id,
};

use crate::database::Database;

pub async fn test_provider_connection(
    provider: &str,
    api_key: &str,
    base_url: Option<&str>,
) -> Result<String, String> {
    let key = api_key.trim();
    if key.is_empty() && provider != "custom" {
        return Err("API key is required".to_string());
    }

    match provider.to_lowercase().as_str() {
        "groq" => providers::groq::test_groq_connection(api_key, base_url).await,
        "openai" => providers::openai::test_openai_connection(api_key, base_url).await,
        "deepgram" => providers::deepgram::test_deepgram_connection(api_key, base_url).await,
        "mistral" => providers::mistral::test_mistral_connection(api_key, base_url).await,
        "custom" => providers::custom::test_custom_connection(api_key, base_url).await,
        _ => Err(format!("Unknown provider: {}", provider)),
    }
}

pub async fn transcribe_with_cloud(
    db: &Database,
    provider: &str,
    model: &str,
    wav_bytes: Vec<u8>,
    language: &str,
    enable_speaker_detection: bool,
) -> Result<String, String> {
    match provider.to_lowercase().as_str() {
        "groq" => {
            providers::groq::transcribe_with_groq(
                db,
                model,
                wav_bytes,
                language,
                enable_speaker_detection,
            )
            .await
        }
        "openai" => {
            providers::openai::transcribe_with_openai(
                db,
                model,
                wav_bytes,
                language,
                enable_speaker_detection,
            )
            .await
        }
        "deepgram" => {
            providers::deepgram::transcribe_with_deepgram(
                db,
                model,
                wav_bytes,
                language,
                enable_speaker_detection,
            )
            .await
        }
        "mistral" => {
            providers::mistral::transcribe_with_mistral(
                db,
                model,
                wav_bytes,
                language,
                enable_speaker_detection,
            )
            .await
        }
        "custom" => {
            providers::custom::transcribe_with_custom(
                db,
                model,
                wav_bytes,
                language,
                enable_speaker_detection,
            )
            .await
        }
        _ => Err(format!("Unsupported cloud provider: {}", provider)),
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CloudTranscriptionUrlOptions {
    pub provider: String,
    pub model: String,
    pub url: String,
    pub language: String,
    pub enable_speaker_detection: bool,
}

pub async fn transcribe_with_cloud_url(
    db: &Database,
    options: CloudTranscriptionUrlOptions,
) -> Result<String, String> {
    match options.provider.to_lowercase().as_str() {
        "deepgram" => {
            providers::deepgram::transcribe_with_deepgram_url(
                db,
                &options.model,
                &options.url,
                &options.language,
                options.enable_speaker_detection,
            )
            .await
        }
        _ => Err(format!(
            "Provider '{}' does not support direct URL transcription yet.",
            options.provider
        )),
    }
}

pub fn format_deepgram_speaker_transcript(words: &[DeepgramWord]) -> String {
    let mut segments: Vec<(u32, String)> = Vec::new();
    let mut current_speaker: Option<u32> = None;
    let mut current_text = String::new();

    for word in words {
        let speaker = word.speaker.unwrap_or(0);
        let w = word.word.trim();
        if w.is_empty() {
            continue;
        }

        if current_speaker != Some(speaker) {
            if !current_text.is_empty() {
                segments.push((current_speaker.unwrap_or(0), current_text));
            }
            current_speaker = Some(speaker);
            current_text = w.to_string();
        } else {
            if !current_text.is_empty() {
                current_text.push(' ');
            }
            current_text.push_str(w);
        }
    }

    if !current_text.is_empty() {
        segments.push((current_speaker.unwrap_or(0), current_text));
    }

    segments
        .into_iter()
        .map(|(speaker, text)| format!("[Speaker {}]: {}", speaker + 1, text))
        .collect::<Vec<_>>()
        .join("\n")
}
