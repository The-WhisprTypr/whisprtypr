use chrono::{Duration, Utc};
use httpmock::prelude::*;
use serde_json::json;
use whisprtypr_lib::database::{Database, LicenseData};
use whisprtypr_lib::downloader::ModelDownloader;
use whisprtypr_lib::license::{clear_cache, LicenseManager, LicenseStatus};
use whisprtypr_lib::post_process::PostProcessor;
use whisprtypr_lib::{
    calculate_trial_integrity_hash, db_license_allows_usage, has_active_trial_core,
};

fn test_database() -> (tempfile::TempDir, Database) {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::new(dir.path().to_path_buf()).unwrap();
    (dir, db)
}

fn can_use_backend(db: &Database) -> (&'static str, bool) {
    let license = db.get_license().unwrap();

    if db_license_allows_usage(&license) {
        return ("licensed", true);
    }

    if has_active_trial_core(&license, Utc::now()) {
        return ("trial", true);
    }

    if license.trial_started_at.is_some() {
        return ("trial_expired", false);
    }

    ("no_license", false)
}

fn trial_license(started_at: chrono::DateTime<Utc>) -> LicenseData {
    let started_at = started_at.to_rfc3339();
    LicenseData {
        status: "trial".to_string(),
        trial_integrity_hash: Some(calculate_trial_integrity_hash(&started_at)),
        trial_started_at: Some(started_at),
        ..LicenseData::default()
    }
}

#[test]
fn e2e_first_launch_trial_then_expiry_blocks_backend_access() {
    let (_dir, db) = test_database();

    let initial = db.get_license().unwrap();
    assert_eq!(initial.status, "inactive");
    assert_eq!(can_use_backend(&db), ("no_license", false));

    db.save_license(&trial_license(Utc::now())).unwrap();
    let active_trial = db.get_license().unwrap();

    assert_eq!(active_trial.status, "trial");
    assert!(active_trial.trial_started_at.is_some());
    assert!(active_trial.trial_integrity_hash.is_some());
    assert_eq!(can_use_backend(&db), ("trial", true));

    db.save_license(&trial_license(Utc::now() - Duration::days(7)))
        .unwrap();

    assert_eq!(can_use_backend(&db), ("trial_expired", false));
}

#[test]
fn e2e_tampered_trial_is_blocked_and_cannot_restore_access_by_clearing_license() {
    let (_dir, db) = test_database();
    let mut license = trial_license(Utc::now());
    license.trial_integrity_hash = Some("tampered".to_string());
    db.save_license(&license).unwrap();

    assert_eq!(can_use_backend(&db), ("trial_expired", false));

    db.clear_license().unwrap();
    let cleared = db.get_license().unwrap();

    assert_eq!(cleared.status, "trial_expired");
    assert!(cleared.trial_started_at.is_some());
    assert_eq!(can_use_backend(&db), ("trial_expired", false));
}

#[test]
fn e2e_onboarding_selects_model_completes_setup_and_persists() {
    let dir = tempfile::tempdir().unwrap();
    let app_data_dir = dir.path().join("app-data");
    let models_dir = app_data_dir.join("models");
    let db = Database::new(app_data_dir.clone()).unwrap();
    let downloader = ModelDownloader::new(models_dir);
    let model_path = downloader.get_model_path("base");

    std::fs::create_dir_all(model_path.parent().unwrap()).unwrap();
    std::fs::write(&model_path, b"fake base model").unwrap();

    assert!(downloader.is_model_downloaded("base"));
    db.set_model_downloaded("base", true, Some(&model_path.to_string_lossy()))
        .unwrap();
    db.set_selected_model(Some("base")).unwrap();
    db.update_setting("language", "en").unwrap();
    db.set_setup_complete(true).unwrap();

    let state = db.get_app_state().unwrap();
    assert!(!state.is_first_launch);
    assert!(state.setup_complete);
    assert_eq!(state.selected_model_id, Some("base".to_string()));

    drop(db);
    let reopened = Database::new(app_data_dir).unwrap();
    let settings = reopened.get_settings().unwrap();
    let state = reopened.get_app_state().unwrap();
    let model = reopened.get_model("base").unwrap().unwrap();

    assert_eq!(settings.selected_model_id, "base");
    assert_eq!(settings.language, "en");
    assert!(state.setup_complete);
    assert!(model.downloaded);
    assert_eq!(
        model.download_path,
        Some(model_path.to_string_lossy().to_string())
    );
}

#[test]
fn e2e_dictation_post_processing_history_search_and_cleanup() {
    let (_dir, db) = test_database();
    let processor = PostProcessor::new();
    let processed = processor
        .process("open file main dot rs new line function create user insert question mark");

    let first_id = db
        .add_transcription(&processed, "base", "en", 1400)
        .unwrap();
    let second_id = db
        .add_transcription("plain follow up note", "base", "en", 800)
        .unwrap();

    assert!(second_id > first_id);
    assert_eq!(db.get_transcription_history_count(None).unwrap(), 2);
    assert!(processed.contains("@main.rs"));
    assert!(processed.contains("createUser()"));

    let code_matches = db
        .get_transcription_history(Some("@main.rs"), 0, 10)
        .unwrap();
    assert_eq!(code_matches.len(), 1);
    assert_eq!(code_matches[0].id, first_id);

    db.delete_transcription(first_id).unwrap();
    assert_eq!(db.get_transcription_history_count(None).unwrap(), 1);

    db.clear_transcription_history().unwrap();
    assert_eq!(db.get_transcription_history_count(None).unwrap(), 0);
}

