use rusqlite::Result;
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Database {
    pub(crate) conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app_data_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&app_data_dir).ok();
        let db_path = app_data_dir.join("Whisprtypr.db");
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
                translation_enabled INTEGER NOT NULL DEFAULT 0,
                translation_hotkey TEXT NOT NULL DEFAULT 'Alt+Shift+T',
                translation_source_language TEXT NOT NULL DEFAULT 'en',
                translation_target_language TEXT NOT NULL DEFAULT 'es',
                translation_api_key TEXT,
                ai_formatting_enabled INTEGER NOT NULL DEFAULT 0,
                ai_formatting_provider_id TEXT NOT NULL DEFAULT 'openai',
                ai_formatting_style TEXT NOT NULL DEFAULT 'clean',
                ai_formatting_model TEXT NOT NULL DEFAULT 'gpt-4o-mini',
                grammar_check_enabled INTEGER NOT NULL DEFAULT 1,
                grammar_check_dialect TEXT NOT NULL DEFAULT 'american',
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

        // Add grammar check settings columns if they don't exist (migration for existing DBs)
        let _ = conn.execute(
            "ALTER TABLE settings ADD COLUMN grammar_check_enabled INTEGER NOT NULL DEFAULT 1",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE settings ADD COLUMN grammar_check_dialect TEXT NOT NULL DEFAULT 'american'",
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
                trial_salt TEXT,
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
        let _ = conn.execute("ALTER TABLE license ADD COLUMN trial_salt TEXT", []);

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
        let _ = conn.execute(
            "ALTER TABLE cloud_providers_new RENAME TO cloud_providers",
            [],
        );

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
}
