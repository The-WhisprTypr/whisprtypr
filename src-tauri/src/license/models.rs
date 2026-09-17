use log::warn;
use serde::{Deserialize, Serialize};

/// Polar.sh Customer Portal API endpoint (no auth required for client apps)
pub const POLAR_API_BASE: &str = "https://api.polar.sh/v1/customer-portal/license-keys";

/// Your Polar.sh Organization UUID - get from polar.sh dashboard settings
pub const POLAR_ORG_ID: &str = "51b31898-f34d-4f72-a175-26c8f6c8d828";

/// Offline grace period in hours - license works offline for this duration
pub const OFFLINE_GRACE_HOURS: i64 = 168; // 7 days

/// Minimum interval between online license re-validations.
///
/// Hotkey presses trigger `start_recording` → `ensure_app_access_verified`,
/// so a naive implementation would call Polar on every push-to-talk. We
/// trust a fresh-enough local cache for up to this many hours between
/// online checks, which keeps the dictation flow responsive and offline-
/// friendly while still catching revocations within a day.
pub const ONLINE_VALIDATION_MIN_INTERVAL_HOURS: i64 = 24;

/// HTTP request timeout
pub const REQUEST_TIMEOUT_SECS: u64 = 30;

/// Cache version for migrations
pub const CACHE_VERSION: i32 = 3;

#[cfg(target_os = "windows")]
pub const CREATE_NO_WINDOW: u32 = 0x08000000;

/// License information returned to the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub license_key: String,
    pub display_key: String,
    pub status: LicenseStatus,
    pub activation_id: Option<String>,
    pub customer_email: Option<String>,
    pub customer_name: Option<String>,
    pub benefit_id: Option<String>,
    pub expires_at: Option<String>,
    pub limit_activations: Option<i32>,
    pub usage: i32,
    pub limit_usage: Option<i32>,
    pub validations: i32,
    pub last_validated_at: Option<String>,
    pub device_id: String,
    pub device_label: String,
}

/// License status enum matching Polar API statuses
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LicenseStatus {
    /// License is valid and active
    Granted,
    /// License has been revoked
    Revoked,
    /// License has been disabled
    Disabled,
    /// License has expired
    Expired,
    /// License key is invalid/not found
    Invalid,
    /// Activation limit reached
    ActivationLimitReached,
    /// Network error - using cached license
    Offline,
    /// No license activated
    #[default]
    NotActivated,
}

impl std::fmt::Display for LicenseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LicenseStatus::Granted => write!(f, "granted"),
            LicenseStatus::Revoked => write!(f, "revoked"),
            LicenseStatus::Disabled => write!(f, "disabled"),
            LicenseStatus::Expired => write!(f, "expired"),
            LicenseStatus::Invalid => write!(f, "invalid"),
            LicenseStatus::ActivationLimitReached => write!(f, "activation_limit_reached"),
            LicenseStatus::Offline => write!(f, "offline"),
            LicenseStatus::NotActivated => write!(f, "not_activated"),
        }
    }
}

impl LicenseStatus {
    /// Check if the license allows app usage
    pub fn allows_usage(&self) -> bool {
        matches!(self, LicenseStatus::Granted | LicenseStatus::Offline)
    }

    /// Parse from Polar API status string
    pub fn from_polar_status(status: &str) -> Self {
        match status.to_lowercase().as_str() {
            "granted" => LicenseStatus::Granted,
            "revoked" => LicenseStatus::Revoked,
            "disabled" => LicenseStatus::Disabled,
            "expired" => LicenseStatus::Expired,
            _ => LicenseStatus::Invalid,
        }
    }
}

/// Outcome of an online license validation attempt.
///
/// This distinguishes authoritative rejections (revoked, disabled, invalid,
/// or activation limit reached) from transient network failures so callers
/// can decide whether to fall back to the local cache or the offline grace
/// period. Authoritative rejections must never grant access.
#[derive(Debug, Clone)]
pub enum LicenseValidationOutcome {
    /// Online validation succeeded and the license allows usage.
    Granted,
    /// The license server rejected the key/activation (revoked, disabled,
    /// invalid, or activation limit reached). Callers must NOT grant access
    /// and should not fall back to a stale local copy.
    Rejected,
    /// The request failed for a non-authoritative reason (network error,
    /// timeout, 5xx server error). Callers may fall back to the cached
    /// license or the offline grace period.
    OfflineFallback,
}

