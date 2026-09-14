use reqwest::Client;
use serde::Serialize;

const MYMEMORY_API_URL: &str = "https://api.mymemory.translated.net/get";

#[derive(Debug, Clone, Serialize)]
struct TranslationRequest {
    q: String,
    langpair: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    key: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TranslationResponse {
    #[serde(rename = "responseData")]
    response_data: ResponseData,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ResponseData {
    #[serde(rename = "translatedText")]
    translated_text: String,
}

pub async fn translate(
    client: &Client,
    text: &str,
    source: &str,
    target: &str,
    api_key: Option<&str>,
) -> Result<String, String> {
    if text.trim().is_empty() {
        return Ok(String::new());
    }

    if source == target {
        return Ok(text.to_string());
    }

    let request = TranslationRequest {
        q: text.to_string(),
        langpair: format!("{}|{}", source, target),
        key: api_key.and_then(|k| {
            let trimmed = k.trim();
            if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
        }),
    };

    let response = client
        .get(MYMEMORY_API_URL)
        .query(&request)
        .send()
        .await
        .map_err(|e| format!("Translation request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Translation error {}: {}", status, body));
    }

    let result: TranslationResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse translation response: {}", e))?;

    Ok(result.response_data.translated_text)
}
