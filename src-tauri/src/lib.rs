#![recursion_limit = "512"]

pub mod ai_formatting;
pub mod audio;
pub mod cloud_transcription;
pub mod commands;
pub mod database;
pub mod downloader;
pub mod error_reporting;
pub mod license;
pub mod post_process;
pub mod providers;
pub mod security;
mod text_inject;
pub mod transcription;
mod translation;
pub mod utils;

pub use commands::*;
pub use utils::*;

use audio::AudioRecorder;
use database::Database;
use downloader::ModelDownloader;
use error_reporting::ErrorReporter;
use license::{get_device_id, LicenseManager, LicenseStatus};
use log::{debug, info, warn};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent,
};
use transcription::Transcriber;

// Application version from Cargo.toml
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_NAME: &str = "Whisprtypr";
const SENTRY_DSN: Option<&str> = option_env!("SENTRY_DSN");
const APP_ICON_BYTES: &[u8] = include_bytes!("../icons/icon.png");
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
            let diagnostics_enabled = db
                .get_settings()
                .map(|s| s.diagnostics_enabled)
                .unwrap_or(false);
            app.manage(DbState(Arc::new(db)));

            // Initialize Sentry if diagnostics are enabled
            if let Some(dsn) = SENTRY_DSN {
                if diagnostics_enabled {
                    let _guard = sentry::init((
                        dsn,
                        sentry::ClientOptions {
                            release: Some(APP_VERSION.into()),
                            environment: Some(if cfg!(debug_assertions) {
                                std::borrow::Cow::Borrowed("dev")
                            } else {
                                std::borrow::Cow::Borrowed("production")
                            }),
                            ..Default::default()
                        },
                    ));
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
                                        warn!(
                                            "Failed to read settings for close behavior: {}",
                                            err
                                        );
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
    let title_item = MenuItem::with_id(app, "title", "Whisprtypr", false, None::<&str>)?;
    let show_item = MenuItem::with_id(app, "show", "Open Whisprtypr", true, None::<&str>)?;
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
        .tooltip("Whisprtypr - Voice to Text")
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
                warn!(
                    "App access denied: license authoritatively rejected: {}",
                    error
                );
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
