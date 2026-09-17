use crate::cloud_transcription::models::DEFAULT_MISTRAL_BASE_URL;
use crate::cloud_transcription::util::build_http_client;
use crate::database::Database;
use reqwest::multipart::{Form, Part};

pub async fn test_mistral_connection(
    api_key: &str,
    base_url: Option<&str>,
) -> Result<String, String> {
    let key = api_key.trim();
    let client = build_http_client(10)?;

    let url = format!(
        "{}/models",
        base_url
            .unwrap_or(DEFAULT_MISTRAL_BASE_URL)
            .trim_end_matches('/')
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

pub async fn transcribe_with_mistral(
    db: &Database,
    model: &str,
    wav_bytes: Vec<u8>,
    language: &str,
    _enable_speaker_detection: bool,
) -> Result<String, String> {
    let provider_record = db
        .get_cloud_provider("mistral")
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| {
            format!(
                "Cloud provider 'mistral' is not configured. Please add an API key in Models settings."
            )
        })?;

    let api_key = crate::cloud_transcription::util::decrypt_api_key(&provider_record.api_key);
    if api_key.trim().is_empty() {
        return Err(format!(
            "API key for 'mistral' is missing. Please configure it in Models."
        ));
    }

    let client = build_http_client(60)?;

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
        return Err(format!(
            "Mistral transcription failed ({}): {}",
            status, err_text
        ));
    }

    #[derive(serde::Deserialize)]
    struct MistralResponse {
        text: String,
    }

    let json: MistralResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse Mistral response: {}", e))?;

    Ok(json.text.trim().to_string())
}
