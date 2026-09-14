#![recursion_limit = "512"]

mod audio;
pub mod cloud_transcription;
pub mod database;
pub mod downloader;
pub mod ai_formatting;
mod error_reporting;
pub mod license;
pub mod post_process;
pub mod security;
mod text_inject;
pub mod transcription;
mod translation;

use audio::{AudioCaptureSource, AudioInputDevice, AudioOutputDevice, AudioRecorder};
use cloud_transcription::build_http_client;
use database::{AppSettings, AppState, Database, LicenseData, TranscriptionHistory, VocabularyEntry, WhisperModel, AiFormattingProviderRecord};
use downloader::{DownloadProgress, ModelDownloader};
use error_reporting::{ErrorCategory, ErrorReport, ErrorReporter, ErrorSeverity, ErrorStats};
use license::{
    clear_cache, get_device_id, get_device_label, LicenseInfo, LicenseManager,
    LicenseStatus,
};
use log::{debug, error, info, warn};
use post_process::PostProcessor;
use sentry::Level as SentryLogLevel;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, State, WindowEvent,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use transcription::Transcriber;

// Application version from Cargo.toml
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_NAME: &str = "WhisprTypr";
const SENTRY_DSN: Option<&str> = option_env!("SENTRY_DSN");
const APP_ICON_BYTES: &[u8] = include_bytes!("../icons/icon.png");
const AUDIO_TARGET_SAMPLE_RATE: u32 = 16_000;
const MAX_FILE_TRANSCRIPTION_SECONDS: usize = 30 * 60;
const MAX_FILE_AUDIO_SAMPLES: usize =
    AUDIO_TARGET_SAMPLE_RATE as usize * MAX_FILE_TRANSCRIPTION_SECONDS;

// Rate limiter for preventing abuse
pub struct RateLimiter {
    requests: Mutex<HashMap<String, Vec<Instant>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            requests: Mutex::new(HashMap::new()),
            max_requests,
            window: Duration::from_secs(window_secs),
        }
    }

    pub fn check(&self, key: &str) -> bool {
        let mut requests = self.requests.lock().unwrap();
        let now = Instant::now();

        let timestamps = requests.entry(key.to_string()).or_default();

        // Remove old timestamps outside the window
        timestamps.retain(|&t| now.duration_since(t) < self.window);

        if timestamps.len() >= self.max_requests {
            warn!("Rate limit exceeded for action: {}", key);
            false
        } else {
            timestamps.push(now);
            true
        }
    }
}

pub struct RateLimiterState(pub Arc<RateLimiter>);

const AUDIO_FILE_EXTENSIONS: &[&str] = &["wav", "mp3", "m4a", "ogg", "flac", "aac", "webm", "mkv"];
const EXPORT_FILE_EXTENSIONS: &[&str] = &["json", "md", "markdown"];
const MAX_EXPORT_BYTES: usize = 10 * 1024 * 1024;

// Input sanitization utilities
fn canonicalize_existing_file_path(path: &str) -> Result<std::path::PathBuf, String> {
    if path.trim().is_empty() || path.contains('\0') {
        return Err("Invalid path".to_string());
    }

    let path = std::path::Path::new(path);
    let canonical = path
        .canonicalize()
        .map_err(|e| format!("Cannot access selected file: {}", e))?;
    let metadata =
        std::fs::metadata(&canonical).map_err(|e| format!("Cannot read selected file: {}", e))?;

    if !metadata.is_file() {
        return Err("Selected path is not a file".to_string());
    }

    Ok(canonical)
}

fn path_has_extension(path: &std::path::Path, allowed: &[&str]) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|extension| {
            allowed
                .iter()
                .any(|allowed| extension.eq_ignore_ascii_case(allowed))
        })
        .unwrap_or(false)
}

fn validate_export_path(path: &str) -> Result<std::path::PathBuf, String> {
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

fn sanitize_text(text: &str, max_len: usize) -> Result<String, String> {
    if text.len() > max_len {
        return Err(format!("Text exceeds maximum length of {} bytes", max_len));
    }

    // Remove null bytes and other control characters that could cause issues
    let sanitized: String = text
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t')
        .collect();

    Ok(sanitized)
}

fn is_valid_language_code(language: &str) -> bool {
    language == "auto"
        || ((2..=4).contains(&language.len()) && language.chars().all(|c| c.is_ascii_lowercase()))
}

const PARAKEET_V3_LANGUAGES: &[&str] = &[
    "bg", "hr", "cs", "da", "nl", "en", "et", "fi", "fr", "de", "el", "hu", "it",
    "lv", "lt", "mt", "pl", "pt", "ro", "sk", "sl", "es", "sv", "ru", "uk",
];

const QWEN3_ASR_LANGUAGES: &[&str] = &[
    "zh", "en", "yue", "ar", "de", "fr", "es", "pt", "id", "it", "ko", "ru",
    "th", "vi", "ja", "tr", "hi", "ms", "nl", "sv", "da", "fi", "pl", "cs",
    "fil", "fa", "el", "hu", "mk", "ro",
];

fn is_model_language_supported(model_id: &str, language: &str) -> bool {
    match model_id {
        // English-only models
        "tiny.en" | "base.en" | "small.en" | "medium.en" | "distil-small.en" | "parakeet-v2" => {
            language == "en"
        }

        // Parakeet v3 supported languages
        "parakeet-v3" => {
            language == "auto" || PARAKEET_V3_LANGUAGES.contains(&language)
        }

        // Qwen3-ASR supported languages
        "qwen3-asr-0.6b" => {
            language == "auto" || QWEN3_ASR_LANGUAGES.contains(&language)
        }

        // Multilingual Whisper models (tiny, base, small, medium, large-v3, large-v3-turbo)
        _ => language == "auto" || is_valid_language_code(language),
    }
}

// State wrappers
pub struct DbState(pub Arc<Database>);
pub struct RecorderState(pub Arc<Mutex<Option<AudioRecorder>>>);
pub struct TranscriberState(pub Arc<Mutex<Option<Transcriber>>>);
pub struct DownloaderState(pub Arc<ModelDownloader>);
pub struct LicenseManagerState(pub Arc<LicenseManager>);
pub struct TextInjectorState(pub Arc<Mutex<text_inject::TextInjector>>);
// Rate limiter: 100 requests per minute per action
pub struct RecordingRateLimiter(pub Arc<RateLimiter>);
pub struct TranscriptionRateLimiter(pub Arc<RateLimiter>);

// Error type for commands
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Recording error: {0}")]
    Recording(String),
    #[error("Transcription error: {0}")]
    Transcription(String),
    #[error("Download error: {0}")]
    Download(String),
    #[error("Text injection error: {0}")]
    TextInjection(String),
    #[error("License error: {0}")]
    License(String),
    #[error("Post-processing error: {0}")]
    PostProcessing(String),
}

fn user_facing_license_error(error: &str) -> &'static str {
    let lower = error.to_lowercase();

    if lower.contains("no license") || lower.contains("not activated") {
        return "No active license was found. Please activate your license key.";
    }

    if lower.contains("activation limit") {
        return "This license has reached its device limit. Please deactivate it on another device first.";
    }

    if lower.contains("invalid license") || lower.contains("invalid request") {
        return "That license key could not be verified. Please check the key and try again.";
    }

    if lower.contains("network") || lower.contains("connect") || lower.contains("offline") {
        return "Could not reach the license server. Please check your internet connection and try again.";
    }

    if lower.contains("expired") {
        return "This license has expired. Please renew or use a different license key.";
    }

    if lower.contains("revoked") || lower.contains("disabled") || lower.contains("rejected") {
        return "This license could not be verified. Please contact support if you think this is a mistake.";
    }

    "License verification failed. Please try again."
}

impl serde::Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            CommandError::License(error) => {
                let message = format!("License error: {}", user_facing_license_error(error));
                serializer.serialize_str(&message)
            }
            _ => serializer.serialize_str(self.to_string().as_ref()),
        }
    }
}

type CommandResult<T> = Result<T, CommandError>;

fn license_status_to_response(status: &LicenseStatus) -> String {
    match status {
        LicenseStatus::Granted | LicenseStatus::Offline => "active".to_string(),
        LicenseStatus::Revoked => "revoked".to_string(),
        LicenseStatus::Disabled => "disabled".to_string(),
        LicenseStatus::Expired => "expired".to_string(),
        LicenseStatus::Invalid => "invalid".to_string(),
        LicenseStatus::ActivationLimitReached => "activation_limit".to_string(),
        LicenseStatus::NotActivated => "not_activated".to_string(),
    }
}

#[allow(unused_variables)]
async fn verified_license_check(license_manager: &LicenseManager) -> VerifiedLicenseResult {
    // Fast path: trust the local cache so hotkey-driven calls don't hit
    // Polar on every push-to-talk and dictation still works offline.
    if license_manager.is_cached_license_valid() {
        return VerifiedLicenseResult::Granted;
    }

    match license_manager.validate_throttled().await {
        Ok(info) => {
            if info.status.allows_usage() {
                VerifiedLicenseResult::Granted
            } else {
                VerifiedLicenseResult::Rejected
            }
        }
        Err(error) => {
            if license_manager.is_authoritative_validate_error(&error) {
                // Authoritative rejection (revoked/disabled/invalid). The
                // cache has already been cleared by LicenseManager. Callers
                // must NOT fall back to a stale DB row.
                VerifiedLicenseResult::Rejected
            } else {
                // Transient failure (network/5xx). Callers may fall back to
                // the DB row / offline grace period.
                VerifiedLicenseResult::OfflineFallback
            }
        }
    }
}

/// Outcome of an online license check.
enum VerifiedLicenseResult {
    Granted,
    Rejected,
    OfflineFallback,
}

#[allow(unused_variables)]
fn has_active_trial(db: &Database) -> bool {
    let Ok(license) = db.get_license() else {
        return false;
    };

    has_active_trial_core(&license, chrono::Utc::now())
}

pub fn has_active_trial_core(
    license: &database::LicenseData,
    now: chrono::DateTime<chrono::Utc>,
) -> bool {
    if license.status != "trial" {
        return false;
    }

    let Some(trial_started) = &license.trial_started_at else {
        return false;
    };

    let expected_hash = calculate_trial_integrity_hash(trial_started);
    if license.trial_integrity_hash.as_deref() != Some(expected_hash.as_str()) {
        warn!("Trial integrity check failed");
        return false;
    }

    let Ok(start_date) = chrono::DateTime::parse_from_rfc3339(trial_started) else {
        return false;
    };

    let start_date = start_date.with_timezone(&chrono::Utc);
    if start_date > now {
        warn!("Trial start date is in the future");
        return false;
    }

    let days_since_start = (now - start_date).num_days();
    (0..7).contains(&days_since_start)
}

pub fn db_license_allows_usage(license: &database::LicenseData) -> bool {
    db_license_allows_usage_core(license, chrono::Utc::now())
}

pub fn db_license_allows_usage_core(
    license: &database::LicenseData,
    now: chrono::DateTime<chrono::Utc>,
) -> bool {
    if !license.is_activated || license.status != "active" {
        return false;
    }

    if license.license_key.is_none() || license.activation_id.is_none() {
        return false;
    }

    if let Some(expires_at) = &license.expires_at {
        let Ok(expiry) = chrono::DateTime::parse_from_rfc3339(expires_at) else {
            warn!("Stored license has invalid expiration timestamp");
            return false;
        };

        if expiry < now {
            return false;
        }
    }

    let Some(last_validated_at) = &license.last_validated_at else {
        return false;
    };

    let Ok(last_validated) = chrono::DateTime::parse_from_rfc3339(last_validated_at) else {
        warn!("Stored license has invalid validation timestamp");
        return false;
    };

    let last_validated = last_validated.with_timezone(&chrono::Utc);
    if last_validated > now {
        warn!("Stored license validation timestamp is in the future");
        return false;
    }

    (now - last_validated).num_hours() < 168
}

pub fn calculate_trial_integrity_hash(trial_started_at: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(trial_started_at.as_bytes());
    hasher.update(get_device_id().as_bytes());
    hasher.update(b"whisprtypr-trial-integrity-v1");
    hex::encode(hasher.finalize())
}

// ==================== Settings Commands ====================

#[tauri::command]
fn get_settings(db: State<DbState>) -> CommandResult<AppSettings> {
    db.0.get_settings().map_err(Into::into)
}

#[tauri::command]
fn update_settings(db: State<DbState>, settings: AppSettings) -> CommandResult<()> {
    db.0.update_settings(&settings).map_err(Into::into)
}

#[tauri::command]
fn update_setting(db: State<DbState>, key: String, value: String) -> CommandResult<()> {
    db.0.update_setting(&key, &value).map_err(Into::into)
}

// ==================== App State Commands ====================

#[tauri::command]
fn get_app_state(db: State<DbState>) -> CommandResult<AppState> {
    db.0.get_app_state().map_err(Into::into)
}

#[tauri::command]
fn update_app_state(db: State<DbState>, state: AppState) -> CommandResult<()> {
    db.0.update_app_state(&state).map_err(Into::into)
}

#[tauri::command]
fn set_setup_complete(db: State<DbState>, complete: bool) -> CommandResult<()> {
    db.0.set_setup_complete(complete).map_err(Into::into)
}

#[tauri::command]
fn set_current_setup_step(db: State<DbState>, step: i32) -> CommandResult<()> {
    db.0.set_current_setup_step(step).map_err(Into::into)
}

// ==================== Model Commands ====================

#[tauri::command]
fn get_models(
    db: State<DbState>,
    downloader: State<DownloaderState>,
) -> CommandResult<Vec<WhisperModel>> {
    let mut models = db.0.get_models().map_err(CommandError::Database)?;

    for model in models.iter_mut() {
        let exists_on_disk = downloader.0.is_model_downloaded(&model.id);
        if model.downloaded != exists_on_disk {
            let path = if exists_on_disk {
                Some(
                    downloader
                        .0
                        .get_model_path(&model.id)
                        .to_string_lossy()
                        .to_string(),
                )
            } else {
                None
            };

            db.0.set_model_downloaded(&model.id, exists_on_disk, path.as_deref())
                .map_err(CommandError::Database)?;
            model.downloaded = exists_on_disk;
            model.download_path = path;
        }
    }

    Ok(models)
}

