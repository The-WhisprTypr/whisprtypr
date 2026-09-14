use httpmock::prelude::*;
use serde_json::json;
use whisprtypr_lib::license::{clear_cache, LicenseManager, LicenseStatus};

#[tokio::test]
async fn license_manager_activation_success() {
    let _ = clear_cache();
    let server = MockServer::start();

    // Mock an activation route
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/activate")
            .body_contains("test-key")
            .body_contains("test-org-id");
        then.status(200).json_body(json!({
            "id": "activation_123",
            "license_key_id": "lk_123",
            "label": "Test Device",
            "created_at": "2023-01-01T00:00:00Z",
            "license_key": {
                "id": "lk_123",
                "organization_id": "test-org-id",
                "customer_id": "cust_123",
                "customer": {
                    "id": "cust_123",
                    "email": "tester@whisprtypr.test",
                    "name": "Test User"
                },
                "benefit_id": "ben_123",
                "key": "test-key",
                "display_key": "****-test",
                "status": "granted",
                "usage": 1,
                "validations": 1,
                "limit_activations": 3,
                "expires_at": null,
            }
        }));
    });

    // We also need to mock a validate route, since activation automatically tries to perform validation right after
    let validate_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/validate")
            .body_contains("test-key");
        then.status(200).json_body(json!({
            "id": "val_123",
            "organization_id": "test-org-id",
            "customer_id": "cust_123",
            "customer": null,
            "benefit_id": "ben_123",
            "key": "test-key",
            "display_key": "****-test",
            "status": "granted",
            "usage": 1,
            "limit_activations": 3,
            "validations": 2,
            "expires_at": null,
            "activation": {
                "id": "activation_123",
                "license_key_id": "lk_123",
                "label": "Test Device",
                "created_at": "2023-01-01T00:00:00Z"
            }
        }));
    });

    let manager = LicenseManager::with_org_id("test-org-id", &server.base_url());
    let result = manager
        .activate("test-key")
        .await
        .expect("activation should succeed");

    mock.assert();
    validate_mock.assert();

    assert_eq!(result.status, LicenseStatus::Granted);
    assert_eq!(result.activation_id.unwrap(), "activation_123");
    let _ = clear_cache();
}

#[tokio::test]
async fn license_manager_activation_limit_reached() {
    let _ = clear_cache();
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(POST).path("/activate");
        then.status(403).json_body(json!({
            "error": "Activation limit reached",
            "type": "limit_reached"
        }));
    });

    let manager = LicenseManager::with_org_id("test-org-id", &server.base_url());
    let result = manager.activate("test-key").await;

    mock.assert();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("limit"));
    let _ = clear_cache();
}

#[tokio::test]
async fn license_manager_invalid_key_returns_friendly_error() {
    let _ = clear_cache();
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(POST).path("/activate");
        then.status(404).json_body(json!({
            "error": "Not found",
            "detail": "License key does not exist"
        }));
    });

    let manager = LicenseManager::with_org_id("test-org-id", &server.base_url());
    let result = manager.activate("bad-key").await;

    mock.assert();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Invalid license key. Please check and try again."
    );
    let _ = clear_cache();
}

#[tokio::test]
async fn license_manager_validation_rejection_is_sanitized() {
    let _ = clear_cache();
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/validate")
            .body_contains("test-activation-id");
        then.status(422).json_body(json!({
            "error": "BadRequest",
            "detail": "License key only has 0 more usages"
        }));
    });

    let manager = LicenseManager::with_org_id("test-org", &server.base_url());
    let result = manager
        .validate_activation("test-key", "test-activation-id")
        .await;

    mock.assert();
    assert!(result.is_err());
    let message = result.unwrap_err();
    assert_eq!(
        message,
        "License validation was rejected by the license server. HTTP 422"
    );
    assert!(!message.contains("0 more usages"));
    assert!(!message.contains("BadRequest"));
    let _ = clear_cache();
}

#[tokio::test]
async fn license_manager_validation_success() {
    let _ = clear_cache();
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/validate")
            .body_contains("test-activation-id");
        then.status(200).json_body(json!({
            "id": "val_123",
            "organization_id": "test-org",
            "customer_id": "cust_123",
            "customer": null,
            "benefit_id": "ben_123",
            "key": "test-key",
            "display_key": "****",
            "status": "granted",
            "usage": 2,
            "limit_activations": 3,
            "validations": 10,
            "expires_at": null,
            "activation": {
                "id": "test-activation-id",
                "license_key_id": "lk_123",
                "label": "Device",
                "created_at": "2023-01-01T00:00:00Z"
            }
        }));
    });

    let manager = LicenseManager::with_org_id("test-org", &server.base_url());
    // Directly use validate_activation to skip the local cache requirement for this test
    let result = manager
        .validate_activation("test-key", "test-activation-id")
        .await
        .expect("validation should succeed");

    mock.assert();
    assert_eq!(result.status, LicenseStatus::Granted);
    assert_eq!(result.usage, 2);
    let _ = clear_cache();
}

#[tokio::test]
async fn license_manager_deactivation_success() {
    let _ = clear_cache();
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/deactivate")
            .body_contains("test-activation-id")
            .body_contains("test-key");
        then.status(204);
    });

    let manager = LicenseManager::with_org_id("test-org", &server.base_url());
    let result = manager
        .deactivate_activation("test-key", "test-activation-id")
        .await;

    mock.assert();
    assert!(result.is_ok());
    let _ = clear_cache();
}

#[tokio::test]
async fn license_manager_deactivation_404_is_treated_as_local_success() {
    let _ = clear_cache();
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/deactivate")
            .body_contains("test-activation-id")
            .body_contains("test-key");
        then.status(404).json_body(json!({
            "error": "Not found"
        }));
    });

    let manager = LicenseManager::with_org_id("test-org", &server.base_url());
    let result = manager
        .deactivate_activation("test-key", "test-activation-id")
        .await;

    mock.assert();
    assert!(result.is_ok());
    let _ = clear_cache();
}

#[tokio::test]
async fn license_manager_deactivation_failure_is_sanitized() {
    let _ = clear_cache();
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(POST).path("/deactivate");
        then.status(500).json_body(json!({
            "error": "InternalError",
            "detail": "raw server body"
        }));
    });

    let manager = LicenseManager::with_org_id("test-org", &server.base_url());
    let result = manager
        .deactivate_activation("test-key", "test-activation-id")
        .await;

    mock.assert();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "License deactivation failed. Please try again."
    );
    let _ = clear_cache();
}
