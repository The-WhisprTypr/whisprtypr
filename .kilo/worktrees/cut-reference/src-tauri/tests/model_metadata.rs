use vox_ai_lib::transcription::{
    get_model_filename, get_model_url, get_parakeet_files, get_qwen3_asr_files,
};

#[test]
fn whisper_model_urls_are_https_and_known_models_resolve() {
    for model_id in [
        "tiny",
        "base",
        "small",
        "medium",
        "large-v3",
        "large-v3-turbo",
    ] {
        let url = get_model_url(model_id).unwrap();
        assert!(url.starts_with("https://"));
        assert!(url.contains("huggingface.co"));
        assert!(url.ends_with(&get_model_filename(model_id)));
    }
}

#[test]
fn unknown_model_has_no_download_url_or_directory_manifest() {
    assert!(get_model_url("unknown-model").is_none());
    assert!(get_parakeet_files("unknown-model").is_none());
    assert!(get_qwen3_asr_files("unknown-model").is_none());
}

#[test]
fn model_filename_mapping_handles_special_models() {
    assert_eq!(get_model_filename("base"), "ggml-base.bin");
    assert_eq!(get_model_filename("base.en"), "ggml-base.en.bin");
    assert_eq!(
        get_model_filename("distil-small.en"),
        "ggml-distil-small.en.bin"
    );
    assert_eq!(
        get_model_filename("parakeet-v3"),
        "parakeet-tdt-0.6b-v3-int8"
    );
    assert_eq!(get_model_filename("qwen3-asr-0.6b"), "qwen3-asr-0.6b");
}

#[test]
fn directory_model_manifests_include_required_files_and_https_urls() {
    let parakeet = get_parakeet_files("parakeet-v3").unwrap();
    assert_eq!(parakeet.len(), 4);
    assert!(parakeet
        .iter()
        .any(|file| file.filename == "encoder-model.int8.onnx"));
    assert!(parakeet.iter().all(|file| file.url.starts_with("https://")));

    let qwen = get_qwen3_asr_files("qwen3-asr-0.6b").unwrap();
    assert!(qwen.iter().any(|file| file.filename == "model.safetensors"));
    assert!(qwen.iter().any(|file| file.filename == "vocab.json"));
    assert!(qwen.iter().all(|file| file.url.starts_with("https://")));
}