#[tauri::command]
fn get_model(
    db: State<DbState>,
    downloader: State<DownloaderState>,
    id: String,
) -> CommandResult<Option<WhisperModel>> {
    let mut model = db.0.get_model(&id).map_err(CommandError::Database)?;

    if let Some(ref mut model) = model {
        let exists_on_disk = downloader.0.is_model_downloaded(&model.id);
        if model.downloaded != exists_on_disk {
            let path = if exists_on_disk {
                Some(
                    downloader
                        .0
                        .get_model_path(&model.id)
                        .to_string_lossy()
                        .to_string(),
                )
            } else {
                None
            };

            db.0.set_model_downloaded(&model.id, exists_on_disk, path.as_deref())
                .map_err(CommandError::Database)?;
            model.downloaded = exists_on_disk;
            model.download_path = path;
        }
    }

    Ok(model)
}

#[tauri::command]
fn set_model_downloaded(
    db: State<DbState>,
    id: String,
    downloaded: bool,
    path: Option<String>,
) -> CommandResult<()> {
    db.0.set_model_downloaded(&id, downloaded, path.as_deref())
        .map_err(Into::into)
}

#[tauri::command]
fn set_selected_model(db: State<DbState>, model_id: Option<String>) -> CommandResult<()> {
    db.0.set_selected_model(model_id.as_deref())
        .map_err(Into::into)
}

// ==================== Recording Commands ====================

#[tauri::command]
fn get_audio_input_devices() -> CommandResult<Vec<AudioInputDevice>> {
    AudioRecorder::list_input_devices().map_err(CommandError::Recording)
}

#[tauri::command]
fn get_audio_output_devices() -> CommandResult<Vec<AudioOutputDevice>> {
    AudioRecorder::list_output_devices().map_err(CommandError::Recording)
}

#[tauri::command]
fn set_audio_input_device(
    recorder: State<RecorderState>,
    device_name: Option<String>,
) -> CommandResult<()> {
    let mut recorder_guard = recorder.0.lock().unwrap();

    if recorder_guard.is_none() {
        *recorder_guard = Some(AudioRecorder::new().map_err(CommandError::Recording)?);
    }

    if let Some(ref mut rec) = *recorder_guard {
        rec.set_input_device(device_name)
            .map_err(CommandError::Recording)?;
    }

    Ok(())
}

#[tauri::command]
fn set_audio_capture_config(
    recorder: State<RecorderState>,
    capture_source: AudioCaptureSource,
    input_device_name: Option<String>,
    output_device_name: Option<String>,
) -> CommandResult<()> {
    let mut recorder_guard = recorder.0.lock().unwrap();

    if recorder_guard.is_none() {
        *recorder_guard = Some(AudioRecorder::new().map_err(CommandError::Recording)?);
    }

    if let Some(ref mut rec) = *recorder_guard {
        rec.set_capture_config(capture_source, input_device_name, output_device_name)
            .map_err(CommandError::Recording)?;
    }

    Ok(())
}

#[tauri::command]
async fn start_recording(
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
    recorder: State<'_, RecorderState>,
    rate_limiter: State<'_, RecordingRateLimiter>,
) -> CommandResult<()> {
    let db = db.0.clone();
    let license_manager = license_manager.0.clone();
    let recorder = recorder.0.clone();
    let rate_limiter = rate_limiter.0.clone();

    ensure_app_access_verified(&db, &license_manager).await?;

    // Rate limiting check
    if !rate_limiter.check("start_recording") {
        return Err(CommandError::Recording(
            "Rate limit exceeded. Please wait before starting another recording.".to_string(),
        ));
    }

    debug!("start_recording called");
    let mut recorder_guard = recorder.lock().unwrap();

    if recorder_guard.is_none() {
        debug!("Creating new AudioRecorder");
        *recorder_guard = Some(AudioRecorder::new().map_err(|e| {
            error!("Failed to create AudioRecorder: {}", e);
            CommandError::Recording(e)
        })?)
    }

    if let Some(ref mut rec) = *recorder_guard {
        debug!("Starting recording...");
        rec.start_recording().map_err(|e| {
            error!("Failed to start recording: {}", e);
            CommandError::Recording(e)
        })?;
        debug!("Recording started successfully");
    }

    Ok(())
}

#[tauri::command]
fn stop_recording(recorder: State<RecorderState>) -> CommandResult<Vec<f32>> {
    let mut recorder_guard = recorder.0.lock().unwrap();

    if let Some(ref mut rec) = *recorder_guard {
        rec.stop_recording().map_err(|e| {
            error!("Failed to stop recording: {}", e);
            CommandError::Recording(e)
        })
    } else {
        Err(CommandError::Recording(
            "No recorder initialized".to_string(),
        ))
    }
}

#[tauri::command]
async fn save_temp_audio(
    app: tauri::AppHandle,
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
    samples: Vec<f32>,
) -> CommandResult<String> {
    let db = db.0.clone();
    let license_manager = license_manager.0.clone();

    ensure_app_access_verified(&db, &license_manager).await?;

    let temp_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| CommandError::Io(std::io::Error::other(e.to_string())))?;
    std::fs::create_dir_all(&temp_dir).map_err(CommandError::Io)?;

    let file_path = temp_dir.join(format!("temp_recording_{}.wav", uuid::Uuid::new_v4()));
    let path_str = file_path
        .to_str()
        .ok_or_else(|| CommandError::Io(std::io::Error::other("Invalid path")))?;

    audio::save_wav(&samples, path_str).map_err(CommandError::Recording)?;

    Ok(path_str.to_string())
}

#[tauri::command]
fn cancel_recording(recorder: State<RecorderState>) -> CommandResult<()> {
    let mut recorder_guard = recorder.0.lock().unwrap();

    if let Some(ref mut rec) = *recorder_guard {
        rec.cancel_recording();
    }

    Ok(())
}

#[tauri::command]
fn is_recording(recorder: State<RecorderState>) -> bool {
    let recorder_guard = recorder.0.lock().unwrap();
    recorder_guard
        .as_ref()
        .map(|r| r.is_recording())
        .unwrap_or(false)
}

// ==================== Recording Overlay Commands ====================

#[tauri::command]
async fn show_recording_overlay(app: tauri::AppHandle, position: Option<String>) -> CommandResult<()> {
    use tauri::Manager;

    if let Some(overlay_window) = app.get_webview_window("recording-overlay") {
        overlay_window.show().map_err(|e| {
            error!("Failed to show overlay window: {}", e);
            CommandError::Recording(format!("Failed to show overlay: {}", e))
        })?;

        overlay_window.set_fullscreen(true).map_err(|e| {
            warn!("Failed to set fullscreen: {}", e);
            CommandError::Recording(format!("Failed to set fullscreen: {}", e))
        })?;

        overlay_window.set_always_on_top(true).map_err(|e| {
            warn!("Failed to set always on top: {}", e);
            CommandError::Recording(format!("Failed to set always on top: {}", e))
        })?;

        if let Some(pos) = position {
            let _ = app.emit("overlay-position", pos);
        }

        debug!("Recording overlay shown");
    } else {
        warn!("Recording window not found");
    }

    Ok(())
}

#[tauri::command]
async fn hide_recording_overlay(app: tauri::AppHandle) -> CommandResult<()> {
    use tauri::Manager;

    if let Some(overlay_window) = app.get_webview_window("recording-overlay") {
        // Hide the overlay window
        overlay_window.hide().map_err(|e| {
            error!("Failed to hide overlay window: {}", e);
            CommandError::Recording(format!("Failed to hide overlay: {}", e))
        })?;

        debug!("Recording overlay hidden");
    }

    Ok(())
}

// ==================== Transcription Commands ====================

#[tauri::command]
async fn load_model(
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
    transcriber: State<'_, TranscriberState>,
    downloader: State<'_, DownloaderState>,
    model_id: String,
    language: String,
) -> CommandResult<()> {
    let db = db.0.clone();
    let license_manager = license_manager.0.clone();
    let transcriber = transcriber.0.clone();
    let downloader = downloader.0.clone();

    ensure_app_access_verified(&db, &license_manager).await?;

    // Cloud models do not require downloading local weights
    if model_id.starts_with("cloud:") {
        let mut transcriber_guard = transcriber.lock().unwrap();
        *transcriber_guard = None;
        info!("Cloud model selected: {} (language: {})", model_id, language);
        return Ok(());
    }

    if !is_valid_language_code(&language) {
        return Err(CommandError::Transcription(format!(
            "Invalid language code: {}",
            language
        )));
    }
    if !is_model_language_supported(&model_id, &language) {
        return Err(CommandError::Transcription(format!(
            "Language '{}' is not supported by model '{}'",
            language, model_id
        )));
    }

    let model_path = downloader.get_model_path(&model_id);

    if !model_path.exists() {
        return Err(CommandError::Transcription(format!(
            "Model {} is not downloaded",
            model_id
        )));
    }

    // Fast path: the same model + language is already loaded, so we don't
    // need to pay the multi-second reload cost. Loading from ONNX disk
    // takes 3-4s on Parakeet, so this is critical for the hotkey latency.
    {
        let transcriber_guard = transcriber.lock().unwrap();
        if let Some(existing) = transcriber_guard.as_ref() {
            if existing.model_id() == model_id && existing.language() == language {
                debug!(
                    "Model already loaded: {} (language: {}) — skipping reload",
                    model_id, language
                );
                return Ok(());
            }
        }
    }

    // Drop existing model first to free memory before loading new one
    {
        let mut transcriber_guard = transcriber.lock().unwrap();
        *transcriber_guard = None;
        // Force memory release by dropping the guard
        drop(transcriber_guard);
    }

    // Load new model
    let mut new_transcriber = Transcriber::new(&model_id, model_path.to_str().unwrap(), &language)
        .map_err(CommandError::Transcription)?;

    // Warm up the model now, during the (already slow) load path, so the
    // first hotkey press doesn't pay the multi-second CUDA kernel compile /
    // ONNX graph optimization cost. This is a tiny dummy sample so it's cheap
    // relative to the load itself.
    if let Err(e) = new_transcriber.warm_up() {
        warn!("Model warm-up failed (first dictation may be slower): {}", e);
    }

    let mut transcriber_guard = transcriber.lock().unwrap();
    *transcriber_guard = Some(new_transcriber);

    info!("Model loaded: {} (language: {})", model_id, language);

    Ok(())
}

#[tauri::command]
fn unload_model(transcriber: State<TranscriberState>) -> CommandResult<()> {
    let mut transcriber_guard = transcriber.0.lock().unwrap();
    *transcriber_guard = None;
    info!("Model unloaded");
    Ok(())
}

#[derive(serde::Serialize, Clone)]
struct LoadedModelInfo {
    model_id: String,
    language: String,
}

#[tauri::command]
fn get_loaded_model(transcriber: State<TranscriberState>) -> Option<LoadedModelInfo> {
    let transcriber_guard = transcriber.0.lock().unwrap();
    transcriber_guard
        .as_ref()
        .map(|t| LoadedModelInfo {
            model_id: t.model_id().to_string(),
            language: t.language().to_string(),
        })
}

#[tauri::command]
async fn transcribe_audio(
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
    transcriber: State<'_, TranscriberState>,
    audio_samples: Vec<f32>,
) -> CommandResult<String> {
    let db = db.0.clone();
    let license_manager = license_manager.0.clone();
    let transcriber = transcriber.0.clone();

    ensure_app_access_verified(&db, &license_manager).await?;

    let settings = db.get_settings().map_err(CommandError::Database)?;
    if settings.selected_model_id.starts_with("cloud:") {
        if let Some((provider, model)) = cloud_transcription::parse_cloud_model_id(&settings.selected_model_id) {
            let wav_bytes = cloud_transcription::encode_samples_to_wav(&audio_samples)
                .map_err(CommandError::Recording)?;
            let text = cloud_transcription::transcribe_with_cloud(&db, provider, model, wav_bytes, &settings.language, false)
                .await
                .map_err(CommandError::Transcription)?;
            return Ok(text);
        }
    }

    // Local transcription is blocking (CPU/GPU inference). Run it on a
    // dedicated blocking thread so the async runtime (and therefore the UI)
    // stays responsive while the model runs. The transcriber is Send+Sync so
    // we can move it into the blocking closure.
    tokio::task::spawn_blocking(move || {
        let mut transcriber_guard = transcriber.lock().unwrap();
        if let Some(ref mut t) = *transcriber_guard {
            t.transcribe(&audio_samples).map_err(CommandError::Transcription)
        } else {
            Err(CommandError::Transcription("No model loaded".to_string()))
        }
    })
    .await
    .map_err(|_| CommandError::Transcription("Transcription task panicked".to_string()))?
}

#[tauri::command]
async fn record_and_transcribe(
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
    recorder: State<'_, RecorderState>,
    transcriber: State<'_, TranscriberState>,
) -> CommandResult<String> {
    let _db = db.0.clone();
    let license_manager = license_manager.0.clone();
    let recorder = recorder.0.clone();
    let transcriber = transcriber.0.clone();

    ensure_app_access_verified(&_db, &license_manager).await?;

    // Stop recording first
    let samples = {
        let mut recorder_guard = recorder.lock().unwrap();
        if let Some(ref mut rec) = *recorder_guard {
            rec.stop_recording().map_err(CommandError::Recording)?
        } else {
            return Err(CommandError::Recording(
                "No recorder initialized".to_string(),
            ));
        }
    };

    // Transcribe
    let settings = _db.get_settings().map_err(CommandError::Database)?;
    if settings.selected_model_id.starts_with("cloud:") {
        if let Some((provider, model)) = cloud_transcription::parse_cloud_model_id(&settings.selected_model_id) {
            let wav_bytes = cloud_transcription::encode_samples_to_wav(&samples)
                .map_err(CommandError::Recording)?;
            let text = cloud_transcription::transcribe_with_cloud(&_db, provider, model, wav_bytes, &settings.language, false)
                .await
                .map_err(CommandError::Transcription)?;
            return Ok(text);
        }
    }

    // Local transcription is blocking (CPU/GPU inference). Run it on a
    // dedicated blocking thread so the async runtime (and therefore the UI)
    // stays responsive while the model runs.
    tokio::task::spawn_blocking(move || {
        let mut transcriber_guard = transcriber.lock().unwrap();
        if let Some(ref mut t) = *transcriber_guard {
            t.transcribe(&samples).map_err(CommandError::Transcription)
        } else {
            Err(CommandError::Transcription("No model loaded".to_string()))
        }
    })
    .await
    .map_err(|_| CommandError::Transcription("Transcription task panicked".to_string()))?
}

