pub mod directory;
pub mod single;

use reqwest::Client;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::fs;

#[derive(Clone, serde::Serialize)]
pub struct DownloadProgress {
    pub model_id: String,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub percentage: f32,
}

pub struct ModelDownloader {
    pub(crate) client: Client,
    pub(crate) models_dir: PathBuf,
    pub(crate) cancel_tokens: Mutex<Vec<(String, Arc<AtomicBool>)>>,
    pub test_url_override: Option<String>,
}

impl ModelDownloader {
    pub fn new(models_dir: PathBuf) -> Self {
        Self {
            client: Client::new(),
            models_dir,
            cancel_tokens: Mutex::new(Vec::new()),
            test_url_override: None,
        }
    }

    pub(crate) fn create_cancel_token(&self, model_id: &str) -> Arc<AtomicBool> {
        let token = Arc::new(AtomicBool::new(false));
        self.cancel_tokens
            .lock()
            .unwrap()
            .push((model_id.to_string(), token.clone()));
        token
    }

    pub(crate) fn clear_cancel_token(&self, model_id: &str) {
        self.cancel_tokens
            .lock()
            .unwrap()
            .retain(|(id, _)| id != model_id);
    }

    pub(crate) fn is_cancelled(cancel_token: &AtomicBool) -> bool {
        cancel_token.load(Ordering::SeqCst)
    }

    pub fn cancel_download(&self, model_id: &str) -> bool {
        if let Some((_, token)) = self
            .cancel_tokens
            .lock()
            .unwrap()
            .iter()
            .find(|(id, _)| id == model_id)
        {
            token.store(true, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    pub fn get_model_path(&self, model_id: &str) -> PathBuf {
        self.models_dir
            .join(crate::transcription::get_model_filename(model_id))
    }

    pub fn is_model_downloaded(&self, model_id: &str) -> bool {
        directory::is_model_downloaded(self, model_id)
    }

    pub async fn download_model<F>(
        &self,
        model_id: &str,
        progress_callback: F,
    ) -> Result<PathBuf, String>
    where
        F: Fn(DownloadProgress) + Send + 'static,
    {
        let cancel_token = self.create_cancel_token(model_id);
        let result =
            single::download_model_inner(self, model_id, progress_callback, cancel_token).await;
        self.clear_cancel_token(model_id);
        result
    }

    pub async fn delete_model(&self, model_id: &str) -> Result<(), String> {
        let model_path = self.get_model_path(model_id);

        if model_path.is_dir() {
            fs::remove_dir_all(&model_path)
                .await
                .map_err(|e| format!("Failed to delete model: {}", e))?;
        } else if model_path.exists() {
            fs::remove_file(&model_path)
                .await
                .map_err(|e| format!("Failed to delete model: {}", e))?;
        }

        Ok(())
    }

    pub fn get_downloaded_models(&self) -> Vec<String> {
        let models = [
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
        models
            .iter()
            .filter(|&&id| self.is_model_downloaded(id))
            .map(|&s| s.to_string())
            .collect()
    }
}
