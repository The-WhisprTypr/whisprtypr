use whisprtypr_lib::database::{Database, LicenseData};

fn test_database() -> (tempfile::TempDir, Database) {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::new(dir.path().to_path_buf()).unwrap();
    (dir, db)
}

#[test]
fn new_database_creates_default_license_row() {
    let (_dir, db) = test_database();

    let license = db.get_license().unwrap();

    assert_eq!(license.status, "inactive");
    assert!(!license.is_activated);
    assert!(license.license_key.is_none());
    assert!(license.activation_id.is_none());
    assert!(license.trial_started_at.is_none());
}

#[test]
fn save_license_persists_active_license_data() {
    let (_dir, db) = test_database();
    let license = LicenseData {
        license_key: Some("WVT-TEST-LICENSE".to_string()),
        activation_id: Some("activation_123".to_string()),
        status: "active".to_string(),
        customer_email: Some("tester@whisprtypr.test".to_string()),
        customer_name: Some("Test User".to_string()),
        expires_at: None,
        is_activated: true,
        last_validated_at: Some("2026-04-19T00:00:00+00:00".to_string()),
        trial_started_at: None,
        trial_integrity_hash: None,
        usage: 3,
        validations: 4,
    };

    db.save_license(&license).unwrap();
    let stored = db.get_license().unwrap();

    assert_eq!(stored.license_key, license.license_key);
    assert_eq!(stored.activation_id, license.activation_id);
    assert_eq!(stored.status, "active");
    assert!(stored.is_activated);
    assert_eq!(stored.customer_email, Some("tester@whisprtypr.test".to_string()));
    assert_eq!(stored.usage, 3);
    assert_eq!(stored.validations, 4);
}

#[test]
fn clear_license_without_trial_returns_to_inactive() {
    let (_dir, db) = test_database();
    let mut license = LicenseData::default();
    license.license_key = Some("WVT-TEST-LICENSE".to_string());
    license.activation_id = Some("activation_123".to_string());
    license.status = "active".to_string();
    license.is_activated = true;
    license.last_validated_at = Some("2026-04-19T00:00:00+00:00".to_string());

    db.save_license(&license).unwrap();
    db.clear_license().unwrap();
    let stored = db.get_license().unwrap();

    assert_eq!(stored.status, "inactive");
    assert!(!stored.is_activated);
    assert!(stored.license_key.is_none());
    assert!(stored.activation_id.is_none());
    assert!(stored.last_validated_at.is_none());
}

#[test]
fn clear_license_preserves_trial_history_and_marks_expired() {
    let (_dir, db) = test_database();
    let trial_started_at = "2026-04-01T00:00:00+00:00".to_string();
    let trial_hash = "trial-hash".to_string();
    let license = LicenseData {
        license_key: Some("WVT-TEST-LICENSE".to_string()),
        activation_id: Some("activation_123".to_string()),
        status: "active".to_string(),
        customer_email: Some("tester@whisprtypr.test".to_string()),
        customer_name: None,
        expires_at: None,
        is_activated: true,
        last_validated_at: Some("2026-04-19T00:00:00+00:00".to_string()),
        trial_started_at: Some(trial_started_at.clone()),
        trial_integrity_hash: Some(trial_hash.clone()),
        usage: 5,
        validations: 6,
    };

    db.save_license(&license).unwrap();
    db.clear_license().unwrap();
    let stored = db.get_license().unwrap();

    assert_eq!(stored.status, "trial_expired");
    assert!(!stored.is_activated);
    assert!(stored.license_key.is_none());
    assert!(stored.activation_id.is_none());
    assert_eq!(stored.trial_started_at, Some(trial_started_at));
    assert_eq!(stored.trial_integrity_hash, Some(trial_hash));
    assert_eq!(stored.usage, 0);
    assert_eq!(stored.validations, 0);
}
