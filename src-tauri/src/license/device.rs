use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use crate::license::models::{CachedLicense, CACHE_VERSION};

#[cfg(target_os = "windows")]
use crate::license::models::CREATE_NO_WINDOW;
use crate::security;

pub fn get_device_id() -> String {
    static DEVICE_ID: std::sync::OnceLock<String> = std::sync::OnceLock::new();

    DEVICE_ID.get_or_init(compute_device_id).clone()
}

fn compute_device_id() -> String {
    let mut hasher = Sha256::new();

    if let Ok(hostname) = hostname::get() {
        hasher.update(hostname.to_string_lossy().as_bytes());
    }

    hasher.update(std::env::consts::OS.as_bytes());
    hasher.update(std::env::consts::ARCH.as_bytes());

    if let Ok(user) = std::env::var("USER").or_else(|_| std::env::var("USERNAME")) {
        hasher.update(user.as_bytes());
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(output) =
            command_output_hidden("ioreg", &["-rd1", "-c", "IOPlatformExpertDevice"])
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = output_str.lines().find(|l| l.contains("IOPlatformUUID")) {
                hasher.update(line.as_bytes());
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = command_output_hidden("wmic", &["csproduct", "get", "UUID"]) {
            hasher.update(&output.stdout);
        }
    }

    #[cfg(target_os = "linux")]
    {
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
            hasher.update(b"whisprtypr-linux-device-id-fallback");
        }
    }

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
    let key = security::derive_encryption_key(&device_id);
    security::encrypt_data(data, &key)
}

pub(crate) fn decrypt_data(data: &[u8]) -> Result<Vec<u8>, String> {
    let device_id = get_device_id();
    let key = security::derive_encryption_key(&device_id);
    security::decrypt_data(data, &key)
}
