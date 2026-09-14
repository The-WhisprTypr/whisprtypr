use vox_ai_lib::{database, db_license_allows_usage, CommandError};

fn active_license() -> database::LicenseData {
    database::LicenseData {
        license_key: Some("WVT-TEST-LICENSE".to_string()),
        activation_id: Some("activation_123".to_string()),
        status: "active".to_string(),
        customer_email: Some("tester@whisprtypr.test".to_string()),
        customer_name: Some("Test User".to_string()),
        expires_at: None,
        is_activated: true,
        last_validated_at: Some(chrono::Utc::now().to_rfc3339()),
        trial_started_at: None,
        trial_integrity_hash: None,
        usage: 1,
        validations: 1,
    }
}

#[test]
fn db_license_allows_recent_active_license() {
    assert!(db_license_allows_usage(&active_license()));
}

#[test]
fn db_license_rejects_missing_key_or_activation() {
    let mut missing_key = active_license();
    missing_key.license_key = None;
    assert!(!db_license_allows_usage(&missing_key));

    let mut missing_activation = active_license();
    missing_activation.activation_id = None;
    assert!(!db_license_allows_usage(&missing_activation));
}

#[test]
fn db_license_rejects_inactive_status() {
    let mut license = active_license();
    license.status = "trial_expired".to_string();
    assert!(!db_license_allows_usage(&license));

    let mut license = active_license();
    license.is_activated = false;
    assert!(!db_license_allows_usage(&license));
}

#[test]
fn db_license_rejects_expired_license() {
    let mut license = active_license();
    license.expires_at = Some((chrono::Utc::now() - chrono::Duration::days(1)).to_rfc3339());

    assert!(!db_license_allows_usage(&license));
}

#[test]
fn db_license_rejects_missing_stale_or_future_validation_time() {
    let mut missing_validation = active_license();
    missing_validation.last_validated_at = None;
    assert!(!db_license_allows_usage(&missing_validation));

    let mut stale_validation = active_license();
    stale_validation.last_validated_at =
        Some((chrono::Utc::now() - chrono::Duration::hours(168)).to_rfc3339());
    assert!(!db_license_allows_usage(&stale_validation));

    let mut future_validation = active_license();
    future_validation.last_validated_at =
        Some((chrono::Utc::now() + chrono::Duration::minutes(1)).to_rfc3339());
    assert!(!db_license_allows_usage(&future_validation));
}

#[test]
fn license_errors_serialize_without_vendor_details() {
    let raw_error = CommandError::License(
        "Validate failed: HTTP 400 Bad Request - {\"error\":\"BadRequest\",\"detail\":\"License key only has 0 more usages.\"}".to_string(),
    );

    let serialized = serde_json::to_string(&raw_error).unwrap();

    assert!(serialized.contains("License error:"));
    assert!(!serialized.contains("HTTP 400"));
    assert!(!serialized.contains("BadRequest"));
    assert!(!serialized.contains("0 more usages"));
    assert!(!serialized.contains("detail"));
}

#[test]
fn license_errors_serialize_to_cause_specific_user_messages() {
    let no_license =
        serde_json::to_string(&CommandError::License("No license activated".to_string())).unwrap();
    assert!(no_license.contains("No active license was found"));

    let activation_limit = serde_json::to_string(&CommandError::License(
        "Activation limit reached".to_string(),
    ))
    .unwrap();
    assert!(activation_limit.contains("device limit"));

    let network = serde_json::to_string(&CommandError::License(
        "Network error during validate".to_string(),
    ))
    .unwrap();
    assert!(network.contains("internet connection"));
}
