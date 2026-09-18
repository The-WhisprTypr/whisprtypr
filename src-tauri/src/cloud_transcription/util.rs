use crate::license::get_device_id;
use crate::security::{decrypt_data, encrypt_data};
use reqwest::Client;
use std::io::Cursor;
use std::time::Duration;

pub fn encode_samples_to_wav(samples: &[f32]) -> Result<Vec<u8>, String> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut cursor = Cursor::new(Vec::with_capacity(44 + samples.len() * 2));
    {
        let mut writer = hound::WavWriter::new(&mut cursor, spec)
            .map_err(|e| format!("Failed to create in-memory WAV writer: {}", e))?;

        for &sample in samples {
            let clamped = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
            writer
                .write_sample(clamped)
                .map_err(|e| format!("Failed to write audio sample: {}", e))?;
        }

        writer
            .finalize()
            .map_err(|e| format!("Failed to finalize in-memory WAV: {}", e))?;
    }

    Ok(cursor.into_inner())
}

pub fn encrypt_api_key(api_key: &str) -> Result<String, String> {
    if api_key.trim().is_empty() {
        return Ok(String::new());
    }
    let device_id = get_device_id();
    let encrypted = encrypt_data(api_key.as_bytes(), &device_id)?;
    Ok(hex::encode(encrypted))
}

pub fn decrypt_api_key(stored_key: &str) -> String {
    let trimmed = stored_key.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    if let Ok(bytes) = hex::decode(trimmed) {
        let device_id = get_device_id();
        if let Ok(decrypted) = decrypt_data(&bytes, &device_id) {
            if let Ok(s) = String::from_utf8(decrypted) {
                return s;
            }
        }
    }

    trimmed.to_string()
}

pub fn mask_key(key: &str) -> String {
    let key = key.trim();
    if key.is_empty() {
        return String::new();
    }
    if key.len() <= 8 {
        return "••••••••".to_string();
    }
    let prefix = &key[..std::cmp::min(4, key.len())];
    let suffix = &key[key.len() - std::cmp::min(4, key.len())..];
    format!("{}••••{}", prefix, suffix)
}

pub fn parse_cloud_model_id(model_id: &str) -> Option<(&str, &str)> {
    if let Some(rest) = model_id.strip_prefix("cloud:") {
        if let Some((provider, model)) = rest.split_once(':') {
            return Some((provider, model));
        }
    }
    None
}

pub fn build_http_client(timeout_secs: u64) -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_samples_to_wav() {
        let samples = vec![0.0, 0.5, -0.5, 1.0, -1.0];
        let result = encode_samples_to_wav(&samples);
        assert!(result.is_ok());
        let wav = result.unwrap();
        assert!(wav.len() > 44);
        assert_eq!(&wav[0..4], b"RIFF");
    }

    #[test]
    fn test_mask_key() {
        assert_eq!(mask_key("short"), "••••••••");
        assert_eq!(mask_key(""), "");
        let long_key = "abcdefghijklmnopqrstuvwxyz123456";
        let masked = mask_key(long_key);
        assert!(masked.starts_with("abcd"));
        assert!(masked.ends_with("3456"));
        assert!(masked.contains("•"));
    }

    #[test]
    fn test_encrypt_decrypt_api_key_roundtrip() {
        let api_key = "sk-test-key-12345";
        let encrypted = encrypt_api_key(api_key).unwrap();
        assert!(!encrypted.is_empty());
        assert_ne!(encrypted, api_key);
        let decrypted = decrypt_api_key(&encrypted);
        assert_eq!(decrypted, api_key);
    }

    #[test]
    fn test_encrypt_decrypt_api_key_empty() {
        let encrypted = encrypt_api_key("").unwrap();
        assert!(encrypted.is_empty());
        let decrypted = decrypt_api_key("");
        assert!(decrypted.is_empty());
    }

    #[test]
    fn test_encrypt_decrypt_api_key_wrong_key() {
        let api_key = "sk-test-key-12345";
        let encrypted = encrypt_api_key(api_key).unwrap();
        let wrong_key = "sk-wrong-key-67890";
        let encrypted_wrong = encrypt_api_key(wrong_key).unwrap();
        let decrypted = decrypt_api_key(&encrypted_wrong);
        assert_ne!(decrypted, api_key);
    }

    #[test]
    fn test_encrypt_api_key_deterministic() {
        let key = "sk-deterministic-test";
        let enc1 = encrypt_api_key(key).unwrap();
        let enc2 = encrypt_api_key(key).unwrap();
        assert_eq!(decrypt_api_key(&enc1), key);
        assert_eq!(decrypt_api_key(&enc2), key);
    }
}
