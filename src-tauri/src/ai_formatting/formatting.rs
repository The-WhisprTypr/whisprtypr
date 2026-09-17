use crate::cloud_transcription::build_http_client;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};

use crate::ai_formatting::default_base_url_for_provider;
use crate::ai_formatting::default_model_for_provider;
use crate::ai_formatting::DEFAULT_ANTHROPIC_BASE_URL;
use crate::ai_formatting::DEFAULT_DEEPSEEK_BASE_URL;
use crate::ai_formatting::DEFAULT_GEMINI_BASE_URL;
use crate::ai_formatting::DEFAULT_OPENAI_BASE_URL;

pub fn build_formatting_prompt(system_prompt: &str, user_text: &str) -> String {
    format!(
        "{{\"role\":\"system\",\"content\":\"{}\"}} {{\"role\":\"user\",\"content\":\"{}\"}}",
        system_prompt, user_text
    )
}

pub async fn test_ai_provider_connection(
    provider: &str,
    api_key: &str,
    base_url: Option<&str>,
    _model: Option<&str>,
) -> Result<String, String> {
    let key = api_key.trim();
    if key.is_empty() && provider != "custom" {
        return Err("API key is required".to_string());
    }

    let client = build_http_client(10)?;

    match provider.to_lowercase().as_str() {
        "openai" => {
            let url = format!(
                "{}/models",
                base_url
                    .unwrap_or(DEFAULT_OPENAI_BASE_URL)
                    .trim_end_matches('/')
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
        "deepseek" => {
            let url = format!(
                "{}/models",
                base_url
                    .unwrap_or(DEFAULT_DEEPSEEK_BASE_URL)
                    .trim_end_matches('/')
            );
            let resp = client
                .get(&url)
                .header("Authorization", format!("Bearer {}", key))
                .send()
                .await
                .map_err(|e| format!("Failed to reach DeepSeek API: {}", e))?;

            if resp.status().is_success() {
                Ok("Connected successfully! DeepSeek API key is valid.".to_string())
            } else if resp.status().as_u16() == 401 {
                Err(
                    "Authentication failed: Invalid DeepSeek API key (401 Unauthorized)."
                        .to_string(),
                )
            } else {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                Err(format!("DeepSeek API error ({}): {}", status, text))
            }
        }
        "gemini" => {
            let base = base_url
                .unwrap_or(DEFAULT_GEMINI_BASE_URL)
                .trim_end_matches('/');
            let url = format!("{}/models?key={}", base, key);
            let resp = client
                .get(&url)
                .send()
                .await
                .map_err(|e| format!("Failed to reach Gemini API: {}", e))?;

            if resp.status().is_success() {
                Ok("Connected successfully! Gemini API key is valid.".to_string())
            } else if resp.status().as_u16() == 400 || resp.status().as_u16() == 401 {
                Err("Authentication failed: Invalid Gemini API key.".to_string())
            } else {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                Err(format!("Gemini API error ({}): {}", status, text))
            }
        }
        "anthropic" => {
            let base = base_url
                .unwrap_or(DEFAULT_ANTHROPIC_BASE_URL)
                .trim_end_matches('/');
            let url = format!("{}/models", base);

            let mut headers = HeaderMap::new();
            headers.insert(
                "x-api-key",
                HeaderValue::from_str(key).map_err(|e| format!("Invalid API key format: {}", e))?,
            );
            headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));

            let resp = client
                .get(&url)
                .headers(headers)
                .send()
                .await
                .map_err(|e| format!("Failed to reach Anthropic API: {}", e))?;

            if resp.status().is_success() {
                Ok("Connected successfully! Anthropic API key is valid.".to_string())
            } else if resp.status().as_u16() == 401 {
                Err(
                    "Authentication failed: Invalid Anthropic API key (401 Unauthorized)."
                        .to_string(),
                )
            } else {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                Err(format!("Anthropic API error ({}): {}", status, text))
            }
        }
        "custom" => {
            let base = base_url
                .unwrap_or("http://localhost:8000/v1")
                .trim_end_matches('/');
            let url = format!("{}/models", base);

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

pub async fn format_text_with_ai(
    provider: &str,
    model: &str,
    api_key: &str,
    base_url: Option<&str>,
    system_prompt: &str,
    user_text: &str,
) -> Result<String, String> {
    if user_text.trim().is_empty() {
        return Ok(String::new());
    }

    let key = api_key.trim();
    if key.is_empty() && provider != "custom" {
        return Err("No API key configured for AI formatting provider".to_string());
    }

    let effective_model = if model.trim().is_empty() {
        default_model_for_provider(provider)
    } else {
        model
    };

    let client = build_http_client(60)?;

    match provider.to_lowercase().as_str() {
        "openai" | "deepseek" => {
            let base = base_url
                .unwrap_or(default_base_url_for_provider(provider))
                .trim_end_matches('/');
            let url = format!("{}/chat/completions", base);

            let req_body = serde_json::json!({
                "model": effective_model,
                "messages": [
                    {"role": "system", "content": system_prompt},
                    {"role": "user", "content": user_text}
                ],
                "temperature": 0.3,
                "max_tokens": 4000
            });

            let resp = client
                .post(&url)
                .header("Authorization", format!("Bearer {}", key))
                .json(&req_body)
                .send()
                .await
                .map_err(|e| format!("{} request failed: {}", provider, e))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let err_text = resp.text().await.unwrap_or_default();
                return Err(format!(
                    "{} formatting failed ({}): {}",
                    provider, status, err_text
                ));
            }

            let json: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse {} response: {}", provider, e))?;

            let formatted = json["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("")
                .trim()
                .to_string();

            Ok(formatted)
        }

        "anthropic" => {
            let base = base_url
                .unwrap_or(DEFAULT_ANTHROPIC_BASE_URL)
                .trim_end_matches('/');
            let url = format!("{}/messages", base);

            let req_body = serde_json::json!({
                "model": effective_model,
                "max_tokens": 4000,
                "messages": [{"role": "user", "content": format!("INSTRUCTIONS: {}\n\nTRANSCRIPT:\n{}", system_prompt, user_text)}],
            });

            let resp = client
                .post(&url)
                .header("x-api-key", key)
                .header("anthropic-version", "2023-06-01")
                .header(CONTENT_TYPE, "application/json")
                .json(&req_body)
                .send()
                .await
                .map_err(|e| format!("Anthropic request failed: {}", e))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let err_text = resp.text().await.unwrap_or_default();
                return Err(format!(
                    "Anthropic formatting failed ({}): {}",
                    status, err_text
                ));
            }

            let json: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse Anthropic response: {}", e))?;

            let formatted = json["content"][0]["text"]
                .as_str()
                .unwrap_or("")
                .trim()
                .to_string();

            Ok(formatted)
        }

        "gemini" => {
            let base = base_url
                .unwrap_or(DEFAULT_GEMINI_BASE_URL)
                .trim_end_matches('/');
            let effective_model = if effective_model.starts_with("gemini") {
                effective_model.to_string()
            } else {
                format!("models/{}", effective_model)
            };
            let url = format!("{}/{}:generateContent?key={}", base, effective_model, key);

            let combined = format!("{}\n\n{}", system_prompt, user_text);

            let req_body = serde_json::json!({
                "contents": [{"parts": [{"text": combined}]}]
            });

            let resp = client
                .post(&url)
                .header(CONTENT_TYPE, "application/json")
                .json(&req_body)
                .send()
                .await
                .map_err(|e| format!("Gemini request failed: {}", e))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let err_text = resp.text().await.unwrap_or_default();
                return Err(format!(
                    "Gemini formatting failed ({}): {}",
                    status, err_text
                ));
            }

            let json: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse Gemini response: {}", e))?;

            let formatted = json["candidates"][0]["content"]["parts"][0]["text"]
                .as_str()
                .unwrap_or("")
                .trim()
                .to_string();

            Ok(formatted)
        }

        "custom" => {
            let base = base_url
                .unwrap_or("http://localhost:8000/v1")
                .trim_end_matches('/');
            let url = format!("{}/chat/completions", base);

            let req_body = serde_json::json!({
                "model": effective_model,
                "messages": [
                    {"role": "system", "content": system_prompt},
                    {"role": "user", "content": user_text}
                ],
                "temperature": 0.3,
                "max_tokens": 4000
            });

            let mut req = client.post(&url).json(&req_body);
            if !key.is_empty() {
                req = req.header("Authorization", format!("Bearer {}", key));
            }

            let resp = req
                .send()
                .await
                .map_err(|e| format!("Custom endpoint request failed: {}", e))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let err_text = resp.text().await.unwrap_or_default();
                return Err(format!(
                    "Custom endpoint formatting failed ({}): {}",
                    status, err_text
                ));
            }

            let json: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse custom endpoint response: {}", e))?;

            let formatted = json["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("")
                .trim()
                .to_string();

            Ok(formatted)
        }

        _ => Err(format!("Unsupported AI formatting provider: {}", provider)),
    }
}
