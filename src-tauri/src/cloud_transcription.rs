use crate::database::Database;
use crate::license::get_device_id;
use crate::security::{derive_encryption_key, encrypt_data, decrypt_data};
use reqwest::multipart::{Form, Part};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::time::Duration;

pub const DEFAULT_GROQ_BASE_URL: &str = "https://api.groq.com/openai/v1";
pub const DEFAULT_OPENAI_BASE_URL: &str = "https://api.openai.com/v1";
pub const DEFAULT_DEEPGRAM_BASE_URL: &str = "https://api.deepgram.com/v1";
pub const DEFAULT_MISTRAL_BASE_URL: &str = "https://api.mistral.ai/v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudProviderInfo {
    pub id: String,
    pub name: String,
    pub configured: bool,
    pub masked_key: String,
    pub base_url: Option<String>,
    pub custom_model: Option<String>,
}

/// Encode raw mono f32 samples at 16kHz into in-memory 16-bit PCM WAV bytes
pub fn encode_samples_to_wav(samples: &[f32]) -> Result<Vec<u8>, String> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut cursor = Cursor::new(Vec::with_capacity(44 + samples.len() * 2));
    {
        let mut writer = hound::WavWriter::new(&mut cursor, spec)
            .map_err(|e| format!("Failed to create in-memory WAV writer: {}", e))?;

        for &sample in samples {
            let clamped = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
            writer
                .write_sample(clamped)
                .map_err(|e| format!("Failed to write audio sample: {}", e))?;
        }

        writer
            .finalize()
            .map_err(|e| format!("Failed to finalize in-memory WAV: {}", e))?;
    }

    Ok(cursor.into_inner())
}

/// Helper to encrypt an API key using the device ID fingerprint
pub fn encrypt_api_key(api_key: &str) -> Result<String, String> {
    if api_key.trim().is_empty() {
        return Ok(String::new());
    }
    let device_id = get_device_id();
    let enc_key = derive_encryption_key(&device_id);
    let encrypted = encrypt_data(api_key.as_bytes(), &enc_key)?;
    Ok(hex::encode(encrypted))
}

/// Helper to decrypt an API key. Falls back to raw string if not hex/encrypted.
pub fn decrypt_api_key(stored_key: &str) -> String {
    let trimmed = stored_key.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    if let Ok(bytes) = hex::decode(trimmed) {
        let device_id = get_device_id();
        let enc_key = derive_encryption_key(&device_id);
        if let Ok(decrypted) = decrypt_data(&bytes, &enc_key) {
            if let Ok(s) = String::from_utf8(decrypted) {
                return s;
            }
        }
    }

    // Fallback if key was stored plaintext
    trimmed.to_string()
}

/// Mask an API key for safe UI display (e.g. "gsk_...4f2a")
pub fn mask_key(key: &str) -> String {
    let key = key.trim();
    if key.is_empty() {
        return String::new();
    }
    if key.len() <= 8 {
        return "••••••••".to_string();
    }
    let prefix = &key[..std::cmp::min(4, key.len())];
    let suffix = &key[key.len() - std::cmp::min(4, key.len())..];
    format!("{}••••{}", prefix, suffix)
}

/// Parse a cloud model identifier: "cloud:{provider}:{model}" -> (provider, model)
pub fn parse_cloud_model_id(model_id: &str) -> Option<(&str, &str)> {
    if let Some(rest) = model_id.strip_prefix("cloud:") {
        if let Some((provider, model)) = rest.split_once(':') {
            return Some((provider, model));
        }
    }
    None
}

/// Build an HTTP client with sensible timeouts
pub fn build_http_client(timeout_secs: u64) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))
}

