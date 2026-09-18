use crate::{
    database::LicenseData,
    license::{clear_cache, get_device_id, get_device_label, LicenseInfo},
    CommandError, CommandResult, DbState, LicenseManagerState,
};
use tauri::State;

#[derive(Debug, serde::Serialize)]
pub struct LicenseResponse {
    license_key: Option<String>,
    display_key: Option<String>,
    activation_id: Option<String>,
    status: String,
    customer_email: Option<String>,
    customer_name: Option<String>,
    benefit_id: Option<String>,
    expires_at: Option<String>,
    is_activated: bool,
    last_validated_at: Option<String>,
    trial_started_at: Option<String>,
    trial_days_remaining: Option<i64>,
    device_id: String,
    device_label: String,
    limit_activations: Option<i32>,
    usage: i32,
    validations: i32,
}

impl From<LicenseInfo> for LicenseResponse {
    fn from(info: LicenseInfo) -> Self {
        Self {
            license_key: Some(crate::security::mask_license_key(&info.license_key)),
            display_key: Some(info.display_key),
            activation_id: info.activation_id,
            status: crate::license_status_to_response(&info.status),
            customer_email: info.customer_email,
            customer_name: info.customer_name,
            benefit_id: info.benefit_id,
            expires_at: info.expires_at,
            is_activated: info.status.allows_usage(),
            last_validated_at: info.last_validated_at,
            trial_started_at: None,
            trial_days_remaining: None,
            device_id: info.device_id,
            device_label: info.device_label,
            limit_activations: info.limit_activations,
            usage: info.usage,
            validations: info.validations,
        }
    }
}

impl From<LicenseData> for LicenseResponse {
    fn from(data: LicenseData) -> Self {
        let trial_days_remaining = if let Some(ref trial_started) = data.trial_started_at {
            if let Ok(start_date) = chrono::DateTime::parse_from_rfc3339(trial_started) {
                let now = chrono::Utc::now();
                let days_since_start = (now - start_date.with_timezone(&chrono::Utc)).num_days();
                Some((7 - days_since_start).max(0))
            } else {
                None
            }
        } else {
            None
        };

        Self {
            license_key: data
                .license_key
                .as_deref()
                .map(crate::security::mask_license_key),
            display_key: None,
            activation_id: data.activation_id,
            status: data.status,
            customer_email: data.customer_email,
            customer_name: data.customer_name,
            benefit_id: None,
            expires_at: data.expires_at,
            is_activated: data.is_activated,
            last_validated_at: data.last_validated_at,
            trial_started_at: data.trial_started_at,
            trial_days_remaining,
            device_id: get_device_id(),
            device_label: get_device_label(),
            limit_activations: None,
            usage: data.usage,
            validations: data.validations,
        }
    }
}

#[tauri::command]
pub async fn get_license(
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
) -> CommandResult<LicenseResponse> {
    if let Some(info) = license_manager.0.get_cached_info() {
        return Ok(LicenseResponse::from(info));
    }

    let license = db.0.get_license().map_err(CommandError::Database)?;

    if crate::db_license_allows_usage(&license) {
        match license_manager.0.validate().await {
            Ok(info) => return Ok(LicenseResponse::from(info)),
            Err(error) => {
                if license_manager.0.is_authoritative_validate_error(&error) {
                    log::warn!("get_license: license authoritatively rejected: {}", error);
                    let _ = clear_cache();
                    return Ok(LicenseResponse::from(license));
                }
            }
        }
    }

    Ok(LicenseResponse::from(license))
}

