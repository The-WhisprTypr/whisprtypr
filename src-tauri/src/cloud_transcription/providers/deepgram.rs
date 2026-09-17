use crate::cloud_transcription::models::{DeepgramWord, DEFAULT_DEEPGRAM_BASE_URL};
use crate::cloud_transcription::util::build_http_client;
use crate::database::Database;

#[derive(serde::Deserialize)]
pub struct DeepgramAlternative {
    pub transcript: String,
    pub words: Option<Vec<DeepgramWord>>,
}

#[derive(serde::Deserialize)]
pub struct DeepgramChannel {
    pub alternatives: Vec<DeepgramAlternative>,
}

#[derive(serde::Deserialize)]
pub struct DeepgramResults {
    pub channels: Vec<DeepgramChannel>,
}

#[derive(serde::Deserialize)]
pub struct DeepgramResponse {
    pub results: DeepgramResults,
}

pub async fn test_deepgram_connection(
    api_key: &str,
    base_url: Option<&str>,
) -> Result<String, String> {
    let key = api_key.trim();
    let client = build_http_client(10)?;

    let url = format!(
        "{}/projects",
        base_url
            .unwrap_or(DEFAULT_DEEPGRAM_BASE_URL)
            .trim_end_matches('/')
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

pub async fn transcribe_with_deepgram(
    db: &Database,
    model: &str,
    wav_bytes: Vec<u8>,
    language: &str,
    enable_speaker_detection: bool,
) -> Result<String, String> {
    let provider_record = db
        .get_cloud_provider("deepgram")
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| {
            format!(
                "Cloud provider 'deepgram' is not configured. Please add an API key in Models settings."
            )
        })?;

    let api_key = crate::cloud_transcription::util::decrypt_api_key(&provider_record.api_key);
    if api_key.trim().is_empty() {
        return Err(format!(
            "API key for 'deepgram' is missing. Please configure it in Models."
        ));
    }

    let client = build_http_client(60)?;

    let base = provider_record
        .base_url
        .as_deref()
        .unwrap_or(DEFAULT_DEEPGRAM_BASE_URL)
        .trim_end_matches('/');
    let mut url = format!(
        "{}/listen?model={}&smart_format=true&punctuate=true",
        base, model
    );
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
        return Err(format!(
            "Deepgram transcription failed ({}): {}",
            status, err_text
        ));
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
                crate::cloud_transcription::format_deepgram_speaker_transcript(words)
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

pub async fn transcribe_with_deepgram_url(
    db: &Database,
    model: &str,
    url: &str,
    language: &str,
    enable_speaker_detection: bool,
) -> Result<String, String> {
    let provider_record = db
        .get_cloud_provider("deepgram")
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| format!("Cloud provider 'deepgram' is not configured."))?;

    let api_key = crate::cloud_transcription::util::decrypt_api_key(&provider_record.api_key);
    if api_key.trim().is_empty() {
        return Err(format!(
            "API key for 'deepgram' is missing. Please configure it in Models."
        ));
    }

    let client = build_http_client(60)?;

    let base = provider_record
        .base_url
        .as_deref()
        .unwrap_or(DEFAULT_DEEPGRAM_BASE_URL)
        .trim_end_matches('/');
    let mut transcribe_url = format!(
        "{}/listen?url={}&model={}&smart_format=true&punctuate=true",
        base,
        urlencoding::encode(url),
        model
    );
    if !language.is_empty() && language != "auto" {
        transcribe_url.push_str(&format!("&language={}", language));
    }
    if enable_speaker_detection {
        transcribe_url.push_str("&diarize=true&diarize_version=2");
    }

    let resp = client
        .get(&transcribe_url)
        .header("Authorization", format!("Token {}", api_key))
        .send()
        .await
        .map_err(|e| format!("Deepgram request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let err_text = resp.text().await.unwrap_or_default();
        return Err(format!(
            "Deepgram transcription failed ({}): {}",
            status, err_text
        ));
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
                crate::cloud_transcription::format_deepgram_speaker_transcript(words)
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
