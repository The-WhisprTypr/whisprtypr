use rusqlite::{params, Result};

use super::{Database, WhisperModel};

pub fn get_models(db: &Database) -> Result<Vec<WhisperModel>> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, name, size, size_bytes, description, languages, downloaded, download_path
             FROM models ORDER BY size_bytes ASC",
    )?;

    let models = stmt
        .query_map([], |row| {
            Ok(WhisperModel {
                id: row.get(0)?,
                name: row.get(1)?,
                size: row.get(2)?,
                size_bytes: row.get(3)?,
                description: row.get(4)?,
                languages: row.get(5)?,
                downloaded: row.get::<_, i32>(6)? == 1,
                download_path: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(models)
}

pub fn get_model(db: &Database, id: &str) -> Result<Option<WhisperModel>> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, name, size, size_bytes, description, languages, downloaded, download_path
             FROM models WHERE id = ?1",
    )?;

    let model = stmt
        .query_row(params![id], |row| {
            Ok(WhisperModel {
                id: row.get(0)?,
                name: row.get(1)?,
                size: row.get(2)?,
                size_bytes: row.get(3)?,
                description: row.get(4)?,
                languages: row.get(5)?,
                downloaded: row.get::<_, i32>(6)? == 1,
                download_path: row.get(7)?,
            })
        })
        .ok();

    Ok(model)
}

pub fn set_model_downloaded(
    db: &Database,
    id: &str,
    downloaded: bool,
    path: Option<&str>,
) -> Result<()> {
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "UPDATE models SET downloaded = ?1, download_path = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?3",
        params![downloaded as i32, path, id],
    )?;
    Ok(())
}

pub fn set_selected_model(db: &Database, model_id: Option<&str>) -> Result<()> {
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "UPDATE app_state SET selected_model_id = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = 1",
        params![model_id],
    )?;
    // Also update in settings
    if let Some(id) = model_id {
        conn.execute(
            "UPDATE settings SET selected_model_id = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = 1",
            params![id],
        )?;
    }
    Ok(())
}