#[tauri::command]
pub async fn activate_license(
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
    license_key: String,
) -> CommandResult<LicenseResponse> {
    log::info!("Activating license key...");

    let license_info = license_manager
        .0
        .activate(&license_key)
        .await
        .map_err(CommandError::License)?;

    if !license_info.status.allows_usage() {
        let _ = clear_cache();
        return Err(CommandError::License(format!(
            "License activation did not grant access: {}",
            crate::license_status_to_response(&license_info.status)
        )));
    }

    let is_activated = license_info.status.allows_usage();
    let license_data = LicenseData {
        license_key: Some(license_key),
        activation_id: license_info.activation_id.clone(),
        status: crate::license_status_to_response(&license_info.status),
        customer_email: license_info.customer_email.clone(),
        customer_name: license_info.customer_name.clone(),
        expires_at: license_info.expires_at.clone(),
        is_activated,
        last_validated_at: Some(chrono::Utc::now().to_rfc3339()),
        trial_started_at: None,
        trial_integrity_hash: None,
        trial_salt: None,
        usage: license_info.usage,
        validations: license_info.validations,
    };

    db.0.save_license(&license_data)
        .map_err(CommandError::Database)?;

    log::info!("License activated successfully!");
    Ok(LicenseResponse::from(license_info))
}

#[tauri::command]
pub async fn validate_license(
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
) -> CommandResult<LicenseResponse> {
    log::info!("Validating license...");

    let license_info = match license_manager.0.validate().await {
        Ok(info) => info,
        Err(error) => {
            if license_manager.0.is_authoritative_validate_error(&error) {
                log::warn!("License authoritatively rejected: {}", error);
                let _ = clear_cache();
                return Err(CommandError::License(error));
            }

            let stored_license = db.0.get_license().map_err(CommandError::Database)?;
            if !crate::db_license_allows_usage(&stored_license) {
                return Err(CommandError::License(error));
            }

            return Ok(LicenseResponse::from(stored_license));
        }
    };

    let license_data = LicenseData {
        license_key: Some(license_info.license_key.clone()),
        activation_id: license_info.activation_id.clone(),
        status: crate::license_status_to_response(&license_info.status),
        customer_email: license_info.customer_email.clone(),
        customer_name: license_info.customer_name.clone(),
        expires_at: license_info.expires_at.clone(),
        is_activated: license_info.status.allows_usage(),
        last_validated_at: license_info.last_validated_at.clone(),
        trial_started_at: None,
        trial_integrity_hash: None,
        trial_salt: None,
        usage: license_info.usage,
        validations: license_info.validations,
    };

    let _ = db.0.save_license(&license_data);

    log::info!("License validated: {:?}", license_info.status);
    Ok(LicenseResponse::from(license_info))
}

#[tauri::command]
pub async fn deactivate_license(
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
) -> CommandResult<()> {
    log::info!("Deactivating license...");

    let deactivate_result = license_manager.0.deactivate().await;
    if let Err(error) = deactivate_result {
        let stored_license = db.0.get_license().map_err(CommandError::Database)?;
        let Some(license_key) = stored_license.license_key.as_deref() else {
            return Err(CommandError::License(error));
        };
        let Some(activation_id) = stored_license.activation_id.as_deref() else {
            return Err(CommandError::License(error));
        };

        if !crate::db_license_allows_usage(&stored_license) {
            return Err(CommandError::License(error));
        }

        license_manager
            .0
            .deactivate_activation(license_key, activation_id)
            .await
            .map_err(CommandError::License)?;
    }

    db.0.clear_license().map_err(CommandError::Database)?;

    log::info!("License deactivated successfully");
    Ok(())
}

#[tauri::command]
pub fn clear_stored_license(db: State<DbState>) -> CommandResult<()> {
    let _ = clear_cache();
    db.0.clear_license().map_err(Into::into)
}

#[tauri::command]
#[allow(unused_variables)]
pub async fn is_license_valid(
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
) -> CommandResult<bool> {
    let db = db.0.clone();
    let license_manager = license_manager.0.clone();

    match crate::verified_license_check(&license_manager).await {
        crate::VerifiedLicenseResult::Granted => return Ok(true),
        crate::VerifiedLicenseResult::Rejected => return Ok(false),
        crate::VerifiedLicenseResult::OfflineFallback => {}
    }

    if let Ok(license) = db.get_license() {
        if crate::db_license_allows_usage(&license) {
            return Ok(true);
        }
    }

    Ok(crate::has_active_trial(&db))
}