// ==================== Translation Commands ====================

#[tauri::command]
async fn translate_text(
    _db: State<'_, DbState>,
    app: tauri::AppHandle,
    text: String,
    source_language: String,
    target_language: String,
    api_key: Option<String>,
) -> CommandResult<String> {
    if text.trim().is_empty() {
        return Ok(String::new());
    }

    if source_language == target_language {
        return Ok(text);
    }

    let sanitized = sanitize_text(&text, 100_000).map_err(CommandError::PostProcessing)?;
    if sanitized.is_empty() {
        return Ok(String::new());
    }

    let _ = app.emit(
        "translation-status",
        serde_json::json!({
            "status": "translating",
            "source": source_language,
            "target": target_language,
        }),
    );

    let client = build_http_client(30).map_err(CommandError::Transcription)?;
    let result = translation::translate(
        &client,
        &sanitized,
        &source_language,
        &target_language,
        api_key.as_deref(),
    )
    .await
    .map_err(CommandError::Transcription)?;

    let _ = app.emit(
        "translation-status",
        serde_json::json!({
            "status": "complete",
            "source": source_language,
            "target": target_language,
        }),
    );

    info!(
        "MyMemory translation completed: {} chars -> {} chars",
        sanitized.len(),
        result.len()
    );
    Ok(result)
}

#[tauri::command]
async fn record_and_translate(
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
    recorder: State<'_, RecorderState>,
    transcriber: State<'_, TranscriberState>,
    app: tauri::AppHandle,
    source_language: String,
    target_language: String,
    api_key: Option<String>,
) -> CommandResult<String> {
    let db = db.0.clone();
    let license_manager = license_manager.0.clone();
    let recorder = recorder.0.clone();
    let transcriber = transcriber.0.clone();

    ensure_app_access_verified(&db, &license_manager).await?;

    let samples = {
        let mut recorder_guard = recorder.lock().unwrap();
        if let Some(ref mut rec) = *recorder_guard {
            rec.stop_recording().map_err(|e| {
                error!("Failed to stop recording: {}", e);
                CommandError::Recording(e)
            })?
        } else {
            return Err(CommandError::Recording(
                "No recorder initialized".to_string(),
            ));
        }
    };

    let settings = db.get_settings().map_err(CommandError::Database)?;
    let mut text = if settings.selected_model_id.starts_with("cloud:") {
        if let Some((provider, model)) = cloud_transcription::parse_cloud_model_id(&settings.selected_model_id) {
            let wav_bytes = cloud_transcription::encode_samples_to_wav(&samples)
                .map_err(CommandError::Recording)?;
            cloud_transcription::transcribe_with_cloud(&db, provider, model, wav_bytes, &source_language, false)
                .await
                .map_err(CommandError::Transcription)?
        } else {
            return Err(CommandError::Transcription("No model loaded".to_string()));
        }
    } else {
        let mut transcriber_guard = transcriber.lock().unwrap();
        if let Some(ref mut t) = *transcriber_guard {
            t.transcribe(&samples).map_err(CommandError::Transcription)?
        } else {
            return Err(CommandError::Transcription("No model loaded".to_string()));
        }
    };

    if text.trim().is_empty() {
        return Ok(String::new());
    }

    if source_language != target_language {
        let _ = app.emit(
            "translation-status",
            serde_json::json!({
                "status": "translating",
                "source": source_language,
                "target": target_language,
            }),
        );

        text = translation::translate(
            &build_http_client(30).map_err(CommandError::Transcription)?,
            &text,
            &source_language,
            &target_language,
            api_key.as_deref(),
        )
        .await
        .map_err(CommandError::Transcription)?;

        let _ = app.emit(
            "translation-status",
            serde_json::json!({
                "status": "complete",
                "source": source_language,
                "target": target_language,
            }),
        );
    }

    Ok(text)
}

#[tauri::command]
async fn transcribe_file(
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
    transcriber: State<'_, TranscriberState>,
    rate_limiter: State<'_, TranscriptionRateLimiter>,
    file_path: String,
) -> CommandResult<String> {
    let db = db.0.clone();
    let license_manager = license_manager.0.clone();
    let transcriber = transcriber.0.clone();
    let rate_limiter = rate_limiter.0.clone();

    ensure_app_access_verified(&db, &license_manager).await?;

    // Rate limiting check
    if !rate_limiter.check("transcribe_file") {
        return Err(CommandError::Transcription(
            "Rate limit exceeded. Please wait before transcribing another file.".to_string(),
        ));
    }

    let safe_path =
        canonicalize_existing_file_path(&file_path).map_err(CommandError::Transcription)?;

    if !path_has_extension(&safe_path, AUDIO_FILE_EXTENSIONS) {
        return Err(CommandError::Transcription(
            "Unsupported audio format. Please use WAV, MP3, M4A, OGG, FLAC, AAC, or WebM."
                .to_string(),
        ));
    }

    // Check file size (max 500MB)
    let metadata = std::fs::metadata(&safe_path)
        .map_err(|e| CommandError::Transcription(format!("Cannot read file: {}", e)))?;
    if metadata.len() > 500 * 1024 * 1024 {
        return Err(CommandError::Transcription(
            "File too large. Maximum size is 500MB.".to_string(),
        ));
    }

    // Read audio file and convert to capped 16kHz mono samples.
    let samples = read_audio_file(&safe_path)
        .map_err(|e| CommandError::Transcription(format!("Failed to read audio file: {}", e)))?;

    // Transcribe
    let settings = db.get_settings().map_err(CommandError::Database)?;
    if settings.selected_model_id.starts_with("cloud:") {
        if let Some((provider, model)) = cloud_transcription::parse_cloud_model_id(&settings.selected_model_id) {
            let wav_bytes = cloud_transcription::encode_samples_to_wav(&samples)
                .map_err(CommandError::Recording)?;
            let text = cloud_transcription::transcribe_with_cloud(&db, provider, model, wav_bytes, &settings.language, false)
                .await
                .map_err(CommandError::Transcription)?;
            return Ok(text);
        }
    }

    let mut transcriber_guard = transcriber.lock().unwrap();
    if let Some(ref mut t) = *transcriber_guard {
        let text = t
            .transcribe(&samples)
            .map_err(CommandError::Transcription)?;
        Ok(text)
    } else {
        Err(CommandError::Transcription("No model loaded".to_string()))
    }
}

// ==================== URL Transcription Commands ====================

fn is_youtube_url(url: &str) -> bool {
    let lower = url.to_lowercase();
    lower.contains("youtube.com")
        || lower.contains("youtu.be")
        || lower.contains("youtube.co")
        || lower.contains("yt.be")
}

fn sanitize_url(url: &str) -> Result<String, String> {
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

async fn download_url_to_temp(
    app: &tauri::AppHandle,
    url: &str,
) -> Result<std::path::PathBuf, String> {
    let client = build_http_client(120)?;
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to download URL: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed with status {}",
            response.status()
        ));
    }

    let temp_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;

    let ext = std::path::Path::new(url)
        .extension()
        .and_then(|e| e.to_str())
        .filter(|e| !e.is_empty())
        .unwrap_or("audio");

    let file_path = temp_dir.join(format!("url_audio_{}.{}", uuid::Uuid::new_v4(), ext));
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read download bytes: {}", e))?
        .to_vec();

    std::fs::write(&file_path, bytes).map_err(|e| e.to_string())?;

    Ok(file_path)
}

fn yt_dlp_binary_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "yt-dlp.exe"
    } else {
        "yt-dlp"
    }
}

async fn ensure_yt_dlp(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let bin_dir = app_data_dir.join("bin");
    std::fs::create_dir_all(&bin_dir).map_err(|e| e.to_string())?;

    let binary_path = bin_dir.join(yt_dlp_binary_name());

    if binary_path.exists() {
        return Ok(binary_path);
    }

    let urls = if cfg!(target_os = "windows") {
        vec![
            "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe",
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos",
        ]
    } else {
        vec![
            "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp",
        ]
    };

    let client = build_http_client(120)?;

    for url in &urls {
        let response = client
            .get(*url)
            .send()
            .await
            .map_err(|e| format!("Failed to download yt-dlp: {}", e))?;

        if response.status().is_success() {
            let bytes = response
                .bytes()
                .await
                .map_err(|e| format!("Failed to read yt-dlp download: {}", e))?
                .to_vec();

            std::fs::write(&binary_path, bytes)
                .map_err(|e| format!("Failed to save yt-dlp: {}", e))?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&binary_path)
                    .map_err(|e| e.to_string())?
                    .permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&binary_path, perms)
                    .map_err(|e| e.to_string())?;
            }

            return Ok(binary_path);
        }
    }

    Err("Failed to download yt-dlp. Make sure your network allows access to GitHub releases.".to_string())
}

fn find_system_ffmpeg() -> Result<Option<std::path::PathBuf>, String> {
    let candidates = if cfg!(target_os = "windows") {
        vec![
            std::path::PathBuf::from("ffmpeg.exe"),
            std::path::PathBuf::from(r"C:\ffmpeg\bin\ffmpeg.exe"),
            std::path::PathBuf::from(r"C:\Program Files\ffmpeg\bin\ffmpeg.exe"),
            std::path::PathBuf::from(r"C:\Program Files (x86)\ffmpeg\bin\ffmpeg.exe"),
        ]
    } else {
        vec![
            std::path::PathBuf::from("ffmpeg"),
            std::path::PathBuf::from("/usr/local/bin/ffmpeg"),
            std::path::PathBuf::from("/usr/bin/ffmpeg"),
        ]
    };

    for candidate in candidates {
        if candidate.exists() {
            return Ok(Some(candidate));
        }
    }

    Ok(None)
}

async fn extract_youtube_audio(
    app: &tauri::AppHandle,
    url: &str,
) -> Result<std::path::PathBuf, String> {
    let temp_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;

    let uuid_str = uuid::Uuid::new_v4().to_string();
    let output_template = temp_dir.join(format!("yt_audio_{}.%(ext)s", uuid_str));
    let output_path_str = output_template
        .to_str()
        .ok_or_else(|| "Invalid output path".to_string())?;

    let yt_dlp = ensure_yt_dlp(app).await.map_err(|e| {
        format!("{} Make sure your network allows access to GitHub releases.", e)
    })?;

    let status = std::process::Command::new(yt_dlp)
        .args([
            "-f",
            "bestaudio[ext=m4a][format_id!=140][format_id!=139]/bestaudio[ext=mp3]/bestaudio[format_id!=140][format_id!=139]",
            "--audio-quality",
            "0",
            "-o",
            output_path_str,
            url,
        ])
        .status()
        .map_err(|e| format!("Failed to run yt-dlp: {}", e))?;

    if !status.success() {
        return Err("yt-dlp failed to extract audio from the YouTube video.".to_string());
    }

    let prefix = format!("yt_audio_{}", uuid_str);
    let mut found: Option<std::path::PathBuf> = None;

    for entry in std::fs::read_dir(&temp_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with(&prefix) {
                found = Some(path);
                break;
            }
        }
    }

    let path = found.ok_or_else(|| "yt-dlp did not produce an output file.".to_string())?;

    if let Some(ext) = path.extension().and_then(|e| e.to_str().map(|s| s.to_lowercase())) {
        if ext == "webm" || ext == "opus" || ext == "ogg" {
            return Err(format!(
                "YouTube delivered an unsupported audio format for this video: {}. \
                 Try another video, or use a cloud transcription provider that supports this format.",
                ext
            ));
        }

        if ext == "m4a" || ext == "mp4" {
            if let Ok(Some(ffmpeg)) = find_system_ffmpeg() {
                let wav_path = temp_dir.join(format!("yt_audio_{}.wav", uuid_str));
                let status = std::process::Command::new(ffmpeg)
                    .args([
                        "-y",
                        "-i",
                        path.to_str().unwrap(),
                        "-ac",
                        "1",
                        "-ar",
                        "16000",
                        "-c:a",
                        "pcm_s16le",
                        wav_path.to_str().unwrap(),
                    ])
                    .status()
                    .map_err(|e| format!("Failed to convert audio with ffmpeg: {}", e))?;

                if status.success() && wav_path.exists() {
                    let _ = std::fs::remove_file(&path);
                    return Ok(wav_path);
                }

                let _ = std::fs::remove_file(&path);
            }
        }
    }

    Ok(path)
}

#[tauri::command]
async fn transcribe_url(
    app: tauri::AppHandle,
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
    transcriber: State<'_, TranscriberState>,
    rate_limiter: State<'_, TranscriptionRateLimiter>,
    url: String,
    enable_speaker_detection: bool,
) -> CommandResult<String> {
    let db = db.0.clone();
    let license_manager = license_manager.0.clone();
    let transcriber = transcriber.0.clone();
    let rate_limiter = rate_limiter.0.clone();

    ensure_app_access_verified(&db, &license_manager).await?;

    if !rate_limiter.check("transcribe_url") {
        return Err(CommandError::Transcription(
            "Rate limit exceeded. Please wait before transcribing another URL.".to_string(),
        ));
    }

    let safe_url = sanitize_url(&url).map_err(CommandError::Transcription)?;
    let settings = db.get_settings().map_err(CommandError::Database)?;

    if let Some((provider, model)) = cloud_transcription::parse_cloud_model_id(&settings.selected_model_id) {
        if provider.eq_ignore_ascii_case("deepgram") {
            let text = cloud_transcription::transcribe_with_cloud_url(
                &db,
                cloud_transcription::CloudTranscriptionUrlOptions {
                    provider: provider.to_string(),
                    model: model.to_string(),
                    url: safe_url.clone(),
                    language: settings.language.clone(),
                    enable_speaker_detection,
                },
            )
            .await
            .map_err(CommandError::Transcription)?;

            return Ok(text);
        }
    }

    let temp_path = if is_youtube_url(&safe_url) {
        extract_youtube_audio(&app, &safe_url).await.map_err(CommandError::Transcription)?
    } else {
        download_url_to_temp(&app, &safe_url).await.map_err(CommandError::Transcription)?
    };

    let samples = read_audio_file(&temp_path).map_err(|e| {
        let _ = std::fs::remove_file(&temp_path);
        CommandError::Transcription(format!(
            "Failed to read audio: {}. This format may require audio conversion. \
             Try switching to Deepgram in Models settings, which supports URL-based transcription.",
            e
        ))
    })?;

    let _ = std::fs::remove_file(&temp_path);

    if settings.selected_model_id.starts_with("cloud:") {
        if let Some((provider, model)) = cloud_transcription::parse_cloud_model_id(&settings.selected_model_id) {
            let wav_bytes = cloud_transcription::encode_samples_to_wav(&samples)
                .map_err(CommandError::Recording)?;
            let text = cloud_transcription::transcribe_with_cloud(&db, provider, model, wav_bytes, &settings.language, enable_speaker_detection)
                .await
                .map_err(CommandError::Transcription)?;
            return Ok(text);
        }
    }

    let mut transcriber_guard = transcriber.lock().unwrap();
    if let Some(ref mut t) = *transcriber_guard {
        let text = t
            .transcribe(&samples)
            .map_err(CommandError::Transcription)?;
        Ok(text)
    } else {
        Err(CommandError::Transcription("No model loaded".to_string()))
    }
}

