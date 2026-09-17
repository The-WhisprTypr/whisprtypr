//! Cloud transcription provider abstraction.
//!
//! Each provider (Groq, OpenAI, Deepgram, Mistral, Custom) implements
//! `CloudProvider`. New providers can be added by implementing the trait
//! and registering — no match blocks need modification.

use crate::database::Database;
use std::collections::HashMap;

/// Result of testing a provider connection.
pub type ConnectionTestResult = Result<String, String>;

/// Result of a transcription request.
pub type TranscriptionResult = Result<String, String>;

/// A cloud transcription provider.
///
/// Implementors encapsulate all provider-specific details: API URLs,
/// request formatting, response parsing, and authentication.
pub trait CloudProvider: Send + Sync {
    /// Provider identifier (e.g., "groq", "openai").
    fn id(&self) -> &str;

    /// Human-readable provider name.
    fn name(&self) -> &str;

    /// Default base URL for this provider's API.
    fn default_base_url(&self) -> &str;

    /// Default model identifier for this provider.
    fn default_model(&self) -> &str;

    /// Test the provider connection with the given API key.
    fn test_connection(&self, api_key: &str, base_url: Option<&str>) -> ConnectionTestResult;

    /// Transcribe audio using this provider.
    fn transcribe(
        &self,
        db: &Database,
        model: &str,
        api_key: &str,
        base_url: Option<&str>,
        wav_bytes: Vec<u8>,
        language: &str,
        enable_speaker_detection: bool,
    ) -> TranscriptionResult;

    /// Transcribe from a URL using this provider.
    fn transcribe_url(
        &self,
        db: &Database,
        model: &str,
        api_key: &str,
        base_url: Option<&str>,
        url: &str,
        language: &str,
        enable_speaker_detection: bool,
    ) -> TranscriptionResult;
}

/// A registered cloud provider with its metadata.
#[derive(Debug, Clone)]
pub struct CloudProviderInfo {
    pub id: String,
    pub name: String,
    pub configured: bool,
    pub masked_key: String,
    pub base_url: Option<String>,
    pub custom_model: Option<String>,
}

/// Registry that maps provider IDs to their implementations.
///
/// Adding a new provider only requires calling `register` — the registry
/// handles lookup, so call sites no longer need match blocks.
pub struct CloudProviderRegistry {
    providers: HashMap<String, Box<dyn CloudProvider>>,
}

impl CloudProviderRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Register a provider in the registry.
    pub fn register(&mut self, provider: Box<dyn CloudProvider>) {
        self.providers.insert(provider.id().to_string(), provider);
    }

    /// Get a provider by ID.
    pub fn get(&self, id: &str) -> Option<&dyn CloudProvider> {
        self.providers.get(id).map(|b| b.as_ref())
    }

    /// Check if a provider is registered.
    pub fn contains(&self, id: &str) -> bool {
        self.providers.contains_key(id)
    }

    /// Get all registered provider IDs.
    pub fn ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.providers.keys().cloned().collect();
        ids.sort();
        ids
    }
}

impl Default for CloudProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
