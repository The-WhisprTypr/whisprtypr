use crate::{
    database::{AppSettings, AppState},
    CommandResult, DbState,
};
use tauri::State;

#[tauri::command]
pub fn get_settings(db: State<DbState>) -> CommandResult<AppSettings> {
    db.0.get_settings().map_err(Into::into)
}

#[tauri::command]
pub fn update_settings(db: State<DbState>, settings: AppSettings) -> CommandResult<()> {
    db.0.update_settings(&settings).map_err(Into::into)
}

#[tauri::command]
pub fn update_setting(db: State<DbState>, key: String, value: String) -> CommandResult<()> {
    db.0.update_setting(&key, &value).map_err(Into::into)
}

#[tauri::command]
pub fn get_app_state(db: State<DbState>) -> CommandResult<AppState> {
    db.0.get_app_state().map_err(Into::into)
}

#[tauri::command]
pub fn update_app_state(db: State<DbState>, state: AppState) -> CommandResult<()> {
    db.0.update_app_state(&state).map_err(Into::into)
}

#[tauri::command]
pub fn set_setup_complete(db: State<DbState>, complete: bool) -> CommandResult<()> {
    db.0.set_setup_complete(complete).map_err(Into::into)
}

#[tauri::command]
pub fn set_current_setup_step(db: State<DbState>, step: i32) -> CommandResult<()> {
    db.0.set_current_setup_step(step).map_err(Into::into)
}