#[tauri::command]
async fn transcribe_files_batch(
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
    transcriber: State<'_, TranscriberState>,
    rate_limiter: State<'_, TranscriptionRateLimiter>,
    file_paths: Vec<String>,
) -> CommandResult<Vec<String>> {
    let db = db.0.clone();
    let license_manager = license_manager.0.clone();
    let transcriber = transcriber.0.clone();
    let rate_limiter = rate_limiter.0.clone();

    ensure_app_access_verified(&db, &license_manager).await?;

    if file_paths.is_empty() {
        return Ok(Vec::new());
    }

    if !rate_limiter.check("transcribe_files_batch") {
        return Err(CommandError::Transcription(
            "Rate limit exceeded. Please wait before transcribing again.".to_string(),
        ));
    }

    let settings = db.get_settings().map_err(CommandError::Database)?;
    let mut results = Vec::with_capacity(file_paths.len());

    for file_path in file_paths {
        let safe_path =
            canonicalize_existing_file_path(&file_path).map_err(CommandError::Transcription)?;

        if !path_has_extension(&safe_path, AUDIO_FILE_EXTENSIONS) {
            results.push(format!(
                "Skipped {}: unsupported format",
                safe_path.display()
            ));
            continue;
        }

        let metadata = std::fs::metadata(&safe_path)
            .map_err(|e| CommandError::Transcription(format!("Cannot read file {}: {}", safe_path.display(), e)))?;
        if metadata.len() > 500 * 1024 * 1024 {
            results.push(format!(
                "Skipped {}: file exceeds 500MB limit",
                safe_path.display()
            ));
            continue;
        }

        let samples = match read_audio_file(&safe_path) {
            Ok(s) => s,
            Err(e) => {
                results.push(format!("Failed to read {}: {}", safe_path.display(), e));
                continue;
            }
        };

        let text = if settings.selected_model_id.starts_with("cloud:") {
            if let Some((provider, model)) = cloud_transcription::parse_cloud_model_id(&settings.selected_model_id) {
                let wav_bytes = match cloud_transcription::encode_samples_to_wav(&samples) {
                    Ok(b) => b,
                    Err(e) => {
                        results.push(format!("Failed to encode {}: {}", safe_path.display(), e));
                        continue;
                    }
                };
                match cloud_transcription::transcribe_with_cloud(&db, provider, model, wav_bytes, &settings.language, false).await {
                    Ok(t) => t,
                    Err(e) => {
                        results.push(format!("Transcription failed for {}: {}", safe_path.display(), e));
                        continue;
                    }
                }
            } else {
                results.push(format!("Invalid cloud model for {}", safe_path.display()));
                continue;
            }
        } else {
            let mut transcriber_guard = transcriber.lock().unwrap();
            match transcriber_guard.as_mut() {
                Some(t) => match t.transcribe(&samples) {
                    Ok(t) => t,
                    Err(e) => {
                        results.push(format!("Transcription failed for {}: {}", safe_path.display(), e));
                        continue;
                    }
                },
                None => {
                    results.push(format!("No model loaded for {}", safe_path.display()));
                    continue;
                }
            }
        };

        results.push(text);
    }

    Ok(results)
}

fn read_audio_file(file_path: &std::path::Path) -> Result<Vec<f32>, String> {
    use std::fs::File;
    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    let file = File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?;

    // Create a media source stream
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    // Create a hint to help the format registry guess what format reader is appropriate
    let mut hint = Hint::new();
    if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    // Probe the media source
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| format!("Failed to probe audio format: {}", e))?;

    let mut format = probed.format;

    // Find the first audio track
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or_else(|| "No audio track found".to_string())?;

    let track_id = track.id;
    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
    let channels = track
        .codec_params
        .channels
        .map(|c| c.count())
        .unwrap_or(2)
        .max(1);

    // Create a decoder
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| format!("Failed to create decoder: {}", e))?;

    let mut samples =
        Vec::with_capacity((AUDIO_TARGET_SAMPLE_RATE as usize * 60).min(MAX_FILE_AUDIO_SAMPLES));

    // Decode all packets
    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break
            }
            Err(symphonia::core::errors::Error::ResetRequired) => {
                decoder.reset();
                continue;
            }
            Err(e) => return Err(format!("Failed to read packet: {}", e)),
        };

        // Skip packets from other tracks
        if packet.track_id() != track_id {
            continue;
        }

        // Decode the packet
        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
            Err(e) => return Err(format!("Failed to decode: {}", e)),
        };

        // Convert the current packet to f32 samples, then immediately fold it
        // into the capped mono 16kHz buffer. This avoids retaining the full
        // decoded source stream in memory.
        let spec = *decoded.spec();
        let duration = decoded.capacity() as u64;
        let mut sample_buf = SampleBuffer::<f32>::new(duration, spec);
        sample_buf.copy_interleaved_ref(decoded);

        let mono = interleaved_to_mono(sample_buf.samples(), channels);

        let normalized = if sample_rate != AUDIO_TARGET_SAMPLE_RATE {
            resample_audio(&mono, sample_rate, AUDIO_TARGET_SAMPLE_RATE)
        } else {
            mono
        };

        append_audio_samples_with_limit(&mut samples, &normalized, MAX_FILE_AUDIO_SAMPLES)?;
    }

    Ok(samples)
}

fn append_audio_samples_with_limit(
    target: &mut Vec<f32>,
    source: &[f32],
    max_samples: usize,
) -> Result<(), String> {
    let remaining = max_samples.saturating_sub(target.len());
    if source.len() > remaining {
        return Err(format!(
            "Audio is too long. Maximum supported duration is {} minutes.",
            MAX_FILE_TRANSCRIPTION_SECONDS / 60
        ));
    }

    target.extend_from_slice(source);
    Ok(())
}

fn interleaved_to_mono(samples: &[f32], channels: usize) -> Vec<f32> {
    if channels > 1 {
        samples
            .chunks(channels)
            .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
            .collect()
    } else {
        samples.to_vec()
    }
}

fn resample_audio(samples: &[f32], source_rate: u32, target_rate: u32) -> Vec<f32> {
    let ratio = source_rate as f64 / target_rate as f64;
    let output_len = (samples.len() as f64 / ratio) as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_idx = i as f64 * ratio;
        let idx = src_idx as usize;
        let frac = src_idx - idx as f64;

        let sample = if idx + 1 < samples.len() {
            samples[idx] * (1.0 - frac as f32) + samples[idx + 1] * frac as f32
        } else if idx < samples.len() {
            samples[idx]
        } else {
            0.0
        };

        output.push(sample);
    }

    output
}

// ==================== Download Commands ====================

#[tauri::command]
async fn download_model(
    app: tauri::AppHandle,
    downloader: State<'_, DownloaderState>,
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
    model_id: String,
) -> CommandResult<String> {
    let db = db.0.clone();
    let license_manager = license_manager.0.clone();
    let downloader = downloader.0.clone();

    ensure_app_access_verified(&db, &license_manager).await?;

    let app_clone = app.clone();
    let model_id_clone = model_id.clone();

    let model_path = downloader
        .download_model(&model_id, move |progress: DownloadProgress| {
            // Emit progress event to frontend
            let _ = app_clone.emit("download-progress", progress);
        })
        .await
        .map_err(CommandError::Download)?;

    // Update database
    let path_str = model_path.to_str().unwrap().to_string();
    db.set_model_downloaded(&model_id_clone, true, Some(&path_str))
        .map_err(CommandError::Database)?;

    Ok(path_str)
}

#[tauri::command]
async fn delete_model(
    downloader: State<'_, DownloaderState>,
    db: State<'_, DbState>,
    model_id: String,
) -> CommandResult<()> {
    downloader
        .0
        .delete_model(&model_id)
        .await
        .map_err(CommandError::Download)?;
    db.0.set_model_downloaded(&model_id, false, None)
        .map_err(CommandError::Database)?;
    Ok(())
}

#[tauri::command]
fn cancel_model_download(downloader: State<'_, DownloaderState>, model_id: String) -> bool {
    downloader.0.cancel_download(&model_id)
}

#[tauri::command]
fn is_model_downloaded(
    _db: State<DbState>,
    downloader: State<DownloaderState>,
    model_id: String,
) -> CommandResult<bool> {
    // Validate model_id against allowed values
    const VALID_MODEL_IDS: &[&str] = &[
        "tiny",
        "base",
        "small",
        "medium",
        "large",
        "large-v3",
        "large-v3-turbo",
        "tiny.en",
        "base.en",
        "small.en",
        "medium.en",
        "distil-small.en",
        "parakeet-v2",
        "parakeet-v3",
        "qwen3-asr-0.6b",
    ];
    if !VALID_MODEL_IDS.contains(&model_id.as_str()) {
        return Err(CommandError::Database(
            rusqlite::Error::InvalidParameterName("Invalid model ID".to_string()),
        ));
    }
    Ok(downloader.0.is_model_downloaded(&model_id))
}

#[tauri::command]
fn get_downloaded_models(downloader: State<DownloaderState>) -> Vec<String> {
    downloader.0.get_downloaded_models()
}

#[tauri::command]
fn get_model_path(downloader: State<DownloaderState>, model_id: String) -> CommandResult<String> {
    // Validate model_id against allowed values
    const VALID_MODEL_IDS: &[&str] = &[
        "tiny",
        "base",
        "small",
        "medium",
        "large",
        "large-v3",
        "large-v3-turbo",
        "tiny.en",
        "base.en",
        "small.en",
        "medium.en",
        "distil-small.en",
        "parakeet-v2",
        "parakeet-v3",
        "qwen3-asr-0.6b",
    ];
    if !VALID_MODEL_IDS.contains(&model_id.as_str()) {
        return Err(CommandError::Database(
            rusqlite::Error::InvalidParameterName("Invalid model ID".to_string()),
        ));
    }
    Ok(downloader
        .0
        .get_model_path(&model_id)
        .to_string_lossy()
        .to_string())
}

// ==================== Post-Processing Commands ====================

#[tauri::command]
fn post_process_text(db: State<DbState>, text: String) -> CommandResult<String> {
    let sanitized = sanitize_text(&text, 100_000).map_err(CommandError::PostProcessing)?;

    if sanitized.is_empty() {
        return Ok(String::new());
    }

    let processor = build_processor(&db)?;
    let processed = processor.process(&sanitized);

    Ok(processed)
}

#[tauri::command]
fn extract_voice_commands(db: State<DbState>, text: String) -> CommandResult<String> {
    let sanitized = sanitize_text(&text, 100_000).map_err(CommandError::PostProcessing)?;

    if sanitized.is_empty() {
        return Ok(String::new());
    }

    let processor = build_processor(&db)?;
    let processed = processor.extract_voice_commands(&sanitized);

    Ok(processed)
}

/// Build a PostProcessor seeded with the user's custom vocabulary so
/// post-processing and voice-command extraction both honour domain terms.
fn build_processor(db: &State<DbState>) -> CommandResult<PostProcessor> {
    let settings = db.0.get_settings().map_err(CommandError::Database)?;
    let entries: Vec<post_process::VocabularyEntry> = settings
        .custom_vocabulary
        .into_iter()
        .map(|e: VocabularyEntry| post_process::VocabularyEntry::new(e.spoken, e.written))
        .collect();
    Ok(PostProcessor::with_vocabulary(&entries))
}

// ==================== Text Injection Commands ====================

#[tauri::command]
fn inject_text(injector: State<TextInjectorState>, text: String) -> CommandResult<()> {
    // Sanitize input - limit text length and remove control characters
    let sanitized = sanitize_text(&text, 100_000).map_err(CommandError::TextInjection)?;

    if sanitized.is_empty() {
        return Err(CommandError::TextInjection("No text to inject".to_string()));
    }

    // Reuse injector instance for better performance (avoids recreating each time)
    let mut injector_guard = injector.0.lock().unwrap();
    injector_guard
        .inject_text(&sanitized)
        .map_err(CommandError::TextInjection)
}

#[tauri::command]
fn execute_keyboard_shortcut(
    injector: State<TextInjectorState>,
    shortcut: String,
) -> CommandResult<()> {
    // Validate shortcut against allowed values
    let allowed_shortcuts = [
        "undo",
        "redo",
        "copy",
        "cut",
        "paste",
        "select_all",
        "backspace_word",
        "backspace",
        "delete",
        "delete_word",
        "delete_line",
        "enter",
        "tab",
        "escape",
        "page_up",
        "page_down",
        "left",
        "right",
        "up",
        "down",
        "home",
        "end",
        "word_left",
        "word_right",
        "select_left",
        "select_right",
        "select_up",
        "select_down",
        "select_word_left",
        "select_word_right",
        "select_to_start",
        "select_to_end",
    ];
    if !allowed_shortcuts.contains(&shortcut.as_str()) {
        return Err(CommandError::TextInjection(format!(
            "Invalid shortcut: {}",
            shortcut
        )));
    }

    // Reuse injector instance for better performance
    let mut injector_guard = injector.0.lock().unwrap();
    injector_guard
        .execute_shortcut(&shortcut)
        .map_err(CommandError::TextInjection)
}

// ==================== Transcription History Commands ====================

