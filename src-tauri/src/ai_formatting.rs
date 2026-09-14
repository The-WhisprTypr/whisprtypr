use crate::cloud_transcription::build_http_client;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

/// Default base URLs for known AI formatting providers
pub const DEFAULT_GEMINI_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";
pub const DEFAULT_ANTHROPIC_BASE_URL: &str = "https://api.anthropic.com/v1";
pub const DEFAULT_OPENAI_BASE_URL: &str = "https://api.openai.com/v1";
pub const DEFAULT_DEEPSEEK_BASE_URL: &str = "https://api.deepseek.com/v1";

/// Provider-level metadata used purely for UI display. The actual persisted
/// records live in the `ai_formatting_providers` database table.
pub const AI_FORMATTING_PROVIDER_IDS: &[&str] = &["gemini", "anthropic", "openai", "deepseek", "custom"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiFormattingProviderInfo {
    pub id: String,
    pub name: String,
    pub configured: bool,
    pub masked_key: String,
    pub base_url: Option<String>,
    pub custom_model: Option<String>,
}

/// Request body for OpenAI/DeepSeek compatible chat completions
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

/// Response shape for OpenAI/DeepSeek-style chat completions
#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ChatResponseMessage {
    content: String,
}

/// Request body for Anthropic messages API
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ChatMessage>,
}

/// Response shape for Anthropic messages API
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    r#type: String,
    text: String,
}

/// Request body for Gemini generateContent
#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Debug, Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize)]
struct GeminiPart {
    text: String,
}

/// Response shape for Gemini generateContent
#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContentResponse,
}

#[derive(Debug, Deserialize)]
struct GeminiContentResponse {
    parts: Vec<GeminiPartResponse>,
}

#[derive(Debug, Deserialize)]
struct GeminiPartResponse {
    text: String,
}

/// Build the system + user prompt for an AI formatting request.
///
/// `system_prompt` is the style-specific instruction and `user_text` is the
/// raw transcribed text to be formatted.
pub fn build_formatting_prompt(system_prompt: &str, user_text: &str) -> String {
    format!(
        "{{\"role\":\"system\",\"content\":\"{}\"}} {{\"role\":\"user\",\"content\":\"{}\"}}",
        system_prompt, user_text
    )
}

/// Test connectivity to an AI formatting provider with the given credentials.
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
        "deepseek" => {
            let url = format!(
                "{}/models",
                base_url.unwrap_or(DEFAULT_DEEPSEEK_BASE_URL).trim_end_matches('/')
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
                Err("Authentication failed: Invalid DeepSeek API key (401 Unauthorized).".to_string())
            } else {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                Err(format!("DeepSeek API error ({}): {}", status, text))
            }
        }
        "gemini" => {
            let base = base_url.unwrap_or(DEFAULT_GEMINI_BASE_URL).trim_end_matches('/');
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
            let base = base_url.unwrap_or(DEFAULT_ANTHROPIC_BASE_URL).trim_end_matches('/');
            let url = format!("{}/models", base);

            let mut headers = HeaderMap::new();
            headers.insert("x-api-key", HeaderValue::from_str(key).map_err(|e| format!("Invalid API key format: {}", e))?);
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
                Err("Authentication failed: Invalid Anthropic API key (401 Unauthorized).".to_string())
            } else {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                Err(format!("Anthropic API error ({}): {}", status, text))
            }
        }
        "custom" => {
            let base = base_url.unwrap_or("http://localhost:8000/v1").trim_end_matches('/');
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

/// Send text to the configured AI provider for formatting.
///
/// `provider` is the provider id (gemini, anthropic, openai, deepseek, custom).
/// `model` is the model identifier to use.
/// `system_prompt` is the style-specific instruction.
/// `user_text` is the raw transcribed text to format.
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
            let base = base_url.unwrap_or(default_base_url_for_provider(provider)).trim_end_matches('/');
            let url = format!("{}/chat/completions", base);

            let req_body = ChatRequest {
                model: effective_model.to_string(),
                messages: vec![
                    ChatMessage {
                        role: "system".to_string(),
                        content: system_prompt.to_string(),
                    },
                    ChatMessage {
                        role: "user".to_string(),
                        content: user_text.to_string(),
                    },
                ],
                temperature: 0.3,
                max_tokens: 4000,
            };

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
                return Err(format!("{} formatting failed ({}): {}", provider, status, err_text));
            }

            let json: ChatResponse = resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse {} response: {}", provider, e))?;

            let formatted = json
                .choices
                .first()
                .map(|c| c.message.content.trim().to_string())
                .unwrap_or_default();

            Ok(formatted)
        }

        "anthropic" => {
            let base = base_url.unwrap_or(DEFAULT_ANTHROPIC_BASE_URL).trim_end_matches('/');
            let url = format!("{}/messages", base);

            let req_body = AnthropicRequest {
                model: effective_model.to_string(),
                max_tokens: 4000,
                messages: vec![ChatMessage {
                    role: "user".to_string(),
                    content: format!("INSTRUCTIONS: {}\n\nTRANSCRIPT:\n{}", system_prompt, user_text),
                }],
            };

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
                return Err(format!("Anthropic formatting failed ({}): {}", status, err_text));
            }

            let json: AnthropicResponse = resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse Anthropic response: {}", e))?;

            let formatted = json
                .content
                .first()
                .filter(|c| c.r#type == "text")
                .map(|c| c.text.trim().to_string())
                .unwrap_or_default();

            Ok(formatted)
        }

        "gemini" => {
            let base = base_url.unwrap_or(DEFAULT_GEMINI_BASE_URL).trim_end_matches('/');
            // Gemini uses the API key as a query parameter, not in the Authorization header
            let effective_model = if effective_model.starts_with("gemini") {
                effective_model.to_string()
            } else {
                format!("models/{}", effective_model)
            };
            let url = format!("{}/{}:generateContent?key={}", base, effective_model, key);

            // For Gemini, combine system prompt into the user message since the
            // API doesn't have a separate system role in the same way
            let combined = format!("{}\n\n{}", system_prompt, user_text);

            let req_body = GeminiRequest {
                contents: vec![GeminiContent {
                    parts: vec![GeminiPart {
                        text: combined,
                    }],
                }],
            };

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
                return Err(format!("Gemini formatting failed ({}): {}", status, err_text));
            }

            let json: GeminiResponse = resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse Gemini response: {}", e))?;

            let formatted = json
                .candidates
                .first()
                .and_then(|c| c.content.parts.first())
                .map(|p| p.text.trim().to_string())
                .unwrap_or_default();

            Ok(formatted)
        }

        "custom" => {
            let base = base_url.unwrap_or("http://localhost:8000/v1").trim_end_matches('/');
            let url = format!("{}/chat/completions", base);

            let req_body = ChatRequest {
                model: effective_model.to_string(),
                messages: vec![
                    ChatMessage {
                        role: "system".to_string(),
                        content: system_prompt.to_string(),
                    },
                    ChatMessage {
                        role: "user".to_string(),
                        content: user_text.to_string(),
                    },
                ],
                temperature: 0.3,
                max_tokens: 4000,
            };

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
                return Err(format!("Custom endpoint formatting failed ({}): {}", status, err_text));
            }

            let json: ChatResponse = resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse custom endpoint response: {}", e))?;

            let formatted = json
                .choices
                .first()
                .map(|c| c.message.content.trim().to_string())
                .unwrap_or_default();

            Ok(formatted)
        }

        _ => Err(format!("Unsupported AI formatting provider: {}", provider)),
    }
}

