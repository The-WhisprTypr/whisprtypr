use chrono::{Duration, Utc};
use vox_ai_lib::database::LicenseData;
use vox_ai_lib::{
    calculate_trial_integrity_hash, db_license_allows_usage_core, has_active_trial_core,
};

fn create_trial_license(started_at_offset_days: i64) -> LicenseData {
    let now = Utc::now();
    let started_at = now + Duration::days(started_at_offset_days);
    let started_at_str = started_at.to_rfc3339();
    let hash = calculate_trial_integrity_hash(&started_at_str);

    LicenseData {
        license_key: None,
        activation_id: None,
        status: "trial".to_string(),
        customer_email: None,
        customer_name: None,
        expires_at: None,
        is_activated: false,
        last_validated_at: None,
        trial_started_at: Some(started_at_str),
        trial_integrity_hash: Some(hash),
        usage: 0,
        validations: 0,
    }
}

#[test]
fn active_trial_allows_usage_on_day_0() {
    let license = create_trial_license(0);
    // Evaluating at the exact moment it started
    let now = chrono::DateTime::parse_from_rfc3339(license.trial_started_at.as_ref().unwrap())
        .unwrap()
        .with_timezone(&Utc);
    assert!(has_active_trial_core(&license, now));
}

#[test]
fn active_trial_allows_usage_on_day_6() {
    let license = create_trial_license(-6);
    let now = Utc::now();
    assert!(has_active_trial_core(&license, now));
}

#[test]
fn active_trial_expires_on_day_7() {
    let license = create_trial_license(-7);
    let now = Utc::now();
    // 7 full days have passed, meaning it is the 8th day (0-6 allowed)
    assert!(!has_active_trial_core(&license, now));
}

#[test]
fn active_trial_allows_until_just_before_day_7() {
    let now = Utc::now();
    let started_at = now - Duration::days(7) + Duration::seconds(1);
    let started_at_str = started_at.to_rfc3339();
    let license = LicenseData {
        trial_integrity_hash: Some(calculate_trial_integrity_hash(&started_at_str)),
        trial_started_at: Some(started_at_str),
        status: "trial".to_string(),
        ..LicenseData::default()
    };

    assert!(has_active_trial_core(&license, now));
}

#[test]
fn active_trial_expires_at_exact_day_7_boundary() {
    let now = Utc::now();
    let started_at = now - Duration::days(7);
    let started_at_str = started_at.to_rfc3339();
    let license = LicenseData {
        trial_integrity_hash: Some(calculate_trial_integrity_hash(&started_at_str)),
        trial_started_at: Some(started_at_str),
        status: "trial".to_string(),
        ..LicenseData::default()
    };

    assert!(!has_active_trial_core(&license, now));
}

#[test]
fn trial_rejects_invalid_start_timestamp() {
    let license = LicenseData {
        trial_integrity_hash: Some(calculate_trial_integrity_hash("not-a-date")),
        trial_started_at: Some("not-a-date".to_string()),
        status: "trial".to_string(),
        ..LicenseData::default()
    };

    assert!(!has_active_trial_core(&license, Utc::now()));
}

#[test]
fn trial_rejects_tampered_hash() {
    let mut license = create_trial_license(0);
    license.trial_integrity_hash = Some("invalid_hash_123".to_string());

    let now = Utc::now();
    assert!(!has_active_trial_core(&license, now));
}

#[test]
fn trial_rejects_missing_hash() {
    let mut license = create_trial_license(0);
    license.trial_integrity_hash = None;

    let now = Utc::now();
    assert!(!has_active_trial_core(&license, now));
}

#[test]
fn trial_rejects_future_start_date() {
    let license = create_trial_license(1); // Starts 1 day in the future
    let now = Utc::now();

    // (now - future_date).num_days() will be 0 or -1 depending on precision,
    // wait, if start_date > now, then now - start_date is negative.
    // Let's ensure it handles it!
    assert!(!has_active_trial_core(&license, now));
}

#[test]
fn trial_rejects_inactive_status() {
    let mut license = create_trial_license(0);
    license.status = "trial_expired".to_string(); // Not "trial"

    let now = Utc::now();
    assert!(!has_active_trial_core(&license, now));
}

fn create_active_license(expired: bool, future_validation: bool) -> LicenseData {
    let now = Utc::now();
    let expiry = if expired {
        now - Duration::days(1)
    } else {
        now + Duration::days(365)
    };

    let validation = if future_validation {
        now + Duration::minutes(10)
    } else {
        now - Duration::hours(24) // Verified 24 hours ago
    };

    LicenseData {
        license_key: Some("test-key".to_string()),
        activation_id: Some("test-id".to_string()),
        status: "active".to_string(),
        customer_email: None,
        customer_name: None,
        expires_at: Some(expiry.to_rfc3339()),
        is_activated: true,
        last_validated_at: Some(validation.to_rfc3339()),
        trial_started_at: None,
        trial_integrity_hash: None,
        usage: 0,
        validations: 1,
    }
}

#[test]
fn db_license_core_allows_valid_usage() {
    let license = create_active_license(false, false);
    let now = Utc::now();
    assert!(db_license_allows_usage_core(&license, now));
}

#[test]
fn db_license_core_rejects_expired() {
    let license = create_active_license(true, false);
    let now = Utc::now();
    assert!(!db_license_allows_usage_core(&license, now));
}

#[test]
fn db_license_core_rejects_future_validation() {
    let license = create_active_license(false, true);
    let now = Utc::now();
    assert!(!db_license_allows_usage_core(&license, now));
}

#[test]
fn db_license_core_rejects_stale_validation() {
    let mut license = create_active_license(false, false);
    let now = Utc::now();

    // Set validation to 8 days ago (> 168 hours)
    license.last_validated_at = Some((now - Duration::days(8)).to_rfc3339());
    assert!(!db_license_allows_usage_core(&license, now));
}

#[test]
fn db_license_core_allows_until_just_before_grace_boundary() {
    let now = Utc::now();
    let mut license = create_active_license(false, false);
    license.last_validated_at =
        Some((now - Duration::hours(168) + Duration::seconds(1)).to_rfc3339());

    assert!(db_license_allows_usage_core(&license, now));
}

#[test]
fn db_license_core_rejects_invalid_timestamps() {
    let now = Utc::now();
    let mut bad_expiry = create_active_license(false, false);
    bad_expiry.expires_at = Some("not-a-date".to_string());
    assert!(!db_license_allows_usage_core(&bad_expiry, now));

    let mut bad_validation = create_active_license(false, false);
    bad_validation.last_validated_at = Some("not-a-date".to_string());
    assert!(!db_license_allows_usage_core(&bad_validation, now));
}