/// Request body for /activate endpoint
#[derive(Serialize)]
pub struct ActivateRequest {
    pub key: String,
    pub organization_id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

/// Request body for /validate endpoint
#[derive(Serialize)]
pub struct ValidateRequest {
    pub key: String,
    pub organization_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub benefit_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub increment_usage: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<serde_json::Value>,
}

/// Request body for /deactivate endpoint
#[derive(Serialize)]
pub struct DeactivateRequest {
    pub key: String,
    pub organization_id: String,
    pub activation_id: String,
}

/// Customer info from Polar API
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct PolarCustomer {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
}

/// Activation info from Polar API
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct PolarActivation {
    pub id: String,
    pub license_key_id: String,
    pub label: String,
    #[serde(default)]
    pub meta: Option<serde_json::Value>,
    pub created_at: String,
    pub modified_at: Option<String>,
}

/// License key info from Polar API (embedded in responses)
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct PolarLicenseKey {
    pub id: String,
    pub organization_id: String,
    pub customer_id: String,
    pub customer: Option<PolarCustomer>,
    pub benefit_id: String,
    pub key: String,
    pub display_key: String,
    pub status: String,
    pub limit_activations: Option<i32>,
    pub usage: i32,
    pub limit_usage: Option<i32>,
    pub validations: i32,
    pub last_validated_at: Option<String>,
    pub expires_at: Option<String>,
}

/// Full license key response with all activations (from GET /license-keys/{id})
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct LicenseKeyWithActivations {
    pub id: String,
    pub organization_id: String,
    pub customer_id: String,
    pub customer: Option<PolarCustomer>,
    pub benefit_id: String,
    pub key: String,
    pub display_key: String,
    pub status: String,
    pub limit_activations: Option<i32>,
    pub usage: i32,
    pub limit_usage: Option<i32>,
    pub validations: i32,
    pub last_validated_at: Option<String>,
    pub expires_at: Option<String>,
    pub activations: Vec<PolarActivation>,
}

/// Response from /activate endpoint
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ActivateResponse {
    pub id: String,
    pub license_key_id: String,
    pub label: String,
    #[serde(default)]
    pub meta: Option<serde_json::Value>,
    pub created_at: String,
    pub modified_at: Option<String>,
    pub license_key: PolarLicenseKey,
}

/// Response from /validate endpoint
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ValidateResponse {
    pub id: String,
    pub organization_id: String,
    pub customer_id: String,
    pub customer: Option<PolarCustomer>,
    pub benefit_id: String,
    pub key: String,
    pub display_key: String,
    pub status: String,
    pub limit_activations: Option<i32>,
    pub usage: i32,
    pub limit_usage: Option<i32>,
    pub validations: i32,
    pub last_validated_at: Option<String>,
    pub expires_at: Option<String>,
    pub activation: Option<PolarActivation>,
}

/// Error response from Polar API
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct PolarError {
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub detail: Option<String>,
    #[serde(rename = "type", default)]
    pub error_type: Option<String>,
}

/// Cached license data stored on disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedLicense {
    /// The original license key (stored securely)
    pub license_key: String,
    /// License Key ID from Polar (required to fetch full license key with activations)
    pub license_key_id: String,
    /// Activation ID from Polar (required for validation)
    pub activation_id: String,
    /// Device ID this license was activated on
    pub device_id: String,
    /// Device label for identification
    pub device_label: String,
    /// Customer email
    pub customer_email: Option<String>,
    /// Customer name
    pub customer_name: Option<String>,
    /// Benefit ID for validation
    pub benefit_id: String,
    /// Expiration timestamp
    pub expires_at: Option<String>,
    /// Last successful validation timestamp
    pub last_validated_at: String,
    /// License status at last validation
    pub status: String,
    /// Usage count
    pub usage: i32,
    /// Validation count
    pub validations: i32,
    /// Integrity hash to detect tampering
    pub integrity_hash: String,
    /// Cache version for migrations
    pub cache_version: i32,
}

pub(crate) fn cached_license_allows_offline(cache: &CachedLicense) -> bool {
    if cache.status != "granted" {
        return false;
    }

    if let Some(ref expires_at) = cache.expires_at {
        let Ok(expiry) = chrono::DateTime::parse_from_rfc3339(expires_at) else {
            warn!("License cache has invalid expiration timestamp");
            return false;
        };

        if expiry < chrono::Utc::now() {
            return false;
        }
    }

    let Ok(last_validated) = chrono::DateTime::parse_from_rfc3339(&cache.last_validated_at) else {
        warn!("License cache has invalid validation timestamp");
        return false;
    };

    let now = chrono::Utc::now();
    let last_validated = last_validated.with_timezone(&chrono::Utc);
    if last_validated > now {
        warn!("License cache validation timestamp is in the future");
        return false;
    }

    let hours_since = (now - last_validated).num_hours();
    hours_since < OFFLINE_GRACE_HOURS
}
