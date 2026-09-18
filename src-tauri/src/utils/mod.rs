pub mod audio;
pub use audio::*;

pub use self::path::canonicalize_existing_file_path;
pub use self::path::path_has_extension;
pub use self::path::validate_export_path;
pub use self::text::is_model_language_supported;
pub use self::text::is_valid_language_code;
pub use self::text::is_youtube_url;
pub use self::text::sanitize_text;
pub use self::text::sanitize_url;
pub use self::text::validate_url_host;

pub const AUDIO_FILE_EXTENSIONS: &[&str] =
    &["wav", "mp3", "m4a", "ogg", "flac", "aac", "webm", "mkv"];

pub const EXPORT_FILE_EXTENSIONS: &[&str] = &["json", "md", "markdown"];

pub const MAX_EXPORT_BYTES: usize = 10 * 1024 * 1024;

pub const AUDIO_TARGET_SAMPLE_RATE: u32 = 16_000;

pub const MAX_FILE_TRANSCRIPTION_SECONDS: usize = 30 * 60;

pub const MAX_FILE_AUDIO_SAMPLES: usize =
    AUDIO_TARGET_SAMPLE_RATE as usize * MAX_FILE_TRANSCRIPTION_SECONDS;

pub mod text {
    pub fn sanitize_text(text: &str, max_len: usize) -> Result<String, String> {
        if text.len() > max_len {
            return Err(format!("Text exceeds maximum length of {} bytes", max_len));
        }

        let sanitized: String = text
            .chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t')
            .collect();

        Ok(sanitized)
    }

    pub fn is_valid_language_code(language: &str) -> bool {
        language == "auto"
            || ((2..=4).contains(&language.len())
                && language.chars().all(|c| c.is_ascii_lowercase()))
    }

    const PARAKEET_V3_LANGUAGES: &[&str] = &[
        "bg", "hr", "cs", "da", "nl", "en", "et", "fi", "fr", "de", "el", "hu", "it", "lv", "lt",
        "mt", "pl", "pt", "ro", "sk", "sl", "es", "sv", "ru", "uk",
    ];

    const QWEN3_ASR_LANGUAGES: &[&str] = &[
        "zh", "en", "yue", "ar", "de", "fr", "es", "pt", "id", "it", "ko", "ru", "th", "vi", "ja",
        "tr", "hi", "ms", "nl", "sv", "da", "fi", "pl", "cs", "fil", "fa", "el", "hu", "mk", "ro",
    ];

    pub fn is_model_language_supported(model_id: &str, language: &str) -> bool {
        match model_id {
            "tiny.en" | "base.en" | "small.en" | "medium.en" | "distil-small.en"
            | "parakeet-v2" => language == "en",
            "parakeet-v3" => language == "auto" || PARAKEET_V3_LANGUAGES.contains(&language),
            "qwen3-asr-0.6b" => language == "auto" || QWEN3_ASR_LANGUAGES.contains(&language),
            _ => language == "auto" || is_valid_language_code(language),
        }
    }

    pub fn is_youtube_url(url: &str) -> bool {
        let lower = url.to_lowercase();
        lower.contains("youtube.com")
            || lower.contains("youtu.be")
            || lower.contains("youtube.co")
            || lower.contains("yt.be")
    }

    pub fn sanitize_url(url: &str) -> Result<String, String> {
        let trimmed = url.trim();
        if trimmed.is_empty() {
            return Err("URL is empty".to_string());
        }

        let parsed = url::Url::parse(trimmed).map_err(|e| format!("Invalid URL: {}", e))?;
        if !matches!(parsed.scheme(), "http" | "https") {
            return Err("URL must start with http:// or https://".to_string());
        }

        Ok(trimmed.to_string())
    }

    fn is_private_ip(ip: std::net::IpAddr) -> bool {
        match ip {
            std::net::IpAddr::V4(ipv4) => {
                let octets = ipv4.octets();
                matches!(
                    octets,
                    [10, ..]
                        | [127, ..]
                        | [169, 254, ..]
                        | [172, 16..=31, ..]
                        | [192, 168, ..]
                        | [0, ..]
                        | [255, 255, 255, 255]
                )
            }
            std::net::IpAddr::V6(ipv6) => {
                let segments = ipv6.segments();
                ipv6.is_loopback()
                    || ipv6.is_unspecified()
                    || (segments[0] & 0xfe00 == 0xfc00)
                    || (segments[0] & 0xffc0 == 0xfe80)
                    || (segments[0] == 0x2001 && segments[1] == 0xdb8)
            }
        }
    }

    pub async fn validate_url_host(url: &str) -> Result<(), String> {
        let parsed = url::Url::parse(url).map_err(|e| format!("Invalid URL: {}", e))?;
        let host = parsed.host_str().ok_or("URL missing host")?;

        let addrs = tokio::net::lookup_host(format!("{}:80", host))
            .await
            .map_err(|e| format!("DNS resolution failed: {}", e))?;

        for addr in addrs {
            if is_private_ip(addr.ip()) {
                return Err(format!(
                    "URL resolves to private/internal IP address: {}",
                    addr.ip()
                ));
            }
        }

        Ok(())
    }
}

pub mod path {
    use std::path::PathBuf;

    use super::EXPORT_FILE_EXTENSIONS;

    pub fn canonicalize_existing_file_path(path: &str) -> Result<PathBuf, String> {
        if path.trim().is_empty() || path.contains('\0') {
            return Err("Invalid path".to_string());
        }

        let path = std::path::Path::new(path);
        let canonical = path
            .canonicalize()
            .map_err(|e| format!("Cannot access selected file: {}", e))?;
        let metadata = std::fs::metadata(&canonical)
            .map_err(|e| format!("Cannot read selected file: {}", e))?;

        if !metadata.is_file() {
            return Err("Selected path is not a file".to_string());
        }

        Ok(canonical)
    }

    pub fn path_has_extension(path: &std::path::Path, allowed: &[&str]) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|extension| {
                allowed
                    .iter()
                    .any(|allowed| extension.eq_ignore_ascii_case(allowed))
            })
            .unwrap_or(false)
    }

    pub fn validate_export_path(path: &str) -> Result<PathBuf, String> {
        if path.trim().is_empty() || path.contains('\0') {
            return Err("Invalid export path".to_string());
        }

        let path = std::path::Path::new(path);
        if !path_has_extension(path, EXPORT_FILE_EXTENSIONS) {
            return Err("Export path must end in .json, .md, or .markdown".to_string());
        }

        let parent = path
            .parent()
            .ok_or_else(|| "Export path must include a parent directory".to_string())?;
        let parent = parent
            .canonicalize()
            .map_err(|e| format!("Cannot access export directory: {}", e))?;

        if !parent.is_dir() {
            return Err("Export directory is not a directory".to_string());
        }

        let file_name = path
            .file_name()
            .ok_or_else(|| "Export path must include a file name".to_string())?;

        Ok(parent.join(file_name))
    }
}
