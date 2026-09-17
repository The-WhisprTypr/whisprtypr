use log::{debug, error, info, warn};
use reqwest::Client;
use std::sync::Mutex;
use std::time::Duration;

use crate::license::cache::{clear_cache, load_cache, store_cache};
use crate::license::device::{get_device_conditions, get_device_meta};
use crate::license::models::{
    ActivateRequest, ActivateResponse, DeactivateRequest, LicenseInfo, LicenseStatus,
    LicenseValidationOutcome, PolarError, ValidateResponse, CACHE_VERSION,
    ONLINE_VALIDATION_MIN_INTERVAL_HOURS, POLAR_API_BASE, POLAR_ORG_ID, REQUEST_TIMEOUT_SECS,
};
use crate::security;

pub struct LicenseManager {
    client: Client,
    pub org_id: String,
    pub api_base: String,
    pub last_online_attempt: Mutex<Option<chrono::DateTime<chrono::Utc>>>,
}

impl LicenseManager {
    pub fn new() -> Self {
        Self::with_org_id(POLAR_ORG_ID, POLAR_API_BASE)
    }

    pub fn with_org_id(org_id: &str, api_base: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            org_id: org_id.to_string(),
            api_base: api_base.to_string(),
            last_online_attempt: Mutex::new(None),
        }
    }

    pub fn is_cached_license_valid(&self) -> bool {
        let Some(cache) = load_cache() else {
            return false;
        };
        crate::license::models::cached_license_allows_offline(&cache)
    }

    fn mark_online_attempt(&self) {
        if let Ok(mut guard) = self.last_online_attempt.lock() {
            *guard = Some(chrono::Utc::now());
        }
    }

    pub fn should_throttle_online_validation(&self) -> bool {
        let Ok(guard) = self.last_online_attempt.lock() else {
            return false;
        };
        let Some(last) = *guard else {
            return false;
        };
        let elapsed = (chrono::Utc::now() - last).num_hours();
        elapsed >= 0 && elapsed < ONLINE_VALIDATION_MIN_INTERVAL_HOURS
    }

    pub async fn validate_smart(&self) -> Result<LicenseInfo, String> {
        if self.is_cached_license_valid() {
            if let Some(cache) = load_cache() {
                let info = self.license_info_from_cache(&cache);
                if info.status.allows_usage() {
                    debug!("Using cached license (skipping online validation)");
                    return Ok(info);
                }
            }
        }
        self.validate_throttled().await
    }

    pub fn license_info_from_cache(
        &self,
        cache: &crate::license::models::CachedLicense,
    ) -> LicenseInfo {
        LicenseInfo {
            license_key: cache.license_key.clone(),
            display_key: mask_key(&cache.license_key),
            status: LicenseStatus::Granted,
            activation_id: Some(cache.activation_id.clone()),
            customer_email: cache.customer_email.clone(),
            customer_name: cache.customer_name.clone(),
            benefit_id: Some(cache.benefit_id.clone()),
            expires_at: cache.expires_at.clone(),
            limit_activations: None,
            usage: cache.usage,
            limit_usage: None,
            validations: cache.validations,
            last_validated_at: Some(cache.last_validated_at.clone()),
            device_id: crate::license::device::get_device_id(),
            device_label: crate::license::device::get_device_label(),
        }
    }

    pub async fn validate_throttled(&self) -> Result<LicenseInfo, String> {
        if self.should_throttle_online_validation() && self.is_cached_license_valid() {
            if let Some(cache) = load_cache() {
                debug!("Online validation throttled; serving cached license");
                return Ok(self.license_info_from_cache(&cache));
            }
        }

        self.mark_online_attempt();
        self.validate().await
    }

    pub async fn activate(&self, license_key: &str) -> Result<LicenseInfo, String> {
        let device_id = crate::license::device::get_device_id();
        let device_label = crate::license::device::get_device_label();

        info!(
            "Activating license on device: {} ({})",
            device_label, device_id
        );

        let request = ActivateRequest {
            key: license_key.to_string(),
            organization_id: self.org_id.clone(),
            label: device_label.clone(),
            conditions: Some(get_device_conditions(&device_id)),
            meta: Some(get_device_meta()),
        };

        let url = format!("{}/activate", self.api_base);
        debug!("POST {}", url);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        debug!("Response status: {}", status);
        debug!("Response body: {}", body);

        if status.is_success() {
            let data: ActivateResponse = serde_json::from_str(&body)
                .map_err(|e| format!("Failed to parse response: {} - Body: {}", e, body))?;

            info!("License activated successfully!");
            info!("  Activation ID: {}", data.id);
            info!("  Status: {}", data.license_key.status);
            info!(
                "  Activations: {}/{:?}",
                data.license_key.usage, data.license_key.limit_activations
            );

            let license_status = self.check_license_status(&data.license_key);

            let mut cache = crate::license::models::CachedLicense {
                license_key: license_key.to_string(),
                license_key_id: data.license_key.id.clone(),
                activation_id: data.id.clone(),
                device_id: device_id.clone(),
                device_label: device_label.clone(),
                customer_email: data.license_key.customer.as_ref().map(|c| c.email.clone()),
                customer_name: data
                    .license_key
                    .customer
                    .as_ref()
                    .and_then(|c| c.name.clone()),
                benefit_id: data.license_key.benefit_id.clone(),
                expires_at: data.license_key.expires_at.clone(),
                last_validated_at: chrono::Utc::now().to_rfc3339(),
                status: data.license_key.status.clone(),
                usage: data.license_key.usage,
                validations: data.license_key.validations,
                integrity_hash: String::new(),
                cache_version: CACHE_VERSION,
            };

            store_cache(&cache)?;

            match crate::license::api::perform_validate(
                &self.client,
                &self.org_id,
                &self.api_base,
                &cache.license_key,
                &cache.activation_id,
                Some(cache.benefit_id.clone()),
                Some(1),
            )
            .await
            {
                Ok(validate_resp) => {
                    cache.last_validated_at = chrono::Utc::now().to_rfc3339();
                    cache.status = validate_resp.status.clone();
                    cache.usage = validate_resp.usage;
                    cache.validations = validate_resp.validations;
                    let _ = store_cache(&cache);

                    let license_status = self.check_license_status_from_validate(&validate_resp);

                    return Ok(LicenseInfo {
                        license_key: cache.license_key.clone(),
                        display_key: validate_resp.display_key,
                        status: license_status,
                        activation_id: validate_resp
                            .activation
                            .as_ref()
                            .map(|a| a.id.clone())
                            .or_else(|| Some(cache.activation_id.clone())),
                        customer_email: validate_resp.customer.as_ref().map(|c| c.email.clone()),
                        customer_name: validate_resp.customer.as_ref().and_then(|c| c.name.clone()),
                        benefit_id: Some(validate_resp.benefit_id),
                        expires_at: validate_resp.expires_at,
                        limit_activations: validate_resp.limit_activations,
                        usage: validate_resp.usage,
                        limit_usage: validate_resp.limit_usage,
                        validations: validate_resp.validations,
                        last_validated_at: validate_resp.last_validated_at,
                        device_id,
                        device_label,
                    });
                }
                Err(e) => {
                    warn!("Validation after activation failed: {}", e);
                }
            }

            Ok(LicenseInfo {
                license_key: license_key.to_string(),
                display_key: data.license_key.display_key.clone(),
                status: license_status,
                activation_id: Some(data.id),
                customer_email: data.license_key.customer.as_ref().map(|c| c.email.clone()),
                customer_name: data
                    .license_key
                    .customer
                    .as_ref()
                    .and_then(|c| c.name.clone()),
                benefit_id: Some(data.license_key.benefit_id),
                expires_at: data.license_key.expires_at,
                limit_activations: data.license_key.limit_activations,
                usage: data.license_key.usage,
                limit_usage: data.license_key.limit_usage,
                validations: data.license_key.validations,
                last_validated_at: data.license_key.last_validated_at,
                device_id,
                device_label,
            })
        } else if status.as_u16() == 403 {
            let err: PolarError = serde_json::from_str(&body).unwrap_or(PolarError {
                error: Some("Activation limit reached".to_string()),
                detail: None,
                error_type: None,
            });
            error!("Activation limit reached: {:?}", err);
            Err(
                "Activation limit reached. Please deactivate from another device first."
                    .to_string(),
            )
        } else if status.as_u16() == 404 {
            error!("License key not found");
            Err("Invalid license key. Please check and try again.".to_string())
        } else if status.as_u16() == 422 {
            let _err: PolarError = serde_json::from_str(&body).unwrap_or(PolarError {
                error: Some("Validation error".to_string()),
                detail: None,
                error_type: None,
            });
            error!("Activation request rejected by license server");
            debug!("Activation rejection response body: {}", body);
            Err("Invalid license request. Please check your key and try again.".to_string())
        } else {
            error!("Activation failed: {} - {}", status, body);
            Err(format!("Activation failed: HTTP {}", status))
        }
    }

    pub async fn validate(&self) -> Result<LicenseInfo, String> {
        let device_id = crate::license::device::get_device_id();
        let device_label = crate::license::device::get_device_label();

        let cache = load_cache();

        if let Some(ref cached) = cache {
            info!("Validating license with Polar API...");

            match crate::license::api::perform_validate(
                &self.client,
                &self.org_id,
                &self.api_base,
                &cached.license_key,
                &cached.activation_id,
                Some(cached.benefit_id.clone()),
                Some(1),
            )
            .await
            {
                Ok(data) => {
                    let license_status = self.check_license_status_from_validate(&data);

                    if let Some(limit) = data.limit_activations {
                        if limit > 0 {
                            if let Ok(license_key_info) = crate::license::api::get_license_key(
                                &self.client,
                                &self.api_base,
                                &cached.license_key_id,
                            )
                            .await
                            {
                                let activation_count = license_key_info.activations.len() as i32;
                                if activation_count >= limit {
                                    warn!(
                                        "Activation limit reached: {} activations, limit: {}",
                                        activation_count, limit
                                    );
                                    let _ = clear_cache();
                                    return Err(format!(
                                        "Activation limit reached ({}/{}). Please deactivate from another device first.",
                                        activation_count, limit
                                    ));
                                }
                                info!("Activation count: {}/{}", activation_count, limit);
                            }
                        }
                    }

                    info!("License validated successfully!");
                    info!("  Status: {} -> {:?}", data.status, license_status);
                    info!("  Validations: {}", data.validations);
                    info!("  Has activation: {}", data.activation.is_some());

                    let mut updated_cache = cached.clone();
                    updated_cache.last_validated_at = chrono::Utc::now().to_rfc3339();
                    updated_cache.status = data.status.clone();
                    updated_cache.usage = data.usage;
                    updated_cache.validations = data.validations;
                    let _ = store_cache(&updated_cache);

                    return Ok(LicenseInfo {
                        license_key: cached.license_key.clone(),
                        display_key: data.display_key,
                        status: license_status,
                        activation_id: data
                            .activation
                            .as_ref()
                            .map(|a| a.id.clone())
                            .or_else(|| Some(cached.activation_id.clone())),
                        customer_email: data.customer.as_ref().map(|c| c.email.clone()),
                        customer_name: data.customer.as_ref().and_then(|c| c.name.clone()),
                        benefit_id: Some(data.benefit_id),
                        expires_at: data.expires_at,
                        limit_activations: data.limit_activations,
                        usage: data.usage,
                        limit_usage: data.limit_usage,
                        validations: data.validations,
                        last_validated_at: data.last_validated_at,
                        device_id: device_id.clone(),
                        device_label: device_label.clone(),
                    });
                }
                Err(error) => match self.classify_validate_error(&error) {
                    LicenseValidationOutcome::Granted => {}
                    LicenseValidationOutcome::Rejected => {
                        warn!("License rejected by Polar - clearing cache: {}", error);
                        let _ = clear_cache();
                        return Err(
                            "License validation was rejected. Please activate again.".to_string()
                        );
                    }
                    LicenseValidationOutcome::OfflineFallback => {
                        warn!("Validation failed (non-authoritative): {}", error);
                    }
                },
            }

            return self.validate_offline(cached, &device_id, &device_label);
        }

        Err("No license activated. Please enter your license key.".to_string())
    }

    pub async fn validate_activation(
        &self,
        license_key: &str,
        activation_id: &str,
    ) -> Result<LicenseInfo, String> {
        let device_id = crate::license::device::get_device_id();
        let device_label = crate::license::device::get_device_label();

        info!("Validating license from stored activation...");

        let data = crate::license::api::perform_validate(
            &self.client,
            &self.org_id,
            &self.api_base,
            license_key,
            activation_id,
            None,
            Some(1),
        )
        .await?;

        let license_status = self.check_license_status_from_validate(&data);

        if let Some(limit) = data.limit_activations {
            if limit > 0 {
                if let Ok(license_key_info) =
                    crate::license::api::get_license_key(&self.client, &self.api_base, &data.id)
                        .await
                {
                    let activation_count = license_key_info.activations.len() as i32;
                    if activation_count >= limit {
                        warn!(
                            "Activation limit reached: {} activations, limit: {}",
                            activation_count, limit
                        );
                        let _ = clear_cache();
                        return Err(format!(
                            "Activation limit reached ({}/{}). Please deactivate from another device first.",
                            activation_count, limit
                        ));
                    }
                    info!("Activation count: {}/{}", activation_count, limit);
                }
            }
        }

        let cache = crate::license::models::CachedLicense {
            license_key: license_key.to_string(),
            license_key_id: data.id.clone(),
            activation_id: data
                .activation
                .as_ref()
                .map(|a| a.id.clone())
                .unwrap_or_else(|| activation_id.to_string()),
            device_id: device_id.clone(),
            device_label: device_label.clone(),
            customer_email: data.customer.as_ref().map(|c| c.email.clone()),
            customer_name: data.customer.as_ref().and_then(|c| c.name.clone()),
            benefit_id: data.benefit_id.clone(),
            expires_at: data.expires_at.clone(),
            last_validated_at: chrono::Utc::now().to_rfc3339(),
            status: data.status.clone(),
            usage: data.usage,
            validations: data.validations,
            integrity_hash: String::new(),
            cache_version: CACHE_VERSION,
        };
        let _ = store_cache(&cache);

        Ok(LicenseInfo {
            license_key: license_key.to_string(),
            display_key: data.display_key,
            status: license_status,
            activation_id: Some(cache.activation_id),
            customer_email: data.customer.as_ref().map(|c| c.email.clone()),
            customer_name: data.customer.as_ref().and_then(|c| c.name.clone()),
            benefit_id: Some(data.benefit_id),
            expires_at: data.expires_at,
            limit_activations: data.limit_activations,
            usage: data.usage,
            limit_usage: data.limit_usage,
            validations: data.validations,
            last_validated_at: data.last_validated_at,
            device_id,
            device_label,
        })
    }

    fn validate_offline(
        &self,
        cache: &crate::license::models::CachedLicense,
        device_id: &str,
        device_label: &str,
    ) -> Result<LicenseInfo, String> {
        if crate::license::models::cached_license_allows_offline(cache) {
            info!("Using offline license within grace period");

            return Ok(LicenseInfo {
                license_key: cache.license_key.clone(),
                display_key: mask_key(&cache.license_key),
                status: LicenseStatus::Offline,
                activation_id: Some(cache.activation_id.clone()),
                customer_email: cache.customer_email.clone(),
                customer_name: cache.customer_name.clone(),
                benefit_id: Some(cache.benefit_id.clone()),
                expires_at: cache.expires_at.clone(),
                limit_activations: None,
                usage: cache.usage,
                limit_usage: None,
                validations: cache.validations,
                last_validated_at: Some(cache.last_validated_at.clone()),
                device_id: device_id.to_string(),
                device_label: device_label.to_string(),
            });
        }

        error!("Offline grace period expired or cache is invalid");
        Err("License validation failed and offline grace period expired. Please connect to the internet."
            .to_string())
    }

    pub async fn deactivate(&self) -> Result<(), String> {
        let cache = load_cache().ok_or("No license to deactivate")?;

        self.deactivate_activation(&cache.license_key, &cache.activation_id)
            .await
    }

    pub async fn deactivate_activation(
        &self,
        license_key: &str,
        activation_id: &str,
    ) -> Result<(), String> {
        info!("Deactivating license from device...");

        let request = DeactivateRequest {
            key: license_key.to_string(),
            organization_id: self.org_id.clone(),
            activation_id: activation_id.to_string(),
        };

        let url = format!("{}/deactivate", self.api_base);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        let status = response.status();

        if status.is_success() || status.as_u16() == 204 {
            info!("License deactivated successfully");
            clear_cache()?;
            Ok(())
        } else if status.as_u16() == 404 {
            warn!("Activation not found on server - clearing local cache");
            clear_cache()?;
            Ok(())
        } else {
            let body = response.text().await.unwrap_or_default();
            error!("Deactivation failed: {}", status);
            debug!("Deactivation response body: {}", body);
            Err("License deactivation failed. Please try again.".to_string())
        }
    }

    pub fn is_valid(&self) -> bool {
        if let Some(cache) = load_cache() {
            return crate::license::models::cached_license_allows_offline(&cache);
        }

        false
    }

    pub fn get_cached_info(&self) -> Option<LicenseInfo> {
        let cache = load_cache()?;
        if !crate::license::models::cached_license_allows_offline(&cache) {
            return None;
        }

        let device_id = crate::license::device::get_device_id();
        let device_label = crate::license::device::get_device_label();

        Some(LicenseInfo {
            license_key: cache.license_key.clone(),
            display_key: mask_key(&cache.license_key),
            status: LicenseStatus::from_polar_status(&cache.status),
            activation_id: Some(cache.activation_id),
            customer_email: cache.customer_email,
            customer_name: cache.customer_name,
            benefit_id: Some(cache.benefit_id),
            expires_at: cache.expires_at,
            limit_activations: None,
            usage: cache.usage,
            limit_usage: None,
            validations: cache.validations,
            last_validated_at: Some(cache.last_validated_at),
            device_id,
            device_label,
        })
    }

    fn check_license_status(&self, key: &crate::license::models::PolarLicenseKey) -> LicenseStatus {
        match key.status.as_str() {
            "revoked" => return LicenseStatus::Revoked,
            "disabled" => return LicenseStatus::Disabled,
            _ => {}
        }

        if let Some(ref expires_at) = key.expires_at {
            if let Ok(expiry) = chrono::DateTime::parse_from_rfc3339(expires_at) {
                if expiry < chrono::Utc::now() {
                    return LicenseStatus::Expired;
                }
            }
        }

        LicenseStatus::Granted
    }

    fn check_license_status_from_validate(&self, data: &ValidateResponse) -> LicenseStatus {
        match data.status.as_str() {
            "revoked" => return LicenseStatus::Revoked,
            "disabled" => return LicenseStatus::Disabled,
            _ => {}
        }

        if let Some(ref expires_at) = data.expires_at {
            if let Ok(expiry) = chrono::DateTime::parse_from_rfc3339(expires_at) {
                if expiry < chrono::Utc::now() {
                    return LicenseStatus::Expired;
                }
            }
        }

        LicenseStatus::Granted
    }

    pub fn is_authoritative_validate_error(&self, error: &str) -> bool {
        is_authoritative_validate_error(error)
    }

    pub fn classify_validate_error(&self, error: &str) -> LicenseValidationOutcome {
        if is_authoritative_validate_error(error) {
            LicenseValidationOutcome::Rejected
        } else {
            LicenseValidationOutcome::OfflineFallback
        }
    }
}

