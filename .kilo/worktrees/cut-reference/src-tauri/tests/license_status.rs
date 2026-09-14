use vox_ai_lib::license::LicenseStatus;

#[test]
fn license_status_display_matches_polar_status_names() {
    assert_eq!(LicenseStatus::Granted.to_string(), "granted");
    assert_eq!(LicenseStatus::Revoked.to_string(), "revoked");
    assert_eq!(LicenseStatus::Disabled.to_string(), "disabled");
    assert_eq!(LicenseStatus::Expired.to_string(), "expired");
    assert_eq!(LicenseStatus::Invalid.to_string(), "invalid");
    assert_eq!(
        LicenseStatus::ActivationLimitReached.to_string(),
        "activation_limit_reached"
    );
    assert_eq!(LicenseStatus::Offline.to_string(), "offline");
    assert_eq!(LicenseStatus::NotActivated.to_string(), "not_activated");
}

#[test]
fn license_status_usage_gate_allows_only_granted_or_offline() {
    assert!(LicenseStatus::Granted.allows_usage());
    assert!(LicenseStatus::Offline.allows_usage());

    assert!(!LicenseStatus::Revoked.allows_usage());
    assert!(!LicenseStatus::Disabled.allows_usage());
    assert!(!LicenseStatus::Expired.allows_usage());
    assert!(!LicenseStatus::Invalid.allows_usage());
    assert!(!LicenseStatus::ActivationLimitReached.allows_usage());
    assert!(!LicenseStatus::NotActivated.allows_usage());
}

#[test]
fn polar_status_parser_is_case_insensitive_and_safe_by_default() {
    assert_eq!(
        LicenseStatus::from_polar_status("granted"),
        LicenseStatus::Granted
    );
    assert_eq!(
        LicenseStatus::from_polar_status("GRANTED"),
        LicenseStatus::Granted
    );
    assert_eq!(
        LicenseStatus::from_polar_status("revoked"),
        LicenseStatus::Revoked
    );
    assert_eq!(
        LicenseStatus::from_polar_status("disabled"),
        LicenseStatus::Disabled
    );
    assert_eq!(
        LicenseStatus::from_polar_status("unknown-new-status"),
        LicenseStatus::Invalid
    );
}

#[test]
fn license_status_serializes_to_snake_case() {
    let serialized = serde_json::to_string(&LicenseStatus::ActivationLimitReached).unwrap();

    assert_eq!(serialized, "\"activation_limit_reached\"");
}