#[tokio::test]
async fn e2e_failed_license_activation_keeps_trial_state_and_backend_access() {
    let _ = clear_cache();
    let (_dir, db) = test_database();
    let trial = trial_license(Utc::now());
    let trial_started_at = trial.trial_started_at.clone();
    let trial_hash = trial.trial_integrity_hash.clone();
    db.save_license(&trial).unwrap();

    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST).path("/activate").body_contains("bad-key");
        then.status(404).json_body(json!({
            "error": "Not found",
            "detail": "raw vendor detail"
        }));
    });

    let manager = LicenseManager::with_org_id("e2e-org", &server.base_url());
    let result = manager.activate("bad-key").await;

    mock.assert();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Invalid license key. Please check and try again."
    );

    let stored = db.get_license().unwrap();
    assert_eq!(stored.status, "trial");
    assert_eq!(stored.trial_started_at, trial_started_at);
    assert_eq!(stored.trial_integrity_hash, trial_hash);
    assert!(stored.license_key.is_none());
    assert_eq!(can_use_backend(&db), ("trial", true));
    let _ = clear_cache();
}

#[tokio::test]
async fn e2e_db_backed_license_recovers_after_cache_loss() {
    let _ = clear_cache();
    let (_dir, db) = test_database();
    db.save_license(&LicenseData {
        license_key: Some("db-key".to_string()),
        activation_id: Some("db-activation".to_string()),
        status: "active".to_string(),
        customer_email: Some("db-customer@whisprtypr.test".to_string()),
        customer_name: Some("DB User".to_string()),
        expires_at: None,
        is_activated: true,
        last_validated_at: Some(Utc::now().to_rfc3339()),
        trial_started_at: None,
        trial_integrity_hash: None,
        usage: 1,
        validations: 1,
    })
    .unwrap();

    assert_eq!(can_use_backend(&db), ("licensed", true));

    let server = MockServer::start();
    let validate_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/validate")
            .body_contains("db-key")
            .body_contains("db-activation");
        then.status(200).json_body(json!({
            "id": "validation_db",
            "organization_id": "e2e-org",
            "customer_id": "cust_db",
            "customer": {
                "id": "cust_db",
                "email": "db-customer@whisprtypr.test",
                "name": "DB User"
            },
            "benefit_id": "benefit_db",
            "key": "db-key",
            "display_key": "****-db",
            "status": "granted",
            "usage": 3,
            "limit_activations": 3,
            "validations": 9,
            "expires_at": null,
            "activation": {
                "id": "db-activation",
                "license_key_id": "lk_db",
                "label": "DB Device",
                "created_at": "2026-04-19T00:00:00Z"
            }
        }));
    });

    let manager = LicenseManager::with_org_id("e2e-org", &server.base_url());
    let validated = manager
        .validate_activation("db-key", "db-activation")
        .await
        .unwrap();

    validate_mock.assert();
    assert_eq!(validated.status, LicenseStatus::Granted);
    assert_eq!(validated.validations, 9);

    db.save_license(&LicenseData {
        license_key: Some(validated.license_key.clone()),
        activation_id: validated.activation_id.clone(),
        status: "active".to_string(),
        customer_email: validated.customer_email.clone(),
        customer_name: validated.customer_name.clone(),
        expires_at: validated.expires_at.clone(),
        is_activated: true,
        last_validated_at: Some(Utc::now().to_rfc3339()),
        trial_started_at: None,
        trial_integrity_hash: None,
        usage: validated.usage,
        validations: validated.validations,
    })
    .unwrap();

    let stored = db.get_license().unwrap();
    assert_eq!(stored.usage, 3);
    assert_eq!(stored.validations, 9);
    assert_eq!(can_use_backend(&db), ("licensed", true));
    let _ = clear_cache();
}

