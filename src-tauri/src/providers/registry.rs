//! Provider registry initialization helpers.
//!
//! Provides functions to set up registries with the built-in
//! provider implementations. New providers are added by editing
//! these functions — the only place that knows about concrete
//! provider types.
//!
//! # Migration from match blocks
//!
//! The current `cloud_transcription.rs` and `ai_formatting.rs` modules
//! use large match blocks to dispatch to provider-specific logic.
//! Once the provider implementations are migrated to use this registry,
//! those match blocks can be removed. The pattern is:
//!
//! 1. Implement `CloudProvider` for each provider in `providers/cloud.rs`
//! 2. Register via `default_cloud_providers()`
//! 3. Replace `match provider { ... }` with `registry.get(id).transcribe(...)`
//!
//! This achieves Open/Closed Principle: adding a provider requires
//! no modifications to existing dispatch logic.

use crate::providers::ai_formatting::AiFormattingProviderRegistry;
use crate::providers::cloud::CloudProviderRegistry;

/// Create and populate a `CloudProviderRegistry` with all built-in providers.
pub fn default_cloud_providers() -> CloudProviderRegistry {
    let mut registry = CloudProviderRegistry::new();
    for id in crate::providers::CLOUD_PROVIDER_IDS {
        // TODO: register concrete provider implementations
        // Example:
        // match id {
        //     "groq" => registry.register(Box::new(GroqCloudProvider)),
        //     "openai" => registry.register(Box::new(OpenAiCloudProvider)),
        //     ...
        //     _ => registry.register(Box::new(CustomCloudProvider)),
        // }
    }
    registry
}

/// Create and populate an `AiFormattingProviderRegistry` with all built-in providers.
pub fn default_ai_formatting_providers() -> AiFormattingProviderRegistry {
    let mut registry = AiFormattingProviderRegistry::new();
    for id in crate::providers::AI_FORMATTING_PROVIDER_IDS {
        // TODO: register concrete provider implementations
        // (same pattern as cloud providers)
    }
    registry
}

/// Get a human-readable name for a cloud provider ID.
pub fn cloud_provider_name(id: &str) -> &'static str {
    match id {
        "groq" => "Groq",
        "openai" => "OpenAI",
        "deepgram" => "Deepgram",
        "mistral" => "Mistral",
        "custom" => "Custom Endpoint",
        _ => "Unknown",
    }
}

/// Get a human-readable name for an AI formatting provider ID.
pub fn ai_formatting_provider_name(id: &str) -> &'static str {
    match id {
        "gemini" => "Gemini",
        "anthropic" => "Anthropic",
        "openai" => "OpenAI",
        "deepseek" => "DeepSeek",
        "custom" => "Custom Endpoint",
        _ => "Unknown",
    }
}

/// Build masked key for display (e.g. "gsk_...cdef").
pub fn mask_key(key: &str) -> String {
    if key.len() <= 8 {
        return "••••••••".to_string();
    }
    let prefix = &key[..std::cmp::min(4, key.len())];
    let suffix = &key[key.len() - std::cmp::min(4, key.len())..];
    format!("{}••••{}", prefix, suffix)
}
