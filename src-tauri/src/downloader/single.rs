use crate::downloader::{DownloadProgress, ModelDownloader};
use futures_util::StreamExt;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub async fn download_model_inner<F>(
    downloader: &ModelDownloader,
    model_id: &str,
    progress_callback: F,
    cancel_token: Arc<AtomicBool>,
) -> Result<std::path::PathBuf, String>
where
    F: Fn(DownloadProgress) + Send + 'static,
{
    if crate::transcription::get_parakeet_files(model_id).is_some() {
        return crate::downloader::directory::download_directory_model(
            downloader,
            model_id,
            "Parakeet",
            progress_callback,
            cancel_token,
        )
        .await;
    }

    if crate::transcription::get_qwen3_asr_files(model_id).is_some() {
        return crate::downloader::directory::download_directory_model(
            downloader,
            model_id,
            "Qwen3-ASR",
            progress_callback,
            cancel_token,
        )
        .await;
    }

    let original_url = crate::transcription::get_model_url(model_id)
        .ok_or_else(|| format!("Unknown model: {}", model_id))?;

    let url = downloader
        .test_url_override
        .clone()
        .unwrap_or_else(|| original_url.to_string());

    if !url.starts_with("https://") && downloader.test_url_override.is_none() {
        return Err("Security error: Only HTTPS URLs are allowed for downloads".to_string());
    }

    fs::create_dir_all(&downloader.models_dir)
        .await
        .map_err(|e| format!("Failed to create models directory: {}", e))?;

    let model_path = downloader.get_model_path(model_id);
    let temp_path = model_path.with_extension("bin.tmp");

    let response = downloader
        .client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to start download: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed with status: {}",
            response.status()
        ));
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let mut file = fs::File::create(&temp_path)
        .await
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        if ModelDownloader::is_cancelled(&cancel_token) {
            drop(file);
            let _ = fs::remove_file(&temp_path).await;
            return Err("Download cancelled".to_string());
        }

        let chunk = chunk.map_err(|e| format!("Download error: {}", e))?;

        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Failed to write chunk: {}", e))?;

        downloaded += chunk.len() as u64;

        let percentage = if total_size > 0 {
            (downloaded as f32 / total_size as f32) * 100.0
        } else {
            0.0
        };

        progress_callback(DownloadProgress {
            model_id: model_id.to_string(),
            bytes_downloaded: downloaded,
            total_bytes: total_size,
            percentage,
        });
    }

    file.flush()
        .await
        .map_err(|e| format!("Failed to flush file: {}", e))?;

    fs::rename(&temp_path, &model_path)
        .await
        .map_err(|e| format!("Failed to rename temp file: {}", e))?;

    Ok(model_path)
}