/// Test connection to a cloud provider
pub async fn test_provider_connection(
    provider: &str,
    api_key: &str,
    base_url: Option<&str>,
) -> Result<String, String> {
    let key = api_key.trim();
    if key.is_empty() && provider != "custom" {
        return Err("API key is required".to_string());
    }

    let client = build_http_client(10)?;

    match provider.to_lowercase().as_str() {
        "groq" => {
            let url = format!(
                "{}/models",
                base_url.unwrap_or(DEFAULT_GROQ_BASE_URL).trim_end_matches('/')
            );
            let resp = client
                .get(&url)
                .header("Authorization", format!("Bearer {}", key))
                .send()
                .await
                .map_err(|e| format!("Failed to reach Groq API: {}", e))?;

            if resp.status().is_success() {
                Ok("Connected successfully! Groq API key is valid.".to_string())
            } else if resp.status().as_u16() == 401 {
                Err("Authentication failed: Invalid Groq API key (401 Unauthorized).".to_string())
            } else {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                Err(format!("Groq API error ({}): {}", status, text))
            }
        }
        "openai" => {
            let url = format!(
                "{}/models",
                base_url.unwrap_or(DEFAULT_OPENAI_BASE_URL).trim_end_matches('/')
            );
            let resp = client
                .get(&url)
                .header("Authorization", format!("Bearer {}", key))
                .send()
                .await
                .map_err(|e| format!("Failed to reach OpenAI API: {}", e))?;

            if resp.status().is_success() {
                Ok("Connected successfully! OpenAI API key is valid.".to_string())
            } else if resp.status().as_u16() == 401 {
                Err("Authentication failed: Invalid OpenAI API key (401 Unauthorized).".to_string())
            } else {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                Err(format!("OpenAI API error ({}): {}", status, text))
            }
        }
        "deepgram" => {
            let url = format!(
                "{}/projects",
                base_url.unwrap_or(DEFAULT_DEEPGRAM_BASE_URL).trim_end_matches('/')
            );
            let resp = client
                .get(&url)
                .header("Authorization", format!("Token {}", key))
                .send()
                .await
                .map_err(|e| format!("Failed to reach Deepgram API: {}", e))?;

            if resp.status().is_success() {
                Ok("Connected successfully! Deepgram API key is valid.".to_string())
            } else if resp.status().as_u16() == 401 {
                Err("Authentication failed: Invalid Deepgram API key (401 Unauthorized).".to_string())
            } else {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                Err(format!("Deepgram API error ({}): {}", status, text))
            }
        }
        "mistral" => {
            let url = format!(
                "{}/models",
                base_url.unwrap_or(DEFAULT_MISTRAL_BASE_URL).trim_end_matches('/')
            );
            let resp = client
                .get(&url)
                .header("Authorization", format!("Bearer {}", key))
                .send()
                .await
                .map_err(|e| format!("Failed to reach Mistral API: {}", e))?;

            if resp.status().is_success() {
                Ok("Connected successfully! Mistral API key is valid.".to_string())
            } else if resp.status().as_u16() == 401 {
                Err("Authentication failed: Invalid Mistral API key (401 Unauthorized).".to_string())
            } else {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                Err(format!("Mistral API error ({}): {}", status, text))
            }
        }
        "custom" => {
            let base = base_url.unwrap_or("http://localhost:8000/v1").trim_end_matches('/');
            let url = if base.ends_with("/audio/transcriptions") {
                base.to_string()
            } else {
                format!("{}/models", base)
            };

            let mut req = client.get(&url);
            if !key.is_empty() {
                req = req.header("Authorization", format!("Bearer {}", key));
            }
            let resp = req
                .send()
                .await
                .map_err(|e| format!("Failed to reach custom endpoint at '{}': {}", url, e))?;

            if resp.status().is_success() || resp.status().as_u16() == 405 {
                Ok("Connected successfully to custom endpoint!".to_string())
            } else {
                let status = resp.status();
                Err(format!("Custom endpoint returned HTTP {}.", status))
            }
        }
        _ => Err(format!("Unknown provider: {}", provider)),
    }
}

