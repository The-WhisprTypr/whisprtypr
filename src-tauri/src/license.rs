//! Production-grade License Management for WhisprTypr
//!
//! Implements Polar.sh License Key API integration with:
//! - Device activation with unique device fingerprinting
//! - License validation with activation_id verification  
//! - Secure local caching with offline grace period
//! - Proper error handling for all API responses
//!
//! API Reference: https://polar.sh/docs/api-reference/customer-portal/license-keys/

use log::{debug, error, info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use crate::security;

// =============================================================================
// Configuration Constants
// =============================================================================

/// Polar.sh Customer Portal API endpoint (no auth required for client apps)
const POLAR_API_BASE: &str = "https://api.polar.sh/v1/customer-portal/license-keys";

/// Your Polar.sh Organization UUID - get from polar.sh dashboard settings
const POLAR_ORG_ID: &str = "51b31898-f34d-4f72-a175-26c8f6c8d828";

/// Offline grace period in hours - license works offline for this duration
const OFFLINE_GRACE_HOURS: i64 = 168; // 7 days

/// Minimum interval between online license re-validations.
///
/// Hotkey presses trigger `start_recording` → `ensure_app_access_verified`,
/// so a naive implementation would call Polar on every push-to-talk. We
/// trust a fresh-enough local cache for up to this many hours between
/// online checks, which keeps the dictation flow responsive and offline-
/// friendly while still catching revocations within a day.
const ONLINE_VALIDATION_MIN_INTERVAL_HOURS: i64 = 24;

/// HTTP request timeout
const REQUEST_TIMEOUT_SECS: u64 = 30;

// =============================================================================
// Public Types
// =============================================================================

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

// =============================================================================
// Polar API Request/Response Types
// =============================================================================

/// Request body for /activate endpoint
#[derive(Debug, Serialize)]
struct ActivateRequest {
    key: String,
    organization_id: String,
    label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    conditions: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    meta: Option<serde_json::Value>,
}

/// Request body for /validate endpoint
#[derive(Debug, Serialize)]
struct ValidateRequest {
    key: String,
    organization_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    activation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    benefit_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    increment_usage: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    conditions: Option<serde_json::Value>,
}

/// Request body for /deactivate endpoint
#[derive(Debug, Serialize)]
struct DeactivateRequest {
    key: String,
    organization_id: String,
    activation_id: String,
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

// =============================================================================
// Local Cache Types
// =============================================================================

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

const CACHE_VERSION: i32 = 3;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

// =============================================================================
// Device Identification
// =============================================================================

/// Generate a unique, stable device fingerprint
/// Uses hardware identifiers to create a reproducible ID
pub fn get_device_id() -> String {
    static DEVICE_ID: OnceLock<String> = OnceLock::new();

    DEVICE_ID.get_or_init(compute_device_id).clone()
}

fn compute_device_id() -> String {
    let mut hasher = Sha256::new();

    // Hostname
    if let Ok(hostname) = hostname::get() {
        hasher.update(hostname.to_string_lossy().as_bytes());
    }

    // OS and architecture
    hasher.update(std::env::consts::OS.as_bytes());
    hasher.update(std::env::consts::ARCH.as_bytes());

    // Username for multi-user systems
    if let Ok(user) = std::env::var("USER").or_else(|_| std::env::var("USERNAME")) {
        hasher.update(user.as_bytes());
    }

// Platform-specific hardware identifiers
    #[cfg(target_os = "macos")]
    {
        // Get macOS IOPlatformUUID
        if let Ok(output) = command_output_hidden("ioreg", &["-rd1", "-c", "IOPlatformExpertDevice"]) {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = output_str.lines().find(|l| l.contains("IOPlatformUUID")) {
                hasher.update(line.as_bytes());
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Get Windows machine UUID
        if let Ok(output) = command_output_hidden("wmic", &["csproduct", "get", "UUID"]) {
            hasher.update(&output.stdout);
        }
    }

    #[cfg(target_os = "linux")]
    {
        // D-Bus machine ID is the canonical stable device identifier on
        // Linux. It is unique per machine and survives reboots. Without it,
        // the fingerprint would fall back to hostname + username only, which
        // is not unique and defeats device binding / cache decryption.
        let mut machine_id = None;
        for path in &["/etc/machine-id", "/var/lib/dbus/machine-id"] {
            if let Ok(content) = std::fs::read_to_string(path) {
                let id = content.trim();
                if !id.is_empty() {
                    machine_id = Some(id.as_bytes().to_vec());
                    break;
                }
            }
        }
        if let Some(id) = machine_id {
            hasher.update(&id);
        } else {
            // Last-resort fallback: mix in a fixed salt so the two sources
            // (hostname+username vs. machine-id) never collide.
            hasher.update(b"whisprtypr-linux-device-id-fallback");
        }
    }

    // Create readable device ID with prefix
    let hash = hasher.finalize();
    format!("WVT-{}", hex::encode(&hash[..12]).to_uppercase())
}

#[cfg(target_os = "windows")]
fn command_output_hidden(program: &str, args: &[&str]) -> std::io::Result<std::process::Output> {
    let mut command = std::process::Command::new(program);
    command.args(args).creation_flags(CREATE_NO_WINDOW);
    command.output()
}

#[cfg(not(target_os = "windows"))]
fn command_output_hidden(program: &str, args: &[&str]) -> std::io::Result<std::process::Output> {
    let mut command = std::process::Command::new(program);
    command.args(args).output()
}

/// Get a human-readable device label
pub fn get_device_label() -> String {
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    let os = match std::env::consts::OS {
        "macos" => "macOS",
        "windows" => "Windows",
        other => other,
    };

    format!("{} ({})", hostname, os)
}

/// Get device metadata for Polar activation
fn get_device_meta() -> serde_json::Value {
    serde_json::json!({
        "device_id": get_device_id(),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "hostname": hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_default(),
        "app_version": env!("CARGO_PKG_VERSION"),
        "activated_at": chrono::Utc::now().to_rfc3339(),
    })
}

fn get_device_conditions(device_id: &str) -> serde_json::Value {
    serde_json::json!({
        "device_id": device_id,
    })
}

// =============================================================================
// Secure Storage
// =============================================================================

fn get_cache_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("com.johuniq.whisprtypr"))
}

fn get_cache_path() -> Option<PathBuf> {
    get_cache_dir().map(|d| d.join(".license.dat"))
}

/// Calculate integrity hash for cache tampering detection
fn calculate_integrity_hash(cache: &CachedLicense) -> String {
    let mut hasher = Sha256::new();
    hasher.update(cache.license_key.as_bytes());
    hasher.update(cache.license_key_id.as_bytes());
    hasher.update(cache.activation_id.as_bytes());
    hasher.update(cache.device_id.as_bytes());
    hasher.update(cache.benefit_id.as_bytes());
    hasher.update(b"whisprtypr-integrity-v2");
    hex::encode(hasher.finalize())
}

/// Encrypt data using device-bound key with AES-256-GCM
fn encrypt_data(data: &[u8]) -> Result<Vec<u8>, String> {
    let device_id = get_device_id();
    let key = security::derive_encryption_key(&device_id);
    security::encrypt_data(data, &key)
}

/// Decrypt data using device-bound key with AES-256-GCM
fn decrypt_data(data: &[u8]) -> Result<Vec<u8>, String> {
    let device_id = get_device_id();
    let key = security::derive_encryption_key(&device_id);
    security::decrypt_data(data, &key)
}

/// Store license cache securely
pub fn store_cache(cache: &CachedLicense) -> Result<(), String> {
    let cache_dir = get_cache_dir().ok_or("Failed to get cache directory")?;

    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create cache directory: {}", e))?;

    let cache_path = get_cache_path().ok_or("Failed to get cache path")?;

    // Add integrity hash
    let mut cache_with_hash = cache.clone();
    cache_with_hash.integrity_hash = calculate_integrity_hash(cache);
    cache_with_hash.cache_version = CACHE_VERSION;

    let json = serde_json::to_string(&cache_with_hash)
        .map_err(|e| format!("Failed to serialize cache: {}", e))?;

    let encrypted =
        encrypt_data(json.as_bytes()).map_err(|e| format!("Failed to encrypt cache: {}", e))?;

    std::fs::write(&cache_path, encrypted).map_err(|e| format!("Failed to write cache: {}", e))?;

    debug!("License cache stored successfully");
    Ok(())
}

/// Load license cache from disk
pub fn load_cache() -> Option<CachedLicense> {
    let cache_path = get_cache_path()?;

    let encrypted = std::fs::read(&cache_path).ok()?;
    let decrypted = decrypt_data(&encrypted).ok()?;
    let json = String::from_utf8(decrypted).ok()?;
    let cache: CachedLicense = serde_json::from_str(&json).ok()?;

    // Verify integrity
    let expected_hash = calculate_integrity_hash(&cache);
    if cache.integrity_hash != expected_hash {
        warn!("License cache integrity check failed - possible tampering");
        return None;
    }

    // Verify device binding
    if cache.device_id != get_device_id() {
        warn!("License cache device mismatch");
        return None;
    }

    // Check cache version
    if cache.cache_version != CACHE_VERSION {
        warn!("License cache version mismatch");
        return None;
    }

    debug!("License cache loaded successfully");
    Some(cache)
}

fn cached_license_allows_offline(cache: &CachedLicense) -> bool {
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

/// Clear license cache
pub fn clear_cache() -> Result<(), String> {
    if let Some(path) = get_cache_path() {
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| format!("Failed to delete cache: {}", e))?;
        }
    }
    info!("License cache cleared");
    Ok(())
}

// =============================================================================
// License Manager
// =============================================================================

/// Main license management interface
pub struct LicenseManager {
    client: Client,
    pub org_id: String,
    pub api_base: String,
    /// Timestamp of the most recent online validation attempt (success or
    /// failure). Used to throttle online calls; hotkey-driven paths must
    /// not hit Polar on every press. Wrapped in a Mutex because the
    /// manager is shared across Tauri command handlers running on the
    /// async runtime.
    last_online_attempt: Mutex<Option<chrono::DateTime<chrono::Utc>>>,
}

impl LicenseManager {
    /// Create new license manager
    pub fn new() -> Self {
        Self::with_org_id(POLAR_ORG_ID, POLAR_API_BASE)
    }

    /// Create license manager with custom org ID
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

    /// Returns true when there is a cached license that still satisfies
    /// the offline grace period and is not past its server-side expiry.
    /// This intentionally never touches the network and is cheap enough
    /// to call on every hotkey press.
    pub fn is_cached_license_valid(&self) -> bool {
        let Some(cache) = load_cache() else {
            return false;
        };
        cached_license_allows_offline(&cache)
    }

    /// Records that we just attempted an online validation. Failures to
    /// acquire the lock are ignored: throttling is best-effort and a
    /// missed timestamp only means one extra request, never a denied
    /// user.
    fn mark_online_attempt(&self) {
        if let Ok(mut guard) = self.last_online_attempt.lock() {
            *guard = Some(chrono::Utc::now());
        }
    }

    /// Returns true when the last online validation was recent enough
    /// that we should skip another network call. Always returns false
    /// when no previous attempt has been recorded.
    fn should_throttle_online_validation(&self) -> bool {
        let Ok(guard) = self.last_online_attempt.lock() else {
            return false;
        };
        let Some(last) = *guard else {
            return false;
        };
        let elapsed = (chrono::Utc::now() - last).num_hours();
        elapsed >= 0 && elapsed < ONLINE_VALIDATION_MIN_INTERVAL_HOURS
    }

    /// Validate without touching the network when the local cache is
    /// fresh. On success returns the cached `LicenseInfo` (status
    /// `Granted`). On failure falls through to a throttled online
    /// validation, then to the offline grace period as a last resort.
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

    /// Build a `LicenseInfo` snapshot from a cached license record.
    fn license_info_from_cache(&self, cache: &CachedLicense) -> LicenseInfo {
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
            device_id: get_device_id(),
            device_label: get_device_label(),
        }
    }

    /// Online validation, but only when the throttle window has elapsed
    /// or the cache is missing/expired. Network failures still fall
    /// through to the offline grace period so dictation keeps working
    /// when the user's internet is down.
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

    /// Activate a license key on this device
    ///
    /// This creates an activation instance in Polar and stores the activation_id
    /// locally for future validations.
    pub async fn activate(&self, license_key: &str) -> Result<LicenseInfo, String> {
        let device_id = get_device_id();
        let device_label = get_device_label();

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

            // Check expiration
            let license_status = self.check_license_status(&data.license_key);

            // Store in local cache (initial activation record)
            let mut cache = CachedLicense {
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

            // Persist initial cache
            store_cache(&cache)?;

            // After activation, verify the activation can be validated and increment usage
            match self
                .perform_validate(
                    &cache.license_key,
                    &cache.activation_id,
                    Some(cache.benefit_id.clone()),
                    Some(1),
                )
                .await
            {
                Ok(validate_resp) => {
                    // Update cache from validation response
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
                    // Fallthrough to return activation-derived info
                }
            }

            Ok(LicenseInfo {
                license_key: license_key.to_string(),
                display_key: data.license_key.display_key,
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
            // Activation limit reached
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

    /// Classify a validate() failure into an authoritative rejection or a
    /// transient failure. Used by the Tauri command layer to decide whether
    /// access may still be granted from a stale DB/cache row.
    pub fn is_authoritative_validate_error(&self, error: &str) -> bool {
        is_authoritative_validate_error(error)
    }

    /// Classify a validate() failure into an authoritative rejection or a
    /// transient failure. Used by the Tauri command layer to decide whether
    /// access may still be granted from a stale DB/cache row.
    pub fn classify_validate_error(&self, error: &str) -> LicenseValidationOutcome {
        if is_authoritative_validate_error(error) {
            LicenseValidationOutcome::Rejected
        } else {
            LicenseValidationOutcome::OfflineFallback
        }
    }

    /// Validate the current license
    ///
    /// First tries online validation with Polar API, falls back to cached
    /// license within the offline grace period.
    pub async fn validate(&self) -> Result<LicenseInfo, String> {
        let device_id = get_device_id();
        let device_label = get_device_label();

        // Load cached license
        let cache = load_cache();

        if let Some(ref cached) = cache {
            info!("Validating license with Polar API...");

            match self
                .perform_validate(
                    &cached.license_key,
                    &cached.activation_id,
                    Some(cached.benefit_id.clone()),
                    Some(1), // Increment usage by 1 on each validation
                )
                .await
            {
                Ok(data) => {
                    let license_status = self.check_license_status_from_validate(&data);

                    // Check activation count against limit
                    if let Some(limit) = data.limit_activations {
                        if limit > 0 {
                            // Fetch full license key to get all activations
                            if let Ok(license_key) = self.get_license_key(&cached.license_key_id).await {
                                let activation_count = license_key.activations.len() as i32;
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

                    // Update cache
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
                Err(error) => {
                    match self.classify_validate_error(&error) {
                        LicenseValidationOutcome::Granted => {
                            // Should not happen here (Ok above), but stay safe.
                        }
                        LicenseValidationOutcome::Rejected => {
                            warn!("License rejected by Polar - clearing cache: {}", error);
                            let _ = clear_cache();
                            return Err(
                                "License validation was rejected. Please activate again.".to_string()
                            );
                        }
                        LicenseValidationOutcome::OfflineFallback => {
                            warn!("Validation failed (non-authoritative): {}", error);
                            // Fall through to offline validation
                        }
                    }
                }
            }

            // Offline validation - check grace period
            return self.validate_offline(cached, &device_id, &device_label);
        }

        Err("No license activated. Please enter your license key.".to_string())
    }

    /// Validate a license using credentials restored from the app database.
    ///
    /// This is used when the encrypted cache is missing but the database still
    /// contains a recent activated license record.
    pub async fn validate_activation(
        &self,
        license_key: &str,
        activation_id: &str,
    ) -> Result<LicenseInfo, String> {
        let device_id = get_device_id();
        let device_label = get_device_label();

        info!("Validating license from stored activation...");

        let data = self
            .perform_validate(license_key, activation_id, None, Some(1))
            .await?;

        let license_status = self.check_license_status_from_validate(&data);

        // Check activation count against limit
        if let Some(limit) = data.limit_activations {
            if limit > 0 {
                // Fetch full license key to get all activations
                if let Ok(license_key_info) = self.get_license_key(&data.id).await {
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

        let cache = CachedLicense {
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

    /// Validate license offline using cache
    fn validate_offline(
        &self,
        cache: &CachedLicense,
        device_id: &str,
        device_label: &str,
    ) -> Result<LicenseInfo, String> {
        if cached_license_allows_offline(cache) {
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
        Err("License validation failed and offline grace period expired. Please connect to the internet.".to_string())
    }

    /// Deactivate license from this device
    pub async fn deactivate(&self) -> Result<(), String> {
        let cache = load_cache().ok_or("No license to deactivate")?;

        self.deactivate_activation(&cache.license_key, &cache.activation_id)
            .await
    }

    /// Deactivate a license using explicit credentials.
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

        // 204 No Content = success
        if status.is_success() || status.as_u16() == 204 {
            info!("License deactivated successfully");
            clear_cache()?;
            Ok(())
        } else if status.as_u16() == 404 {
            // Already deactivated or not found - clear local anyway
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

    /// Check if license is currently valid (quick local check)
    pub fn is_valid(&self) -> bool {
        if let Some(cache) = load_cache() {
            return cached_license_allows_offline(&cache);
        }

        false
    }

    /// Get cached license info without validation
    pub fn get_cached_info(&self) -> Option<LicenseInfo> {
        let cache = load_cache()?;
        if !cached_license_allows_offline(&cache) {
            return None;
        }

        let device_id = get_device_id();
        let device_label = get_device_label();

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

    /// Determine license status from license key data
    fn check_license_status(&self, key: &PolarLicenseKey) -> LicenseStatus {
        // Check Polar status
        match key.status.as_str() {
            "revoked" => return LicenseStatus::Revoked,
            "disabled" => return LicenseStatus::Disabled,
            _ => {}
        }

        // Check expiration
        if let Some(ref expires_at) = key.expires_at {
            if let Ok(expiry) = chrono::DateTime::parse_from_rfc3339(expires_at) {
                if expiry < chrono::Utc::now() {
                    return LicenseStatus::Expired;
                }
            }
        }

        LicenseStatus::Granted
    }

    /// Determine license status from validate response
    fn check_license_status_from_validate(&self, data: &ValidateResponse) -> LicenseStatus {
        // Check Polar status
        match data.status.as_str() {
            "revoked" => return LicenseStatus::Revoked,
            "disabled" => return LicenseStatus::Disabled,
            _ => {}
        }

        // Check expiration
        if let Some(ref expires_at) = data.expires_at {
            if let Ok(expiry) = chrono::DateTime::parse_from_rfc3339(expires_at) {
                if expiry < chrono::Utc::now() {
                    return LicenseStatus::Expired;
                }
            }
        }

        LicenseStatus::Granted
    }

    /// Helper: perform a validate call for a given license key + activation id
    async fn perform_validate(
        &self,
        license_key: &str,
        activation_id: &str,
        benefit_id: Option<String>,
        increment_usage: Option<i32>,
    ) -> Result<ValidateResponse, String> {
        let request = ValidateRequest {
            key: license_key.to_string(),
            organization_id: self.org_id.clone(),
            activation_id: Some(activation_id.to_string()),
            benefit_id,
            increment_usage,
            conditions: Some(get_device_conditions(&get_device_id())),
        };

        let body_json = serde_json::to_string(&request).unwrap_or_default();
        debug!("validate request body: {}", body_json);

        let url = format!("{}/validate", self.api_base);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Network error during validate: {}", e))?;

        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        if status.is_success() {
            let data: ValidateResponse = serde_json::from_str(&body).map_err(|e| {
                format!("Failed to parse validate response: {} - Body: {}", e, body)
            })?;
            debug!(
                "validate resp: usage={} limit_usage={:?} validations={} status={}",
                data.usage, data.limit_usage, data.validations, data.status
            );
            return Ok(data);
        }

        warn!("License validation rejected by server: {}", status);
        debug!("License validation response body: {}", body);

        // Distinguish authoritative rejections (4xx) from transient
        // failures (5xx, network). Authoritative rejections clear the
        // cache; transient failures fall through to the offline grace
        // period so a server outage does not lock users out.
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

    /// Fetch full license key info including all activations
    /// Uses GET /v1/customer-portal/license-keys/{license_key_id}
    async fn get_license_key(&self, license_key_id: &str) -> Result<LicenseKeyWithActivations, String> {
        let url = format!("{}/{}", self.api_base, license_key_id);
        debug!("GET {}", url);

        let response = self
            .client
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
}

impl Default for LicenseManager {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Mask license key for display. Delegates to the canonical
/// `security::mask_license_key` so every code path produces the same
/// masked form (first 4 + last 4 characters).
fn mask_key(key: &str) -> String {
    security::mask_license_key(key)
}

fn is_authoritative_validate_error(error: &str) -> bool {
    if error.contains("rejected by the license server") {
        return true;
    }

    // Only 4xx responses are authoritative. 5xx (and any other network
    // failure) must fall through to the offline grace period so a transient
    // server outage does not wipe every user's license cache.
    error.contains("HTTP 400")
        || error.contains("HTTP 401")
        || error.contains("HTTP 403")
        || error.contains("HTTP 404")
        || error.contains("HTTP 422")
}

// =============================================================================
// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
fn classify_validate_error(error: &str) -> LicenseValidationOutcome {
    if is_authoritative_validate_error(error) {
        LicenseValidationOutcome::Rejected
    } else {
        LicenseValidationOutcome::OfflineFallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_validate_error() {
        // 4xx responses are authoritative rejections.
        assert!(matches!(
            classify_validate_error("License validation was rejected by the license server. HTTP 403"),
            LicenseValidationOutcome::Rejected
        ));
        assert!(matches!(
            classify_validate_error("License validation was rejected by the license server. HTTP 401"),
            LicenseValidationOutcome::Rejected
        ));
        // 5xx responses are non-authoritative and fall through to offline grace.
        assert!(matches!(
            classify_validate_error("License validation request failed (non-authoritative). HTTP 500"),
            LicenseValidationOutcome::OfflineFallback
        ));
        assert!(matches!(
            classify_validate_error("License validation request failed (non-authoritative). HTTP 503"),
            LicenseValidationOutcome::OfflineFallback
        ));
        // Network errors are non-authoritative.
        assert!(matches!(
            classify_validate_error("reqwest error: connection reset"),
            LicenseValidationOutcome::OfflineFallback
        ));
    }

    #[test]
    fn test_device_id_is_stable() {
        let id1 = get_device_id();
        let id2 = get_device_id();
        assert_eq!(id1, id2);
        assert!(id1.starts_with("WVT-"));
    }

    #[test]
    fn test_device_label() {
        let label = get_device_label();
        assert!(!label.is_empty());
        assert!(label.contains('('));
    }

    #[test]
    fn test_mask_key() {
        // Unified with security::mask_license_key: first 4 + last 4 chars.
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
        let cache = CachedLicense {
            license_key: "test-license".to_string(),
            license_key_id: "test-license-key-id".to_string(),
            activation_id: "test-activation".to_string(),
            device_id: get_device_id(),
            device_label: get_device_label(),
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

        assert!(!cached_license_allows_offline(&cache));
    }

    fn fresh_granted_cache() -> CachedLicense {
        CachedLicense {
            license_key: "TEST-KEY-1234-ABCD".to_string(),
            license_key_id: "test-license-key-id".to_string(),
            activation_id: "act_123".to_string(),
            device_id: get_device_id(),
            device_label: get_device_label(),
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
        // No prior attempt recorded -> always proceed (i.e. don't throttle).
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
        assert_eq!(info.activation_id.as_deref(), Some(cache.activation_id.as_str()));
        assert!(info.status.allows_usage());
        assert!(matches!(info.status, LicenseStatus::Granted));
    }
}