#[tauri::command]
fn add_transcription(
    db: State<DbState>,
    text: String,
    model_id: String,
    language: String,
    duration_ms: i64,
) -> CommandResult<i64> {
    // Sanitize and validate text input
    let sanitized_text = sanitize_text(&text, 1_000_000)
        .map_err(|e| CommandError::Database(rusqlite::Error::InvalidParameterName(e)))?;

    if sanitized_text.is_empty() {
        return Err(CommandError::Database(
            rusqlite::Error::InvalidParameterName("Text cannot be empty".to_string()),
        ));
    }

    // Validate model_id against allowed values
    const VALID_MODEL_IDS: &[&str] = &[
        // Whisper models
        "tiny",
        "base",
        "small",
        "medium",
        "large",
        "large-v3",
        "large-v3-turbo",
        "tiny.en",
        "base.en",
        "small.en",
        "medium.en",
        "distil-small.en",
        "parakeet-v2",
        "parakeet-v3",
        "qwen3-asr-0.6b",
    ];
    
    // Allow cloud model IDs that start with "cloud:"
    let is_valid = VALID_MODEL_IDS.contains(&model_id.as_str()) || model_id.starts_with("cloud:");
    
    if !is_valid {
        return Err(CommandError::Database(
            rusqlite::Error::InvalidParameterName("Invalid model ID".to_string()),
        ));
    }

    // Validate language code shape. The UI restricts this to the selected model's languages.
    if !is_valid_language_code(&language) {
        return Err(CommandError::Database(
            rusqlite::Error::InvalidParameterName("Invalid language".to_string()),
        ));
    }

    // Validate duration range (0 to 1 hour in milliseconds)
    if !(0..=3_600_000).contains(&duration_ms) {
        return Err(CommandError::Database(
            rusqlite::Error::InvalidParameterName("Invalid duration".to_string()),
        ));
    }

    db.0.add_transcription(&sanitized_text, &model_id, &language, duration_ms)
        .map_err(Into::into)
}

#[tauri::command]
fn get_transcription_history(
    db: State<DbState>,
    limit: Option<i32>,
    offset: Option<i32>,
    search: Option<String>,
) -> CommandResult<Vec<TranscriptionHistory>> {
    // Validate and cap limit
    let safe_limit = limit.unwrap_or(50).clamp(1, 1000);
    let safe_offset = offset.unwrap_or(0).max(0);
    let safe_search = search
        .as_deref()
        .map(|value| sanitize_text(value.trim(), 500))
        .transpose()
        .map_err(|e| {
            CommandError::Database(rusqlite::Error::InvalidParameterName(format!(
                "Invalid search: {}",
                e
            )))
        })?;
    db.0.get_transcription_history(safe_limit, safe_offset, safe_search.as_deref())
        .map_err(Into::into)
}

#[tauri::command]
fn get_transcription_history_count(
    db: State<DbState>,
    search: Option<String>,
) -> CommandResult<i64> {
    let safe_search = search
        .as_deref()
        .map(|value| sanitize_text(value.trim(), 500))
        .transpose()
        .map_err(|e| {
            CommandError::Database(rusqlite::Error::InvalidParameterName(format!(
                "Invalid search: {}",
                e
            )))
        })?;
    db.0.get_transcription_history_count(safe_search.as_deref())
        .map_err(Into::into)
}

#[tauri::command]
fn clear_transcription_history(db: State<DbState>) -> CommandResult<()> {
    db.0.clear_transcription_history().map_err(Into::into)
}

#[tauri::command]
fn delete_transcription(db: State<DbState>, id: i64) -> CommandResult<()> {
    db.0.delete_transcription(id).map_err(Into::into)
}

// ==================== License Commands ====================

// License response for frontend
#[derive(Debug, serde::Serialize)]
struct LicenseResponse {
    license_key: Option<String>,
    display_key: Option<String>,
    activation_id: Option<String>,
    status: String,
    customer_email: Option<String>,
    customer_name: Option<String>,
    benefit_id: Option<String>,
    expires_at: Option<String>,
    is_activated: bool,
    last_validated_at: Option<String>,
    trial_started_at: Option<String>,
    trial_days_remaining: Option<i64>,
    device_id: String,
    device_label: String,
    limit_activations: Option<i32>,
    usage: i32,
    validations: i32,
}

impl From<LicenseInfo> for LicenseResponse {
    fn from(info: LicenseInfo) -> Self {
        Self {
            license_key: Some(security::mask_license_key(&info.license_key)),
            display_key: Some(info.display_key),
            activation_id: info.activation_id,
            status: license_status_to_response(&info.status),
            customer_email: info.customer_email,
            customer_name: info.customer_name,
            benefit_id: info.benefit_id,
            expires_at: info.expires_at,
            is_activated: info.status.allows_usage(),
            last_validated_at: info.last_validated_at,
            trial_started_at: None,
            trial_days_remaining: None,
            device_id: info.device_id,
            device_label: info.device_label,
            limit_activations: info.limit_activations,
            usage: info.usage,
            validations: info.validations,
        }
    }
}

impl From<LicenseData> for LicenseResponse {
    fn from(data: LicenseData) -> Self {
        let trial_days_remaining = if let Some(ref trial_started) = data.trial_started_at {
            if let Ok(start_date) = chrono::DateTime::parse_from_rfc3339(trial_started) {
                let now = chrono::Utc::now();
                let days_since_start = (now - start_date.with_timezone(&chrono::Utc)).num_days();
                Some((7 - days_since_start).max(0))
            } else {
                None
            }
        } else {
            None
        };

        Self {
            license_key: data.license_key.as_deref().map(security::mask_license_key),
            display_key: None,
            activation_id: data.activation_id,
            status: data.status,
            customer_email: data.customer_email,
            customer_name: data.customer_name,
            benefit_id: None,
            expires_at: data.expires_at,
            is_activated: data.is_activated,
            last_validated_at: data.last_validated_at,
            trial_started_at: data.trial_started_at,
            trial_days_remaining,
            device_id: get_device_id(),
            device_label: get_device_label(),
            limit_activations: None,
            usage: data.usage,
            validations: data.validations,
        }
    }
}

#[tauri::command]
async fn get_license(
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
) -> CommandResult<LicenseResponse> {
    // First try to get from secure cache
    if let Some(info) = license_manager.0.get_cached_info() {
        return Ok(LicenseResponse::from(info));
    }

    // Fall back to database
    let license = db.0.get_license().map_err(CommandError::Database)?;

    // If the DB row claims the license is active, re-validate online before
    // returning it. A revoked/disabled license must not be displayed as
    // "active" just because the DB row hasn't been updated yet.
    if db_license_allows_usage(&license) {
        match license_manager.0.validate().await {
            Ok(info) => return Ok(LicenseResponse::from(info)),
            Err(error) => {
                if license_manager.0.is_authoritative_validate_error(&error) {
                    warn!("get_license: license authoritatively rejected: {}", error);
                    let _ = clear_cache();
                    // Return the DB row as-is; the UI will show the
                    // rejected status rather than a stale "active".
                    return Ok(LicenseResponse::from(license));
                }
                // Transient failure: return the DB row (offline grace).
            }
        }
    }

    Ok(LicenseResponse::from(license))
}

#[tauri::command]
async fn activate_license(
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
    license_key: String,
) -> CommandResult<LicenseResponse> {
    info!("Activating license key...");

    // Activate using the new LicenseManager
    let license_info = license_manager
        .0
        .activate(&license_key)
        .await
        .map_err(CommandError::License)?;

    if !license_info.status.allows_usage() {
        let _ = clear_cache();
        return Err(CommandError::License(format!(
            "License activation did not grant access: {}",
            license_status_to_response(&license_info.status)
        )));
    }

    // Also save to database as backup
    let is_activated = license_info.status.allows_usage();
    let license_data = LicenseData {
        license_key: Some(license_key),
        activation_id: license_info.activation_id.clone(),
        status: license_status_to_response(&license_info.status),
        customer_email: license_info.customer_email.clone(),
        customer_name: license_info.customer_name.clone(),
        expires_at: license_info.expires_at.clone(),
        is_activated,
        last_validated_at: Some(chrono::Utc::now().to_rfc3339()),
        trial_started_at: None,
        trial_integrity_hash: None,
        usage: license_info.usage,
        validations: license_info.validations,
    };

    db.0.save_license(&license_data)
        .map_err(CommandError::Database)?;

    info!("License activated successfully!");
    Ok(LicenseResponse::from(license_info))
}

#[tauri::command]
async fn validate_license(
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
) -> CommandResult<LicenseResponse> {
    info!("Validating license...");

    let license_info = match license_manager.0.validate().await {
        Ok(info) => info,
        Err(error) => {
            // Distinguish authoritative rejections (revoked/disabled/invalid)
            // from transient network failures. Authoritative rejections must
            // never grant access from a stale DB row.
            if license_manager.0.is_authoritative_validate_error(&error) {
                warn!("License authoritatively rejected: {}", error);
                let _ = clear_cache();
                return Err(CommandError::License(error));
            }

            // Transient failure (network/5xx): fall back to the stored DB
            // license for offline grace. Do NOT make another online call.
            let stored_license = db.0.get_license().map_err(CommandError::Database)?;
            if !db_license_allows_usage(&stored_license) {
                return Err(CommandError::License(error));
            }

            // Build a LicenseInfo from the stored DB row so the UI reflects
            // the offline-grace state.
            return Ok(LicenseResponse::from(stored_license));
        }
    };

    // Update database
    let license_data = LicenseData {
        license_key: Some(license_info.license_key.clone()),
        activation_id: license_info.activation_id.clone(),
        status: license_status_to_response(&license_info.status),
        customer_email: license_info.customer_email.clone(),
        customer_name: license_info.customer_name.clone(),
        expires_at: license_info.expires_at.clone(),
        is_activated: license_info.status.allows_usage(),
        last_validated_at: license_info.last_validated_at.clone(),
        trial_started_at: None,
        trial_integrity_hash: None,
        usage: license_info.usage,
        validations: license_info.validations,
    };

    let _ = db.0.save_license(&license_data);

    info!("License validated: {:?}", license_info.status);
    Ok(LicenseResponse::from(license_info))
}

#[tauri::command]
async fn deactivate_license(
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
) -> CommandResult<()> {
    info!("Deactivating license...");

    let deactivate_result = license_manager.0.deactivate().await;
    if let Err(error) = deactivate_result {
        let stored_license = db.0.get_license().map_err(CommandError::Database)?;
        let Some(license_key) = stored_license.license_key.as_deref() else {
            return Err(CommandError::License(error));
        };
        let Some(activation_id) = stored_license.activation_id.as_deref() else {
            return Err(CommandError::License(error));
        };

        if !db_license_allows_usage(&stored_license) {
            return Err(CommandError::License(error));
        }

        license_manager
            .0
            .deactivate_activation(license_key, activation_id)
            .await
            .map_err(CommandError::License)?;
    }

    // Clear database
    db.0.clear_license().map_err(CommandError::Database)?;

    info!("License deactivated successfully");
    Ok(())
}

#[tauri::command]
fn clear_stored_license(db: State<DbState>) -> CommandResult<()> {
    let _ = clear_cache();
    db.0.clear_license().map_err(Into::into)
}

#[tauri::command]
#[allow(unused_variables)]
async fn is_license_valid(
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
) -> CommandResult<bool> {
    let db = db.0.clone();
    let license_manager = license_manager.0.clone();

    match verified_license_check(&license_manager).await {
        VerifiedLicenseResult::Granted => return Ok(true),
        VerifiedLicenseResult::Rejected => {
            // Authoritative rejection: a revoked/disabled license must not
            // grant access from a stale DB row.
            return Ok(false);
        }
        VerifiedLicenseResult::OfflineFallback => {
            // Transient failure: fall back to the DB row / trial.
        }
    }

    if let Ok(license) = db.get_license() {
        if db_license_allows_usage(&license) {
            return Ok(true);
        }
    }

    Ok(has_active_trial(&db))
}

#[tauri::command]
fn start_trial(db: State<DbState>) -> CommandResult<LicenseResponse> {
    let mut license = db.0.get_license().map_err(CommandError::Database)?;

    // Check if already has active license
    if license.is_activated && license.status == "active" {
        return Err(CommandError::License(
            "Already have an active license".to_string(),
        ));
    }

    // Check if trial already started
    if license.trial_started_at.is_some() {
        // Check if trial is still valid
        if let Some(ref trial_started) = license.trial_started_at {
            let expected_hash = calculate_trial_integrity_hash(trial_started);
            if license.trial_integrity_hash.as_deref() != Some(expected_hash.as_str()) {
                warn!("Trial integrity check failed during start_trial");
                license.status = "trial_expired".to_string();
                db.0.save_license(&license)
                    .map_err(CommandError::Database)?;
                // Clear any cached license so the UI reflects the expired
                // state rather than a stale granted cache.
                let _ = clear_cache();
                return Err(CommandError::License(
                    "Trial state is invalid. Please activate a license.".to_string(),
                ));
            }

            if let Ok(start_date) = chrono::DateTime::parse_from_rfc3339(trial_started) {
                let now = chrono::Utc::now();
                let days_since_start = (now - start_date.with_timezone(&chrono::Utc)).num_days();
                if days_since_start < 0 {
                    warn!("Invalid future trial start detected");
                    license.status = "trial_expired".to_string();
                    db.0.save_license(&license)
                        .map_err(CommandError::Database)?;
                    let _ = clear_cache();
                    return Err(CommandError::License(
                        "Trial state is invalid. Please activate a license.".to_string(),
                    ));
                }

                if days_since_start >= 7 {
                    license.status = "trial_expired".to_string();
                    db.0.save_license(&license)
                        .map_err(CommandError::Database)?;
                    let _ = clear_cache();
                    return Err(CommandError::License(
                        "Trial has expired. Please purchase a license.".to_string(),
                    ));
                }
            }
        }
        // Trial already active
        return Ok(LicenseResponse::from(license));
    }

    // Start new trial. Clear any cached license so the UI reflects the
    // trial state rather than a stale granted cache.
    let _ = clear_cache();

    let trial_started_at = chrono::Utc::now().to_rfc3339();
    license.status = "trial".to_string();
    license.trial_started_at = Some(trial_started_at.clone());
    license.trial_integrity_hash = Some(calculate_trial_integrity_hash(&trial_started_at));
    license.is_activated = false;

    db.0.save_license(&license)
        .map_err(CommandError::Database)?;

    info!("Trial started");
    Ok(LicenseResponse::from(license))
}

