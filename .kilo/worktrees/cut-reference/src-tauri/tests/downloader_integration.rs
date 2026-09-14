use httpmock::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::fs;
use vox_ai_lib::downloader::{DownloadProgress, ModelDownloader};

#[tokio::test]
async fn downloader_fetches_file_and_updates_progress() {
    let server = MockServer::start();

    // Mock a normal binary file download
    let mock = server.mock(|when, then| {
        when.method(GET).path("/model.bin");
        then.status(200)
            .header("content-length", "1024")
            .body(vec![0u8; 1024]);
    });

    let dir = tempfile::tempdir().unwrap();
    let mut downloader = ModelDownloader::new(dir.path().to_path_buf());

    // Inject mock URL
    downloader.test_url_override = Some(server.url("/model.bin"));

    let progress_count = Arc::new(AtomicU64::new(0));
    let progress_count_clone = progress_count.clone();

    let result = downloader
        .download_model("base", move |progress: DownloadProgress| {
            progress_count_clone.fetch_add(1, Ordering::SeqCst);
            assert_eq!(progress.total_bytes, 1024);
        })
        .await;

    mock.assert();
    assert!(result.is_ok());

    let final_path = result.unwrap();
    assert!(final_path.exists());

    let metadata = fs::metadata(&final_path).await.unwrap();
    assert_eq!(metadata.len(), 1024);

    assert!(progress_count.load(Ordering::SeqCst) > 0);
}

#[tokio::test]
async fn downloader_handles_http_errors_gracefully() {
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(GET).path("/bad_model.bin");
        then.status(404);
    });

    let dir = tempfile::tempdir().unwrap();
    let mut downloader = ModelDownloader::new(dir.path().to_path_buf());
    downloader.test_url_override = Some(server.url("/bad_model.bin"));

    let result = downloader.download_model("base", |_| {}).await;

    mock.assert();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("status: 404"));
}

#[tokio::test]
async fn downloader_rejects_unknown_model_before_network() {
    let dir = tempfile::tempdir().unwrap();
    let downloader = ModelDownloader::new(dir.path().to_path_buf());

    let result = downloader.download_model("missing-model", |_| {}).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown model"));
    assert!(!dir.path().join("missing-model").exists());
}

#[tokio::test]
async fn downloader_does_not_leave_temp_file_after_http_error() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET).path("/bad_model.bin");
        then.status(500);
    });

    let dir = tempfile::tempdir().unwrap();
    let mut downloader = ModelDownloader::new(dir.path().to_path_buf());
    downloader.test_url_override = Some(server.url("/bad_model.bin"));

    let result = downloader.download_model("base", |_| {}).await;

    mock.assert();
    assert!(result.is_err());
    assert!(!downloader.get_model_path("base").exists());
    assert!(!downloader
        .get_model_path("base")
        .with_extension("bin.tmp")
        .exists());
}

#[tokio::test]
async fn downloader_cancels_in_flight_download() {
    let server = MockServer::start();

    // Mock a response with delay to simulate long download
    let _mock = server.mock(|when, then| {
        when.method(GET).path("/slow.bin");
        then.status(200)
            .header("content-length", "1000000")
            .body(vec![0u8; 1000000]) // Give it some body
            .delay(std::time::Duration::from_millis(50));
    });

    let dir = tempfile::tempdir().unwrap();
    let mut downloader = ModelDownloader::new(dir.path().to_path_buf());
    downloader.test_url_override = Some(server.url("/slow.bin"));

    let downloader_arc = Arc::new(downloader);
    let downloader_clone = downloader_arc.clone();

    // Spawn download in background
    let handle = tokio::spawn(async move { downloader_clone.download_model("base", |_| {}).await });

    // Wait a tiny bit then cancel
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    let cancelled = downloader_arc.cancel_download("base");
    assert!(cancelled);

    let result = handle.await.unwrap();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Download cancelled");

    // Ensure no temp file is left
    let temp_path = downloader_arc
        .get_model_path("base")
        .with_extension("bin.tmp");
    assert!(!temp_path.exists());
}