/// Transcribe audio bytes using a configured cloud provider
pub async fn transcribe_with_cloud(
    db: &Database,
    provider: &str,
    model: &str,
    wav_bytes: Vec<u8>,
    language: &str,
    enable_speaker_detection: bool,
) -> Result<String, String> {
    let provider_record = db
        .get_cloud_provider(provider)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| format!("Cloud provider '{}' is not configured. Please add an API key in Models settings.", provider))?;

    let api_key = decrypt_api_key(&provider_record.api_key);
    if api_key.trim().is_empty() && provider != "custom" {
        return Err(format!(
            "API key for '{}' is missing. Please configure it in Models.",
            provider
        ));
    }

    let client = build_http_client(60)?;

    match provider.to_lowercase().as_str() {
        "groq" => {
            let base = provider_record
                .base_url
                .as_deref()
                .unwrap_or(DEFAULT_GROQ_BASE_URL)
                .trim_end_matches('/');
            let url = format!("{}/audio/transcriptions", base);

            let mut form = Form::new()
                .part(
                    "file",
                    Part::bytes(wav_bytes)
                        .file_name("audio.wav")
                        .mime_str("audio/wav")
                        .map_err(|e| e.to_string())?,
                )
                .text("model", model.to_string())
                .text("response_format", "json");

            if !language.is_empty() && language != "auto" {
                form = form.text("language", language.to_string());
            }

            let resp = client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .multipart(form)
                .send()
                .await
                .map_err(|e| format!("Groq request failed: {}", e))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let err_text = resp.text().await.unwrap_or_default();
                return Err(format!("Groq transcription failed ({}): {}", status, err_text));
            }

            #[derive(Deserialize)]
            struct OpenAiResponse {
                text: String,
            }

            let json: OpenAiResponse = resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse Groq response: {}", e))?;

            Ok(json.text.trim().to_string())
        }

        "openai" => {
            let base = provider_record
                .base_url
                .as_deref()
                .unwrap_or(DEFAULT_OPENAI_BASE_URL)
                .trim_end_matches('/');
            let url = format!("{}/audio/transcriptions", base);

            let mut form = Form::new()
                .part(
                    "file",
                    Part::bytes(wav_bytes)
                        .file_name("audio.wav")
                        .mime_str("audio/wav")
                        .map_err(|e| e.to_string())?,
                )
                .text("model", model.to_string())
                .text("response_format", "json");

            if !language.is_empty() && language != "auto" {
                form = form.text("language", language.to_string());
            }

            let resp = client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .multipart(form)
                .send()
                .await
                .map_err(|e| format!("OpenAI request failed: {}", e))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let err_text = resp.text().await.unwrap_or_default();
                return Err(format!("OpenAI transcription failed ({}): {}", status, err_text));
            }

            #[derive(Deserialize)]
            struct OpenAiResponse {
                text: String,
            }

            let json: OpenAiResponse = resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse OpenAI response: {}", e))?;

            Ok(json.text.trim().to_string())
        }

        "deepgram" => {
            let base = provider_record
                .base_url
                .as_deref()
                .unwrap_or(DEFAULT_DEEPGRAM_BASE_URL)
                .trim_end_matches('/');
            let mut url = format!("{}/listen?model={}&smart_format=true&punctuate=true", base, model);
            if !language.is_empty() && language != "auto" {
                url.push_str(&format!("&language={}", language));
            }
            if enable_speaker_detection {
                url.push_str("&diarize=true&diarize_version=2");
            }

            let resp = client
                .post(&url)
                .header("Authorization", format!("Token {}", api_key))
                .header("Content-Type", "audio/wav")
                .body(wav_bytes)
                .send()
                .await
                .map_err(|e| format!("Deepgram request failed: {}", e))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let err_text = resp.text().await.unwrap_or_default();
                return Err(format!("Deepgram transcription failed ({}): {}", status, err_text));
            }

            #[derive(Deserialize)]
            struct DeepgramAlternative {
                transcript: String,
                words: Option<Vec<DeepgramWord>>,
            }
            #[derive(Deserialize)]
            struct DeepgramChannel {
                alternatives: Vec<DeepgramAlternative>,
            }
            #[derive(Deserialize)]
            struct DeepgramResults {
                channels: Vec<DeepgramChannel>,
            }
            #[derive(Deserialize)]
            struct DeepgramResponse {
                results: DeepgramResults,
            }

            let json: DeepgramResponse = resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse Deepgram response: {}", e))?;

            let alternative = json
                .results
                .channels
                .first()
                .and_then(|c| c.alternatives.first());

            let transcript = if enable_speaker_detection {
                if let Some(alt) = alternative {
                    if let Some(words) = &alt.words {
                        format_deepgram_speaker_transcript(words)
                    } else {
                        alt.transcript.trim().to_string()
                    }
                } else {
                    String::new()
                }
            } else {
                alternative
                    .map(|a| a.transcript.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string()
            };

            Ok(transcript)
        }

        "mistral" => {
            let base = provider_record
                .base_url
                .as_deref()
                .unwrap_or(DEFAULT_MISTRAL_BASE_URL)
                .trim_end_matches('/');
            let url = format!("{}/audio/transcriptions", base);

            let mut form = Form::new()
                .part(
                    "file",
                    Part::bytes(wav_bytes)
                        .file_name("audio.wav")
                        .mime_str("audio/wav")
                        .map_err(|e| e.to_string())?,
                )
                .text("model", model.to_string());

            if !language.is_empty() && language != "auto" {
                form = form.text("language", language.to_string());
            }

            let resp = client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .multipart(form)
                .send()
                .await
                .map_err(|e| format!("Mistral request failed: {}", e))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let err_text = resp.text().await.unwrap_or_default();
                return Err(format!("Mistral transcription failed ({}): {}", status, err_text));
            }

            #[derive(Deserialize)]
            struct MistralResponse {
                text: String,
            }

            let json: MistralResponse = resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse Mistral response: {}", e))?;

            Ok(json.text.trim().to_string())
        }

        "custom" => {
            let base = provider_record
                .base_url
                .as_deref()
                .unwrap_or("http://localhost:8000/v1")
                .trim_end_matches('/');
            let url = if base.ends_with("/audio/transcriptions") {
                base.to_string()
            } else {
                format!("{}/audio/transcriptions", base)
            };

            let custom_model_name = provider_record
                .custom_model
                .as_deref()
                .filter(|m| !m.trim().is_empty())
                .unwrap_or(model);

            let mut form = Form::new()
                .part(
                    "file",
                    Part::bytes(wav_bytes)
                        .file_name("audio.wav")
                        .mime_str("audio/wav")
                        .map_err(|e| e.to_string())?,
                )
                .text("model", custom_model_name.to_string())
                .text("response_format", "json");

            if !language.is_empty() && language != "auto" {
                form = form.text("language", language.to_string());
            }

            let mut req = client.post(&url).multipart(form);
            if !api_key.is_empty() {
                req = req.header("Authorization", format!("Bearer {}", api_key));
            }

            let resp = req
                .send()
                .await
                .map_err(|e| format!("Custom endpoint request failed: {}", e))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let err_text = resp.text().await.unwrap_or_default();
                return Err(format!("Custom endpoint transcription failed ({}): {}", status, err_text));
            }

            #[derive(Deserialize)]
            struct OpenAiResponse {
                text: String,
            }

            let json: OpenAiResponse = resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse response from custom endpoint: {}", e))?;

            Ok(json.text.trim().to_string())
        }

        _ => Err(format!("Unsupported cloud provider: {}", provider)),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudTranscriptionUrlOptions {
    pub provider: String,
    pub model: String,
    pub url: String,
    pub language: String,
    pub enable_speaker_detection: bool,
}