/// Get device information for license display
#[tauri::command]
fn get_device_info() -> serde_json::Value {
    serde_json::json!({
        "device_id": get_device_id(),
        "device_label": get_device_label(),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
    })
}

#[tauri::command]
#[allow(unused_variables)]
async fn get_trial_status(
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
) -> CommandResult<serde_json::Value> {
    let db = db.0.clone();
    let license_manager = license_manager.0.clone();

    // Check for active license first with server validation when possible.
    match verified_license_check(&license_manager).await {
        VerifiedLicenseResult::Granted => {
            return Ok(serde_json::json!({
                "isInTrial": false,
                "daysRemaining": 0,
                "trialExpired": false,
                "hasLicense": true
            }));
        }
        VerifiedLicenseResult::Rejected => {
            // Authoritative rejection: do not fall back to a stale DB row.
            // Fall through to trial checks below.
        }
        VerifiedLicenseResult::OfflineFallback => {
            // Transient failure: fall back to the DB row below.
        }
    }

    let license = db.get_license().map_err(CommandError::Database)?;
    if db_license_allows_usage(&license) {
        return Ok(serde_json::json!({
            "isInTrial": false,
            "daysRemaining": 0,
            "trialExpired": false,
            "hasLicense": true
        }));
    }

    // Check trial status
    if let Some(trial_started) = &license.trial_started_at {
        let expected_hash = calculate_trial_integrity_hash(trial_started);
        if license.trial_integrity_hash.as_deref() != Some(expected_hash.as_str()) {
            warn!("Trial integrity check failed in get_trial_status");
            // Persist the expired state so the UI consistently reports it.
            let mut expired = license.clone();
            expired.status = "trial_expired".to_string();
            let _ = db.save_license(&expired);
            return Ok(serde_json::json!({
                "isInTrial": false,
                "daysRemaining": 0,
                "trialExpired": true,
                "hasLicense": false
            }));
        }

        if let Ok(start_date) = chrono::DateTime::parse_from_rfc3339(trial_started) {
            let now = chrono::Utc::now();
            let days_since_start = (now - start_date.with_timezone(&chrono::Utc)).num_days();
            let days_remaining = if days_since_start < 0 {
                0
            } else {
                (7 - days_since_start).max(0)
            };

            if days_remaining <= 0 {
                // Persist the expired state so the UI consistently reports it.
                let mut expired = license.clone();
                expired.status = "trial_expired".to_string();
                let _ = db.save_license(&expired);
            }

            return Ok(serde_json::json!({
                "isInTrial": days_remaining > 0,
                "daysRemaining": days_remaining,
                "trialExpired": days_remaining <= 0,
                "hasLicense": false
            }));
        }
    }

    // No trial started
    Ok(serde_json::json!({
        "isInTrial": false,
        "daysRemaining": 0,
        "trialExpired": false,
        "hasLicense": false
    }))
}

#[tauri::command]
#[allow(unused_variables)]
async fn can_use_app(
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
) -> CommandResult<serde_json::Value> {
    let db = db.0.clone();
    let license_manager = license_manager.0.clone();

    // Prefer server validation. LicenseManager falls back to offline grace
    // only for non-authoritative network failures.
    match verified_license_check(&license_manager).await {
        VerifiedLicenseResult::Granted => {
            return Ok(serde_json::json!({
                "canUse": true,
                "reason": "licensed",
                "daysRemaining": null
            }));
        }
        VerifiedLicenseResult::Rejected => {
            // Authoritative rejection: do not fall back to a stale DB row.
            // Fall through to trial checks below.
        }
        VerifiedLicenseResult::OfflineFallback => {
            // Transient failure: fall back to the DB row below.
        }
    }

    let license = db.get_license().map_err(CommandError::Database)?;
    if db_license_allows_usage(&license) {
        return Ok(serde_json::json!({
            "canUse": true,
            "reason": "licensed",
            "daysRemaining": null
        }));
    }

    // Check trial status
    if let Some(trial_started) = &license.trial_started_at {
        let expected_hash = calculate_trial_integrity_hash(trial_started);
        if license.trial_integrity_hash.as_deref() != Some(expected_hash.as_str()) {
            warn!("Trial integrity check failed in can_use_app");
            // Persist the expired state so subsequent calls don't re-evaluate
            // a corrupted trial.
            let mut expired = license.clone();
            expired.status = "trial_expired".to_string();
            let _ = db.save_license(&expired);
            return Ok(serde_json::json!({
                "canUse": false,
                "reason": "trial_expired",
                "daysRemaining": 0
            }));
        }

        if let Ok(start_date) = chrono::DateTime::parse_from_rfc3339(trial_started) {
            let now = chrono::Utc::now();
            let days_since_start = (now - start_date.with_timezone(&chrono::Utc)).num_days();
            let days_remaining = if days_since_start < 0 {
                0
            } else {
                (7 - days_since_start).max(0)
            };

            if days_remaining > 0 {
                return Ok(serde_json::json!({
                    "canUse": true,
                    "reason": "trial",
                    "daysRemaining": days_remaining
                }));
            } else {
                // Persist the expired state so the UI consistently shows
                // "Trial Period Ended" rather than recomputing each call.
                let mut expired = license.clone();
                expired.status = "trial_expired".to_string();
                let _ = db.save_license(&expired);
                return Ok(serde_json::json!({
                    "canUse": false,
                    "reason": "trial_expired",
                    "daysRemaining": 0
                }));
            }
        }
    }

    // No license and no trial
    Ok(serde_json::json!({
        "canUse": false,
        "reason": "no_license",
        "daysRemaining": null
    }))
}

// ==================== Utility Commands ====================

#[tauri::command]
fn get_app_data_dir(app: tauri::AppHandle) -> CommandResult<String> {
    let path = app.path().app_data_dir().map_err(|e: tauri::Error| {
        std::io::Error::new(std::io::ErrorKind::NotFound, e.to_string())
    })?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn get_models_dir(app: tauri::AppHandle) -> CommandResult<String> {
    let path = app.path().app_data_dir().map_err(|e: tauri::Error| {
        std::io::Error::new(std::io::ErrorKind::NotFound, e.to_string())
    })?;
    Ok(path.join("models").to_string_lossy().to_string())
}

// ==================== Hotkey Commands ====================

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct HotkeyRegistration {
    hotkey: String,
    label: String,
}

#[derive(Debug, serde::Serialize, Clone)]
struct HotkeyEventPayload {
    label: String,
}

#[tauri::command]
fn register_hotkey(app: tauri::AppHandle, registrations: Vec<HotkeyRegistration>) -> CommandResult<()> {
    if registrations.is_empty() {
        return Ok(());
    }

    println!("Registering {} hotkeys", registrations.len());

    if let Err(e) = app.global_shortcut().unregister_all() {
        println!("Warning: Failed to unregister existing shortcuts: {}", e);
    }

    for reg in registrations {
        let shortcut = parse_hotkey(&reg.hotkey)
            .map_err(|e| CommandError::Recording(format!("Invalid hotkey: {}", e)))?;
        let label = reg.label.clone();

        println!("Registering hotkey: {} -> {:?} (label: {})", reg.hotkey, shortcut, label);

        let result = app.global_shortcut().on_shortcut(shortcut, move |app, _shortcut, event| {
            println!("Shortcut event: {:?} for label: {}", event.state(), label);
            match event.state() {
                ShortcutState::Pressed => {
                    let _ = app.emit("hotkey-pressed", HotkeyEventPayload { label: label.clone() });
                }
                ShortcutState::Released => {
                    let _ = app.emit("hotkey-released", HotkeyEventPayload { label: label.clone() });
                }
            }
        });

        if let Err(e) = result {
            println!("Failed to register hotkey {}: {}", reg.hotkey, e);
            return Err(CommandError::Recording(format!(
                "Failed to register hotkey {}: {}",
                reg.hotkey, e
            )));
        }
    }

    println!("All hotkeys registered successfully");
    Ok(())
}

#[tauri::command]
fn unregister_hotkeys(app: tauri::AppHandle) -> CommandResult<()> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| CommandError::Recording(format!("Failed to unregister hotkeys: {}", e)))?;
    Ok(())
}

fn parse_hotkey(hotkey: &str) -> Result<Shortcut, String> {
    let parts: Vec<&str> = hotkey.split('+').map(|s| s.trim()).collect();

    if parts.is_empty() {
        return Err("Empty hotkey".to_string());
    }

    let mut modifiers = Modifiers::empty();
    let mut key_code: Option<Code> = None;

    for part in parts {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "alt" => modifiers |= Modifiers::ALT,
            "shift" => modifiers |= Modifiers::SHIFT,
            "super" | "meta" | "win" | "cmd" => modifiers |= Modifiers::SUPER,
            "space" => key_code = Some(Code::Space),
            "enter" | "return" => key_code = Some(Code::Enter),
            "tab" => key_code = Some(Code::Tab),
            "escape" | "esc" => key_code = Some(Code::Escape),
            "backspace" => key_code = Some(Code::Backspace),
            "delete" => key_code = Some(Code::Delete),
            "f1" => key_code = Some(Code::F1),
            "f2" => key_code = Some(Code::F2),
            "f3" => key_code = Some(Code::F3),
            "f4" => key_code = Some(Code::F4),
            "f5" => key_code = Some(Code::F5),
            "f6" => key_code = Some(Code::F6),
            "f7" => key_code = Some(Code::F7),
            "f8" => key_code = Some(Code::F8),
            "f9" => key_code = Some(Code::F9),
            "f10" => key_code = Some(Code::F10),
            "f11" => key_code = Some(Code::F11),
            "f12" => key_code = Some(Code::F12),
            s if s.len() == 1 => {
                let c = s.chars().next().unwrap().to_ascii_uppercase();
                key_code = match c {
                    'A' => Some(Code::KeyA),
                    'B' => Some(Code::KeyB),
                    'C' => Some(Code::KeyC),
                    'D' => Some(Code::KeyD),
                    'E' => Some(Code::KeyE),
                    'F' => Some(Code::KeyF),
                    'G' => Some(Code::KeyG),
                    'H' => Some(Code::KeyH),
                    'I' => Some(Code::KeyI),
                    'J' => Some(Code::KeyJ),
                    'K' => Some(Code::KeyK),
                    'L' => Some(Code::KeyL),
                    'M' => Some(Code::KeyM),
                    'N' => Some(Code::KeyN),
                    'O' => Some(Code::KeyO),
                    'P' => Some(Code::KeyP),
                    'Q' => Some(Code::KeyQ),
                    'R' => Some(Code::KeyR),
                    'S' => Some(Code::KeyS),
                    'T' => Some(Code::KeyT),
                    'U' => Some(Code::KeyU),
                    'V' => Some(Code::KeyV),
                    'W' => Some(Code::KeyW),
                    'X' => Some(Code::KeyX),
                    'Y' => Some(Code::KeyY),
                    'Z' => Some(Code::KeyZ),
                    '0' => Some(Code::Digit0),
                    '1' => Some(Code::Digit1),
                    '2' => Some(Code::Digit2),
                    '3' => Some(Code::Digit3),
                    '4' => Some(Code::Digit4),
                    '5' => Some(Code::Digit5),
                    '6' => Some(Code::Digit6),
                    '7' => Some(Code::Digit7),
                    '8' => Some(Code::Digit8),
                    '9' => Some(Code::Digit9),
                    _ => return Err(format!("Unknown key: {}", s)),
                };
            }
            _ => return Err(format!("Unknown key or modifier: {}", part)),
        }
    }

    let code = key_code.ok_or("No key specified in hotkey")?;

    Ok(Shortcut::new(Some(modifiers), code))
}

// App version command for frontend
#[tauri::command]
fn get_app_version() -> String {
    APP_VERSION.to_string()
}

#[tauri::command]
fn get_app_name() -> String {
    APP_NAME.to_string()
}

// Error Reporting Commands

#[tauri::command]
async fn report_error(
    app: tauri::AppHandle,
    category: String,
    message: String,
    severity: String,
    stack_trace: Option<String>,
    user_action: Option<String>,
    context: Option<std::collections::HashMap<String, String>>,
) -> Result<(), CommandError> {
    let severity = match severity.to_lowercase().as_str() {
        "debug" => ErrorSeverity::Debug,
        "info" => ErrorSeverity::Info,
        "warning" => ErrorSeverity::Warning,
        "error" => ErrorSeverity::Error,
        "critical" => ErrorSeverity::Critical,
        "fatal" => ErrorSeverity::Fatal,
        _ => ErrorSeverity::Error,
    };

    let category = match category.to_lowercase().as_str() {
        "transcription" => ErrorCategory::Transcription,
        "audio" => ErrorCategory::Audio,
        "model" => ErrorCategory::Model,
        "database" => ErrorCategory::Database,
        "network" => ErrorCategory::Network,
        "filesystem" => ErrorCategory::FileSystem,
        "license" => ErrorCategory::License,
        "ui" => ErrorCategory::Ui,
        "system" => ErrorCategory::System,
        "configuration" => ErrorCategory::Configuration,
        _ => ErrorCategory::Unknown,
    };

    if let Some(reporter) = ErrorReporter::global() {
        let mut report = ErrorReport::new(severity, category, message.clone());

        if let Some(trace) = stack_trace {
            report = report.with_details(trace);
        }

        if let Some(action) = user_action {
            report = report.with_context("user_action", action);
        }

        if let Some(ctx) = context {
            for (key, value) in ctx {
                report = report.with_context(key, value);
            }
        }

        reporter.report(report);

        // Also send to Sentry if available
        if sentry::Hub::current().client().is_some() {
            let sentry_severity = severity.to_sentry_level();
            let _ = sentry::capture_message(&message, sentry_severity);
        }

        // Persist to disk periodically
        if let Ok(app_data_dir) = app.path().app_data_dir() {
            let _ = reporter.persist_to_file(&app_data_dir);
        }
    }

    Ok(())
}

trait ToSentryLevel {
    fn to_sentry_level(&self) -> SentryLogLevel;
}

impl ToSentryLevel for ErrorSeverity {
    fn to_sentry_level(&self) -> SentryLogLevel {
        match self {
            ErrorSeverity::Debug => SentryLogLevel::Debug,
            ErrorSeverity::Info => SentryLogLevel::Info,
            ErrorSeverity::Warning => SentryLogLevel::Warning,
            ErrorSeverity::Error | ErrorSeverity::Critical | ErrorSeverity::Fatal => SentryLogLevel::Error,
        }
    }
}

