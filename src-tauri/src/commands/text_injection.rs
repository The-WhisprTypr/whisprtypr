use crate::{CommandError, CommandResult, TextInjectorState};
use tauri::State;

#[tauri::command]
pub fn inject_text(injector: State<TextInjectorState>, text: String) -> CommandResult<()> {
    let sanitized =
        crate::utils::sanitize_text(&text, 100_000).map_err(CommandError::TextInjection)?;

    if sanitized.is_empty() {
        return Err(CommandError::TextInjection("No text to inject".to_string()));
    }

    let mut injector_guard = injector.0.lock().unwrap();
    injector_guard
        .inject_text(&sanitized)
        .map_err(CommandError::TextInjection)
}

#[tauri::command]
pub fn execute_keyboard_shortcut(
    injector: State<TextInjectorState>,
    shortcut: String,
) -> CommandResult<()> {
    let allowed_shortcuts = [
        "undo",
        "redo",
        "copy",
        "cut",
        "paste",
        "select_all",
        "backspace_word",
        "backspace",
        "delete",
        "delete_word",
        "delete_line",
        "enter",
        "tab",
        "escape",
        "page_up",
        "page_down",
        "left",
        "right",
        "up",
        "down",
        "home",
        "end",
        "word_left",
        "word_right",
        "select_left",
        "select_right",
        "select_up",
        "select_down",
        "select_word_left",
        "select_word_right",
        "select_to_start",
        "select_to_end",
    ];
    if !allowed_shortcuts.contains(&shortcut.as_str()) {
        return Err(CommandError::TextInjection(format!(
            "Invalid shortcut: {}",
            shortcut
        )));
    }

    let mut injector_guard = injector.0.lock().unwrap();
    injector_guard
        .execute_shortcut(&shortcut)
        .map_err(CommandError::TextInjection)
}
