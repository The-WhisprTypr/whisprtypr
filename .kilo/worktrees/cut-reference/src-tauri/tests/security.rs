use vox_ai_lib::security::{decrypt_data, derive_encryption_key, encrypt_data, mask_license_key};

#[test]
fn encryption_key_is_stable_and_256_bit() {
    let key = derive_encryption_key("device-a");

    assert_eq!(key, derive_encryption_key("device-a"));
    assert_ne!(key, derive_encryption_key("device-b"));
    assert_eq!(key.len(), 32);
}

#[test]
fn encrypted_payload_round_trips_and_uses_random_nonce() {
    let key = derive_encryption_key("device-a");
    let plaintext = b"secret license payload";

    let first = encrypt_data(plaintext, &key).unwrap();
    let second = encrypt_data(plaintext, &key).unwrap();

    assert_ne!(first, plaintext);
    assert_ne!(first, second);
    assert_eq!(decrypt_data(&first, &key).unwrap(), plaintext);
    assert_eq!(decrypt_data(&second, &key).unwrap(), plaintext);
}

#[test]
fn decrypt_rejects_wrong_key_and_too_short_payloads() {
    let key = derive_encryption_key("device-a");
    let wrong_key = derive_encryption_key("device-b");
    let encrypted = encrypt_data(b"secret", &key).unwrap();

    assert!(decrypt_data(&encrypted, &wrong_key).is_err());
    assert_eq!(
        decrypt_data(b"short", &key).unwrap_err(),
        "Invalid encrypted data: too short"
    );
}

#[test]
fn license_key_masking_hides_sensitive_middle() {
    assert_eq!(mask_license_key("123"), "****");
    assert_eq!(mask_license_key("12345678"), "****");
    assert_eq!(mask_license_key("1234-5678-9012"), "1234****9012");
    assert_eq!(mask_license_key("WVT-SOLO-F110-C137"), "WVT-****C137");
}
