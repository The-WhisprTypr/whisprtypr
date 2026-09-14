use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

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

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app_data_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&app_data_dir).ok();
        let db_path = app_data_dir.join("WhisprTypr.db");
        let conn = Connection::open(db_path)?;

        let db = Self {
            conn: Mutex::new(conn),
        };

        db.init_tables()?;
        db.init_default_data()?;

        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Settings table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                push_to_talk_key TEXT NOT NULL DEFAULT 'Alt+Shift+S',
                toggle_key TEXT NOT NULL DEFAULT 'Alt+Shift+D',
                hotkey_mode TEXT NOT NULL DEFAULT 'push-to-talk',
                language TEXT NOT NULL DEFAULT 'en',
                selected_model_id TEXT NOT NULL DEFAULT 'base',
                show_recording_indicator INTEGER NOT NULL DEFAULT 1,
                show_recording_overlay INTEGER NOT NULL DEFAULT 1,
                play_audio_feedback INTEGER NOT NULL DEFAULT 1,
                auto_start_on_boot INTEGER NOT NULL DEFAULT 0,
                minimize_to_tray INTEGER NOT NULL DEFAULT 1,
                post_processing_enabled INTEGER NOT NULL DEFAULT 1,
                voice_commands_enabled INTEGER NOT NULL DEFAULT 0,
                clipboard_mode INTEGER NOT NULL DEFAULT 0,
                auto_check_for_updates INTEGER NOT NULL DEFAULT 0,
                diagnostics_enabled INTEGER NOT NULL DEFAULT 1,
                custom_vocabulary TEXT NOT NULL DEFAULT '[]',
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Add post_processing_enabled column if it doesn't exist (migration for existing DBs)
        let _ = conn.execute(
            "ALTER TABLE settings ADD COLUMN post_processing_enabled INTEGER NOT NULL DEFAULT 1",
            [],
        );

        // Add clipboard_mode column if it doesn't exist (migration for existing DBs)
        let _ = conn.execute(
            "ALTER TABLE settings ADD COLUMN clipboard_mode INTEGER NOT NULL DEFAULT 0",
            [],
        );

        // Add voice_commands_enabled column if it doesn't exist. This is
        // deliberately off by default because these commands can mutate the
        // active application.
        let _ = conn.execute(
            "ALTER TABLE settings ADD COLUMN voice_commands_enabled INTEGER NOT NULL DEFAULT 0",
            [],
        );

        // Add show_recording_overlay column if it doesn't exist (migration for existing DBs)
        let _ = conn.execute(
            "ALTER TABLE settings ADD COLUMN show_recording_overlay INTEGER NOT NULL DEFAULT 1",
            [],
        );

        // Add recording_overlay_position column if it doesn't exist (migration for existing DBs)
        let _ = conn.execute(
            "ALTER TABLE settings ADD COLUMN recording_overlay_position TEXT NOT NULL DEFAULT 'top-center'",
            [],
        );

        // Add custom_vocabulary column if it doesn't exist (migration for existing DBs)
        let _ = conn.execute(
            "ALTER TABLE settings ADD COLUMN custom_vocabulary TEXT NOT NULL DEFAULT '[]'",
            [],
        );

        // Add auto_check_for_updates column if it doesn't exist (migration for existing DBs)
        let _ = conn.execute(
            "ALTER TABLE settings ADD COLUMN auto_check_for_updates INTEGER NOT NULL DEFAULT 0",
            [],
        );

        // Add diagnostics_enabled column if it doesn't exist (migration for existing DBs)
        let _ = conn.execute(
            "ALTER TABLE settings ADD COLUMN diagnostics_enabled INTEGER NOT NULL DEFAULT 1",
            [],
        );

        // Add translation settings columns if they don't exist (migration for existing DBs)
        let _ = conn.execute(
            "ALTER TABLE settings ADD COLUMN translation_enabled INTEGER NOT NULL DEFAULT 0",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE settings ADD COLUMN translation_hotkey TEXT NOT NULL DEFAULT 'Alt+Shift+T'",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE settings ADD COLUMN translation_source_language TEXT NOT NULL DEFAULT 'en'",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE settings ADD COLUMN translation_target_language TEXT NOT NULL DEFAULT 'es'",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE settings ADD COLUMN translation_api_key TEXT",
            [],
        );

        // Add AI formatting settings columns if they don't exist (migration for existing DBs)
        let _ = conn.execute(
            "ALTER TABLE settings ADD COLUMN ai_formatting_enabled INTEGER NOT NULL DEFAULT 0",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE settings ADD COLUMN ai_formatting_provider_id TEXT NOT NULL DEFAULT 'openai'",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE settings ADD COLUMN ai_formatting_style TEXT NOT NULL DEFAULT 'clean'",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE settings ADD COLUMN ai_formatting_model TEXT NOT NULL DEFAULT 'gpt-4o-mini'",
            [],
        );

        // Migration: Update default hotkeys if they are still the old ones
        // This ensures existing users get the new non-conflicting defaults
        let _ = conn.execute(
            "UPDATE settings SET push_to_talk_key = 'Alt+Shift+S' WHERE push_to_talk_key = 'Ctrl+Shift+R'",
            [],
        );
        let _ = conn.execute(
            "UPDATE settings SET toggle_key = 'Alt+Shift+D' WHERE toggle_key = 'Ctrl+Shift+T'",
            [],
        );

        // App state table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS app_state (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                is_first_launch INTEGER NOT NULL DEFAULT 1,
                setup_complete INTEGER NOT NULL DEFAULT 0,
                current_setup_step INTEGER NOT NULL DEFAULT 0,
                selected_model_id TEXT,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Models table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS models (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                size TEXT NOT NULL,
                size_bytes INTEGER NOT NULL,
                description TEXT NOT NULL,
                languages TEXT NOT NULL,
                downloaded INTEGER NOT NULL DEFAULT 0,
                download_path TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Transcription history table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS transcription_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                text TEXT NOT NULL,
                model_id TEXT NOT NULL,
                language TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        let _ = conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_transcription_history_created_at ON transcription_history(created_at DESC)",
            [],
        );

        // License table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS license (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                license_key TEXT,
                activation_id TEXT,
                status TEXT NOT NULL DEFAULT 'inactive',
                customer_email TEXT,
                customer_name TEXT,
                expires_at TEXT,
                is_activated INTEGER NOT NULL DEFAULT 0,
                last_validated_at TEXT,
                trial_started_at TEXT,
                trial_integrity_hash TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Migration: add trial_started_at column if it doesn't exist
        let _ = conn.execute("ALTER TABLE license ADD COLUMN trial_started_at TEXT", []);
        let _ = conn.execute(
            "ALTER TABLE license ADD COLUMN trial_integrity_hash TEXT",
            [],
        );

        // Migration: add usage and validations columns if they don't exist
        let _ = conn.execute(
            "ALTER TABLE license ADD COLUMN usage INTEGER NOT NULL DEFAULT 0",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE license ADD COLUMN validations INTEGER NOT NULL DEFAULT 0",
            [],
        );
        // Cloud providers table (BYOK)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS cloud_providers (
                id TEXT PRIMARY KEY,
                api_key TEXT NOT NULL DEFAULT '',
                base_url TEXT,
                custom_model TEXT,
                is_active INTEGER NOT NULL DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Migration: fix any existing cloud_providers table with incorrect schema
        // This handles cases where the table might have been created with wrong column names
        let _ = conn.execute(
            "CREATE TABLE IF NOT EXISTS cloud_providers_new (
                id TEXT PRIMARY KEY,
                api_key TEXT NOT NULL DEFAULT '',
                base_url TEXT,
                custom_model TEXT,
                is_active INTEGER NOT NULL DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        );
        
        // Try to migrate data from old table if it exists with different schema
        let _ = conn.execute(
            "INSERT OR REPLACE INTO cloud_providers_new (id, api_key, base_url, custom_model, is_active, created_at, updated_at)
             SELECT id, api_key, base_url, custom_model, is_active, created_at, updated_at FROM cloud_providers",
            [],
        );
        
        // Drop old table and rename new one
        let _ = conn.execute("DROP TABLE IF EXISTS cloud_providers", []);
        let _ = conn.execute("ALTER TABLE cloud_providers_new RENAME TO cloud_providers", []);

        // AI formatting providers table (BYOK for LLM formatting)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS ai_formatting_providers (
                id TEXT PRIMARY KEY,
                api_key TEXT NOT NULL DEFAULT '',
                base_url TEXT,
                custom_model TEXT,
                is_active INTEGER NOT NULL DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        Ok(())
    }

    fn init_default_data(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Insert default settings if not exists
        conn.execute("INSERT OR IGNORE INTO settings (id) VALUES (1)", [])?;

        // Insert default app state if not exists
        conn.execute("INSERT OR IGNORE INTO app_state (id) VALUES (1)", [])?;

        // Insert default license record if not exists
        conn.execute("INSERT OR IGNORE INTO license (id) VALUES (1)", [])?;

        const REMOVED_MODEL_IDS: &[&str] = &[
            "distil-medium.en",
            "distil-large-v2",
            "distil-large-v3",
            "canary",
            "canary-nvidia",
            "nvidia-canary",
            "nvidia-canary-1b",
            "nvidia-canary-qwen-2.5b",
        ];

        for model_id in REMOVED_MODEL_IDS {
            conn.execute("DELETE FROM models WHERE id = ?1", params![model_id])?;
        }

        // Insert default Whisper models
        let models: Vec<(&str, &str, &str, i64, &str, &str)> = vec![
            (
                "tiny",
                "Whisper Tiny",
                "75 MB",
                75_i64 * 1024 * 1024,
                "Fastest Whisper model. Best for quick notes and low-resource devices.",
                "[\"multilingual\"]",
            ),
            (
                "base",
                "Whisper Base",
                "142 MB",
                142_i64 * 1024 * 1024,
                "Balanced Whisper model for everyday transcription.",
                "[\"multilingual\"]",
            ),
            (
                "small",
                "Whisper Small",
                "466 MB",
                466_i64 * 1024 * 1024,
                "Improved accuracy for longer dictation, meetings, and focused writing.",
                "[\"multilingual\"]",
            ),
            (
                "medium",
                "Whisper Medium",
                "1.5 GB",
                1536_i64 * 1024 * 1024,
                "High-accuracy multilingual transcription for demanding audio.",
                "[\"multilingual\"]",
            ),
            (
                "large-v3",
                "Whisper Large v3",
                "2.9 GB",
                2969_i64 * 1024 * 1024,
                "Highest-accuracy Whisper model for professional workflows.",
                "[\"multilingual\"]",
            ),
            (
                "large-v3-turbo",
                "Whisper Large v3 Turbo",
                "1.6 GB",
                1600_i64 * 1024 * 1024,
                "Fast large Whisper model with a strong speed and accuracy balance.",
                "[\"multilingual\"]",
            ),
            // English-only models (faster)
            (
                "tiny.en",
                "Whisper Tiny English",
                "75 MB",
                75_i64 * 1024 * 1024,
                "Fastest English-only Whisper model. Great for quick notes.",
                "[\"en\"]",
            ),
            (
                "base.en",
                "Whisper Base English",
                "142 MB",
                142_i64 * 1024 * 1024,
                "Fast English-only Whisper model with good accuracy.",
                "[\"en\"]",
            ),
            (
                "small.en",
                "Whisper Small English",
                "466 MB",
                466_i64 * 1024 * 1024,
                "Accurate English-only Whisper model for longer dictation.",
                "[\"en\"]",
            ),
            (
                "medium.en",
                "Whisper Medium English",
                "1.5 GB",
                1536_i64 * 1024 * 1024,
                "High-accuracy English-only Whisper model.",
                "[\"en\"]",
            ),
            // Distil-Whisper models (faster)
            (
                "distil-small.en",
                "Distil Whisper Small English",
                "166 MB",
                166_i64 * 1024 * 1024,
                "Fast English transcription with accuracy close to Whisper Small.",
                "[\"en\"]",
            ),
            (
                "parakeet-v3",
                "Parakeet v3",
                "670 MB",
                670_i64 * 1024 * 1024,
                "Fast multilingual Parakeet model with automatic language detection.",
                "[\"bg\", \"hr\", \"cs\", \"da\", \"nl\", \"en\", \"et\", \"fi\", \"fr\", \"de\", \"el\", \"hu\", \"it\", \"lv\", \"lt\", \"mt\", \"pl\", \"pt\", \"ro\", \"sk\", \"sl\", \"es\", \"sv\", \"ru\", \"uk\"]",
            ),
            (
                "parakeet-v2",
                "Parakeet v2",
                "661 MB",
                661_i64 * 1024 * 1024,
                "Previous Parakeet English model with stable transcription quality.",
                "[\"en\"]",
            ),
            (
                "qwen3-asr-0.6b",
                "Qwen3-ASR 0.6B",
                "1.9 GB",
                1880_i64 * 1024 * 1024,
                "Qwen3-ASR speech recognition model for accurate multilingual transcription.",
                "[\"zh\", \"en\", \"yue\", \"ar\", \"de\", \"fr\", \"es\", \"pt\", \"id\", \"it\", \"ko\", \"ru\", \"th\", \"vi\", \"ja\", \"tr\", \"hi\", \"ms\", \"nl\", \"sv\", \"da\", \"fi\", \"pl\", \"cs\", \"fil\", \"fa\", \"el\", \"hu\", \"mk\", \"ro\"]",
            ),
        ];

        for (id, name, size, size_bytes, description, languages) in models {
            conn.execute(
                "INSERT INTO models (id, name, size, size_bytes, description, languages)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(id) DO UPDATE SET
                    name = excluded.name,
                    size = excluded.size,
                    size_bytes = excluded.size_bytes,
                    description = excluded.description,
                    languages = excluded.languages",
                params![id, name, size, size_bytes, description, languages],
            )?;
        }

        Ok(())
    }

    // Settings operations
    pub fn get_settings(&self) -> Result<AppSettings> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT push_to_talk_key, toggle_key, hotkey_mode, language, selected_model_id,
                    show_recording_indicator, show_recording_overlay, play_audio_feedback, auto_start_on_boot, minimize_to_tray,
                    post_processing_enabled, voice_commands_enabled, clipboard_mode, auto_check_for_updates, recording_overlay_position, custom_vocabulary, diagnostics_enabled,
                    translation_enabled, translation_hotkey, translation_source_language, translation_target_language, translation_api_key,
                    ai_formatting_enabled, ai_formatting_provider_id, ai_formatting_style, ai_formatting_model
                 FROM settings WHERE id = 1",
            [],
            |row| {
                let vocab_json: String = row.get(15).unwrap_or_else(|_| "[]".to_string());
                let custom_vocabulary = serde_json::from_str(&vocab_json)
                    .unwrap_or_else(|_| Vec::new());
                Ok(AppSettings {
                    push_to_talk_key: row.get(0)?,
                    toggle_key: row.get(1)?,
                    hotkey_mode: row.get(2)?,
                    language: row.get(3)?,
                    selected_model_id: row.get(4)?,
                    show_recording_indicator: row.get::<_, i32>(5)? == 1,
                    show_recording_overlay: row.get::<_, i32>(6).unwrap_or(1) == 1,
                    play_audio_feedback: row.get::<_, i32>(7)? == 1,
                    auto_start_on_boot: row.get::<_, i32>(8)? == 1,
                    minimize_to_tray: row.get::<_, i32>(9)? == 1,
                    post_processing_enabled: row.get::<_, i32>(10)? == 1,
                    voice_commands_enabled: row.get::<_, i32>(11)? == 1,
                    clipboard_mode: row.get::<_, i32>(12)? == 1,
                    auto_check_for_updates: row.get::<_, i32>(13).unwrap_or(1) == 1,
                    recording_overlay_position: row.get(14).unwrap_or_else(|_| "top-center".to_string()),
                    custom_vocabulary,
                    diagnostics_enabled: row.get::<_, i32>(16).unwrap_or(1) == 1,
                    translation_enabled: row.get::<_, i32>(17).unwrap_or(0) == 1,
                    translation_hotkey: row.get(18).unwrap_or_else(|_| "Alt+Shift+T".to_string()),
                    translation_source_language: row.get(19).unwrap_or_else(|_| "en".to_string()),
                    translation_target_language: row.get(20).unwrap_or_else(|_| "es".to_string()),
                    translation_api_key: row.get(21).unwrap_or(None),
                    ai_formatting_enabled: row.get::<_, i32>(22).unwrap_or(0) == 1,
                    ai_formatting_provider_id: row.get(23).unwrap_or_else(|_| "openai".to_string()),
                    ai_formatting_style: row.get(24).unwrap_or_else(|_| "clean".to_string()),
                    ai_formatting_model: row.get(25).unwrap_or_else(|_| "gpt-4o-mini".to_string()),
                })
            },
        )
    }

    pub fn update_settings(&self, settings: &AppSettings) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let vocab_json = serde_json::to_string(&settings.custom_vocabulary)
            .unwrap_or_else(|_| "[]".to_string());
        conn.execute(
            "UPDATE settings SET
                push_to_talk_key = ?1,
                toggle_key = ?2,
                hotkey_mode = ?3,
                language = ?4,
                selected_model_id = ?5,
                show_recording_indicator = ?6,
                show_recording_overlay = ?7,
                play_audio_feedback = ?8,
                auto_start_on_boot = ?9,
                minimize_to_tray = ?10,
                post_processing_enabled = ?11,
                voice_commands_enabled = ?12,
                clipboard_mode = ?13,
                auto_check_for_updates = ?14,
                recording_overlay_position = ?15,
                custom_vocabulary = ?16,
                diagnostics_enabled = ?17,
                translation_enabled = ?18,
                translation_hotkey = ?19,
                translation_source_language = ?20,
                translation_target_language = ?21,
                translation_api_key = ?22,
                ai_formatting_enabled = ?23,
                ai_formatting_provider_id = ?24,
                ai_formatting_style = ?25,
                ai_formatting_model = ?26,
                updated_at = CURRENT_TIMESTAMP
             WHERE id = 1",
            params![
                settings.push_to_talk_key,
                settings.toggle_key,
                settings.hotkey_mode,
                settings.language,
                settings.selected_model_id,
                settings.show_recording_indicator as i32,
                settings.show_recording_overlay as i32,
                settings.play_audio_feedback as i32,
                settings.auto_start_on_boot as i32,
                settings.minimize_to_tray as i32,
                settings.post_processing_enabled as i32,
                settings.voice_commands_enabled as i32,
                settings.clipboard_mode as i32,
                settings.auto_check_for_updates as i32,
                settings.recording_overlay_position,
                vocab_json,
                settings.diagnostics_enabled as i32,
                settings.translation_enabled as i32,
                settings.translation_hotkey,
                settings.translation_source_language,
                settings.translation_target_language,
                 settings.translation_api_key,
                 settings.ai_formatting_enabled as i32,
                 settings.ai_formatting_provider_id,
                 settings.ai_formatting_style,
                 settings.ai_formatting_model,
             ],
        )?;
        Ok(())
    }

    pub fn update_setting(&self, key: &str, value: &str) -> Result<()> {
        // Security: Whitelist allowed column names to prevent SQL injection
        const ALLOWED_KEYS: &[&str] = &[
            "push_to_talk_key",
            "toggle_key",
            "hotkey_mode",
            "language",
            "selected_model_id",
            "show_recording_indicator",
            "show_recording_overlay",
            "play_audio_feedback",
            "auto_start_on_boot",
            "minimize_to_tray",
            "post_processing_enabled",
            "voice_commands_enabled",
            "clipboard_mode",
            "auto_check_for_updates",
            "recording_overlay_position",
            "diagnostics_enabled",
            "translation_enabled",
            "translation_hotkey",
            "translation_source_language",
            "translation_target_language",
            "translation_api_key",
            "ai_formatting_enabled",
            "ai_formatting_provider_id",
            "ai_formatting_style",
            "ai_formatting_model",
        ];

        if !ALLOWED_KEYS.contains(&key) {
            return Err(rusqlite::Error::InvalidParameterName(format!(
                "Invalid setting key: {}",
                key
            )));
        }

        let conn = self.conn.lock().unwrap();
        let query = format!(
            "UPDATE settings SET {} = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = 1",
            key
        );
        conn.execute(&query, params![value])?;
        Ok(())
    }

    // App state operations
    pub fn get_app_state(&self) -> Result<AppState> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT is_first_launch, setup_complete, current_setup_step, selected_model_id
             FROM app_state WHERE id = 1",
            [],
            |row| {
                Ok(AppState {
                    is_first_launch: row.get::<_, i32>(0)? == 1,
                    setup_complete: row.get::<_, i32>(1)? == 1,
                    current_setup_step: row.get(2)?,
                    selected_model_id: row.get(3)?,
                })
            },
        )
    }

    pub fn update_app_state(&self, state: &AppState) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE app_state SET
                is_first_launch = ?1,
                setup_complete = ?2,
                current_setup_step = ?3,
                selected_model_id = ?4,
                updated_at = CURRENT_TIMESTAMP
             WHERE id = 1",
            params![
                state.is_first_launch as i32,
                state.setup_complete as i32,
                state.current_setup_step,
                state.selected_model_id,
            ],
        )?;
        Ok(())
    }

    pub fn set_setup_complete(&self, complete: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE app_state SET setup_complete = ?1, is_first_launch = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = 1",
            params![complete as i32, (!complete) as i32],
        )?;
        Ok(())
    }

    pub fn set_current_setup_step(&self, step: i32) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE app_state SET current_setup_step = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = 1",
            params![step],
        )?;
        Ok(())
    }

    // Model operations
    pub fn get_models(&self) -> Result<Vec<WhisperModel>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, size, size_bytes, description, languages, downloaded, download_path
             FROM models ORDER BY size_bytes ASC",
        )?;

        let models = stmt
            .query_map([], |row| {
                Ok(WhisperModel {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    size: row.get(2)?,
                    size_bytes: row.get(3)?,
                    description: row.get(4)?,
                    languages: row.get(5)?,
                    downloaded: row.get::<_, i32>(6)? == 1,
                    download_path: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(models)
    }

    pub fn get_model(&self, id: &str) -> Result<Option<WhisperModel>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, size, size_bytes, description, languages, downloaded, download_path
             FROM models WHERE id = ?1",
        )?;

        let model = stmt
            .query_row(params![id], |row| {
                Ok(WhisperModel {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    size: row.get(2)?,
                    size_bytes: row.get(3)?,
                    description: row.get(4)?,
                    languages: row.get(5)?,
                    downloaded: row.get::<_, i32>(6)? == 1,
                    download_path: row.get(7)?,
                })
            })
            .ok();

        Ok(model)
    }

    pub fn set_model_downloaded(
        &self,
        id: &str,
        downloaded: bool,
        path: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE models SET downloaded = ?1, download_path = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?3",
            params![downloaded as i32, path, id],
        )?;
        Ok(())
    }

    pub fn set_selected_model(&self, model_id: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE app_state SET selected_model_id = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = 1",
            params![model_id],
        )?;
        // Also update in settings
        if let Some(id) = model_id {
            conn.execute(
                "UPDATE settings SET selected_model_id = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = 1",
                params![id],
            )?;
        }
        Ok(())
    }

    // Transcription history operations
    fn escape_like_pattern(search: &str) -> String {
        search
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    }

    pub fn add_transcription(
        &self,
        text: &str,
        model_id: &str,
        language: &str,
        duration_ms: i64,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO transcription_history (text, model_id, language, duration_ms)
             VALUES (?1, ?2, ?3, ?4)",
            params![text, model_id, language, duration_ms],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_transcription_history(
        &self,
        limit: i32,
        offset: i32,
        search: Option<&str>,
    ) -> Result<Vec<TranscriptionHistory>> {
        let conn = self.conn.lock().unwrap();
        let search = search.unwrap_or("").trim();
        let pattern = format!("%{}%", Self::escape_like_pattern(search));
        let mut stmt = conn.prepare(
            "SELECT id, text, model_id, language, duration_ms, created_at
             FROM transcription_history
             WHERE (?3 = '' OR text LIKE ?4 ESCAPE '\\')
             ORDER BY created_at DESC
             LIMIT ?1 OFFSET ?2",
        )?;

        let history = stmt
            .query_map(params![limit, offset, search, pattern], |row| {
                Ok(TranscriptionHistory {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    model_id: row.get(2)?,
                    language: row.get(3)?,
                    duration_ms: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(history)
    }

    pub fn get_transcription_history_count(&self, search: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let search = search.unwrap_or("").trim();
        let pattern = format!("%{}%", Self::escape_like_pattern(search));
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transcription_history
             WHERE (?1 = '' OR text LIKE ?2 ESCAPE '\\')",
            params![search, pattern],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn clear_transcription_history(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM transcription_history", [])?;
        Ok(())
    }

    pub fn delete_transcription(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM transcription_history WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    // License operations
    pub fn get_license(&self) -> Result<LicenseData> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT license_key, activation_id, status, customer_email, customer_name, 
                    expires_at, is_activated, last_validated_at, trial_started_at,
                    trial_integrity_hash, usage, validations
             FROM license WHERE id = 1",
            [],
            |row| {
                Ok(LicenseData {
                    license_key: row.get(0)?,
                    activation_id: row.get(1)?,
                    status: row.get(2)?,
                    customer_email: row.get(3)?,
                    customer_name: row.get(4)?,
                    expires_at: row.get(5)?,
                    is_activated: row.get::<_, i32>(6)? != 0,
                    last_validated_at: row.get(7)?,
                    trial_started_at: row.get(8)?,
                    trial_integrity_hash: row.get(9)?,
                    usage: row.get(10)?,
                    validations: row.get(11)?,
                })
            },
        )
    }

    pub fn save_license(&self, license: &LicenseData) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE license SET 
                license_key = ?1,
                activation_id = ?2,
                status = ?3,
                customer_email = ?4,
                customer_name = ?5,
                expires_at = ?6,
                is_activated = ?7,
                last_validated_at = ?8,
                trial_started_at = ?9,
                trial_integrity_hash = ?10,
                usage = ?11,
                validations = ?12,
                updated_at = CURRENT_TIMESTAMP
             WHERE id = 1",
            params![
                license.license_key,
                license.activation_id,
                license.status,
                license.customer_email,
                license.customer_name,
                license.expires_at,
                license.is_activated as i32,
                license.last_validated_at,
                license.trial_started_at,
                license.trial_integrity_hash,
                license.usage,
                license.validations,
            ],
        )?;
        Ok(())
    }

    pub fn clear_license(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        // IMPORTANT: Preserve trial_started_at to prevent trial abuse
        // Users who have used their trial should not be able to restart it
        conn.execute(
            "UPDATE license SET 
                license_key = NULL,
                activation_id = NULL,
                status = CASE 
                    WHEN trial_started_at IS NOT NULL THEN 'trial_expired'
                    ELSE 'inactive'
                END,
                customer_email = NULL,
                customer_name = NULL,
                expires_at = NULL,
                is_activated = 0,
                last_validated_at = NULL,
                usage = 0,
                validations = 0,
                -- trial_started_at and trial_integrity_hash are preserved intentionally
                updated_at = CURRENT_TIMESTAMP
             WHERE id = 1",
            [],
        )?;
        Ok(())
    }

    // ==================== Cloud Provider Operations (BYOK) ====================

    pub fn get_cloud_providers(&self) -> Result<Vec<CloudProviderRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, api_key, base_url, custom_model, is_active FROM cloud_providers ORDER BY id ASC"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(CloudProviderRecord {
                id: row.get(0)?,
                api_key: row.get(1)?,
                base_url: row.get(2)?,
                custom_model: row.get(3)?,
                is_active: row.get::<_, i32>(4)? == 1,
            })
        })?;

        let mut providers = Vec::new();
        for row in rows {
            providers.push(row?);
        }
        Ok(providers)
    }

    pub fn get_cloud_provider(&self, id: &str) -> Result<Option<CloudProviderRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, api_key, base_url, custom_model, is_active FROM cloud_providers WHERE id = ?1"
        )?;

        let mut rows = stmt.query_map(params![id], |row| {
            Ok(CloudProviderRecord {
                id: row.get(0)?,
                api_key: row.get(1)?,
                base_url: row.get(2)?,
                custom_model: row.get(3)?,
                is_active: row.get::<_, i32>(4)? == 1,
            })
        })?;

        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    pub fn save_cloud_provider(
        &self,
        id: &str,
        api_key: &str,
        base_url: Option<&str>,
        custom_model: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO cloud_providers (id, api_key, base_url, custom_model, is_active, updated_at)
             VALUES (?1, ?2, ?3, ?4, 1, CURRENT_TIMESTAMP)
             ON CONFLICT(id) DO UPDATE SET
                api_key = excluded.api_key,
                base_url = excluded.base_url,
                custom_model = excluded.custom_model,
                is_active = 1,
                updated_at = CURRENT_TIMESTAMP",
            params![id, api_key, base_url, custom_model],
        )?;
        Ok(())
    }

    pub fn delete_cloud_provider(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM cloud_providers WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ==================== AI Formatting Provider Operations (BYOK) ====================

    pub fn get_ai_formatting_providers(&self) -> Result<Vec<AiFormattingProviderRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, api_key, base_url, custom_model, is_active FROM ai_formatting_providers ORDER BY id ASC"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(AiFormattingProviderRecord {
                id: row.get(0)?,
                api_key: row.get(1)?,
                base_url: row.get(2)?,
                custom_model: row.get(3)?,
                is_active: row.get::<_, i32>(4)? == 1,
            })
        })?;

        let mut providers = Vec::new();
        for row in rows {
            providers.push(row?);
        }
        Ok(providers)
    }

    pub fn get_ai_formatting_provider(&self, id: &str) -> Result<Option<AiFormattingProviderRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, api_key, base_url, custom_model, is_active FROM ai_formatting_providers WHERE id = ?1"
        )?;

        let mut rows = stmt.query_map(params![id], |row| {
            Ok(AiFormattingProviderRecord {
                id: row.get(0)?,
                api_key: row.get(1)?,
                base_url: row.get(2)?,
                custom_model: row.get(3)?,
                is_active: row.get::<_, i32>(4)? == 1,
            })
        })?;

        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    pub fn save_ai_formatting_provider(
        &self,
        id: &str,
        api_key: &str,
        base_url: Option<&str>,
        custom_model: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO ai_formatting_providers (id, api_key, base_url, custom_model, is_active, updated_at)
             VALUES (?1, ?2, ?3, ?4, 1, CURRENT_TIMESTAMP)
             ON CONFLICT(id) DO UPDATE SET
                api_key = excluded.api_key,
                base_url = excluded.base_url,
                custom_model = excluded.custom_model,
                is_active = 1,
                updated_at = CURRENT_TIMESTAMP",
            params![id, api_key, base_url, custom_model],
        )?;
        Ok(())
    }

    pub fn delete_ai_formatting_provider(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM ai_formatting_providers WHERE id = ?1", params![id])?;
        Ok(())
    }
}
