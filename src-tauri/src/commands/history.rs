use crate::{database::TranscriptionHistory, CommandResult, DbState};
use tauri::State;

#[tauri::command]
pub fn add_transcription(
    db: State<DbState>,
    text: String,
    model_id: String,
    language: String,
    duration_ms: i64,
) -> CommandResult<i64> {
    let sanitized_text = crate::utils::sanitize_text(&text, 1_000_000)
        .map_err(|e| crate::CommandError::Database(rusqlite::Error::InvalidParameterName(e)))?;

    if sanitized_text.is_empty() {
        return Err(crate::CommandError::Database(
            rusqlite::Error::InvalidParameterName("Text cannot be empty".to_string()),
        ));
    }

    const VALID_MODEL_IDS: &[&str] = &[
        "tiny",
        "base",
        "small",
        "medium",
        "large",
        "large-v3",
        "large-v3-turbo",
        "tiny.en",
        "base.en",
        "small.en",
        "medium.en",
        "distil-small.en",
        "parakeet-v2",
        "parakeet-v3",
        "qwen3-asr-0.6b",
    ];

    let is_valid = VALID_MODEL_IDS.contains(&model_id.as_str()) || model_id.starts_with("cloud:");

    if !is_valid {
        return Err(crate::CommandError::Database(
            rusqlite::Error::InvalidParameterName("Invalid model ID".to_string()),
        ));
    }

    if !crate::utils::is_valid_language_code(&language) {
        return Err(crate::CommandError::Database(
            rusqlite::Error::InvalidParameterName("Invalid language".to_string()),
        ));
    }

    if !(0..=3_600_000).contains(&duration_ms) {
        return Err(crate::CommandError::Database(
            rusqlite::Error::InvalidParameterName("Invalid duration".to_string()),
        ));
    }

    db.0.add_transcription(&sanitized_text, &model_id, &language, duration_ms)
        .map_err(Into::into)
}

#[tauri::command]
pub fn get_transcription_history(
    db: State<DbState>,
    limit: Option<i32>,
    offset: Option<i32>,
    search: Option<String>,
) -> CommandResult<Vec<TranscriptionHistory>> {
    let safe_limit = i64::from(limit.unwrap_or(50).clamp(1, 1000));
    let safe_offset = i64::from(offset.unwrap_or(0).max(0));
    let safe_search = search
        .as_deref()
        .map(|value| crate::utils::sanitize_text(value.trim(), 500))
        .transpose()
        .map_err(|e| {
            crate::CommandError::Database(rusqlite::Error::InvalidParameterName(format!(
                "Invalid search: {}",
                e
            )))
        })?;
    db.0.get_transcription_history(safe_search.as_deref(), safe_offset, safe_limit)
        .map_err(Into::into)
}

#[tauri::command]
pub fn get_transcription_history_count(
    db: State<DbState>,
    search: Option<String>,
) -> CommandResult<i64> {
    let safe_search = search
        .as_deref()
        .map(|value| crate::utils::sanitize_text(value.trim(), 500))
        .transpose()
        .map_err(|e| {
            crate::CommandError::Database(rusqlite::Error::InvalidParameterName(format!(
                "Invalid search: {}",
                e
            )))
        })?;
    db.0.get_transcription_history_count(safe_search.as_deref())
        .map_err(Into::into)
}

#[tauri::command]
pub fn clear_transcription_history(db: State<DbState>) -> CommandResult<()> {
    db.0.clear_transcription_history().map_err(Into::into)
}

#[tauri::command]
pub fn delete_transcription(db: State<DbState>, id: i64) -> CommandResult<()> {
    db.0.delete_transcription(id).map_err(Into::into)
}
