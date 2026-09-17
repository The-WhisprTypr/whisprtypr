use crate::{
    cloud_transcription::build_http_client, CommandError, CommandResult, DbState, DownloaderState,
    LicenseManagerState,
};
use tauri::{AppHandle, Emitter, Manager, State};

pub(crate) async fn download_url_to_temp(
    app: &AppHandle,
    url: &str,
) -> Result<std::path::PathBuf, String> {
    let client = build_http_client(120)?;
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to download URL: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status {}", response.status()));
    }

    let temp_dir = app.path().app_cache_dir().map_err(|e| e.to_string())?;
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

pub(crate) async fn ensure_yt_dlp(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let bin_dir = app_data_dir.join("bin");
    std::fs::create_dir_all(&bin_dir).map_err(|e| e.to_string())?;

    let binary_path = bin_dir.join(yt_dlp_binary_name());

    if binary_path.exists() {
        return Ok(binary_path);
    }

    let urls = if cfg!(target_os = "windows") {
        vec!["https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe"]
    } else if cfg!(target_os = "macos") {
        vec!["https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos"]
    } else {
        vec!["https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp"]
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
                std::fs::set_permissions(&binary_path, perms).map_err(|e| e.to_string())?;
            }

            return Ok(binary_path);
        }
    }

    Err(
        "Failed to download yt-dlp. Make sure your network allows access to GitHub releases."
            .to_string(),
    )
}

pub(crate) fn find_system_ffmpeg() -> Result<Option<std::path::PathBuf>, String> {
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

pub(crate) async fn extract_youtube_audio(
    app: &AppHandle,
    url: &str,
) -> Result<std::path::PathBuf, String> {
    let temp_dir = app.path().app_cache_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;

    let uuid_str = uuid::Uuid::new_v4().to_string();
    let output_template = temp_dir.join(format!("yt_audio_{}.%(ext)s", uuid_str));
    let output_path_str = output_template
        .to_str()
        .ok_or_else(|| "Invalid output path".to_string())?;

    let yt_dlp = ensure_yt_dlp(app).await.map_err(|e| {
        format!(
            "{} Make sure your network allows access to GitHub releases.",
            e
        )
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

    if let Some(ext) = path
        .extension()
        .and_then(|e| e.to_str().map(|s| s.to_lowercase()))
    {
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
pub async fn download_model(
    app: AppHandle,
    downloader: State<'_, DownloaderState>,
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
    model_id: String,
) -> CommandResult<String> {
    let db = db.0.clone();
    let license_manager = license_manager.0.clone();
    let downloader = downloader.0.clone();

    crate::ensure_app_access_verified(&db, &license_manager).await?;

    let app_clone = app.clone();
    let model_id_clone = model_id.clone();

    let model_path = downloader
        .download_model(
            &model_id,
            move |progress: crate::downloader::DownloadProgress| {
                let _ = app_clone.emit("download-progress", progress);
            },
        )
        .await
        .map_err(CommandError::Download)?;

    let path_str = model_path.to_str().unwrap().to_string();
    db.set_model_downloaded(&model_id_clone, true, Some(&path_str))
        .map_err(CommandError::Database)?;

    Ok(path_str)
}

#[tauri::command]
pub async fn delete_model(
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
pub fn cancel_model_download(downloader: State<'_, DownloaderState>, model_id: String) -> bool {
    downloader.0.cancel_download(&model_id)
}

#[tauri::command]
pub fn is_model_downloaded(
    _db: State<DbState>,
    downloader: State<DownloaderState>,
    model_id: String,
) -> CommandResult<bool> {
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
pub fn get_downloaded_models(downloader: State<DownloaderState>) -> Vec<String> {
    downloader.0.get_downloaded_models()
}

#[tauri::command]
pub fn get_model_path(
    downloader: State<DownloaderState>,
    model_id: String,
) -> CommandResult<String> {
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
