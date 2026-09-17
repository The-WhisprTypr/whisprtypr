use crate::cloud_transcription::build_http_client;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

pub const DEFAULT_GEMINI_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";
pub const DEFAULT_ANTHROPIC_BASE_URL: &str = "https://api.anthropic.com/v1";
pub const DEFAULT_OPENAI_BASE_URL: &str = "https://api.openai.com/v1";
pub const DEFAULT_DEEPSEEK_BASE_URL: &str = "https://api.deepseek.com/v1";

pub const AI_FORMATTING_PROVIDER_IDS: &[&str] =
    &["gemini", "anthropic", "openai", "deepseek", "custom"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiFormattingProviderInfo {
    pub id: String,
    pub name: String,
    pub configured: bool,
    pub masked_key: String,
    pub base_url: Option<String>,
    pub custom_model: Option<String>,
}

pub mod types;
pub use types::*;

pub mod formatting;
pub use formatting::{build_formatting_prompt, format_text_with_ai, test_ai_provider_connection};

pub fn default_base_url_for_provider(provider: &str) -> &str {
    match provider.to_lowercase().as_str() {
        "openai" => DEFAULT_OPENAI_BASE_URL,
        "deepseek" => DEFAULT_DEEPSEEK_BASE_URL,
        "anthropic" => DEFAULT_ANTHROPIC_BASE_URL,
        "gemini" => DEFAULT_GEMINI_BASE_URL,
        _ => "http://localhost:8000/v1",
    }
}

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
        assert_eq!(
            default_model_for_provider("anthropic"),
            "claude-3-5-sonnet-20241022"
        );
        assert_eq!(default_model_for_provider("gemini"), "gemini-1.5-flash");
        assert_eq!(default_model_for_provider("custom"), "gpt-4o-mini");
    }

    #[test]
    fn test_default_base_url_for_provider() {
        assert_eq!(
            default_base_url_for_provider("openai"),
            DEFAULT_OPENAI_BASE_URL
        );
        assert_eq!(
            default_base_url_for_provider("deepseek"),
            DEFAULT_DEEPSEEK_BASE_URL
        );
        assert_eq!(
            default_base_url_for_provider("anthropic"),
            DEFAULT_ANTHROPIC_BASE_URL
        );
        assert_eq!(
            default_base_url_for_provider("gemini"),
            DEFAULT_GEMINI_BASE_URL
        );
        assert_eq!(
            default_base_url_for_provider("custom"),
            "http://localhost:8000/v1"
        );
    }

    #[test]
    fn test_formatting_prompt_structure() {
        let prompt = formatting::build_formatting_prompt("Clean the text.", "Hello world");
        assert!(prompt.contains("system"));
        assert!(prompt.contains("user"));
        assert!(prompt.contains("Hello world"));
        assert!(prompt.contains("Clean the text."));
    }

    #[test]
    fn test_format_text_with_ai_empty_input() {
        let result = formatting::format_text_with_ai(
            "openai",
            "gpt-4o-mini",
            "fake_key",
            None,
            "Clean text.",
            "",
        );
        let _ = result;
    }
}
