use serde::{Deserialize, Serialize};

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

#[derive(Debug, Deserialize)]
pub struct DeepgramWord {
    pub speaker: Option<u32>,
    pub word: String,
}
