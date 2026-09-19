use crate::{
    database::VocabularyEntry,
    post_process::{PostProcessor, VocabularyEntry as PostProcessVocabularyEntry},
    CommandError, CommandResult, DbState,
};
use tauri::State;

#[tauri::command]
pub fn post_process_text(db: State<DbState>, text: String) -> CommandResult<String> {
    let sanitized =
        crate::utils::sanitize_text(&text, 100_000).map_err(crate::CommandError::PostProcessing)?;

    if sanitized.is_empty() {
        return Ok(String::new());
    }

    let settings = db.0.get_settings().map_err(CommandError::Database)?;
    let entries: Vec<PostProcessVocabularyEntry> = settings
        .custom_vocabulary
        .into_iter()
        .map(|e: VocabularyEntry| PostProcessVocabularyEntry::new(e.spoken, e.written))
        .collect();
    let processor = PostProcessor::with_vocabulary(&entries);
    let processed = processor.process(&sanitized);

    let final_text = if settings.grammar_check_enabled {
        let dialect = settings
            .grammar_check_dialect
            .parse::<crate::GrammarDialect>()
            .unwrap_or_default();
        crate::grammar::fix_grammar(&processed, dialect)
    } else {
        processed
    };

    Ok(final_text)
}

#[tauri::command]
pub fn extract_voice_commands(db: State<DbState>, text: String) -> CommandResult<String> {
    let sanitized =
        crate::utils::sanitize_text(&text, 100_000).map_err(crate::CommandError::PostProcessing)?;

    if sanitized.is_empty() {
        return Ok(String::new());
    }

    let processor = build_processor(&db)?;
    let processed = processor.extract_voice_commands(&sanitized);

    Ok(processed)
}

fn build_processor(db: &State<DbState>) -> CommandResult<PostProcessor> {
    let settings = db.0.get_settings().map_err(CommandError::Database)?;
    let entries: Vec<PostProcessVocabularyEntry> = settings
        .custom_vocabulary
        .into_iter()
        .map(|e: VocabularyEntry| PostProcessVocabularyEntry::new(e.spoken, e.written))
        .collect();
    Ok(PostProcessor::with_vocabulary(&entries))
}