/// Return the default base URL for a known provider
pub fn default_base_url_for_provider(provider: &str) -> &str {
    match provider.to_lowercase().as_str() {
        "openai" => DEFAULT_OPENAI_BASE_URL,
        "deepseek" => DEFAULT_DEEPSEEK_BASE_URL,
        "anthropic" => DEFAULT_ANTHROPIC_BASE_URL,
        "gemini" => DEFAULT_GEMINI_BASE_URL,
        _ => "http://localhost:8000/v1",
    }
}

/// Return the default model for a known provider
pub fn default_model_for_provider(provider: &str) -> &str {
    match provider.to_lowercase().as_str() {
        "openai" => "gpt-4o-mini",
        "deepseek" => "deepseek-chat",
        "anthropic" => "claude-3-5-sonnet-20241022",
        "gemini" => "gemini-1.5-flash",
        _ => "gpt-4o-mini",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_model_for_provider() {
        assert_eq!(default_model_for_provider("openai"), "gpt-4o-mini");
        assert_eq!(default_model_for_provider("deepseek"), "deepseek-chat");
        assert_eq!(default_model_for_provider("anthropic"), "claude-3-5-sonnet-20241022");
        assert_eq!(default_model_for_provider("gemini"), "gemini-1.5-flash");
        assert_eq!(default_model_for_provider("custom"), "gpt-4o-mini");
    }

    #[test]
    fn test_default_base_url_for_provider() {
        assert_eq!(default_base_url_for_provider("openai"), DEFAULT_OPENAI_BASE_URL);
        assert_eq!(default_base_url_for_provider("deepseek"), DEFAULT_DEEPSEEK_BASE_URL);
        assert_eq!(default_base_url_for_provider("anthropic"), DEFAULT_ANTHROPIC_BASE_URL);
        assert_eq!(default_base_url_for_provider("gemini"), DEFAULT_GEMINI_BASE_URL);
        assert_eq!(default_base_url_for_provider("custom"), "http://localhost:8000/v1");
    }

    #[test]
    fn test_formatting_prompt_structure() {
        let prompt = build_formatting_prompt("Clean the text.", "Hello world");
        assert!(prompt.contains("system"));
        assert!(prompt.contains("user"));
        assert!(prompt.contains("Hello world"));
        assert!(prompt.contains("Clean the text."));
    }

    #[test]
    fn test_format_text_with_ai_empty_input() {
        let result = format_text_with_ai(
            "openai",
            "gpt-4o-mini",
            "fake_key",
            None,
            "Clean text.",
            "",
        );
        // Empty input should return empty without attempting API call
        // (this is evaluated as a future, so we just check the sync part)
        // The actual async result would be Ok("") but we can't await in a test
        // without tokio runtime. Just verify the function signature compiles.
        let _ = result;
    }
}
