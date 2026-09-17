use rusqlite::Connection;
use whisprtypr_lib::database::{AppSettings, AppState, Database, LicenseData};

#[test]
fn database_state_survives_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().to_path_buf();

    {
        let db = Database::new(db_path.clone()).unwrap();
        db.update_settings(&AppSettings {
            push_to_talk_key: "Ctrl+Alt+Space".to_string(),
            toggle_key: "Ctrl+Alt+T".to_string(),
            hotkey_mode: "toggle".to_string(),
            language: "fr".to_string(),
            selected_model_id: "small".to_string(),
            show_recording_indicator: false,
            show_recording_overlay: false,
            play_audio_feedback: false,
            auto_start_on_boot: true,
            minimize_to_tray: false,
            post_processing_enabled: false,
            voice_commands_enabled: true,
            clipboard_mode: true,
            ..Default::default()
        })
        .unwrap();
        db.update_app_state(&AppState {
            is_first_launch: false,
            setup_complete: true,
            current_setup_step: 5,
            selected_model_id: Some("small".to_string()),
        })
        .unwrap();
        db.set_model_downloaded("small", true, Some("C:\\models\\small.bin"))
            .unwrap();
        db.add_transcription("persist me", "small", "fr", 321)
            .unwrap();
        db.save_license(&LicenseData {
            license_key: Some("WVT-PERSIST".to_string()),
            activation_id: Some("activation_persist".to_string()),
            status: "active".to_string(),
            customer_email: Some("persistence-customer@whisprtypr.test".to_string()),
            customer_name: Some("Persist User".to_string()),
            expires_at: None,
            is_activated: true,
            last_validated_at: Some("2026-04-19T00:00:00+00:00".to_string()),
            trial_started_at: Some("2026-04-01T00:00:00+00:00".to_string()),
            trial_integrity_hash: Some("trial-hash".to_string()),
            usage: 7,
            validations: 8,
        })
        .unwrap();
    }

    let reopened = Database::new(db_path).unwrap();

    let settings = reopened.get_settings().unwrap();
    assert_eq!(settings.hotkey_mode, "toggle");
    assert_eq!(settings.language, "fr");
    assert!(settings.clipboard_mode);

    let state = reopened.get_app_state().unwrap();
    assert!(state.setup_complete);
    assert_eq!(state.current_setup_step, 5);

    let model = reopened.get_model("small").unwrap().unwrap();
    assert!(model.downloaded);
    assert_eq!(
        model.download_path,
        Some("C:\\models\\small.bin".to_string())
    );

    assert_eq!(reopened.get_transcription_history_count(None).unwrap(), 1);
    assert_eq!(
        reopened.get_transcription_history(10, 0, None).unwrap()[0].text,
        "persist me"
    );

    let license = reopened.get_license().unwrap();
    assert_eq!(license.status, "active");
    assert_eq!(license.license_key, Some("WVT-PERSIST".to_string()));
    assert_eq!(license.usage, 7);
    assert_eq!(license.validations, 8);
}

#[test]
fn legacy_database_is_migrated_without_losing_existing_state() {
    let dir = tempfile::tempdir().unwrap();
    let conn = Connection::open(dir.path().join("Whisprtypr.db")).unwrap();

    conn.execute_batch(
        "
        CREATE TABLE settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            push_to_talk_key TEXT NOT NULL DEFAULT 'Ctrl+Shift+R',
            toggle_key TEXT NOT NULL DEFAULT 'Ctrl+Shift+T',
            hotkey_mode TEXT NOT NULL DEFAULT 'push-to-talk',
            language TEXT NOT NULL DEFAULT 'en',
            selected_model_id TEXT NOT NULL DEFAULT 'base',
            show_recording_indicator INTEGER NOT NULL DEFAULT 1,
            play_audio_feedback INTEGER NOT NULL DEFAULT 1,
            auto_start_on_boot INTEGER NOT NULL DEFAULT 0,
            minimize_to_tray INTEGER NOT NULL DEFAULT 1
        );
        INSERT INTO settings (id, push_to_talk_key, toggle_key) VALUES (1, 'Ctrl+Shift+R', 'Ctrl+Shift+T');

        CREATE TABLE app_state (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            is_first_launch INTEGER NOT NULL DEFAULT 1,
            setup_complete INTEGER NOT NULL DEFAULT 0,
            current_setup_step INTEGER NOT NULL DEFAULT 0,
            selected_model_id TEXT
        );
        INSERT INTO app_state (id, setup_complete, current_setup_step) VALUES (1, 1, 3);

        CREATE TABLE models (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            size TEXT NOT NULL,
            size_bytes INTEGER NOT NULL,
            description TEXT NOT NULL,
            languages TEXT NOT NULL,
            downloaded INTEGER NOT NULL DEFAULT 0,
            download_path TEXT
        );
        INSERT INTO models (id, name, size, size_bytes, description, languages)
        VALUES ('canary', 'Removed Canary', '1 GB', 1024, 'removed', '[\"en\"]');

        CREATE TABLE transcription_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            text TEXT NOT NULL,
            model_id TEXT NOT NULL,
            language TEXT NOT NULL,
            duration_ms INTEGER NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE license (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            license_key TEXT,
            activation_id TEXT,
            status TEXT NOT NULL DEFAULT 'inactive',
            customer_email TEXT,
            customer_name TEXT,
            expires_at TEXT,
            is_activated INTEGER NOT NULL DEFAULT 0,
            last_validated_at TEXT
        );
        INSERT INTO license (id, license_key, activation_id, status, is_activated, last_validated_at)
        VALUES (1, 'WVT-OLD', 'activation_old', 'active', 1, '2026-04-19T00:00:00+00:00');
        ",
    )
    .unwrap();
    drop(conn);

    let db = Database::new(dir.path().to_path_buf()).unwrap();

    let settings = db.get_settings().unwrap();
    assert_eq!(settings.push_to_talk_key, "Alt+Shift+S");
    assert_eq!(settings.toggle_key, "Alt+Shift+D");
    assert!(settings.post_processing_enabled);
    assert!(!settings.voice_commands_enabled);
    assert!(!settings.clipboard_mode);
    assert!(settings.show_recording_overlay);

    let state = db.get_app_state().unwrap();
    assert!(state.setup_complete);
    assert_eq!(state.current_setup_step, 3);

    assert!(db.get_model("canary").unwrap().is_none());
    assert!(db.get_model("qwen3-asr-0.6b").unwrap().is_some());

    let license = db.get_license().unwrap();
    assert_eq!(license.license_key, Some("WVT-OLD".to_string()));
    assert_eq!(license.activation_id, Some("activation_old".to_string()));
    assert_eq!(license.trial_started_at, None);
    assert_eq!(license.usage, 0);
    assert_eq!(license.validations, 0);
}