#[tokio::test]
async fn e2e_trial_to_license_activation_validation_and_deactivation() {
    let _ = clear_cache();
    let (_dir, db) = test_database();
    db.save_license(&trial_license(Utc::now() - Duration::days(6)))
        .unwrap();
    assert_eq!(can_use_backend(&db), ("trial", true));

    let server = MockServer::start();
    let activate_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/activate")
            .body_contains("e2e-key")
            .body_contains("e2e-org");
        then.status(200).json_body(json!({
            "id": "activation_e2e",
            "license_key_id": "lk_e2e",
            "label": "E2E Device",
            "created_at": "2026-04-19T00:00:00Z",
            "license_key": {
                "id": "lk_e2e",
                "organization_id": "e2e-org",
                "customer_id": "cust_e2e",
                "customer": {
                    "id": "cust_e2e",
                    "email": "e2e-customer@whisprtypr.test",
                    "name": "E2E User"
                },
                "benefit_id": "benefit_e2e",
                "key": "e2e-key",
                "display_key": "****-e2e",
                "status": "granted",
                "usage": 1,
                "validations": 1,
                "limit_activations": 3,
                "expires_at": null
            }
        }));
    });
    let validate_after_activate_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/validate")
            .body_contains("e2e-key")
            .body_contains("activation_e2e");
        then.status(200).json_body(json!({
            "id": "validation_e2e",
            "organization_id": "e2e-org",
            "customer_id": "cust_e2e",
            "customer": {
                "id": "cust_e2e",
                "email": "e2e-customer@whisprtypr.test",
                "name": "E2E User"
            },
            "benefit_id": "benefit_e2e",
            "key": "e2e-key",
            "display_key": "****-e2e",
            "status": "granted",
            "usage": 1,
            "limit_activations": 3,
            "validations": 2,
            "expires_at": null,
            "activation": {
                "id": "activation_e2e",
                "license_key_id": "lk_e2e",
                "label": "E2E Device",
                "created_at": "2026-04-19T00:00:00Z"
            }
        }));
    });
    let deactivate_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/deactivate")
            .body_contains("e2e-key")
            .body_contains("activation_e2e");
        then.status(204);
    });

    let manager = LicenseManager::with_org_id("e2e-org", &server.base_url());
    let activated = manager.activate("e2e-key").await.unwrap();

    activate_mock.assert();
    validate_after_activate_mock.assert();
    assert_eq!(activated.status, LicenseStatus::Granted);
    assert_eq!(activated.activation_id.as_deref(), Some("activation_e2e"));

    db.save_license(&LicenseData {
        license_key: Some(activated.license_key.clone()),
        activation_id: activated.activation_id.clone(),
        status: "active".to_string(),
        customer_email: activated.customer_email.clone(),
        customer_name: activated.customer_name.clone(),
        expires_at: activated.expires_at.clone(),
        is_activated: true,
        last_validated_at: Some(Utc::now().to_rfc3339()),
        trial_started_at: db.get_license().unwrap().trial_started_at,
        trial_integrity_hash: db.get_license().unwrap().trial_integrity_hash,
        usage: activated.usage,
        validations: activated.validations,
    })
    .unwrap();

    assert_eq!(can_use_backend(&db), ("licensed", true));

    let validated = manager
        .validate_activation("e2e-key", "activation_e2e")
        .await
        .unwrap();
    assert_eq!(validated.status, LicenseStatus::Granted);
    assert_eq!(validated.validations, 2);

    db.save_license(&LicenseData {
        license_key: Some(validated.license_key.clone()),
        activation_id: validated.activation_id.clone(),
        status: "active".to_string(),
        customer_email: validated.customer_email.clone(),
        customer_name: validated.customer_name.clone(),
        expires_at: validated.expires_at.clone(),
        is_activated: true,
        last_validated_at: Some(Utc::now().to_rfc3339()),
        trial_started_at: db.get_license().unwrap().trial_started_at,
        trial_integrity_hash: db.get_license().unwrap().trial_integrity_hash,
        usage: validated.usage,
        validations: validated.validations,
    })
    .unwrap();

    assert_eq!(db.get_license().unwrap().validations, 2);
    assert_eq!(can_use_backend(&db), ("licensed", true));

    manager
        .deactivate_activation("e2e-key", "activation_e2e")
        .await
        .unwrap();
    deactivate_mock.assert();
    db.clear_license().unwrap();

    let after_deactivation = db.get_license().unwrap();
    assert_eq!(after_deactivation.status, "trial_expired");
    assert!(after_deactivation.license_key.is_none());
    assert_eq!(can_use_backend(&db), ("trial_expired", false));
    let _ = clear_cache();
}

#[tokio::test]
async fn e2e_model_download_updates_database_and_survives_reopen() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET).path("/ggml-base.bin");
        then.status(200)
            .header("content-length", "4096")
            .body(vec![7u8; 4096]);
    });

    let dir = tempfile::tempdir().unwrap();
    let app_data_dir = dir.path().join("app-data");
    let models_dir = app_data_dir.join("models");
    let db = Database::new(app_data_dir.clone()).unwrap();
    let mut downloader = ModelDownloader::new(models_dir);
    downloader.test_url_override = Some(server.url("/ggml-base.bin"));

    let downloaded_path = downloader.download_model("base", |_| {}).await.unwrap();

    mock.assert();
    assert!(downloaded_path.exists());
    assert!(downloader.is_model_downloaded("base"));

    db.set_model_downloaded("base", true, Some(&downloaded_path.to_string_lossy()))
        .unwrap();

    let model = db.get_model("base").unwrap().unwrap();
    assert!(model.downloaded);
    assert_eq!(
        model.download_path,
        Some(downloaded_path.to_string_lossy().to_string())
    );

    drop(db);
    let reopened = Database::new(app_data_dir).unwrap();
    let reopened_model = reopened.get_model("base").unwrap().unwrap();

    assert!(reopened_model.downloaded);
    assert_eq!(
        reopened_model.download_path,
        Some(downloaded_path.to_string_lossy().to_string())
    );
}
