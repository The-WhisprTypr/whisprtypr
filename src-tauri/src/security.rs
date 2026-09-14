use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use sha2::{Digest, Sha256};

/// Derive a strong 256-bit encryption key from device ID
pub fn derive_encryption_key(device_id: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(device_id.as_bytes());
    hasher.update(b"whisprtypr-secure-v3-key-derivation");
    hasher.finalize().to_vec()
}

/// Encrypt data using a key
pub fn encrypt_data(data: &[u8], key_bytes: &[u8]) -> Result<Vec<u8>, String> {
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);

    // Generate a random 96-bit nonce
    let mut nonce_bytes = [0u8; 12];
    getrandom::getrandom(&mut nonce_bytes)
        .map_err(|e| format!("Failed to generate nonce: {}", e))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // Prepend nonce to ciphertext
    let mut result = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

/// Decrypt data using a key
pub fn decrypt_data(data: &[u8], key_bytes: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < 12 {
        return Err("Invalid encrypted data: too short".to_string());
    }

    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))
}

/// Mask a license key for safe logging/display
#[allow(dead_code)]
pub fn mask_license_key(key: &str) -> String {
    if key.len() <= 8 {
        return "****".to_string();
    }
    format!("{}****{}", &key[..4], &key[key.len() - 4..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_encryption_key_stability() {
        let device_id = "test-device-123";
        let key1 = derive_encryption_key(device_id);
        let key2 = derive_encryption_key(device_id);
        assert_eq!(key1, key2);
        assert_eq!(key1.len(), 32); // 256-bit
    }

    #[test]
    fn test_encryption_decryption_roundtrip() {
        let device_id = "test-device-456";
        let key = derive_encryption_key(device_id);
        let original_data = b"Hello, WhisprTypr Secure Data!";

        let encrypted = encrypt_data(original_data, &key).unwrap();
        assert_ne!(encrypted, original_data);
        assert!(encrypted.len() > original_data.len());

        let decrypted = decrypt_data(&encrypted, &key).unwrap();
        assert_eq!(decrypted, original_data);
    }

    #[test]
    fn test_decryption_with_wrong_key() {
        let key1 = derive_encryption_key("device-1");
        let key2 = derive_encryption_key("device-2");
        let data = b"Secret message";

        let encrypted = encrypt_data(data, &key1).unwrap();
        let result = decrypt_data(&encrypted, &key2);
        assert!(result.is_err());
    }

    #[test]
    fn test_decryption_invalid_data() {
        let key = derive_encryption_key("test");
        let result = decrypt_data(b"too-short", &key);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid encrypted data: too short");
    }

    #[test]
    fn test_mask_license_key() {
        assert_eq!(mask_license_key("123"), "****");
        assert_eq!(mask_license_key("12345678"), "****");
        assert_eq!(mask_license_key("1234-5678-9012"), "1234****9012");
        assert_eq!(mask_license_key("POLAR-KEY-ABC-DEF"), "POLA****-DEF");
    }
}
