use crate::{
    grammar::{check_grammar as do_check_grammar, fix_grammar as do_fix_grammar, GrammarDialect, GrammarError},
    CommandError, CommandResult, DbState,
};
use tauri::State;

#[tauri::command]
pub fn check_grammar(
    db: State<'_, DbState>,
    text: String,
) -> CommandResult<Vec<GrammarError>> {
    if text.is_empty() {
        return Ok(Vec::new());
    }

    let dialect = read_dialect(&db)?;
    let errors = do_check_grammar(&text, dialect);
    Ok(errors)
}

#[tauri::command]
pub fn fix_grammar(db: State<'_, DbState>, text: String) -> CommandResult<String> {
    if text.is_empty() {
        return Ok(String::new());
    }

    let dialect = read_dialect(&db)?;
    let fixed = do_fix_grammar(&text, dialect);
    Ok(fixed)
}

fn read_dialect(db: &State<'_, DbState>) -> CommandResult<GrammarDialect> {
    let settings = db.0.get_settings().map_err(CommandError::Database)?;
    settings
        .grammar_check_dialect
        .parse::<GrammarDialect>()
        .map_err(CommandError::Grammar)
}
