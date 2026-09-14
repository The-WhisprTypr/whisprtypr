use vox_ai_lib::downloader::ModelDownloader;
use vox_ai_lib::transcription::{get_model_filename, get_parakeet_files};

#[test]
fn downloader_resolves_model_paths_from_metadata() {
    let dir = tempfile::tempdir().unwrap();
    let downloader = ModelDownloader::new(dir.path().to_path_buf());

    assert_eq!(
        downloader.get_model_path("base"),
        dir.path().join(get_model_filename("base"))
    );
    assert_eq!(
        downloader.get_model_path("parakeet-v3"),
        dir.path().join(get_model_filename("parakeet-v3"))
    );
}

#[test]
fn single_file_model_download_state_uses_file_existence() {
    let dir = tempfile::tempdir().unwrap();
    let downloader = ModelDownloader::new(dir.path().to_path_buf());

    assert!(!downloader.is_model_downloaded("base"));

    std::fs::write(downloader.get_model_path("base"), b"fake model").unwrap();

    assert!(downloader.is_model_downloaded("base"));
    assert_eq!(downloader.get_downloaded_models(), vec!["base".to_string()]);
}

#[test]
fn directory_model_download_state_requires_all_manifest_files() {
    let dir = tempfile::tempdir().unwrap();
    let downloader = ModelDownloader::new(dir.path().to_path_buf());
    let model_dir = downloader.get_model_path("parakeet-v3");
    std::fs::create_dir_all(&model_dir).unwrap();

    let files = get_parakeet_files("parakeet-v3").unwrap();
    for file in files.iter().take(files.len() - 1) {
        std::fs::write(model_dir.join(file.filename), b"fake").unwrap();
    }
    assert!(!downloader.is_model_downloaded("parakeet-v3"));

    let last = files.last().unwrap();
    std::fs::write(model_dir.join(last.filename), b"fake").unwrap();

    assert!(downloader.is_model_downloaded("parakeet-v3"));
}

#[test]
fn cancel_download_returns_false_when_no_download_is_active() {
    let dir = tempfile::tempdir().unwrap();
    let downloader = ModelDownloader::new(dir.path().to_path_buf());

    assert!(!downloader.cancel_download("base"));
}

#[tokio::test]
async fn delete_model_removes_file_or_directory_models() {
    let dir = tempfile::tempdir().unwrap();
    let downloader = ModelDownloader::new(dir.path().to_path_buf());

    let base_path = downloader.get_model_path("base");
    std::fs::write(&base_path, b"fake").unwrap();
    downloader.delete_model("base").await.unwrap();
    assert!(!base_path.exists());

    let parakeet_path = downloader.get_model_path("parakeet-v3");
    std::fs::create_dir_all(&parakeet_path).unwrap();
    std::fs::write(parakeet_path.join("vocab.txt"), b"fake").unwrap();
    downloader.delete_model("parakeet-v3").await.unwrap();
    assert!(!parakeet_path.exists());
}
