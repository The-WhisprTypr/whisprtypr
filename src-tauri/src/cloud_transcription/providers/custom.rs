use crate::cloud_transcription::util::build_http_client;
use crate::database::Database;
use reqwest::multipart::{Form, Part};

pub async fn test_custom_connection(
    api_key: &str,
    base_url: Option<&str>,
) -> Result<String, String> {
    let key = api_key.trim();
    let client = build_http_client(10)?;

    let base = base_url
        .unwrap_or("http://localhost:8000/v1")
        .trim_end_matches('/');
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

pub async fn transcribe_with_custom(
    db: &Database,
    model: &str,
    wav_bytes: Vec<u8>,
    language: &str,
    _enable_speaker_detection: bool,
) -> Result<String, String> {
    let provider_record = db
        .get_cloud_provider("custom")
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| {
            format!(
                "Cloud provider 'custom' is not configured. Please add an API key in Models settings."
            )
        })?;

    let api_key = crate::cloud_transcription::util::decrypt_api_key(&provider_record.api_key);

    let client = build_http_client(60)?;

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
        return Err(format!(
            "Custom endpoint transcription failed ({}): {}",
            status, err_text
        ));
    }

    #[derive(serde::Deserialize)]
    struct OpenAiResponse {
        text: String,
    }

    let json: OpenAiResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response from custom endpoint: {}", e))?;

    Ok(json.text.trim().to_string())
}
