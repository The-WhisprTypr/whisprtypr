use crate::{
    cloud_transcription,
    commands::download::{download_url_to_temp, extract_youtube_audio},
    transcription::Transcriber,
    utils::{
        is_model_language_supported, is_valid_language_code, path_has_extension, read_audio_file,
        sanitize_url, AUDIO_FILE_EXTENSIONS,
    },
    CommandError, CommandResult, DbState, DownloaderState, LicenseManagerState, RecorderState,
    TranscriberState, TranscriptionRateLimiter,
};
use log::{debug, error, info, warn};
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub async fn load_model(
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

    crate::ensure_app_access_verified(&db, &license_manager).await?;

    if model_id.starts_with("cloud:") {
        let mut transcriber_guard = transcriber.lock().unwrap();
        *transcriber_guard = None;
        info!(
            "Cloud model selected: {} (language: {})",
            model_id, language
        );
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

    {
        let mut transcriber_guard = transcriber.lock().unwrap();
        *transcriber_guard = None;
        drop(transcriber_guard);
    }

    let mut new_transcriber = Transcriber::new(&model_id, model_path.to_str().unwrap(), &language)
        .map_err(CommandError::Transcription)?;

    if let Err(e) = new_transcriber.warm_up() {
        warn!(
            "Model warm-up failed (first dictation may be slower): {}",
            e
        );
    }

    let mut transcriber_guard = transcriber.lock().unwrap();
    *transcriber_guard = Some(new_transcriber);

    info!("Model loaded: {} (language: {})", model_id, language);

    Ok(())
}

#[tauri::command]
pub fn unload_model(transcriber: State<TranscriberState>) -> CommandResult<()> {
    let mut transcriber_guard = transcriber.0.lock().unwrap();
    *transcriber_guard = None;
    info!("Model unloaded");
    Ok(())
}

#[derive(serde::Serialize, Clone)]
pub struct LoadedModelInfo {
    model_id: String,
    language: String,
}

#[tauri::command]
pub fn get_loaded_model(transcriber: State<TranscriberState>) -> Option<LoadedModelInfo> {
    let transcriber_guard = transcriber.0.lock().unwrap();
    transcriber_guard.as_ref().map(|t| LoadedModelInfo {
        model_id: t.model_id().to_string(),
        language: t.language().to_string(),
    })
}

#[tauri::command]
pub async fn transcribe_audio(
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
    transcriber: State<'_, TranscriberState>,
    audio_samples: Vec<f32>,
) -> CommandResult<String> {
    let db = db.0.clone();
    let license_manager = license_manager.0.clone();
    let transcriber = transcriber.0.clone();

    crate::ensure_app_access_verified(&db, &license_manager).await?;

    let settings = db.get_settings().map_err(CommandError::Database)?;
    if settings.selected_model_id.starts_with("cloud:") {
        if let Some((provider, model)) =
            cloud_transcription::parse_cloud_model_id(&settings.selected_model_id)
        {
            let wav_bytes = cloud_transcription::encode_samples_to_wav(&audio_samples)
                .map_err(CommandError::Recording)?;
            let text = cloud_transcription::transcribe_with_cloud(
                &db,
                provider,
                model,
                wav_bytes,
                &settings.language,
                false,
            )
            .await
            .map_err(CommandError::Transcription)?;
            return Ok(text);
        }
    }

    tokio::task::spawn_blocking(move || {
        let mut transcriber_guard = transcriber.lock().unwrap();
        if let Some(ref mut t) = *transcriber_guard {
            t.transcribe(&audio_samples)
                .map_err(CommandError::Transcription)
        } else {
            Err(CommandError::Transcription("No model loaded".to_string()))
        }
    })
    .await
    .map_err(|_| CommandError::Transcription("Transcription task panicked".to_string()))?
}

#[tauri::command]
pub async fn record_and_transcribe(
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
    recorder: State<'_, RecorderState>,
    transcriber: State<'_, TranscriberState>,
) -> CommandResult<String> {
    let _db = db.0.clone();
    let license_manager = license_manager.0.clone();
    let recorder = recorder.0.clone();
    let transcriber = transcriber.0.clone();

    crate::ensure_app_access_verified(&_db, &license_manager).await?;

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

    let settings = _db.get_settings().map_err(CommandError::Database)?;
    if settings.selected_model_id.starts_with("cloud:") {
        if let Some((provider, model)) =
            cloud_transcription::parse_cloud_model_id(&settings.selected_model_id)
        {
            let wav_bytes = cloud_transcription::encode_samples_to_wav(&samples)
                .map_err(CommandError::Recording)?;
            let text = cloud_transcription::transcribe_with_cloud(
                &_db,
                provider,
                model,
                wav_bytes,
                &settings.language,
                false,
            )
            .await
            .map_err(CommandError::Transcription)?;
            return Ok(text);
        }
    }

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

#[tauri::command]
pub async fn translate_text(
    _db: State<'_, DbState>,
    app: AppHandle,
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

    let sanitized =
        crate::utils::sanitize_text(&text, 100_000).map_err(CommandError::PostProcessing)?;
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

    let client = cloud_transcription::build_http_client(30).map_err(CommandError::Transcription)?;
    let result = crate::translation::translate(
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
pub async fn record_and_translate(
    db: State<'_, DbState>,
    license_manager: State<'_, LicenseManagerState>,
    recorder: State<'_, RecorderState>,
    transcriber: State<'_, TranscriberState>,
    app: AppHandle,
    source_language: String,
    target_language: String,
    api_key: Option<String>,
) -> CommandResult<String> {
    let db = db.0.clone();
    let license_manager = license_manager.0.clone();
    let recorder = recorder.0.clone();
    let transcriber = transcriber.0.clone();

    crate::ensure_app_access_verified(&db, &license_manager).await?;

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
        if let Some((provider, model)) =
            cloud_transcription::parse_cloud_model_id(&settings.selected_model_id)
        {
            let wav_bytes = cloud_transcription::encode_samples_to_wav(&samples)
                .map_err(CommandError::Recording)?;
            cloud_transcription::transcribe_with_cloud(
                &db,
                provider,
                model,
                wav_bytes,
                &source_language,
                false,
            )
            .await
            .map_err(CommandError::Transcription)?
        } else {
            return Err(CommandError::Transcription("No model loaded".to_string()));
        }
    } else {
        let mut transcriber_guard = transcriber.lock().unwrap();
        if let Some(ref mut t) = *transcriber_guard {
            t.transcribe(&samples)
                .map_err(CommandError::Transcription)?
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

        text = crate::translation::translate(
            &cloud_transcription::build_http_client(30).map_err(CommandError::Transcription)?,
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
pub async fn transcribe_file(
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

    crate::ensure_app_access_verified(&db, &license_manager).await?;

    if !rate_limiter.check("transcribe_file") {
        return Err(CommandError::Transcription(
            "Rate limit exceeded. Please wait before transcribing another file.".to_string(),
        ));
    }

    let safe_path = crate::utils::canonicalize_existing_file_path(&file_path)
        .map_err(CommandError::Transcription)?;

    if !path_has_extension(&safe_path, AUDIO_FILE_EXTENSIONS) {
        return Err(CommandError::Transcription(
            "Unsupported audio format. Please use WAV, MP3, M4A, OGG, FLAC, AAC, or WebM."
                .to_string(),
        ));
    }

    let metadata = std::fs::metadata(&safe_path)
        .map_err(|e| CommandError::Transcription(format!("Cannot read file: {}", e)))?;
    if metadata.len() > 500 * 1024 * 1024 {
        return Err(CommandError::Transcription(
            "File too large. Maximum size is 500MB.".to_string(),
        ));
    }

    let samples = read_audio_file(&safe_path)
        .map_err(|e| CommandError::Transcription(format!("Failed to read audio file: {}", e)))?;

    let settings = db.get_settings().map_err(CommandError::Database)?;
    if settings.selected_model_id.starts_with("cloud:") {
        if let Some((provider, model)) =
            cloud_transcription::parse_cloud_model_id(&settings.selected_model_id)
        {
            let wav_bytes = cloud_transcription::encode_samples_to_wav(&samples)
                .map_err(CommandError::Recording)?;
            let text = cloud_transcription::transcribe_with_cloud(
                &db,
                provider,
                model,
                wav_bytes,
                &settings.language,
                false,
            )
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
pub async fn transcribe_url(
    app: AppHandle,
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

    crate::ensure_app_access_verified(&db, &license_manager).await?;

    if !rate_limiter.check("transcribe_url") {
        return Err(CommandError::Transcription(
            "Rate limit exceeded. Please wait before transcribing another URL.".to_string(),
        ));
    }

    let safe_url = sanitize_url(&url).map_err(CommandError::Transcription)?;
    crate::utils::validate_url_host(&safe_url)
        .await
        .map_err(CommandError::Transcription)?;
    let settings = db.get_settings().map_err(CommandError::Database)?;

    if let Some((provider, model)) =
        cloud_transcription::parse_cloud_model_id(&settings.selected_model_id)
    {
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

    let temp_path = if crate::utils::is_youtube_url(&safe_url) {
        extract_youtube_audio(&app, &safe_url)
            .await
            .map_err(CommandError::Transcription)?
    } else {
        download_url_to_temp(&app, &safe_url)
            .await
            .map_err(CommandError::Transcription)?
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
        if let Some((provider, model)) =
            cloud_transcription::parse_cloud_model_id(&settings.selected_model_id)
        {
            let wav_bytes = cloud_transcription::encode_samples_to_wav(&samples)
                .map_err(CommandError::Recording)?;
            let text = cloud_transcription::transcribe_with_cloud(
                &db,
                provider,
                model,
                wav_bytes,
                &settings.language,
                enable_speaker_detection,
            )
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
pub async fn transcribe_files_batch(
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

    crate::ensure_app_access_verified(&db, &license_manager).await?;

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
        let safe_path = crate::utils::canonicalize_existing_file_path(&file_path)
            .map_err(CommandError::Transcription)?;

        if !path_has_extension(&safe_path, AUDIO_FILE_EXTENSIONS) {
            results.push(format!(
                "Skipped {}: unsupported format",
                safe_path.display()
            ));
            continue;
        }

        let metadata = std::fs::metadata(&safe_path).map_err(|e| {
            CommandError::Transcription(format!("Cannot read file {}: {}", safe_path.display(), e))
        })?;
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
            if let Some((provider, model)) =
                cloud_transcription::parse_cloud_model_id(&settings.selected_model_id)
            {
                let wav_bytes = match cloud_transcription::encode_samples_to_wav(&samples) {
                    Ok(b) => b,
                    Err(e) => {
                        results.push(format!("Failed to encode {}: {}", safe_path.display(), e));
                        continue;
                    }
                };
                match cloud_transcription::transcribe_with_cloud(
                    &db,
                    provider,
                    model,
                    wav_bytes,
                    &settings.language,
                    false,
                )
                .await
                {
                    Ok(t) => t,
                    Err(e) => {
                        results.push(format!(
                            "Transcription failed for {}: {}",
                            safe_path.display(),
                            e
                        ));
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
                        results.push(format!(
                            "Transcription failed for {}: {}",
                            safe_path.display(),
                            e
                        ));
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
