use crate::{CommandResult, DbState};
use tauri::State;

#[tauri::command]
pub fn get_cloud_providers(
    db: State<'_, DbState>,
) -> CommandResult<Vec<crate::cloud_transcription::CloudProviderInfo>> {
    let records =
        db.0.get_cloud_providers()
            .map_err(crate::CommandError::Database)?;
    let mut map: std::collections::HashMap<String, crate::database::CloudProviderRecord> =
        records.into_iter().map(|r| (r.id.clone(), r)).collect();

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
            let decrypted_key = crate::cloud_transcription::decrypt_api_key(&record.api_key);
            let has_key = !decrypted_key.trim().is_empty() || id == "custom";
            let masked = if has_key && !decrypted_key.trim().is_empty() {
                crate::cloud_transcription::mask_key(&decrypted_key)
            } else {
                String::new()
            };
            result.push(crate::cloud_transcription::CloudProviderInfo {
                id: id.to_string(),
                name: name.to_string(),
                configured: has_key,
                masked_key: masked,
                base_url: record.base_url,
                custom_model: record.custom_model,
            });
        } else {
            result.push(crate::cloud_transcription::CloudProviderInfo {
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
pub async fn save_cloud_provider(
    db: State<'_, DbState>,
    provider: String,
    api_key: String,
    base_url: Option<String>,
    custom_model: Option<String>,
) -> CommandResult<()> {
    let encrypted = crate::cloud_transcription::encrypt_api_key(&api_key)
        .map_err(|e| crate::CommandError::Transcription(format!("Encryption failed: {}", e)))?;
    let record = crate::database::CloudProviderRecord {
        id: provider,
        api_key: encrypted,
        base_url,
        custom_model,
        is_active: true,
    };
    db.0.save_cloud_provider(&record)
        .map_err(crate::CommandError::Database)?;
    Ok(())
}

#[tauri::command]
pub async fn delete_cloud_provider(db: State<'_, DbState>, provider: String) -> CommandResult<()> {
    db.0.delete_cloud_provider(&provider)
        .map_err(crate::CommandError::Database)?;
    Ok(())
}

#[tauri::command]
pub async fn test_cloud_connection(
    db: State<'_, DbState>,
    provider: String,
    api_key: Option<String>,
    base_url: Option<String>,
) -> CommandResult<String> {
    let key = if let Some(k) = api_key.filter(|k| !k.trim().is_empty()) {
        k
    } else if let Some(record) =
        db.0.get_cloud_provider(&provider)
            .map_err(crate::CommandError::Database)?
    {
        crate::cloud_transcription::decrypt_api_key(&record.api_key)
    } else {
        String::new()
    };

    let url = base_url.or_else(|| {
        db.0.get_cloud_provider(&provider)
            .ok()
            .flatten()
            .and_then(|r| r.base_url)
    });

    crate::cloud_transcription::test_provider_connection(&provider, &key, url.as_deref())
        .await
        .map_err(crate::CommandError::Transcription)
}

#[tauri::command]
pub async fn get_ai_formatting_providers(
    db: State<'_, DbState>,
) -> CommandResult<Vec<crate::ai_formatting::AiFormattingProviderInfo>> {
    let records =
        db.0.get_ai_formatting_providers()
            .map_err(crate::CommandError::Database)?;
    let mut map: std::collections::HashMap<String, crate::database::AiFormattingProviderRecord> =
        records.into_iter().map(|r| (r.id.clone(), r)).collect();

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
            let decrypted_key = crate::cloud_transcription::decrypt_api_key(&record.api_key);
            let has_key = !decrypted_key.trim().is_empty() || id == "custom";
            let masked = if has_key && !decrypted_key.trim().is_empty() {
                crate::cloud_transcription::mask_key(&decrypted_key)
            } else {
                String::new()
            };
            result.push(crate::ai_formatting::AiFormattingProviderInfo {
                id: id.to_string(),
                name: name.to_string(),
                configured: has_key,
                masked_key: masked,
                base_url: record.base_url,
                custom_model: record.custom_model,
            });
        } else {
            result.push(crate::ai_formatting::AiFormattingProviderInfo {
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
pub async fn save_ai_formatting_provider(
    db: State<'_, DbState>,
    provider: String,
    api_key: String,
    base_url: Option<String>,
    custom_model: Option<String>,
) -> CommandResult<()> {
    let encrypted = crate::cloud_transcription::encrypt_api_key(&api_key)
        .map_err(|e| crate::CommandError::Transcription(format!("Encryption failed: {}", e)))?;
    let record = crate::database::AiFormattingProviderRecord {
        id: provider,
        api_key: encrypted,
        base_url,
        custom_model,
        is_active: true,
    };
    db.0.save_ai_formatting_provider(&record)
        .map_err(crate::CommandError::Database)?;
    Ok(())
}

#[tauri::command]
pub async fn delete_ai_formatting_provider(
    db: State<'_, DbState>,
    provider: String,
) -> CommandResult<()> {
    db.0.delete_ai_formatting_provider(&provider)
        .map_err(crate::CommandError::Database)?;
    Ok(())
}

#[tauri::command]
pub async fn test_ai_formatting_connection(
    db: State<'_, DbState>,
    provider: String,
    api_key: Option<String>,
    base_url: Option<String>,
    model: Option<String>,
) -> CommandResult<String> {
    let key = if let Some(k) = api_key.filter(|k| !k.trim().is_empty()) {
        k
    } else if let Some(record) =
        db.0.get_ai_formatting_provider(&provider)
            .map_err(crate::CommandError::Database)?
    {
        crate::cloud_transcription::decrypt_api_key(&record.api_key)
    } else {
        String::new()
    };

    let url = base_url.or_else(|| {
        db.0.get_ai_formatting_provider(&provider)
            .ok()
            .flatten()
            .and_then(|r| r.base_url)
    });

    let model = model.or_else(|| {
        db.0.get_ai_formatting_provider(&provider)
            .ok()
            .flatten()
            .and_then(|r| r.custom_model)
    });

    crate::ai_formatting::test_ai_provider_connection(
        &provider,
        &key,
        url.as_deref(),
        model.as_deref(),
    )
    .await
    .map_err(crate::CommandError::Transcription)
}

#[tauri::command]
pub async fn format_text_with_ai(
    db: State<'_, DbState>,
    license_manager: State<'_, crate::LicenseManagerState>,
    text: String,
    style: String,
    provider: Option<String>,
    model: Option<String>,
) -> CommandResult<String> {
    let db = db.0.clone();
    let license_manager = license_manager.0.clone();

    crate::ensure_app_access_verified(&db, &license_manager).await?;

    if text.trim().is_empty() {
        return Ok(String::new());
    }

    let settings = db.get_settings().map_err(crate::CommandError::Database)?;

    let provider_id = provider.unwrap_or_else(|| settings.ai_formatting_provider_id.clone());
    let model_name = model.unwrap_or_else(|| settings.ai_formatting_model.clone());

    let record = db
        .get_ai_formatting_provider(&provider_id)
        .map_err(crate::CommandError::Database)?
        .ok_or_else(|| {
            crate::CommandError::Transcription(format!(
                "AI formatting provider '{}' is not configured. Please add an API key in AI Formatting settings.",
                provider_id
            ))
        })?;

    let api_key = crate::cloud_transcription::decrypt_api_key(&record.api_key);
    let base_url = record.base_url.as_deref();
    let custom_model = record.custom_model.as_deref().or(Some(&model_name));

    let style_prompt = crate::commands::utility::ai_formatting_style_prompt(&style);

    let formatted = crate::ai_formatting::format_text_with_ai(
        &provider_id,
        custom_model.unwrap_or("gpt-4o-mini"),
        &api_key,
        base_url,
        style_prompt,
        &text,
    )
    .await
    .map_err(crate::CommandError::Transcription)?;

    Ok(formatted)
}
