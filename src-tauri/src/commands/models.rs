use crate::{database::WhisperModel, CommandResult, DbState, DownloaderState};
use tauri::State;

#[tauri::command]
pub fn get_models(
    db: State<DbState>,
    downloader: State<DownloaderState>,
) -> CommandResult<Vec<WhisperModel>> {
    let mut models = db.0.get_models().map_err(crate::CommandError::Database)?;

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
                .map_err(crate::CommandError::Database)?;
            model.downloaded = exists_on_disk;
            model.download_path = path;
        }
    }

    Ok(models)
}

#[tauri::command]
pub fn get_model(
    db: State<DbState>,
    downloader: State<DownloaderState>,
    id: String,
) -> CommandResult<Option<WhisperModel>> {
    let mut model = db.0.get_model(&id).map_err(crate::CommandError::Database)?;

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
                .map_err(crate::CommandError::Database)?;
            model.downloaded = exists_on_disk;
            model.download_path = path;
        }
    }

    Ok(model)
}

#[tauri::command]
pub fn set_model_downloaded(
    db: State<DbState>,
    id: String,
    downloaded: bool,
    path: Option<String>,
) -> CommandResult<()> {
    db.0.set_model_downloaded(&id, downloaded, path.as_deref())
        .map_err(Into::into)
}

#[tauri::command]
pub fn set_selected_model(db: State<DbState>, model_id: Option<String>) -> CommandResult<()> {
    db.0.set_selected_model(model_id.as_deref())
        .map_err(Into::into)
}