/// Transcribe from a URL using a configured cloud provider.
///
/// Currently only Deepgram supports direct URL transcription.
pub async fn transcribe_with_cloud_url(
    db: &Database,
    options: CloudTranscriptionUrlOptions,
) -> Result<String, String> {
    let provider_record = db
        .get_cloud_provider(&options.provider)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| format!("Cloud provider '{}' is not configured.", options.provider))?;

    let api_key = decrypt_api_key(&provider_record.api_key);
    if api_key.trim().is_empty() && options.provider != "custom" {
        return Err(format!(
            "API key for '{}' is missing. Please configure it in Models.",
            options.provider
        ));
    }

    let client = build_http_client(60)?;

    match options.provider.to_lowercase().as_str() {
        "deepgram" => {
            let base = provider_record
                .base_url
                .as_deref()
                .unwrap_or(DEFAULT_DEEPGRAM_BASE_URL)
                .trim_end_matches('/');
            let mut url = format!(
                "{}/listen?url={}&model={}&smart_format=true&punctuate=true",
                base,
                urlencoding::encode(&options.url),
                options.model
            );
            if !options.language.is_empty() && options.language != "auto" {
                url.push_str(&format!("&language={}", options.language));
            }
            if options.enable_speaker_detection {
                url.push_str("&diarize=true&diarize_version=2");
            }

            let resp = client
                .get(&url)
                .header("Authorization", format!("Token {}", api_key))
                .send()
                .await
                .map_err(|e| format!("Deepgram request failed: {}", e))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let err_text = resp.text().await.unwrap_or_default();
                return Err(format!("Deepgram transcription failed ({}): {}", status, err_text));
            }

            #[derive(Deserialize)]
            struct DeepgramAlternative {
                transcript: String,
                words: Option<Vec<DeepgramWord>>,
            }
            #[derive(Deserialize)]
            struct DeepgramChannel {
                alternatives: Vec<DeepgramAlternative>,
            }
            #[derive(Deserialize)]
            struct DeepgramResults {
                channels: Vec<DeepgramChannel>,
            }
            #[derive(Deserialize)]
            struct DeepgramResponse {
                results: DeepgramResults,
            }

            let json: DeepgramResponse = resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse Deepgram response: {}", e))?;

            let alternative = json
                .results
                .channels
                .first()
                .and_then(|c| c.alternatives.first());

            let transcript = if options.enable_speaker_detection {
                if let Some(alt) = alternative {
                    if let Some(words) = &alt.words {
                        format_deepgram_speaker_transcript(words)
                    } else {
                        alt.transcript.trim().to_string()
                    }
                } else {
                    String::new()
                }
            } else {
                alternative
                    .map(|a| a.transcript.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string()
            };

            Ok(transcript)
        }

        _ => Err(format!(
            "Provider '{}' does not support direct URL transcription yet.",
            options.provider
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct DeepgramWord {
    pub speaker: Option<u32>,
    pub word: String,
}

/// Format Deepgram diarized words into speaker-segmented transcript lines
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_samples_to_wav() {
        let samples = vec![0.0f32; 16000]; // 1 second of silence
        let wav = encode_samples_to_wav(&samples).expect("Encoding should succeed");
        // Check RIFF header
        assert!(wav.len() > 44);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
    }

    #[test]
    fn test_parse_cloud_model_id() {
        assert_eq!(
            parse_cloud_model_id("cloud:groq:whisper-large-v3"),
            Some(("groq", "whisper-large-v3"))
        );
        assert_eq!(
            parse_cloud_model_id("cloud:openai:whisper-1"),
            Some(("openai", "whisper-1"))
        );
        assert_eq!(
            parse_cloud_model_id("cloud:deepgram:nova-3"),
            Some(("deepgram", "nova-3"))
        );
        assert_eq!(parse_cloud_model_id("base"), None);
        assert_eq!(parse_cloud_model_id("parakeet-v3"), None);
    }

    #[test]
    fn test_mask_key() {
        assert_eq!(mask_key(""), "");
        assert_eq!(mask_key("short"), "••••••••");
        assert_eq!(mask_key("gsk_123456789abcdef"), "gsk_••••cdef");
    }

    #[test]
    fn test_encrypt_decrypt_api_key_roundtrip() {
        let key = "gsk_test_api_key_secret_12345";
        let encrypted = encrypt_api_key(key).expect("Encryption should succeed");
        assert_ne!(key, encrypted);
        let decrypted = decrypt_api_key(&encrypted);
        assert_eq!(key, decrypted);
    }
}
