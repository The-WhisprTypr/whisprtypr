use serde::{Deserialize, Serialize};

/// Request body for OpenAI/DeepSeek compatible chat completions
#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Response shape for OpenAI/DeepSeek-style chat completions
#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
pub struct ChatChoice {
    pub message: ChatResponseMessage,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponseMessage {
    pub content: String,
}

/// Request body for Anthropic messages API
#[derive(Debug, Serialize)]
pub struct AnthropicRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<ChatMessage>,
}

/// Response shape for Anthropic messages API
#[derive(Debug, Deserialize)]
pub struct AnthropicResponse {
    pub content: Vec<AnthropicContent>,
}

#[derive(Debug, Deserialize)]
pub struct AnthropicContent {
    pub r#type: String,
    pub text: String,
}

/// Request body for Gemini generateContent
#[derive(Debug, Serialize)]
pub struct GeminiRequest {
    pub contents: Vec<GeminiContent>,
}

#[derive(Debug, Serialize)]
pub struct GeminiContent {
    pub parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize)]
pub struct GeminiPart {
    pub text: String,
}

/// Response shape for Gemini generateContent
#[derive(Debug, Deserialize)]
pub struct GeminiResponse {
    pub candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
pub struct GeminiCandidate {
    pub content: GeminiContentResponse,
}

#[derive(Debug, Deserialize)]
pub struct GeminiContentResponse {
    pub parts: Vec<GeminiPartResponse>,
}

#[derive(Debug, Deserialize)]
pub struct GeminiPartResponse {
    pub text: String,
}