#[tauri::command]
pub fn start_trial(db: State<DbState>) -> CommandResult<LicenseResponse> {
    let mut license = db.0.get_license().map_err(CommandError::Database)?;

    if license.is_activated && license.status == "active" {
        return Err(CommandError::License(
            "Already have an active license".to_string(),
        ));
    }

    if license.trial_started_at.is_some() {
        if let Some(ref trial_started) = license.trial_started_at {
            let Some(ref trial_salt) = license.trial_salt else {
                log::warn!("Trial salt missing, treating as invalid");
                license.status = "trial_expired".to_string();
                db.0.save_license(&license)
                    .map_err(CommandError::Database)?;
                let _ = clear_cache();
                return Err(CommandError::License(
                    "Trial state is invalid. Please activate a license.".to_string(),
                ));
            };
            let expected_hash = crate::calculate_trial_integrity_hash(trial_started, trial_salt);
            if license.trial_integrity_hash.as_deref() != Some(expected_hash.as_str()) {
                log::warn!("Trial integrity check failed during start_trial");
                license.status = "trial_expired".to_string();
                db.0.save_license(&license)
                    .map_err(CommandError::Database)?;
                let _ = clear_cache();
                return Err(CommandError::License(
                    "Trial state is invalid. Please activate a license.".to_string(),
                ));
            }

            if let Ok(start_date) = chrono::DateTime::parse_from_rfc3339(trial_started) {
                let now = chrono::Utc::now();
                let days_since_start = (now - start_date.with_timezone(&chrono::Utc)).num_days();
                if days_since_start < 0 {
                    log::warn!("Invalid future trial start detected");
                    license.status = "trial_expired".to_string();
                    db.0.save_license(&license)
                        .map_err(CommandError::Database)?;
                    let _ = clear_cache();
                    return Err(CommandError::License(
                        "Trial state is invalid. Please activate a license.".to_string(),
                    ));
                }

                if days_since_start >= 7 {
                    license.status = "trial_expired".to_string();
                    db.0.save_license(&license)
                        .map_err(CommandError::Database)?;
                    let _ = clear_cache();
                    return Err(CommandError::License(
                        "Trial has expired. Please purchase a license.".to_string(),
                    ));
                }
            }
        }
        return Ok(LicenseResponse::from(license));
    }

    let _ = clear_cache();

    let trial_started_at = chrono::Utc::now().to_rfc3339();
    let trial_salt = crate::generate_trial_salt();
    license.status = "trial".to_string();
    license.trial_started_at = Some(trial_started_at.clone());
    license.trial_salt = Some(trial_salt.clone());
    license.trial_integrity_hash = Some(crate::calculate_trial_integrity_hash(&trial_started_at, &trial_salt));
    license.is_activated = false;

    db.0.save_license(&license)
        .map_err(CommandError::Database)?;

    log::info!("Trial started");
    Ok(LicenseResponse::from(license))
}

#[tauri::command]
pub fn get_device_info() -> serde_json::Value {
    serde_json::json!({
        "device_id": get_device_id(),
        "device_label": get_device_label(),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
    })
}

