use rusqlite::{params, Result};

use super::{AppSettings, Database};

pub fn get_settings(db: &Database) -> Result<AppSettings> {
    let conn = db.conn.lock().unwrap();
    conn.query_row(
        "SELECT push_to_talk_key, toggle_key, hotkey_mode, language, selected_model_id,
                    show_recording_indicator, show_recording_overlay, play_audio_feedback, auto_start_on_boot, minimize_to_tray,
                    post_processing_enabled, voice_commands_enabled, clipboard_mode, auto_check_for_updates, recording_overlay_position, custom_vocabulary, diagnostics_enabled,
                    translation_enabled, translation_hotkey, translation_source_language, translation_target_language, translation_api_key,
                    ai_formatting_enabled, ai_formatting_provider_id, ai_formatting_style, ai_formatting_model,
                    grammar_check_enabled, grammar_check_dialect
                 FROM settings WHERE id = 1",
        [],
        |row| {
            let vocab_json: String = row.get(15).unwrap_or_else(|_| "[]".to_string());
            let custom_vocabulary =
                serde_json::from_str(&vocab_json).unwrap_or_else(|_| Vec::new());
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
                recording_overlay_position: row
                    .get(14)
                    .unwrap_or_else(|_| "top-center".to_string()),
                custom_vocabulary,
                diagnostics_enabled: row.get::<_, i32>(16).unwrap_or(1) == 1,
                translation_enabled: row.get::<_, i32>(17).unwrap_or(0) == 1,
                translation_hotkey: row
                    .get(18)
                    .unwrap_or_else(|_| "Alt+Shift+T".to_string()),
                translation_source_language: row
                    .get(19)
                    .unwrap_or_else(|_| "en".to_string()),
                translation_target_language: row
                    .get(20)
                    .unwrap_or_else(|_| "es".to_string()),
                translation_api_key: row.get(21).unwrap_or(None),
                ai_formatting_enabled: row.get::<_, i32>(22).unwrap_or(0) == 1,
                ai_formatting_provider_id: row
                    .get(23)
                    .unwrap_or_else(|_| "openai".to_string()),
                ai_formatting_style: row
                    .get(24)
                    .unwrap_or_else(|_| "clean".to_string()),
                ai_formatting_model: row
                    .get(25)
                    .unwrap_or_else(|_| "gpt-4o-mini".to_string()),
                grammar_check_enabled: row.get::<_, i32>(26).unwrap_or(1) == 1,
                grammar_check_dialect: row
                    .get(27)
                    .unwrap_or_else(|_| "american".to_string()),
            })
        },
    )
}

pub fn update_settings(db: &Database, settings: &AppSettings) -> Result<()> {
    let conn = db.conn.lock().unwrap();
    let vocab_json =
        serde_json::to_string(&settings.custom_vocabulary).unwrap_or_else(|_| "[]".to_string());
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
                grammar_check_enabled = ?27,
                grammar_check_dialect = ?28,
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
            settings.grammar_check_enabled as i32,
            settings.grammar_check_dialect,
        ],
    )?;
    Ok(())
}

pub fn update_setting(db: &Database, key: &str, value: &str) -> Result<()> {
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
        "grammar_check_enabled",
        "grammar_check_dialect",
    ];

    if !ALLOWED_KEYS.contains(&key) {
        return Err(rusqlite::Error::InvalidParameterName(format!(
            "Invalid setting key: {}",
            key
        )));
    }

    let conn = db.conn.lock().unwrap();
    let query = format!(
        "UPDATE settings SET {} = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = 1",
        key
    );
    conn.execute(&query, params![value])?;
    Ok(())
}
