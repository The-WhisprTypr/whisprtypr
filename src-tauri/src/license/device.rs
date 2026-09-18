use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::OnceLock;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use crate::license::models::CachedLicense;
#[cfg(target_os = "windows")]
use crate::license::models::CREATE_NO_WINDOW;
use crate::security;
use hex;
use getrandom;

fn get_or_create_installation_id() -> String {
    let cache_dir = get_cache_dir();
    if let Some(dir) = cache_dir {
        let installation_file = dir.join(".installation_id");
        if let Ok(content) = std::fs::read_to_string(&installation_file) {
            let id = content.trim();
            if !id.is_empty() && id.len() >= 32 {
                return id.to_string();
            }
        }
        // Generate new installation ID
        let new_id = generate_random_id();
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(&installation_file, &new_id);
        return new_id;
    }
    // Fallback: generate ephemeral ID (won't persist across runs)
    generate_random_id()
}

fn generate_random_id() -> String {
    use getrandom::getrandom;
    let mut bytes = [0u8; 32];
    if getrandom(&mut bytes).is_ok() {
        hex::encode(bytes)
    } else {
        // Fallback using timestamp + PID if getrandom fails
        use std::time::{SystemTime, UNIX_EPOCH};
        let mut hasher = Sha256::new();
        hasher.update(SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos().to_le_bytes());
        hasher.update(std::process::id().to_le_bytes());
        hex::encode(hasher.finalize())
    }
}

pub fn get_device_id() -> String {
    static DEVICE_ID: OnceLock<String> = OnceLock::new();

    DEVICE_ID.get_or_init(compute_device_id).clone()
}

fn compute_device_id() -> String {
    let mut hasher = Sha256::new();

    // Stable per-installation random component (generated once, stored in app data)
    let installation_id = get_or_create_installation_id();
    hasher.update(installation_id.as_bytes());

    // Platform-specific hardware identifiers
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = command_output_hidden("ioreg", &["-rd1", "-c", "IOPlatformExpertDevice"]) {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = output_str.lines().find(|l| l.contains("IOPlatformUUID")) {
                hasher.update(line.as_bytes());
            }
        }
        // Also add serial number for additional entropy
        if let Ok(output) = command_output_hidden("ioreg", &["-rd1", "-c", "IOPlatformExpertDevice"]) {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = output_str.lines().find(|l| l.contains("IOPlatformSerialNumber")) {
                hasher.update(line.as_bytes());
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // System UUID (motherboard)
        if let Ok(output) = command_output_hidden("wmic", &["csproduct", "get", "UUID"]) {
            hasher.update(&output.stdout);
        }
        // BIOS serial
        if let Ok(output) = command_output_hidden("wmic", &["bios", "get", "SerialNumber"]) {
            hasher.update(&output.stdout);
        }
        // Disk serial (first physical drive)
        if let Ok(output) = command_output_hidden("wmic", &["diskdrive", "get", "SerialNumber"]) {
            hasher.update(&output.stdout);
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Primary: /etc/machine-id (systemd) or /var/lib/dbus/machine-id
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
        if let Some(ref id) = machine_id {
            hasher.update(id);
        }

        // Secondary: CPU info for additional entropy
        if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
            for line in content.lines() {
                if line.starts_with("Serial") || line.starts_with("Hardware") {
                    hasher.update(line.as_bytes());
                }
            }
        }

        // Tertiary: First disk UUID
        if let Ok(entries) = std::fs::read_dir("/dev/disk/by-uuid") {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    hasher.update(name.as_bytes());
                    break;
                }
            }
        }

        // Fallback: if no machine-id found, use a hash of available identifiers
        if machine_id.is_none() {
            let mut fallback_hasher = Sha256::new();
            fallback_hasher.update(std::env::consts::OS.as_bytes());
            fallback_hasher.update(std::env::consts::ARCH.as_bytes());
            if let Ok(hostname) = hostname::get() {
                fallback_hasher.update(hostname.to_string_lossy().as_bytes());
            }
            if let Ok(user) = std::env::var("USER").or_else(|_| std::env::var("USERNAME")) {
                fallback_hasher.update(user.as_bytes());
            }
            hasher.update(&fallback_hasher.finalize());
        }
    }

    // Cross-platform: add OS/arch as last resort (low entropy but stable)
    hasher.update(std::env::consts::OS.as_bytes());
    hasher.update(std::env::consts::ARCH.as_bytes());

    let hash = hasher.finalize();
    format!("WVT-{}", hex::encode(&hash[..16]).to_uppercase())
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

pub fn get_device_meta() -> serde_json::Value {
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

pub fn get_device_conditions(device_id: &str) -> serde_json::Value {
    serde_json::json!({
        "device_id": device_id,
    })
}

pub(crate) fn get_cache_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("com.johuniq.whisprtypr"))
}

pub(crate) fn get_cache_path() -> Option<PathBuf> {
    get_cache_dir().map(|d| d.join(".license.dat"))
}

pub(crate) fn calculate_integrity_hash(cache: &CachedLicense) -> String {
    let mut hasher = Sha256::new();
    hasher.update(cache.license_key.as_bytes());
    hasher.update(cache.license_key_id.as_bytes());
    hasher.update(cache.activation_id.as_bytes());
    hasher.update(cache.device_id.as_bytes());
    hasher.update(cache.benefit_id.as_bytes());
    hasher.update(b"whisprtypr-integrity-v2");
    hex::encode(hasher.finalize())
}

pub(crate) fn encrypt_data(data: &[u8]) -> Result<Vec<u8>, String> {
    let device_id = get_device_id();
    security::encrypt_data(data, &device_id)
}

pub(crate) fn decrypt_data(data: &[u8]) -> Result<Vec<u8>, String> {
    let device_id = get_device_id();
    security::decrypt_data(data, &device_id)
}