#[tauri::command]
#[allow(unused_variables)]
pub async fn get_trial_status(
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
) -> CommandResult<serde_json::Value> {
    let db = db.0.clone();
    let license_manager = license_manager.0.clone();

    match crate::verified_license_check(&license_manager).await {
        crate::VerifiedLicenseResult::Granted => {
            return Ok(serde_json::json!({
                "isInTrial": false,
                "daysRemaining": 0,
                "trialExpired": false,
                "hasLicense": true
            }));
        }
        crate::VerifiedLicenseResult::Rejected => {}
        crate::VerifiedLicenseResult::OfflineFallback => {}
    }

    let license = db.get_license().map_err(CommandError::Database)?;
    if crate::db_license_allows_usage(&license) {
        return Ok(serde_json::json!({
            "isInTrial": false,
            "daysRemaining": 0,
            "trialExpired": false,
            "hasLicense": true
        }));
    }

    if let Some(trial_started) = &license.trial_started_at {
        let Some(ref trial_salt) = license.trial_salt else {
            log::warn!("Trial salt missing in get_trial_status");
            let mut expired = license.clone();
            expired.status = "trial_expired".to_string();
            let _ = db.save_license(&expired);
            return Ok(serde_json::json!({
                "isInTrial": false,
                "daysRemaining": 0,
                "trialExpired": true,
                "hasLicense": false
            }));
        };
        let expected_hash = crate::calculate_trial_integrity_hash(trial_started, trial_salt);
        if license.trial_integrity_hash.as_deref() != Some(expected_hash.as_str()) {
            log::warn!("Trial integrity check failed in get_trial_status");
            let mut expired = license.clone();
            expired.status = "trial_expired".to_string();
            let _ = db.save_license(&expired);
            return Ok(serde_json::json!({
                "isInTrial": false,
                "daysRemaining": 0,
                "trialExpired": true,
                "hasLicense": false
            }));
        }

        if let Ok(start_date) = chrono::DateTime::parse_from_rfc3339(trial_started) {
            let now = chrono::Utc::now();
            let days_since_start = (now - start_date.with_timezone(&chrono::Utc)).num_days();
            let days_remaining = if days_since_start < 0 {
                0
            } else {
                (7 - days_since_start).max(0)
            };

            if days_remaining <= 0 {
                let mut expired = license.clone();
                expired.status = "trial_expired".to_string();
                let _ = db.save_license(&expired);
            }

            return Ok(serde_json::json!({
                "isInTrial": days_remaining > 0,
                "daysRemaining": days_remaining,
                "trialExpired": days_remaining <= 0,
                "hasLicense": false
            }));
        }
    }

    Ok(serde_json::json!({
        "isInTrial": false,
        "daysRemaining": 0,
        "trialExpired": false,
        "hasLicense": false
    }))
}

#[tauri::command]
#[allow(unused_variables)]
pub async fn can_use_app(
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
) -> CommandResult<serde_json::Value> {
    let db = db.0.clone();
    let license_manager = license_manager.0.clone();

    match crate::verified_license_check(&license_manager).await {
        crate::VerifiedLicenseResult::Granted => {
            return Ok(serde_json::json!({
                "canUse": true,
                "reason": "licensed",
                "daysRemaining": null
            }));
        }
        crate::VerifiedLicenseResult::Rejected => {}
        crate::VerifiedLicenseResult::OfflineFallback => {}
    }

    let license = db.get_license().map_err(CommandError::Database)?;
    if crate::db_license_allows_usage(&license) {
        return Ok(serde_json::json!({
            "canUse": true,
            "reason": "licensed",
            "daysRemaining": null
        }));
    }

    if let Some(trial_started) = &license.trial_started_at {
        let Some(ref trial_salt) = license.trial_salt else {
            log::warn!("Trial salt missing in can_use_app");
            let mut expired = license.clone();
            expired.status = "trial_expired".to_string();
            let _ = db.save_license(&expired);
            return Ok(serde_json::json!({
                "canUse": false,
                "reason": "trial_expired",
                "daysRemaining": 0
            }));
        };
        let expected_hash = crate::calculate_trial_integrity_hash(trial_started, trial_salt);
        if license.trial_integrity_hash.as_deref() != Some(expected_hash.as_str()) {
            log::warn!("Trial integrity check failed in can_use_app");
            let mut expired = license.clone();
            expired.status = "trial_expired".to_string();
            let _ = db.save_license(&expired);
            return Ok(serde_json::json!({
                "canUse": false,
                "reason": "trial_expired",
                "daysRemaining": 0
            }));
        }

        if let Ok(start_date) = chrono::DateTime::parse_from_rfc3339(trial_started) {
            let now = chrono::Utc::now();
            let days_since_start = (now - start_date.with_timezone(&chrono::Utc)).num_days();
            let days_remaining = if days_since_start < 0 {
                0
            } else {
                (7 - days_since_start).max(0)
            };

            if days_remaining > 0 {
                return Ok(serde_json::json!({
                    "canUse": true,
                    "reason": "trial",
                    "daysRemaining": days_remaining
                }));
            }

            let mut expired = license.clone();
            expired.status = "trial_expired".to_string();
            let _ = db.save_license(&expired);
            return Ok(serde_json::json!({
                "canUse": false,
                "reason": "trial_expired",
                "daysRemaining": 0
            }));
        }
    }

    Ok(serde_json::json!({
        "canUse": false,
        "reason": "no_license",
        "daysRemaining": null
    }))
}
