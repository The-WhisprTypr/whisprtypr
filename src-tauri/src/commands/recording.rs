use crate::{
    audio::{AudioCaptureSource, AudioInputDevice, AudioOutputDevice, AudioRecorder},
    CommandError, CommandResult, DbState, LicenseManagerState, RecorderState, RecordingRateLimiter,
};
use log::{debug, error, warn};
use tauri::{AppHandle, Emitter, Manager, State};

#[tauri::command]
pub fn get_audio_input_devices() -> CommandResult<Vec<AudioInputDevice>> {
    AudioRecorder::list_input_devices().map_err(CommandError::Recording)
}

#[tauri::command]
pub fn get_audio_output_devices() -> CommandResult<Vec<AudioOutputDevice>> {
    AudioRecorder::list_output_devices().map_err(CommandError::Recording)
}

#[tauri::command]
pub fn set_audio_input_device(
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
pub fn set_audio_capture_config(
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
pub async fn start_recording(
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
    recorder: State<'_, RecorderState>,
    rate_limiter: State<'_, RecordingRateLimiter>,
) -> CommandResult<()> {
    let db = db.0.clone();
    let license_manager = license_manager.0.clone();
    let recorder = recorder.0.clone();
    let rate_limiter = rate_limiter.0.clone();

    crate::ensure_app_access_verified(&db, &license_manager).await?;

    if !rate_limiter.check("start_recording") {
        return Err(crate::CommandError::Recording(
            "Rate limit exceeded. Please wait before starting another recording.".to_string(),
        ));
    }

    debug!("start_recording called");
    let mut recorder_guard = recorder.lock().unwrap();

    if recorder_guard.is_none() {
        debug!("Creating new AudioRecorder");
        *recorder_guard = Some(AudioRecorder::new().map_err(|e| {
            error!("Failed to create AudioRecorder: {}", e);
            crate::CommandError::Recording(e)
        })?)
    }

    if let Some(ref mut rec) = *recorder_guard {
        debug!("Starting recording...");
        rec.start_recording().map_err(|e| {
            error!("Failed to start recording: {}", e);
            crate::CommandError::Recording(e)
        })?;
        debug!("Recording started successfully");
    }

    Ok(())
}

#[tauri::command]
pub fn stop_recording(recorder: State<RecorderState>) -> CommandResult<Vec<f32>> {
    let mut recorder_guard = recorder.0.lock().unwrap();

    if let Some(ref mut rec) = *recorder_guard {
        rec.stop_recording().map_err(|e| {
            error!("Failed to stop recording: {}", e);
            crate::CommandError::Recording(e)
        })
    } else {
        Err(crate::CommandError::Recording(
            "No recorder initialized".to_string(),
        ))
    }
}

#[tauri::command]
pub async fn save_temp_audio(
    app: AppHandle,
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
    samples: Vec<f32>,
) -> CommandResult<String> {
    let db = db.0.clone();
    let license_manager = license_manager.0.clone();

    crate::ensure_app_access_verified(&db, &license_manager).await?;

    let temp_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| crate::CommandError::Io(std::io::Error::other(e.to_string())))?;
    std::fs::create_dir_all(&temp_dir).map_err(crate::CommandError::Io)?;

    let file_path = temp_dir.join(format!("temp_recording_{}.wav", uuid::Uuid::new_v4()));
    let path_str = file_path
        .to_str()
        .ok_or_else(|| crate::CommandError::Io(std::io::Error::other("Invalid path")))?;

    crate::audio::save_wav(&samples, path_str).map_err(crate::CommandError::Recording)?;

    Ok(path_str.to_string())
}

#[tauri::command]
pub fn cancel_recording(recorder: State<RecorderState>) -> CommandResult<()> {
    let mut recorder_guard = recorder.0.lock().unwrap();

    if let Some(ref mut rec) = *recorder_guard {
        rec.cancel_recording();
    }

    Ok(())
}

#[tauri::command]
pub fn is_recording(recorder: State<RecorderState>) -> bool {
    let recorder_guard = recorder.0.lock().unwrap();
    recorder_guard
        .as_ref()
        .map(|r| r.is_recording())
        .unwrap_or(false)
}

#[tauri::command]
pub async fn show_recording_overlay(app: AppHandle, position: Option<String>) -> CommandResult<()> {
    if let Some(overlay_window) = app.get_webview_window("recording-overlay") {
        overlay_window.show().map_err(|e| {
            error!("Failed to show overlay window: {}", e);
            crate::CommandError::Recording(format!("Failed to show overlay: {}", e))
        })?;

        overlay_window.set_fullscreen(true).map_err(|e| {
            warn!("Failed to set fullscreen: {}", e);
            crate::CommandError::Recording(format!("Failed to set fullscreen: {}", e))
        })?;

        overlay_window.set_always_on_top(true).map_err(|e| {
            warn!("Failed to set always on top: {}", e);
            crate::CommandError::Recording(format!("Failed to set always on top: {}", e))
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
pub async fn hide_recording_overlay(app: AppHandle) -> CommandResult<()> {
    if let Some(overlay_window) = app.get_webview_window("recording-overlay") {
        overlay_window.hide().map_err(|e| {
            error!("Failed to hide overlay window: {}", e);
            crate::CommandError::Recording(format!("Failed to hide overlay: {}", e))
        })?;

        debug!("Recording overlay hidden");
    }

    Ok(())
}
