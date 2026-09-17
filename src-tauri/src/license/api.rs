use log::{debug, error, warn};
use reqwest::Client;

use crate::license::models::{LicenseKeyWithActivations, ValidateRequest, ValidateResponse};

pub async fn perform_validate(
    client: &Client,
    org_id: &str,
    api_base: &str,
    license_key: &str,
    activation_id: &str,
    benefit_id: Option<String>,
    increment_usage: Option<i32>,
) -> Result<ValidateResponse, String> {
    let request = ValidateRequest {
        key: license_key.to_string(),
        organization_id: org_id.to_string(),
        activation_id: Some(activation_id.to_string()),
        benefit_id,
        increment_usage,
        conditions: Some(crate::license::device::get_device_conditions(
            &crate::license::device::get_device_id(),
        )),
    };

    let body_json = serde_json::to_string(&request).unwrap_or_default();
    debug!("validate request body: {}", body_json);

    let url = format!("{}/validate", api_base);

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Network error during validate: {}", e))?;

    let status = response.status();
    let body = response.text().await.unwrap_or_default();

    if status.is_success() {
        let data: ValidateResponse = serde_json::from_str(&body)
            .map_err(|e| format!("Failed to parse validate response: {} - Body: {}", e, body))?;
        debug!(
            "validate resp: usage={} limit_usage={:?} validations={} status={}",
            data.usage, data.limit_usage, data.validations, data.status
        );
        return Ok(data);
    }

    warn!("License validation rejected by server: {}", status);
    debug!("License validation response body: {}", body);

    if status.is_client_error() {
        Err(format!(
            "License validation was rejected by the license server. HTTP {}",
            status.as_u16()
        ))
    } else {
        Err(format!(
            "License validation request failed (non-authoritative). HTTP {}",
            status.as_u16()
        ))
    }
}

pub async fn get_license_key(
    client: &Client,
    api_base: &str,
    license_key_id: &str,
) -> Result<LicenseKeyWithActivations, String> {
    let url = format!("{}/{}", api_base, license_key_id);
    debug!("GET {}", url);

    let response = client
        .get(&url)
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let status = response.status();
    let body = response.text().await.unwrap_or_default();

    debug!("Response status: {}", status);
    debug!("Response body: {}", body);

    if status.is_success() {
        let data: LicenseKeyWithActivations = serde_json::from_str(&body)
            .map_err(|e| format!("Failed to parse response: {} - Body: {}", e, body))?;
        Ok(data)
    } else if status.as_u16() == 404 {
        error!("License key not found");
        Err("License key not found".to_string())
    } else {
        error!("Failed to fetch license key: {} - {}", status, body);
        Err(format!("Failed to fetch license key: HTTP {}", status))
    }
}
