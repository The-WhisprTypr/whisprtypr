use rusqlite::Result;

use super::{Database, LicenseData};

pub fn get_license(db: &Database) -> Result<LicenseData> {
    let conn = db.conn.lock().unwrap();
    conn.query_row(
        "SELECT license_key, activation_id, status, customer_email, customer_name, 
                    expires_at, is_activated, last_validated_at, trial_started_at,
                    trial_integrity_hash, trial_salt, usage, validations
             FROM license WHERE id = 1",
        [],
        |row| {
            Ok(LicenseData {
                license_key: row.get(0)?,
                activation_id: row.get(1)?,
                status: row.get(2)?,
                customer_email: row.get(3)?,
                customer_name: row.get(4)?,
                expires_at: row.get(5)?,
                is_activated: row.get::<_, i32>(6)? != 0,
                last_validated_at: row.get(7)?,
                trial_started_at: row.get(8)?,
                trial_integrity_hash: row.get(9)?,
                trial_salt: row.get(10)?,
                usage: row.get(11)?,
                validations: row.get(12)?,
            })
        },
    )
}

pub fn save_license(db: &Database, license: &LicenseData) -> Result<()> {
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "UPDATE license SET 
                license_key = ?1,
                activation_id = ?2,
                status = ?3,
                customer_email = ?4,
                customer_name = ?5,
                expires_at = ?6,
                is_activated = ?7,
                last_validated_at = ?8,
                trial_started_at = ?9,
                trial_integrity_hash = ?10,
                trial_salt = ?11,
                usage = ?12,
                validations = ?13,
                updated_at = CURRENT_TIMESTAMP
             WHERE id = 1",
        rusqlite::params![
            license.license_key,
            license.activation_id,
            license.status,
            license.customer_email,
            license.customer_name,
            license.expires_at,
            license.is_activated as i32,
            license.last_validated_at,
            license.trial_started_at,
            license.trial_integrity_hash,
            license.trial_salt,
            license.usage,
            license.validations,
        ],
    )?;
    Ok(())
}

pub fn clear_license(db: &Database) -> Result<()> {
    let conn = db.conn.lock().unwrap();
    // IMPORTANT: Preserve trial_started_at to prevent trial abuse
    // Users who have used their trial should not be able to restart it
    conn.execute(
        "UPDATE license SET 
                license_key = NULL,
                activation_id = NULL,
                status = CASE 
                    WHEN trial_started_at IS NOT NULL THEN 'trial_expired'
                    ELSE 'inactive'
                END,
                customer_email = NULL,
                customer_name = NULL,
                expires_at = NULL,
                is_activated = 0,
                last_validated_at = NULL,
                usage = 0,
                validations = 0,
                -- trial_started_at and trial_integrity_hash are preserved intentionally
                updated_at = CURRENT_TIMESTAMP
             WHERE id = 1",
        [],
    )?;
    Ok(())
}
