use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct TranslationRequest {
    pub q: String,
    pub langpair: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TranslationResponse {
    #[serde(rename = "responseData")]
    pub response_data: ResponseData,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ResponseData {
    #[serde(rename = "translatedText")]
    pub translated_text: String,
}
