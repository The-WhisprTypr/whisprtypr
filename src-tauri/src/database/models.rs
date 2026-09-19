use serde::{Deserialize, Serialize};

/// A single user-defined custom vocabulary entry.
///
/// `spoken` is the phrase the user expects to say (and what Whisper is most
/// likely to mishear) and `written` is the canonical text that should appear
/// in the final output. Matching is case-insensitive and word-boundary
/// aware, so longer words that happen to contain a `spoken` substring are
/// not affected.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct VocabularyEntry {
    pub spoken: String,
    pub written: String,
}

// Types for database operations
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub push_to_talk_key: String,
    pub toggle_key: String,
    pub hotkey_mode: String,
    pub language: String,
    pub selected_model_id: String,
    pub show_recording_indicator: bool,
    pub show_recording_overlay: bool,
    pub play_audio_feedback: bool,
    pub auto_start_on_boot: bool,
    pub minimize_to_tray: bool,
    pub post_processing_enabled: bool,
    pub voice_commands_enabled: bool,
    pub clipboard_mode: bool,
    pub auto_check_for_updates: bool,
    pub diagnostics_enabled: bool,
    pub recording_overlay_position: String,
    /// User-defined domain vocabulary. Stored as a JSON array so we can
    /// support an arbitrary number of entries without a schema change.
    pub custom_vocabulary: Vec<VocabularyEntry>,
    pub translation_enabled: bool,
    pub translation_hotkey: String,
    pub translation_source_language: String,
    pub translation_target_language: String,
    pub translation_api_key: Option<String>,
    pub ai_formatting_enabled: bool,
    pub ai_formatting_provider_id: String,
    pub ai_formatting_style: String,
    pub ai_formatting_model: String,
    pub grammar_check_enabled: bool,
    pub grammar_check_dialect: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            push_to_talk_key: "Alt+Shift+S".to_string(),
            toggle_key: "Alt+Shift+D".to_string(),
            hotkey_mode: "push-to-talk".to_string(),
            language: "en".to_string(),
            selected_model_id: "base".to_string(),
            show_recording_indicator: true,
            show_recording_overlay: true,
            play_audio_feedback: true,
            auto_start_on_boot: false,
            minimize_to_tray: true,
            post_processing_enabled: true,
            voice_commands_enabled: false,
            clipboard_mode: false,
            auto_check_for_updates: false,
            diagnostics_enabled: true,
            recording_overlay_position: "top-center".to_string(),
            custom_vocabulary: Vec::new(),
            translation_enabled: false,
            translation_hotkey: "Alt+Shift+T".to_string(),
            translation_source_language: "en".to_string(),
            translation_target_language: "es".to_string(),
            translation_api_key: None,
            ai_formatting_enabled: false,
            ai_formatting_provider_id: "openai".to_string(),
            ai_formatting_style: "clean".to_string(),
            ai_formatting_model: "gpt-4o-mini".to_string(),
            grammar_check_enabled: true,
            grammar_check_dialect: "american".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WhisperModel {
    pub id: String,
    pub name: String,
    pub size: String,
    pub size_bytes: i64,
    pub description: String,
    pub languages: String, // JSON array stored as string
    pub downloaded: bool,
    pub download_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LicenseData {
    pub license_key: Option<String>,
    pub activation_id: Option<String>,
    pub status: String,
    pub customer_email: Option<String>,
    pub customer_name: Option<String>,
    pub expires_at: Option<String>,
    pub is_activated: bool,
    pub last_validated_at: Option<String>,
    pub trial_started_at: Option<String>,
    pub trial_integrity_hash: Option<String>,
    pub trial_salt: Option<String>,
    pub usage: i32,
    pub validations: i32,
}

impl Default for LicenseData {
    fn default() -> Self {
        Self {
            license_key: None,
            activation_id: None,
            status: "inactive".to_string(),
            customer_email: None,
            customer_name: None,
            expires_at: None,
            is_activated: false,
            last_validated_at: None,
            trial_started_at: None,
            trial_integrity_hash: None,
            trial_salt: None,
            usage: 0,
            validations: 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppState {
    pub is_first_launch: bool,
    pub setup_complete: bool,
    pub current_setup_step: i32,
    pub selected_model_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CloudProviderRecord {
    pub id: String,
    pub api_key: String,
    pub base_url: Option<String>,
    pub custom_model: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiFormattingProviderRecord {
    pub id: String,
    pub api_key: String,
    pub base_url: Option<String>,
    pub custom_model: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TranscriptionHistory {
    pub id: i64,
    pub text: String,
    pub model_id: String,
    pub language: String,
    pub duration_ms: i64,
    pub created_at: String,
}