#[tauri::command]
async fn get_error_reports(limit: Option<usize>) -> Result<Vec<ErrorReport>, CommandError> {
    if let Some(reporter) = ErrorReporter::global() {
        Ok(reporter.get_reports(limit))
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
async fn get_error_stats() -> Result<ErrorStats, CommandError> {
    if let Some(reporter) = ErrorReporter::global() {
        Ok(reporter.get_stats())
    } else {
        Ok(ErrorStats {
            total_errors: 0,
            by_category: std::collections::HashMap::new(),
            by_severity: std::collections::HashMap::new(),
        })
    }
}

#[tauri::command]
async fn export_error_reports(
    app: tauri::AppHandle,
    format: Option<String>,
) -> Result<String, CommandError> {
    let format_str = format.unwrap_or_else(|| "json".to_string());

    let content = if let Some(reporter) = ErrorReporter::global() {
        match format_str.to_lowercase().as_str() {
            "markdown" | "md" => reporter.export_to_markdown(),
            _ => reporter.export_to_json(),
        }
    } else {
        "{}".to_string()
    };

    // Also save to file
    if let Ok(app_data_dir) = app.path().app_data_dir() {
        let reports_dir = app_data_dir.join("reports");
        std::fs::create_dir_all(&reports_dir).ok();

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let extension = if format_str == "markdown" || format_str == "md" {
            "md"
        } else {
            "json"
        };
        let filename = format!("error_report_{}.{}", timestamp, extension);
        let filepath = reports_dir.join(&filename);

        std::fs::write(&filepath, &content)?;

        info!("Error report exported to: {:?}", filepath);
    }

    Ok(content)
}

#[tauri::command]
async fn test_sentry() -> Result<String, CommandError> {
    // Test if Sentry is configured and working
    if sentry::Hub::current().client().is_some() {
        // Send a test message to Sentry
        sentry::capture_message("Sentry test from Rust backend - this is a test message", SentryLogLevel::Info);
        
        // Also test with the error reporter
        if let Some(reporter) = ErrorReporter::global() {
            let report = ErrorReport::new(
                ErrorSeverity::Info, 
                ErrorCategory::System, 
                "Sentry integration test".to_string()
            ).with_context("test", "true");
            reporter.report(report);
        }
        
        Ok("Sentry is configured and test message sent! Check your Sentry dashboard.".to_string())
    } else {
        Ok("Sentry is not configured. Set SENTRY_DSN environment variable.".to_string())
    }
}

#[tauri::command]
async fn save_export_file(path: String, content: String) -> Result<(), CommandError> {
    let sanitized_content =
        sanitize_text(&content, MAX_EXPORT_BYTES).map_err(CommandError::PostProcessing)?;
    let export_path =
        validate_export_path(&path).map_err(|e| CommandError::Io(std::io::Error::other(e)))?;

    std::fs::write(&export_path, sanitized_content)?;
    info!("Export saved to: {:?}", export_path);

    Ok(())
}

#[tauri::command]
async fn clear_error_reports() -> Result<(), CommandError> {
    if let Some(reporter) = ErrorReporter::global() {
        reporter.clear();
    }
    info!("Error reports cleared");
    Ok(())
}

#[tauri::command]
async fn load_error_reports(app: tauri::AppHandle) -> Result<usize, CommandError> {
    if let Some(reporter) = ErrorReporter::global() {
        if let Ok(app_data_dir) = app.path().app_data_dir() {
            match reporter.load_from_file(&app_data_dir) {
                Ok(count) => {
                    info!("Loaded {} error reports from disk", count);
                    return Ok(count);
                }
                Err(e) => {
                    warn!("Failed to load error reports: {}", e);
                }
            }
        }
    }
    Ok(0)
}

// ==================== Cloud Provider Commands (BYOK) ====================

#[tauri::command]
async fn get_cloud_providers(
    db: State<'_, DbState>,
) -> CommandResult<Vec<cloud_transcription::CloudProviderInfo>> {
    let records = db.0.get_cloud_providers().map_err(CommandError::Database)?;
    let mut map: HashMap<String, database::CloudProviderRecord> = records
        .into_iter()
        .map(|r| (r.id.clone(), r))
        .collect();

    let standard_providers = vec![
        ("groq", "Groq"),
        ("openai", "OpenAI"),
        ("deepgram", "Deepgram"),
        ("mistral", "Mistral"),
        ("custom", "Custom Endpoint"),
    ];

    let mut result = Vec::new();
    for (id, name) in standard_providers {
        if let Some(record) = map.remove(id) {
            let decrypted_key = cloud_transcription::decrypt_api_key(&record.api_key);
            let has_key = !decrypted_key.trim().is_empty() || id == "custom";
            let masked = if has_key && !decrypted_key.trim().is_empty() {
                cloud_transcription::mask_key(&decrypted_key)
            } else {
                String::new()
            };
            result.push(cloud_transcription::CloudProviderInfo {
                id: id.to_string(),
                name: name.to_string(),
                configured: has_key,
                masked_key: masked,
                base_url: record.base_url,
                custom_model: record.custom_model,
            });
        } else {
            result.push(cloud_transcription::CloudProviderInfo {
                id: id.to_string(),
                name: name.to_string(),
                configured: false,
                masked_key: String::new(),
                base_url: None,
                custom_model: None,
            });
        }
    }
    Ok(result)
}

#[tauri::command]
async fn save_cloud_provider(
    db: State<'_, DbState>,
    provider: String,
    api_key: String,
    base_url: Option<String>,
    custom_model: Option<String>,
) -> CommandResult<()> {
    let encrypted = cloud_transcription::encrypt_api_key(&api_key)
        .map_err(|e| CommandError::Transcription(format!("Encryption failed: {}", e)))?;
    db.0.save_cloud_provider(
        &provider,
        &encrypted,
        base_url.as_deref(),
        custom_model.as_deref(),
    )
    .map_err(CommandError::Database)?;
    Ok(())
}

#[tauri::command]
async fn delete_cloud_provider(
    db: State<'_, DbState>,
    provider: String,
) -> CommandResult<()> {
    db.0.delete_cloud_provider(&provider).map_err(CommandError::Database)?;
    Ok(())
}

#[tauri::command]
async fn test_cloud_connection(
    db: State<'_, DbState>,
    provider: String,
    api_key: Option<String>,
    base_url: Option<String>,
) -> CommandResult<String> {
    let key = if let Some(k) = api_key.filter(|k| !k.trim().is_empty()) {
        k
    } else if let Some(record) = db.0.get_cloud_provider(&provider).map_err(CommandError::Database)? {
        cloud_transcription::decrypt_api_key(&record.api_key)
    } else {
        String::new()
    };

    let url = base_url.or_else(|| {
        db.0.get_cloud_provider(&provider).ok().flatten().and_then(|r| r.base_url)
    });

    cloud_transcription::test_provider_connection(&provider, &key, url.as_deref())
        .await
        .map_err(CommandError::Transcription)
}

// ==================== AI Formatting Provider Commands (BYOK) ====================

#[tauri::command]
async fn get_ai_formatting_providers(
    db: State<'_, DbState>,
) -> CommandResult<Vec<ai_formatting::AiFormattingProviderInfo>> {
    let records = db.0.get_ai_formatting_providers().map_err(CommandError::Database)?;
    let mut map: HashMap<String, AiFormattingProviderRecord> = records
        .into_iter()
        .map(|r| (r.id.clone(), r))
        .collect();

    let standard_providers = vec![
        ("gemini", "Gemini"),
        ("anthropic", "Anthropic"),
        ("openai", "OpenAI"),
        ("deepseek", "DeepSeek"),
        ("custom", "Custom Endpoint"),
    ];

    let mut result = Vec::new();
    for (id, name) in standard_providers {
        if let Some(record) = map.remove(id) {
            let decrypted_key = cloud_transcription::decrypt_api_key(&record.api_key);
            let has_key = !decrypted_key.trim().is_empty() || id == "custom";
            let masked = if has_key && !decrypted_key.trim().is_empty() {
                cloud_transcription::mask_key(&decrypted_key)
            } else {
                String::new()
            };
            result.push(ai_formatting::AiFormattingProviderInfo {
                id: id.to_string(),
                name: name.to_string(),
                configured: has_key,
                masked_key: masked,
                base_url: record.base_url,
                custom_model: record.custom_model,
            });
        } else {
            result.push(ai_formatting::AiFormattingProviderInfo {
                id: id.to_string(),
                name: name.to_string(),
                configured: false,
                masked_key: String::new(),
                base_url: None,
                custom_model: None,
            });
        }
    }
    Ok(result)
}

#[tauri::command]
async fn save_ai_formatting_provider(
    db: State<'_, DbState>,
    provider: String,
    api_key: String,
    base_url: Option<String>,
    custom_model: Option<String>,
) -> CommandResult<()> {
    let encrypted = cloud_transcription::encrypt_api_key(&api_key)
        .map_err(|e| CommandError::Transcription(format!("Encryption failed: {}", e)))?;
    db.0.save_ai_formatting_provider(
        &provider,
        &encrypted,
        base_url.as_deref(),
        custom_model.as_deref(),
    )
    .map_err(CommandError::Database)?;
    Ok(())
}

#[tauri::command]
async fn delete_ai_formatting_provider(
    db: State<'_, DbState>,
    provider: String,
) -> CommandResult<()> {
    db.0.delete_ai_formatting_provider(&provider).map_err(CommandError::Database)?;
    Ok(())
}

#[tauri::command]
async fn test_ai_formatting_connection(
    db: State<'_, DbState>,
    provider: String,
    api_key: Option<String>,
    base_url: Option<String>,
    model: Option<String>,
) -> CommandResult<String> {
    let key = if let Some(k) = api_key.filter(|k| !k.trim().is_empty()) {
        k
    } else if let Some(record) = db.0.get_ai_formatting_provider(&provider).map_err(CommandError::Database)? {
        cloud_transcription::decrypt_api_key(&record.api_key)
    } else {
        String::new()
    };

    let url = base_url.or_else(|| {
        db.0.get_ai_formatting_provider(&provider).ok().flatten().and_then(|r| r.base_url)
    });

    let model = model.or_else(|| {
        db.0.get_ai_formatting_provider(&provider).ok().flatten().and_then(|r| r.custom_model)
    });

    ai_formatting::test_ai_provider_connection(&provider, &key, url.as_deref(), model.as_deref())
        .await
        .map_err(CommandError::Transcription)
}

#[tauri::command]
async fn format_text_with_ai(
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
    text: String,
    style: String,
    provider: Option<String>,
    model: Option<String>,
) -> CommandResult<String> {
    let db = db.0.clone();
    let license_manager = license_manager.0.clone();

    ensure_app_access_verified(&db, &license_manager).await?;

    if text.trim().is_empty() {
        return Ok(String::new());
    }

    let settings = db.get_settings().map_err(CommandError::Database)?;

    // Use provided values or fall back to settings
    let provider_id = provider.unwrap_or_else(|| settings.ai_formatting_provider_id.clone());
    let model_name = model.unwrap_or_else(|| settings.ai_formatting_model.clone());

    let record = db
        .get_ai_formatting_provider(&provider_id)
        .map_err(CommandError::Database)?
        .ok_or_else(|| {
            CommandError::Transcription(format!(
                "AI formatting provider '{}' is not configured. Please add an API key in AI Formatting settings.",
                provider_id
            ))
        })?;

    let api_key = cloud_transcription::decrypt_api_key(&record.api_key);
    let base_url = record.base_url.as_deref();
    let custom_model = record.custom_model.as_deref().or(Some(&model_name));

    let style_prompt = ai_formatting_style_prompt(&style);

    let formatted = ai_formatting::format_text_with_ai(
        &provider_id,
        custom_model.unwrap_or("gpt-4o-mini"),
        &api_key,
        base_url,
        style_prompt,
        &text,
    )
    .await
    .map_err(CommandError::Transcription)?;

    Ok(formatted)
}

/// Map a style id to its system prompt. Mirrors the frontend
/// `AI_FORMATTING_STYLES` definitions so the backend can format
/// independently of the frontend.
fn ai_formatting_style_prompt(style: &str) -> &'static str {
    match style {
        "personal" => "You are a helpful dictation assistant. The user has spoken the following text which was transcribed from voice. Your job is to lightly clean up obvious transcription errors, punctuation, and filler words while preserving the speaker's casual, conversational tone and personality. Do not over-edit or change the speaker's voice. Return only the formatted text, with no preamble or explanation.",
        "clean" => "You are a professional dictation editor. The user has spoken the following text which was transcribed from voice. Your job is to produce clean, professional prose: fix grammar, remove filler words (um, uh, like, you know), add proper punctuation, ensure proper sentence structure, and create clean paragraph breaks. Return only the formatted text, with no preamble or explanation.",
        "writing" => "You are an editor helping someone turn spoken dictation into polished written content. Format the following transcribed text as a well-structured article or blog post. Use proper paragraphs, fix grammar and flow, and polish the prose to sound professional and engaging. Return only the formatted text, with no preamble or explanation.",
        "notes" => "You are a note-taking assistant. Extract the key points and important information from the following transcribed dictation. Format as concise bullet points or short phrases, capturing the essential information. Remove filler words and redundant phrasing. Return only the formatted notes, with no preamble or explanation.",
        "email" => "You are a professional email assistant. Format the following transcribed dictation as a polished business email. Add an appropriate greeting, structure the body with clear paragraphs, and include a professional sign-off. Ensure the tone is appropriate and professional. Return only the formatted email text, with no preamble or explanation.",
        "code" => "You are a coding assistant. The following text was transcribed from voice and contains spoken programming terms, code snippets, and technical instructions. Convert spoken descriptions of code into clean, properly formatted code. Apply appropriate casing (camelCase, PascalCase, snake_case), insert code symbols (brackets, braces, operators) that were spoken as words, and organize into logical blocks. Preserve any literal code. Return only the formatted code, with no preamble or explanation.",
        "social" => "You are a social media assistant. The following text was transcribed from casual voice dictation. Format it for social media: keep a conversational and engaging tone, use short paragraphs or sentences, add appropriate line breaks, and make it easy to read. Return only the formatted text, with no preamble or explanation.",
        "academic" => "You are an academic writing assistant. The following text was transcribed from voice. Rewrite it in a formal academic style: use precise language, proper sentence structure, formal tone, and structured paragraphs. Remove colloquialisms and filler words. Return only the formatted text, with no preamble or explanation.",
        "business" => "You are a business communication assistant. The following text was transcribed from voice. Format it as a professional business document: use clear, concise language, structured paragraphs, bullet points where appropriate, and a professional tone. Return only the formatted text, with no preamble or explanation.",
        "transcript" => "You are a transcription editor. The following text was transcribed from voice. Create a clean transcript-style output: preserve the original meaning and content, but fix obvious transcription errors, add proper punctuation, and organize into readable paragraphs. Do not change the tone or style of the original speech. Return only the formatted text, with no preamble or explanation.",
        "legal" => "You are a legal transcription assistant. The following text was transcribed from voice. Rewrite it in a formal legal style: use precise terminology, structured paragraphs, and proper legal phrasing. Maintain the original meaning while ensuring legal accuracy. Return only the formatted text, with no preamble or explanation.",
        "meeting" => "You are a meeting minutes assistant. The following text was transcribed from a meeting recording. Format it as professional meeting minutes: include a brief summary, list key discussion points, highlight decisions made, and extract action items with responsible parties and deadlines. Use clear headings and bullet points. Return only the formatted text, with no preamble or explanation.",
        "journaling" => "You are a journaling assistant. The following text was transcribed from a personal voice journal entry. Format it as a thoughtful, well-structured journal entry: preserve the personal, reflective tone, organize thoughts into coherent paragraphs, add proper punctuation, and maintain the authentic voice of the writer. Return only the formatted text, with no preamble or explanation.",
        _ => "You are a professional dictation editor. Clean up the following transcribed text: fix grammar, add proper punctuation, and create clean paragraph breaks. Return only the formatted text, with no preamble or explanation.",
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    info!("Starting {} v{}", APP_NAME, APP_VERSION);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .setup(|app| {
            info!("Initializing application...");

            // Initialize database
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");

            info!("App data directory: {:?}", app_data_dir);

            // Initialize error reporter early for crash handling
            let error_log_dir = app_data_dir.join("logs");
            ErrorReporter::init(error_log_dir);

            let db = Database::new(app_data_dir.clone()).expect("Failed to initialize database");
            let diagnostics_enabled = db.get_settings().map(|s| s.diagnostics_enabled).unwrap_or(false);
            app.manage(DbState(Arc::new(db)));

            // Initialize Sentry if diagnostics are enabled
            if let Some(dsn) = SENTRY_DSN {
                if diagnostics_enabled {
                    let _guard = sentry::init((dsn, sentry::ClientOptions {
                        release: Some(APP_VERSION.into()),
                        environment: Some(if cfg!(debug_assertions) {
                            std::borrow::Cow::Borrowed("dev")
                        } else {
                            std::borrow::Cow::Borrowed("production")
                        }),
                        ..Default::default()
                    }));
                    info!("Sentry initialized");
                    // Keep guard alive for the app's lifetime
                    let _ = _guard;
                }
            }

            // Initialize recorder state
            app.manage(RecorderState(Arc::new(Mutex::new(None))));

            // Initialize transcriber state
            app.manage(TranscriberState(Arc::new(Mutex::new(None))));

            // Initialize downloader
            let models_dir = app_data_dir.join("models");
            app.manage(DownloaderState(Arc::new(ModelDownloader::new(models_dir))));

            // Initialize license manager
            app.manage(LicenseManagerState(Arc::new(LicenseManager::new())));

            // Initialize text injector (reused for better performance)
            let text_injector =
                text_inject::TextInjector::new().expect("Failed to initialize text injector");
            app.manage(TextInjectorState(Arc::new(Mutex::new(text_injector))));

// Initialize rate limiters (100 requests per 60 seconds)
            app.manage(RecordingRateLimiter(Arc::new(RateLimiter::new(100, 60))));
            app.manage(TranscriptionRateLimiter(Arc::new(RateLimiter::new(50, 60))));

            setup_window_icons(app)?;

            // Setup system tray
            setup_tray(app)?;

            // Handle startup args (e.g. --minimized from autostart). When the
            // app is launched via autostart we hide the main window so it
            // sits in the tray until the user opens it. This only applies
            // when autostart is actually enabled — toggling it off in
            // settings does not remove an existing launch argument, but the
            // window visibility is driven by the stored setting at runtime.
            let args: Vec<String> = std::env::args().collect();
            let is_minimized = args.contains(&"--minimized".to_string());
            if is_minimized {
                if let Some(main_window) = app.get_webview_window("main") {
                    let _ = main_window.hide();
                }
            }

            info!("Application initialized successfully");

            // Note: Hotkey is registered from the frontend via register_hotkey command
            // This allows the frontend to control which hotkey is used based on settings

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    // Respect the user's "Minimize to tray" setting. When
                    // disabled, closing the window should exit the app
                    // entirely instead of hiding it in the tray.
                    let minimize_to_tray = {
                        let app_handle = window.app_handle();
                        match app_handle.try_state::<DbState>() {
                            Some(db_state) => {
                                match db_state.0.get_settings() {
                                    Ok(settings) => settings.minimize_to_tray,
                                    Err(err) => {
                                        warn!("Failed to read settings for close behavior: {}", err);
                                        // Default to hiding on error to avoid accidental exits
                                        true
                                    }
                                }
                            }
                            None => {
                                warn!("DbState not available; defaulting to minimize-to-tray");
                                true
                            }
                        }
                    };

                    if minimize_to_tray {
                        debug!("Window close requested, hiding to tray");
                        let _ = window.hide();
                        api.prevent_close();
                    } else {
                        debug!("Window close requested and minimize-to-tray disabled; exiting app");
                        window.app_handle().exit(0);
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            // Settings
            get_settings,
            update_settings,
            update_setting,
            // App state
            get_app_state,
            update_app_state,
            set_setup_complete,
            set_current_setup_step,
            // Models
            get_models,
            get_model,
            set_model_downloaded,
            set_selected_model,
            // Cloud providers (BYOK)
            get_cloud_providers,
            save_cloud_provider,
            delete_cloud_provider,
            test_cloud_connection,
            // AI Formatting providers (BYOK)
            get_ai_formatting_providers,
            save_ai_formatting_provider,
            delete_ai_formatting_provider,
            test_ai_formatting_connection,
            format_text_with_ai,
            // Recording
            get_audio_input_devices,
            get_audio_output_devices,
            set_audio_input_device,
            set_audio_capture_config,
            start_recording,
            stop_recording,
            save_temp_audio,
            cancel_recording,
            is_recording,
            // Recording overlay
            show_recording_overlay,
            hide_recording_overlay,
            // Transcription
            load_model,
            unload_model,
            get_loaded_model,
            transcribe_audio,
            record_and_transcribe,
            record_and_translate,
            translate_text,
            transcribe_file,
            transcribe_url,
            transcribe_files_batch,
            // Download
            download_model,
            cancel_model_download,
            delete_model,
            is_model_downloaded,
            get_downloaded_models,
            get_model_path,
            // Text injection
            inject_text,
            execute_keyboard_shortcut,
            // Post-processing
            post_process_text,
            extract_voice_commands,
            // Transcription history
            add_transcription,
            get_transcription_history,
            get_transcription_history_count,
            clear_transcription_history,
            delete_transcription,
            // License
            get_license,
            activate_license,
            validate_license,
            deactivate_license,
            clear_stored_license,
            is_license_valid,
            start_trial,
            get_trial_status,
            get_device_info,
            can_use_app,
            // Utility
            get_app_data_dir,
            get_models_dir,
            // Hotkeys
            register_hotkey,
            unregister_hotkeys,
            // App info
            get_app_version,
            get_app_name,
            // Error reporting
            report_error,
            test_sentry,
            get_error_reports,
            get_error_stats,
            export_error_reports,
            save_export_file,
            clear_error_reports,
            load_error_reports,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn app_icon() -> tauri::Result<Image<'static>> {
    Image::from_bytes(APP_ICON_BYTES)
}

fn setup_window_icons(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let icon = app_icon()?;

    if let Some(window) = app.get_webview_window("main") {
        window.set_icon(icon.clone())?;
        window.set_skip_taskbar(true)?;
    }

    if let Some(window) = app.get_webview_window("recording-overlay") {
        window.set_icon(icon)?;
    }

    Ok(())
}

fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Create tray menu items
    let title_item = MenuItem::with_id(app, "title", "WhisprTypr", false, None::<&str>)?;
    let show_item = MenuItem::with_id(app, "show", "Open WhisprTypr", true, None::<&str>)?;
    let start_recording_item = MenuItem::with_id(
        app,
        "start_recording",
        "Start Recording",
        true,
        None::<&str>,
    )?;
    let stop_recording_item =
        MenuItem::with_id(app, "stop_recording", "Stop Recording", true, None::<&str>)?;
    let transcribe_file_item =
        MenuItem::with_id(app, "transcribe", "Transcribe File...", true, None::<&str>)?;
    let history_item = MenuItem::with_id(app, "history", "History", true, None::<&str>)?;
    let models_item = MenuItem::with_id(app, "models", "Models", true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let help_item = MenuItem::with_id(app, "help", "Help & Support", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let separator_2 = PredefinedMenuItem::separator(app)?;
    let separator_3 = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    // Build menu
    let menu = Menu::with_items(
        app,
        &[
            &title_item,
            &separator,
            &show_item,
            &transcribe_file_item,
            &start_recording_item,
            &stop_recording_item,
            &separator_2,
            &history_item,
            &models_item,
            &settings_item,
            &help_item,
            &separator_3,
            &quit_item,
        ],
    )?;

    let icon = app_icon()?;

    // Build tray icon
    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .tooltip("WhisprTypr - Voice to Text")
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                show_main_window(app);
                let _ = app.emit("tray-navigate", "main");
            }
            "start_recording" => {
                let _ = app.emit("tray-start-recording", ());
            }
            "stop_recording" => {
                let _ = app.emit("tray-stop-recording", ());
            }
            "transcribe" => {
                show_main_window(app);
                let _ = app.emit("tray-navigate", "transcribe");
            }
            "history" => {
                show_main_window(app);
                let _ = app.emit("tray-navigate", "history");
            }
            "models" => {
                show_main_window(app);
                let _ = app.emit("tray-navigate", "models");
            }
            "settings" => {
                show_main_window(app);
                let _ = app.emit("tray-navigate", "settings");
            }
            "help" => {
                show_main_window(app);
                let _ = app.emit("tray-navigate", "help");
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                show_main_window(&app);
                let _ = app.emit("tray-navigate", "main");
            }
        })
        .build(app)?;

    Ok(())
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

async fn ensure_app_access_verified(
    db: &Database,
    license_manager: &LicenseManager,
) -> CommandResult<()> {
    let has_db_license = db
        .get_license()
        .map(|license| db_license_allows_usage(&license))
        .unwrap_or(false);

    let has_trial = has_active_trial(db);

    match license_manager.validate().await {
        Ok(_) => Ok(()),
        Err(error) => {
            if license_manager.is_authoritative_validate_error(&error) {
                // Authoritative rejection (revoked/disabled/invalid). The
                // cache has already been cleared by LicenseManager. Do NOT
                // fall back to the DB row — a revoked license must not
                // continue dictating.
                warn!("App access denied: license authoritatively rejected: {}", error);
                return Err(CommandError::License(
                    "A valid license or active trial is required.".to_string(),
                ));
            }

            // Transient failure (network/5xx): fall back to the DB row and
            // the trial. This is the offline grace period.
            if has_db_license || has_trial {
                Ok(())
            } else {
                Err(CommandError::License(
                    "A valid license or active trial is required.".to_string(),
                ))
            }
        }
    }
}

#[cfg(test)]
mod audio_ingestion_tests {
    use super::*;

    #[test]
    fn append_audio_samples_rejects_over_limit() {
        let mut target = vec![0.0, 0.1];
        let source = vec![0.2, 0.3];

        let result = append_audio_samples_with_limit(&mut target, &source, 3);

        assert!(result.is_err());
        assert_eq!(target, vec![0.0, 0.1]);
    }

    #[test]
    fn interleaved_to_mono_averages_channels() {
        let mono = interleaved_to_mono(&[1.0, -1.0, 0.5, 0.25], 2);

        assert_eq!(mono, vec![0.0, 0.375]);
    }
}

#[cfg(test)]
mod text_injection_tests {
    use super::*;

    #[test]
    fn sanitize_text_removes_unwanted_control_characters() {
        // Keeps \n, \r, \t but trims bad null bytes \0 etc
        let dirty = "Hello\nWorld\t\r\0\x07";
        let clean = sanitize_text(dirty, 100).unwrap();
        assert_eq!(clean, "Hello\nWorld\t\r");
    }

    #[test]
    fn sanitize_text_respects_byte_limits() {
        let text = "Exactly ten!";
        // 12 bytes
        assert!(sanitize_text(text, 12).is_ok());
        assert!(sanitize_text(text, 10).is_err()); // Exceeds limit
    }
}

#[cfg(test)]
mod model_compatibility_tests {
    use super::*;

    #[test]
    fn is_valid_language_code_checks() {
        assert!(is_valid_language_code("auto"));
        assert!(is_valid_language_code("en"));
        assert!(is_valid_language_code("spa"));
        assert!(!is_valid_language_code("e")); // Too short
        assert!(!is_valid_language_code("EN")); // Must be lowercase
        assert!(!is_valid_language_code("english")); // Too long
    }

    #[test]
    fn expected_models_lock_onto_specific_languages() {
        // English-only models
        assert!(is_model_language_supported("tiny.en", "en"));
        assert!(!is_model_language_supported("tiny.en", "es"));
        assert!(!is_model_language_supported("tiny.en", "auto"));

        // Parakeet allows specific langs
        assert!(is_model_language_supported("parakeet-v3", "auto"));
        assert!(is_model_language_supported("parakeet-v3", "fr"));
        assert!(!is_model_language_supported("parakeet-v3", "invalid"));

        // Multilingual whisper allows standard codes or auto
        assert!(is_model_language_supported("large-v3", "auto"));
        assert!(is_model_language_supported("large-v3", "de"));
    }
}
