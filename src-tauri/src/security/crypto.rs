use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{Argon2, Params, Algorithm, Version};
use getrandom::getrandom;

const SALT_LEN: usize = 16;
const KEY_LEN: usize = 32;
const ARGON2_PARAMS: Params = match Params::new(65536, 3, 1, Some(KEY_LEN)) {
    Ok(p) => p,
    Err(_) => panic!("Invalid Argon2 parameters"),
};

fn derive_key(device_id: &str, salt: &[u8]) -> Vec<u8> {
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, ARGON2_PARAMS);
    let mut key = vec![0u8; KEY_LEN];
    argon2.hash_password_into(device_id.as_bytes(), salt, &mut key)
        .expect("Argon2 key derivation failed");
    key
}

pub fn encrypt_data(data: &[u8], device_id: &str) -> Result<Vec<u8>, String> {
    let mut salt = [0u8; SALT_LEN];
    getrandom(&mut salt)
        .map_err(|e| format!("Failed to generate salt: {}", e))?;

    let key = derive_key(device_id, &salt);

    let cipher = Aes256Gcm::new(aes_gcm::Key::<Aes256Gcm>::from_slice(&key));

    let mut nonce_bytes = [0u8; 12];
    getrandom(&mut nonce_bytes)
        .map_err(|e| format!("Failed to generate nonce: {}", e))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut result = Vec::with_capacity(SALT_LEN + nonce_bytes.len() + ciphertext.len());
    result.extend_from_slice(&salt);
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

pub fn decrypt_data(data: &[u8], device_id: &str) -> Result<Vec<u8>, String> {
    if data.len() < SALT_LEN + 12 {
        return Err("Invalid encrypted data: too short".to_string());
    }

    let (salt, rest) = data.split_at(SALT_LEN);
    if rest.len() < 12 {
        return Err("Invalid encrypted data: too short".to_string());
    }

    let (nonce_bytes, ciphertext) = rest.split_at(12);

    let key = derive_key(device_id, salt);

    let cipher = Aes256Gcm::new(aes_gcm::Key::<Aes256Gcm>::from_slice(&key));
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption_roundtrip() {
        let device_id = "test-device-123";
        let original_data = b"Hello, Whisprtypr Secure Data!";

        let encrypted = encrypt_data(original_data, device_id).unwrap();
        assert_ne!(encrypted, original_data);
        assert!(encrypted.len() > original_data.len());

        let decrypted = decrypt_data(&encrypted, device_id).unwrap();
        assert_eq!(decrypted, original_data);
    }

    #[test]
    fn test_encryption_uses_random_salt() {
        let device_id = "test-device-123";
        let data = b"test data";

        let first = encrypt_data(data, device_id).unwrap();
        let second = encrypt_data(data, device_id).unwrap();

        assert_ne!(first, second);
        assert_eq!(decrypt_data(&first, device_id).unwrap(), data);
        assert_eq!(decrypt_data(&second, device_id).unwrap(), data);
    }

    #[test]
    fn test_decryption_with_wrong_device_id() {
        let data = b"Secret message";
        let encrypted = encrypt_data(data, "device-1").unwrap();
        let result = decrypt_data(&encrypted, "device-2");
        assert!(result.is_err());
    }

    #[test]
    fn test_decryption_invalid_data() {
        let result = decrypt_data(b"too-short", "test");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid encrypted data: too short");
    }
}