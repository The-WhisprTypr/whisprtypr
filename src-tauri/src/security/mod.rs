pub mod crypto;
pub use crypto::decrypt_data;
pub use crypto::encrypt_data;

pub fn mask_license_key(key: &str) -> String {
    if key.len() <= 8 {
        return "****".to_string();
    }
    format!("{}****{}", &key[..4], &key[key.len() - 4..])
}

#[cfg(test)]
mod tests {
    use super::crypto;
    use super::mask_license_key;

    #[test]
    fn test_encryption_decryption_roundtrip() {
        let device_id = "test-device-456";
        let original_data = b"Hello, Whisprtypr Secure Data!";

        let encrypted = crypto::encrypt_data(original_data, device_id).unwrap();
        assert_ne!(encrypted, original_data);
        assert!(encrypted.len() > original_data.len());

        let decrypted = crypto::decrypt_data(&encrypted, device_id).unwrap();
        assert_eq!(decrypted, original_data);
    }

    #[test]
    fn test_encryption_uses_random_salt() {
        let device_id = "test-device-456";
        let original_data = b"Hello, Whisprtypr Secure Data!";

        let first = crypto::encrypt_data(original_data, device_id).unwrap();
        let second = crypto::encrypt_data(original_data, device_id).unwrap();
        assert_ne!(first, second);
        assert_eq!(crypto::decrypt_data(&first, device_id).unwrap(), original_data);
        assert_eq!(crypto::decrypt_data(&second, device_id).unwrap(), original_data);
    }

    #[test]
    fn test_decryption_with_wrong_device_id() {
        let data = b"Secret message";
        let encrypted = crypto::encrypt_data(data, "device-1").unwrap();
        let result = crypto::decrypt_data(&encrypted, "device-2");
        assert!(result.is_err());
    }

    #[test]
    fn test_decryption_invalid_data() {
        let result = crypto::decrypt_data(b"too-short", "test");
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