use futures_util::StreamExt;
use reqwest::Client;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub struct ModelDownloader {
    client: Client,
    models_dir: PathBuf,
    cancel_tokens: Mutex<HashMap<String, Arc<AtomicBool>>>,
    pub test_url_override: Option<String>,
}

#[derive(Clone, serde::Serialize)]
pub struct DownloadProgress {
    pub model_id: String,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub percentage: f32,
}

impl ModelDownloader {
    pub fn new(models_dir: PathBuf) -> Self {
        Self {
            client: Client::new(),
            models_dir,
            cancel_tokens: Mutex::new(HashMap::new()),
            test_url_override: None,
        }
    }

    fn create_cancel_token(&self, model_id: &str) -> Arc<AtomicBool> {
        let token = Arc::new(AtomicBool::new(false));
        self.cancel_tokens
            .lock()
            .unwrap()
            .insert(model_id.to_string(), token.clone());
        token
    }

    fn clear_cancel_token(&self, model_id: &str) {
        self.cancel_tokens.lock().unwrap().remove(model_id);
    }

    fn is_cancelled(cancel_token: &AtomicBool) -> bool {
        cancel_token.load(Ordering::SeqCst)
    }

    pub fn cancel_download(&self, model_id: &str) -> bool {
        if let Some(token) = self.cancel_tokens.lock().unwrap().get(model_id) {
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
        if let Some(files) = crate::transcription::get_parakeet_files(model_id)
            .or_else(|| crate::transcription::get_qwen3_asr_files(model_id))
        {
            let model_dir = self.get_model_path(model_id);
            let all_downloaded = model_dir.is_dir()
                && files
                    .iter()
                    .all(|file| model_dir.join(file.filename).is_file());

            if model_id == "qwen3-asr-0.6b" {
                return all_downloaded
                    && model_dir.join("tokenizer.json").is_file()
                    && model_dir
                        .join("model.safetensors")
                        .metadata()
                        .map(|metadata| metadata.len() >= 1_800_000_000)
                        .unwrap_or(false);
            }

            return all_downloaded;
        }

        self.get_model_path(model_id).exists()
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
        let result = self
            .download_model_inner(model_id, progress_callback, cancel_token)
            .await;
        self.clear_cancel_token(model_id);
        result
    }

    async fn download_model_inner<F>(
        &self,
        model_id: &str,
        progress_callback: F,
        cancel_token: Arc<AtomicBool>,
    ) -> Result<PathBuf, String>
    where
        F: Fn(DownloadProgress) + Send + 'static,
    {
        if crate::transcription::get_parakeet_files(model_id).is_some() {
            return self
                .download_directory_model(model_id, "Parakeet", progress_callback, cancel_token)
                .await;
        }

        if crate::transcription::get_qwen3_asr_files(model_id).is_some() {
            return self
                .download_directory_model(model_id, "Qwen3-ASR", progress_callback, cancel_token)
                .await;
        }

        let original_url = crate::transcription::get_model_url(model_id)
            .ok_or_else(|| format!("Unknown model: {}", model_id))?;

        let url = self
            .test_url_override
            .clone()
            .unwrap_or_else(|| original_url.to_string());

        // Security: Enforce HTTPS only
        if !url.starts_with("https://") && self.test_url_override.is_none() {
            return Err("Security error: Only HTTPS URLs are allowed for downloads".to_string());
        }

        // Create models directory if it doesn't exist
        tokio::fs::create_dir_all(&self.models_dir)
            .await
            .map_err(|e| format!("Failed to create models directory: {}", e))?;

        let model_path = self.get_model_path(model_id);
        let temp_path = model_path.with_extension("bin.tmp");

        // Start download
        let response = self
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

        let mut file = File::create(&temp_path)
            .await
            .map_err(|e| format!("Failed to create temp file: {}", e))?;

        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            if Self::is_cancelled(&cancel_token) {
                drop(file);
                let _ = tokio::fs::remove_file(&temp_path).await;
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

        // Rename temp file to final path
        tokio::fs::rename(&temp_path, &model_path)
            .await
            .map_err(|e| format!("Failed to rename temp file: {}", e))?;

        Ok(model_path)
    }

    async fn download_directory_model<F>(
        &self,
        model_id: &str,
        model_name: &str,
        progress_callback: F,
        cancel_token: Arc<AtomicBool>,
    ) -> Result<PathBuf, String>
    where
        F: Fn(DownloadProgress) + Send + 'static,
    {
        let files = crate::transcription::get_parakeet_files(model_id)
            .or_else(|| crate::transcription::get_qwen3_asr_files(model_id))
            .ok_or_else(|| format!("Unknown {} model: {}", model_name, model_id))?;

        tokio::fs::create_dir_all(&self.models_dir)
            .await
            .map_err(|e| format!("Failed to create models directory: {}", e))?;

        let model_dir = self.get_model_path(model_id);
        tokio::fs::create_dir_all(&model_dir)
            .await
            .map_err(|e| format!("Failed to create {} model directory: {}", model_name, e))?;

        for file in files {
            if !file.url.starts_with("https://") {
                return Err("Security error: Only HTTPS URLs are allowed for downloads".to_string());
            }
        }

        let mut total_size = expected_directory_model_size(model_id).unwrap_or(0);
        if total_size == 0 {
            for file in files {
                if let Ok(response) = self.client.head(file.url).send().await {
                    if response.status().is_success() {
                        total_size =
                            total_size.saturating_add(response.content_length().unwrap_or(0));
                    }
                }
            }
        }

        let mut total_downloaded = 0u64;

        for file in files {
            if Self::is_cancelled(&cancel_token) {
                let _ = tokio::fs::remove_dir_all(&model_dir).await;
                return Err("Download cancelled".to_string());
            }

            let final_path = model_dir.join(file.filename);
            let temp_path = final_path.with_extension("tmp");

            let response =
                self.client.get(file.url).send().await.map_err(|e| {
                    format!("Failed to start download for {}: {}", file.filename, e)
                })?;

            if !response.status().is_success() {
                return Err(format!(
                    "Download failed for {} with status: {}",
                    file.filename,
                    response.status()
                ));
            }

            let file_size = response.content_length().unwrap_or(0);
            if total_size == 0 {
                total_size = total_size.saturating_add(file_size);
            }

            let mut output = File::create(&temp_path)
                .await
                .map_err(|e| format!("Failed to create temp file for {}: {}", file.filename, e))?;

            let mut stream = response.bytes_stream();
            while let Some(chunk) = stream.next().await {
                if Self::is_cancelled(&cancel_token) {
                    drop(output);
                    let _ = tokio::fs::remove_file(&temp_path).await;
                    let _ = tokio::fs::remove_dir_all(&model_dir).await;
                    return Err("Download cancelled".to_string());
                }

                let chunk =
                    chunk.map_err(|e| format!("Download error for {}: {}", file.filename, e))?;

                output
                    .write_all(&chunk)
                    .await
                    .map_err(|e| format!("Failed to write {}: {}", file.filename, e))?;

                total_downloaded = total_downloaded.saturating_add(chunk.len() as u64);

                let percentage = if total_size > 0 {
                    (total_downloaded as f32 / total_size as f32 * 100.0).min(100.0)
                } else {
                    0.0
                };

                progress_callback(DownloadProgress {
                    model_id: model_id.to_string(),
                    bytes_downloaded: total_downloaded,
                    total_bytes: total_size,
                    percentage,
                });
            }

            output
                .flush()
                .await
                .map_err(|e| format!("Failed to flush {}: {}", file.filename, e))?;

            tokio::fs::rename(&temp_path, &final_path)
                .await
                .map_err(|e| format!("Failed to finalize {}: {}", file.filename, e))?;
        }

        validate_directory_model(model_id, &model_dir).await?;

        if model_id == "qwen3-asr-0.6b" {
            self.generate_qwen3_asr_tokenizer(&model_dir).await?;
        }

        progress_callback(DownloadProgress {
            model_id: model_id.to_string(),
            bytes_downloaded: total_downloaded,
            total_bytes: total_size,
            percentage: 100.0,
        });

        Ok(model_dir)
    }

    async fn generate_qwen3_asr_tokenizer(
        &self,
        model_dir: &std::path::Path,
    ) -> Result<(), String> {
        let tokenizer_config = tokio::fs::read_to_string(model_dir.join("tokenizer_config.json"))
            .await
            .map_err(|e| format!("Failed to read Qwen3-ASR tokenizer config: {}", e))?;
        let vocab = tokio::fs::read_to_string(model_dir.join("vocab.json"))
            .await
            .map_err(|e| format!("Failed to read Qwen3-ASR vocab: {}", e))?;
        let merges = tokio::fs::read_to_string(model_dir.join("merges.txt"))
            .await
            .map_err(|e| format!("Failed to read Qwen3-ASR merges: {}", e))?;

        let tokenizer_json = build_qwen3_asr_tokenizer_json(&vocab, &merges, &tokenizer_config)
            .map_err(|e| format!("Failed to build Qwen3-ASR tokenizer: {}", e))?;

        tokio::fs::write(model_dir.join("tokenizer.json"), tokenizer_json)
            .await
            .map_err(|e| format!("Failed to write Qwen3-ASR tokenizer: {}", e))?;

        Ok(())
    }

    pub async fn delete_model(&self, model_id: &str) -> Result<(), String> {
        let model_path = self.get_model_path(model_id);

        if model_path.is_dir() {
            tokio::fs::remove_dir_all(&model_path)
                .await
                .map_err(|e| format!("Failed to delete model: {}", e))?;
        } else if model_path.exists() {
            tokio::fs::remove_file(&model_path)
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

fn expected_directory_model_size(model_id: &str) -> Option<u64> {
    match model_id {
        // Hugging Face/Xet does not always expose a useful Content-Length for
        // the large safetensors redirect, so use the published file sizes for
        // stable progress reporting.
        "qwen3-asr-0.6b" => Some(
            1_880_000_000 // model.safetensors
                + 1_671_853 // merges.txt
                + 2_780_000 // vocab.json
                + 6_193 // config.json
                + 12_500 // tokenizer_config.json
                + 1_161 // chat_template.json
                + 330 // preprocessor_config.json
                + 142, // generation_config.json
        ),
        _ => None,
    }
}

async fn validate_directory_model(
    model_id: &str,
    model_dir: &std::path::Path,
) -> Result<(), String> {
    if model_id == "qwen3-asr-0.6b" {
        let weights_path = model_dir.join("model.safetensors");
        let weights_size = tokio::fs::metadata(&weights_path)
            .await
            .map_err(|e| format!("Qwen3-ASR weights are missing: {}", e))?
            .len();

        if weights_size < 1_800_000_000 {
            return Err(format!(
                "Qwen3-ASR weights download is incomplete: expected about 1.88 GB, got {:.2} MB",
                weights_size as f64 / 1_048_576.0
            ));
        }
    }

    Ok(())
}

fn build_qwen3_asr_tokenizer_json(
    vocab: &str,
    merges: &str,
    tokenizer_config: &str,
) -> Result<Vec<u8>, serde_json::Error> {
    let vocab_value: serde_json::Value = serde_json::from_str(vocab)?;
    let merges_value: Vec<&str> = merges
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .collect();

    let tokenizer_config_value: serde_json::Value = serde_json::from_str(tokenizer_config)?;
    let mut added_tokens: Vec<serde_json::Value> = Vec::new();

    if let Some(decoder_map) = tokenizer_config_value["added_tokens_decoder"].as_object() {
        let mut entries: Vec<(u64, &serde_json::Value)> = decoder_map
            .iter()
            .filter_map(|(key, value)| key.parse::<u64>().ok().map(|id| (id, value)))
            .collect();

        entries.sort_by_key(|(id, _)| *id);

        for (id, value) in entries {
            added_tokens.push(serde_json::json!({
                "id": id,
                "content": value["content"],
                "single_word": false,
                "lstrip": false,
                "rstrip": false,
                "normalized": false,
                "special": value["special"]
            }));
        }
    }

    let tokenizer_json = serde_json::json!({
        "version": "1.0",
        "truncation": null,
        "padding": null,
        "added_tokens": added_tokens,
        "normalizer": { "type": "NFC" },
        "pre_tokenizer": {
            "type": "Sequence",
            "pretokenizers": [
                {
                    "type": "Split",
                    "pattern": { "Regex": "(?i:'s|'t|'re|'ve|'m|'ll|'d)|[^\\r\\n\\p{L}\\p{N}]?\\p{L}+|\\p{N}| ?[^\\s\\p{L}\\p{N}]+[\\r\\n]*|\\s*[\\r\\n]+|\\s+(?!\\S)|\\s+" },
                    "behavior": "Isolated",
                    "invert": false
                },
                {
                    "type": "ByteLevel",
                    "add_prefix_space": false,
                    "trim_offsets": false,
                    "use_regex": false
                }
            ]
        },
        "post_processor": {
            "type": "ByteLevel",
            "add_prefix_space": false,
            "trim_offsets": false,
            "use_regex": false
        },
        "decoder": {
            "type": "ByteLevel",
            "add_prefix_space": false,
            "trim_offsets": false,
            "use_regex": false
        },
        "model": {
            "type": "BPE",
            "dropout": null,
            "unk_token": null,
            "continuing_subword_prefix": "",
            "end_of_word_suffix": "",
            "fuse_unk": false,
            "byte_fallback": false,
            "ignore_merges": false,
            "vocab": vocab_value,
            "merges": merges_value
        }
    });

    serde_json::to_vec(&tokenizer_json)
}
