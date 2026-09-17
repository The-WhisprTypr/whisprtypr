use log::{debug, info, warn};

use crate::license::device::{
    calculate_integrity_hash, decrypt_data, encrypt_data, get_cache_dir, get_cache_path,
    get_device_id,
};
use crate::license::models::{CachedLicense, CACHE_VERSION};

pub fn store_cache(cache: &CachedLicense) -> Result<(), String> {
    let cache_dir = get_cache_dir().ok_or("Failed to get cache directory")?;

    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create cache directory: {}", e))?;

    let cache_path = get_cache_path().ok_or("Failed to get cache path")?;

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

pub fn load_cache() -> Option<CachedLicense> {
    let cache_path = get_cache_path()?;

    let encrypted = std::fs::read(&cache_path).ok()?;
    let decrypted = decrypt_data(&encrypted).ok()?;
    let json = String::from_utf8(decrypted).ok()?;
    let cache: CachedLicense = serde_json::from_str(&json).ok()?;

    let expected_hash = calculate_integrity_hash(&cache);
    if cache.integrity_hash != expected_hash {
        warn!("License cache integrity check failed - possible tampering");
        return None;
    }

    if cache.device_id != get_device_id() {
        warn!("License cache device mismatch");
        return None;
    }

    if cache.cache_version != CACHE_VERSION {
        warn!("License cache version mismatch");
        return None;
    }

    debug!("License cache loaded successfully");
    Some(cache)
}

pub fn clear_cache() -> Result<(), String> {
    if let Some(path) = get_cache_path() {
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| format!("Failed to delete cache: {}", e))?;
        }
    }
    info!("License cache cleared");
    Ok(())
}
