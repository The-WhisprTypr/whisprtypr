use rusqlite::{params, Result};

use super::{AiFormattingProviderRecord, CloudProviderRecord, Database};

pub fn get_cloud_providers(db: &Database) -> Result<Vec<CloudProviderRecord>> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, api_key, base_url, custom_model, is_active FROM cloud_providers ORDER BY id ASC"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(CloudProviderRecord {
            id: row.get(0)?,
            api_key: row.get(1)?,
            base_url: row.get(2)?,
            custom_model: row.get(3)?,
            is_active: row.get::<_, i32>(4)? == 1,
        })
    })?;

    let mut providers = Vec::new();
    for row in rows {
        providers.push(row?);
    }
    Ok(providers)
}

pub fn get_cloud_provider(db: &Database, id: &str) -> Result<Option<CloudProviderRecord>> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, api_key, base_url, custom_model, is_active FROM cloud_providers WHERE id = ?1",
    )?;

    let mut rows = stmt.query_map(params![id], |row| {
        Ok(CloudProviderRecord {
            id: row.get(0)?,
            api_key: row.get(1)?,
            base_url: row.get(2)?,
            custom_model: row.get(3)?,
            is_active: row.get::<_, i32>(4)? == 1,
        })
    })?;

    if let Some(row) = rows.next() {
        Ok(Some(row?))
    } else {
        Ok(None)
    }
}

pub fn save_cloud_provider(db: &Database, record: &CloudProviderRecord) -> Result<()> {
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO cloud_providers (id, api_key, base_url, custom_model, is_active, updated_at)
             VALUES (?1, ?2, ?3, ?4, 1, CURRENT_TIMESTAMP)
             ON CONFLICT(id) DO UPDATE SET
                api_key = excluded.api_key,
                base_url = excluded.base_url,
                custom_model = excluded.custom_model,
                is_active = 1,
                updated_at = CURRENT_TIMESTAMP",
        params![
            &record.id,
            &record.api_key,
            record.base_url.as_deref(),
            record.custom_model.as_deref()
        ],
    )?;
    Ok(())
}

pub fn delete_cloud_provider(db: &Database, id: &str) -> Result<()> {
    let conn = db.conn.lock().unwrap();
    conn.execute("DELETE FROM cloud_providers WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn get_ai_formatting_providers(db: &Database) -> Result<Vec<AiFormattingProviderRecord>> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, api_key, base_url, custom_model, is_active FROM ai_formatting_providers ORDER BY id ASC"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(AiFormattingProviderRecord {
            id: row.get(0)?,
            api_key: row.get(1)?,
            base_url: row.get(2)?,
            custom_model: row.get(3)?,
            is_active: row.get::<_, i32>(4)? == 1,
        })
    })?;

    let mut providers = Vec::new();
    for row in rows {
        providers.push(row?);
    }
    Ok(providers)
}

pub fn get_ai_formatting_provider(
    db: &Database,
    id: &str,
) -> Result<Option<AiFormattingProviderRecord>> {
    let conn = db.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, api_key, base_url, custom_model, is_active FROM ai_formatting_providers WHERE id = ?1"
    )?;

    let mut rows = stmt.query_map(params![id], |row| {
        Ok(AiFormattingProviderRecord {
            id: row.get(0)?,
            api_key: row.get(1)?,
            base_url: row.get(2)?,
            custom_model: row.get(3)?,
            is_active: row.get::<_, i32>(4)? == 1,
        })
    })?;

    if let Some(row) = rows.next() {
        Ok(Some(row?))
    } else {
        Ok(None)
    }
}

pub fn save_ai_formatting_provider(
    db: &Database,
    record: &AiFormattingProviderRecord,
) -> Result<()> {
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO ai_formatting_providers (id, api_key, base_url, custom_model, is_active, updated_at)
             VALUES (?1, ?2, ?3, ?4, 1, CURRENT_TIMESTAMP)
             ON CONFLICT(id) DO UPDATE SET
                api_key = excluded.api_key,
                base_url = excluded.base_url,
                custom_model = excluded.custom_model,
                is_active = 1,
                updated_at = CURRENT_TIMESTAMP",
        params![
            &record.id,
            &record.api_key,
            record.base_url.as_deref(),
            record.custom_model.as_deref()
        ],
    )?;
    Ok(())
}

pub fn delete_ai_formatting_provider(db: &Database, id: &str) -> Result<()> {
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "DELETE FROM ai_formatting_providers WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}
