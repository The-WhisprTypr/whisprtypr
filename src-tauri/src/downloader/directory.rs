use crate::downloader::{DownloadProgress, ModelDownloader};
use futures_util::StreamExt;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub async fn download_directory_model<F>(
    downloader: &ModelDownloader,
    model_id: &str,
    model_name: &str,
    progress_callback: F,
    cancel_token: Arc<AtomicBool>,
) -> Result<std::path::PathBuf, String>
where
    F: Fn(DownloadProgress) + Send + 'static,
{
    let files = crate::transcription::get_parakeet_files(model_id)
        .or_else(|| crate::transcription::get_qwen3_asr_files(model_id))
        .ok_or_else(|| format!("Unknown {} model: {}", model_name, model_id))?;

    fs::create_dir_all(&downloader.models_dir)
        .await
        .map_err(|e| format!("Failed to create models directory: {}", e))?;

    let model_dir = downloader.get_model_path(model_id);
    fs::create_dir_all(&model_dir)
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
            if let Ok(response) = downloader.client.head(file.url).send().await {
                if response.status().is_success() {
                    total_size = total_size.saturating_add(response.content_length().unwrap_or(0));
                }
            }
        }
    }

    let mut total_downloaded = 0u64;

    for file in files {
        if ModelDownloader::is_cancelled(&cancel_token) {
            let _ = fs::remove_dir_all(&model_dir).await;
            return Err("Download cancelled".to_string());
        }

        let final_path = model_dir.join(file.filename);
        let temp_path = final_path.with_extension("tmp");

        let response = downloader
            .client
            .get(file.url)
            .send()
            .await
            .map_err(|e| format!("Failed to start download for {}: {}", file.filename, e))?;

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

        let mut output = fs::File::create(&temp_path)
            .await
            .map_err(|e| format!("Failed to create temp file for {}: {}", file.filename, e))?;

        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            if ModelDownloader::is_cancelled(&cancel_token) {
                drop(output);
                let _ = fs::remove_file(&temp_path).await;
                let _ = fs::remove_dir_all(&model_dir).await;
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

        fs::rename(&temp_path, &final_path)
            .await
            .map_err(|e| format!("Failed to finalize {}: {}", file.filename, e))?;
    }

    validate_directory_model(model_id, &model_dir).await?;

    if model_id == "qwen3-asr-0.6b" {
        crate::downloader::directory::generate_qwen3_asr_tokenizer(downloader, &model_dir).await?;
    }

    progress_callback(DownloadProgress {
        model_id: model_id.to_string(),
        bytes_downloaded: total_downloaded,
        total_bytes: total_size,
        percentage: 100.0,
    });

    Ok(model_dir)
}

pub fn is_model_downloaded(downloader: &ModelDownloader, model_id: &str) -> bool {
    if let Some(files) = crate::transcription::get_parakeet_files(model_id)
        .or_else(|| crate::transcription::get_qwen3_asr_files(model_id))
    {
        let model_dir = downloader.get_model_path(model_id);
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

    downloader.get_model_path(model_id).exists()
}

pub fn expected_directory_model_size(model_id: &str) -> Option<u64> {
    match model_id {
        "qwen3-asr-0.6b" => {
            Some(1_880_000_000 + 1_671_853 + 2_780_000 + 6_193 + 12_500 + 1_161 + 330 + 142)
        }
        _ => None,
    }
}

pub async fn validate_directory_model(
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

pub async fn generate_qwen3_asr_tokenizer(
    _downloader: &ModelDownloader,
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

pub fn build_qwen3_asr_tokenizer_json(
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