impl Default for LicenseManager {
    fn default() -> Self {
        Self::new()
    }
}

fn mask_key(key: &str) -> String {
    security::mask_license_key(key)
}

fn is_authoritative_validate_error(error: &str) -> bool {
    if error.contains("rejected by the license server") {
        return true;
    }

    error.contains("HTTP 400")
        || error.contains("HTTP 401")
        || error.contains("HTTP 403")
        || error.contains("HTTP 404")
        || error.contains("HTTP 422")
}

#[cfg(test)]
fn classify_validate_error(error: &str) -> crate::license::models::LicenseValidationOutcome {
    if is_authoritative_validate_error(error) {
        crate::license::models::LicenseValidationOutcome::Rejected
    } else {
        crate::license::models::LicenseValidationOutcome::OfflineFallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_validate_error() {
        assert!(matches!(
            classify_validate_error(
                "License validation was rejected by the license server. HTTP 403"
            ),
            LicenseValidationOutcome::Rejected
        ));
        assert!(matches!(
            classify_validate_error(
                "License validation was rejected by the license server. HTTP 401"
            ),
            LicenseValidationOutcome::Rejected
        ));
        assert!(matches!(
            classify_validate_error(
                "License validation request failed (non-authoritative). HTTP 500"
            ),
            LicenseValidationOutcome::OfflineFallback
        ));
        assert!(matches!(
            classify_validate_error(
                "License validation request failed (non-authoritative). HTTP 503"
            ),
            LicenseValidationOutcome::OfflineFallback
        ));
        assert!(matches!(
            classify_validate_error("reqwest error: connection reset"),
            LicenseValidationOutcome::OfflineFallback
        ));
    }

    #[test]
    fn test_device_id_is_stable() {
        let id1 = crate::license::device::get_device_id();
        let id2 = crate::license::device::get_device_id();
        assert_eq!(id1, id2);
        assert!(id1.starts_with("WVT-"));
    }

    #[test]
    fn test_device_label() {
        let label = crate::license::device::get_device_label();
        assert!(!label.is_empty());
        assert!(label.contains('('));
    }

    #[test]
    fn test_mask_key() {
        assert_eq!(mask_key("ABC"), "****");
        assert_eq!(mask_key("ABC-DEF-GHI-JKL"), "ABC-****-JKL");
        assert_eq!(mask_key("1234-5678-9012"), "1234****9012");
    }

    #[test]
    fn test_license_status_allows_usage() {
        assert!(LicenseStatus::Granted.allows_usage());
        assert!(LicenseStatus::Offline.allows_usage());
        assert!(!LicenseStatus::Revoked.allows_usage());
        assert!(!LicenseStatus::Expired.allows_usage());
    }

    #[test]
    fn test_cached_license_rejects_future_validation_time() {
        let cache = crate::license::models::CachedLicense {
            license_key: "test-license".to_string(),
            license_key_id: "test-license-key-id".to_string(),
            activation_id: "test-activation".to_string(),
            device_id: crate::license::device::get_device_id(),
            device_label: crate::license::device::get_device_label(),
            customer_email: None,
            customer_name: None,
            benefit_id: "test-benefit".to_string(),
            expires_at: None,
            last_validated_at: (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339(),
            status: "granted".to_string(),
            usage: 0,
            validations: 0,
            integrity_hash: String::new(),
            cache_version: CACHE_VERSION,
        };

        assert!(!crate::license::models::cached_license_allows_offline(
            &cache
        ));
    }

    fn fresh_granted_cache() -> crate::license::models::CachedLicense {
        crate::license::models::CachedLicense {
            license_key: "TEST-KEY-1234-ABCD".to_string(),
            license_key_id: "test-license-key-id".to_string(),
            activation_id: "act_123".to_string(),
            device_id: crate::license::device::get_device_id(),
            device_label: crate::license::device::get_device_label(),
            customer_email: None,
            customer_name: None,
            benefit_id: "benefit_xyz".to_string(),
            expires_at: None,
            last_validated_at: chrono::Utc::now().to_rfc3339(),
            status: "granted".to_string(),
            usage: 0,
            validations: 1,
            integrity_hash: String::new(),
            cache_version: CACHE_VERSION,
        }
    }

    #[test]
    fn throttle_is_inert_without_prior_attempt() {
        let manager = LicenseManager::with_org_id("test", "https://invalid.test");
        assert!(!manager.should_throttle_online_validation());
    }

    #[test]
    fn throttle_window_blocks_subsequent_attempts() {
        let manager = LicenseManager::with_org_id("test", "https://invalid.test");
        manager.mark_online_attempt();
        assert!(
            manager.should_throttle_online_validation(),
            "should throttle within the window after a fresh attempt"
        );
    }

    #[test]
    fn license_info_from_cache_reports_granted() {
        let manager = LicenseManager::with_org_id("test", "https://invalid.test");
        let cache = fresh_granted_cache();

        let info = manager.license_info_from_cache(&cache);

        assert_eq!(info.license_key, cache.license_key);
        assert_eq!(
            info.activation_id.as_deref(),
            Some(cache.activation_id.as_str())
        );
        assert!(info.status.allows_usage());
        assert!(matches!(info.status, LicenseStatus::Granted));
    }
}
