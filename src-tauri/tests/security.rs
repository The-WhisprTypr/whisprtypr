use whisprtypr_lib::security::{
    decrypt_data, encrypt_data, mask_license_key,
};

#[test]
fn encrypted_payload_round_trips_and_uses_random_salt() {
    let device_id = "device-a";
    let plaintext = b"secret license payload";

    let first = encrypt_data(plaintext, device_id).unwrap();
    let second = encrypt_data(plaintext, device_id).unwrap();

    assert_ne!(first, plaintext);
    assert_ne!(first, second);
    assert_eq!(decrypt_data(&first, device_id).unwrap(), plaintext);
    assert_eq!(decrypt_data(&second, device_id).unwrap(), plaintext);
}

#[test]
fn decrypt_rejects_wrong_device_id_and_too_short_payloads() {
    let encrypted = encrypt_data(b"secret", "device-a").unwrap();

    assert!(decrypt_data(&encrypted, "device-b").is_err());
    assert_eq!(
        decrypt_data(b"short", "device-a").unwrap_err(),
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