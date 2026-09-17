use rusqlite::{params, Result};

use super::{AppState, Database};

pub fn get_app_state(db: &Database) -> Result<AppState> {
    let conn = db.conn.lock().unwrap();
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

pub fn update_app_state(db: &Database, state: &AppState) -> Result<()> {
    let conn = db.conn.lock().unwrap();
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

pub fn set_setup_complete(db: &Database, complete: bool) -> Result<()> {
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "UPDATE app_state SET setup_complete = ?1, is_first_launch = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = 1",
        params![complete as i32, (!complete) as i32],
    )?;
    Ok(())
}

pub fn set_current_setup_step(db: &Database, step: i32) -> Result<()> {
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "UPDATE app_state SET current_setup_step = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = 1",
        params![step],
    )?;
    Ok(())
}
