//! Provider abstraction layer for cloud transcription and AI formatting.
//!
//! This module defines trait-based abstractions so that adding a new provider
//! requires only implementing the trait — no match blocks need modification
//! (Open/Closed Principle). Commands depend on the trait, not concrete
//! provider types (Dependency Inversion Principle).

pub mod ai_formatting;
pub mod cloud;
pub mod registry;

/// Shared defaults across all cloud transcription providers.
pub const DEFAULT_GROQ_BASE_URL: &str = "https://api.groq.com/openai/v1";
pub const DEFAULT_OPENAI_BASE_URL: &str = "https://api.openai.com/v1";
pub const DEFAULT_DEEPGRAM_BASE_URL: &str = "https://api.deepgram.com/v1";
pub const DEFAULT_MISTRAL_BASE_URL: &str = "https://api.mistral.ai/v1";

/// Shared defaults across all AI formatting providers.
pub const DEFAULT_GEMINI_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";
pub const DEFAULT_ANTHROPIC_BASE_URL: &str = "https://api.anthropic.com/v1";
pub const DEFAULT_DEEPSEEK_BASE_URL: &str = "https://api.deepseek.com/v1";

/// Known cloud transcription provider IDs.
pub const CLOUD_PROVIDER_IDS: &[&str] = &["groq", "openai", "deepgram", "mistral", "custom"];

/// Known AI formatting provider IDs.
pub const AI_FORMATTING_PROVIDER_IDS: &[&str] =
    &["gemini", "anthropic", "openai", "deepseek", "custom"];
