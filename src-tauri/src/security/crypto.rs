use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use sha2::{Digest, Sha256};

pub fn derive_encryption_key(device_id: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(device_id.as_bytes());
    hasher.update(b"whisprtypr-secure-v3-key-derivation");
    hasher.finalize().to_vec()
}

pub fn encrypt_data(data: &[u8], key_bytes: &[u8]) -> Result<Vec<u8>, String> {
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key_bytes);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; 12];
    getrandom::getrandom(&mut nonce_bytes)
        .map_err(|e| format!("Failed to generate nonce: {}", e))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut result = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

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
