use rusqlite::{params, Result};

use super::{Database, TranscriptionHistory};

fn escape_like_pattern(search: &str) -> String {
    search
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

pub fn add_transcription(
    db: &Database,
    text: &str,
    model_id: &str,
    language: &str,
    duration_ms: i64,
) -> Result<i64> {
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO transcription_history (text, model_id, language, duration_ms)
             VALUES (?1, ?2, ?3, ?4)",
        params![text, model_id, language, duration_ms],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_transcription_history(
    db: &Database,
    search: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<Vec<TranscriptionHistory>> {
    let conn = db.conn.lock().unwrap();
    let search = search.unwrap_or("").trim();
    let pattern = format!("%{}%", escape_like_pattern(search));
    let mut stmt = conn.prepare(
        "SELECT id, text, model_id, language, duration_ms, created_at
             FROM transcription_history
             WHERE (?3 = '' OR text LIKE ?4 ESCAPE '\\')
             ORDER BY created_at DESC
             LIMIT ?1 OFFSET ?2",
    )?;

    let history = stmt
        .query_map(params![limit, offset, search, pattern], |row| {
            Ok(TranscriptionHistory {
                id: row.get(0)?,
                text: row.get(1)?,
                model_id: row.get(2)?,
                language: row.get(3)?,
                duration_ms: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(history)
}

pub fn get_transcription_history_count(db: &Database, search: Option<&str>) -> Result<i64> {
    let conn = db.conn.lock().unwrap();
    let search = search.unwrap_or("").trim();
    let pattern = format!("%{}%", escape_like_pattern(search));
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM transcription_history
             WHERE (?1 = '' OR text LIKE ?2 ESCAPE '\\')",
        params![search, pattern],
        |row| row.get(0),
    )?;
    Ok(count)
}

pub fn clear_transcription_history(db: &Database) -> Result<()> {
    let conn = db.conn.lock().unwrap();
    conn.execute("DELETE FROM transcription_history", [])?;
    Ok(())
}

pub fn delete_transcription(db: &Database, id: i64) -> Result<()> {
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "DELETE FROM transcription_history WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}
