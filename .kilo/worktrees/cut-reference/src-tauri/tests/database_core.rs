use vox_ai_lib::database::{AppSettings, AppState, Database};

fn test_database() -> (tempfile::TempDir, Database) {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::new(dir.path().to_path_buf()).unwrap();
    (dir, db)
}

#[test]
fn default_settings_are_created() {
    let (_dir, db) = test_database();

    let settings = db.get_settings().unwrap();

    assert_eq!(settings.push_to_talk_key, "Alt+Shift+S");
    assert_eq!(settings.toggle_key, "Alt+Shift+D");
    assert_eq!(settings.hotkey_mode, "push-to-talk");
    assert_eq!(settings.language, "en");
    assert_eq!(settings.selected_model_id, "base");
    assert!(settings.show_recording_indicator);
    assert!(settings.show_recording_overlay);
    assert!(settings.play_audio_feedback);
    assert!(!settings.auto_start_on_boot);
    assert!(settings.minimize_to_tray);
    assert!(settings.post_processing_enabled);
    assert!(!settings.voice_commands_enabled);
    assert!(!settings.clipboard_mode);
}

#[test]
fn update_settings_persists_every_field() {
    let (_dir, db) = test_database();
    let settings = AppSettings {
        push_to_talk_key: "Ctrl+Alt+Space".to_string(),
        toggle_key: "Ctrl+Alt+T".to_string(),
        hotkey_mode: "toggle".to_string(),
        language: "es".to_string(),
        selected_model_id: "small".to_string(),
        show_recording_indicator: false,
        show_recording_overlay: false,
        play_audio_feedback: false,
        auto_start_on_boot: true,
        minimize_to_tray: false,
        post_processing_enabled: false,
        voice_commands_enabled: true,
        clipboard_mode: true,
    };

    db.update_settings(&settings).unwrap();
    let stored = db.get_settings().unwrap();

    assert_eq!(stored.push_to_talk_key, "Ctrl+Alt+Space");
    assert_eq!(stored.toggle_key, "Ctrl+Alt+T");
    assert_eq!(stored.hotkey_mode, "toggle");
    assert_eq!(stored.language, "es");
    assert_eq!(stored.selected_model_id, "small");
    assert!(!stored.show_recording_indicator);
    assert!(!stored.show_recording_overlay);
    assert!(!stored.play_audio_feedback);
    assert!(stored.auto_start_on_boot);
    assert!(!stored.minimize_to_tray);
    assert!(!stored.post_processing_enabled);
    assert!(stored.voice_commands_enabled);
    assert!(stored.clipboard_mode);
}

#[test]
fn update_setting_rejects_unknown_keys() {
    let (_dir, db) = test_database();

    let result = db.update_setting("selected_model_id = 'small', language", "oops");

    assert!(result.is_err());
    let settings = db.get_settings().unwrap();
    assert_eq!(settings.selected_model_id, "base");
}

#[test]
fn app_state_lifecycle_persists_setup_progress() {
    let (_dir, db) = test_database();

    let state = db.get_app_state().unwrap();
    assert!(state.is_first_launch);
    assert!(!state.setup_complete);
    assert_eq!(state.current_setup_step, 0);
    assert!(state.selected_model_id.is_none());

    db.update_app_state(&AppState {
        is_first_launch: false,
        setup_complete: true,
        current_setup_step: 4,
        selected_model_id: Some("small".to_string()),
    })
    .unwrap();

    let stored = db.get_app_state().unwrap();
    assert!(!stored.is_first_launch);
    assert!(stored.setup_complete);
    assert_eq!(stored.current_setup_step, 4);
    assert_eq!(stored.selected_model_id, Some("small".to_string()));
}

#[test]
fn setup_helpers_keep_first_launch_in_sync() {
    let (_dir, db) = test_database();

    db.set_setup_complete(true).unwrap();
    let state = db.get_app_state().unwrap();
    assert!(state.setup_complete);
    assert!(!state.is_first_launch);

    db.set_setup_complete(false).unwrap();
    let state = db.get_app_state().unwrap();
    assert!(!state.setup_complete);
    assert!(state.is_first_launch);

    db.set_current_setup_step(3).unwrap();
    assert_eq!(db.get_app_state().unwrap().current_setup_step, 3);
}

#[test]
fn selected_model_updates_app_state_and_settings() {
    let (_dir, db) = test_database();

    db.set_selected_model(Some("small")).unwrap();

    assert_eq!(
        db.get_app_state().unwrap().selected_model_id,
        Some("small".to_string())
    );
    assert_eq!(db.get_settings().unwrap().selected_model_id, "small");

    db.set_selected_model(None).unwrap();
    assert!(db.get_app_state().unwrap().selected_model_id.is_none());
    assert_eq!(db.get_settings().unwrap().selected_model_id, "small");
}

#[test]
fn default_models_are_seeded_and_sorted_by_size() {
    let (_dir, db) = test_database();

    let models = db.get_models().unwrap();

    assert!(models.len() >= 10);
    assert_eq!(models.first().unwrap().id, "tiny");
    assert!(models
        .windows(2)
        .all(|pair| pair[0].size_bytes <= pair[1].size_bytes));
    assert!(models.iter().any(|model| model.id == "qwen3-asr-0.6b"));
}

#[test]
fn model_download_state_round_trips() {
    let (_dir, db) = test_database();

    db.set_model_downloaded("base", true, Some("C:\\models\\base.bin"))
        .unwrap();
    let model = db.get_model("base").unwrap().unwrap();

    assert!(model.downloaded);
    assert_eq!(
        model.download_path,
        Some("C:\\models\\base.bin".to_string())
    );

    db.set_model_downloaded("base", false, None).unwrap();
    let model = db.get_model("base").unwrap().unwrap();
    assert!(!model.downloaded);
    assert!(model.download_path.is_none());
}
