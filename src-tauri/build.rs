// Build script for Whisprtypr.
//
// GPU acceleration for the Qwen3-ASR model is provided by the vendored
// `qwen3-asr` crate, which uses Candle's CUDA/Metal backends. We detect
// the toolchain at build time and emit bare cfg keys (`cuda`, `metal`)
// that the crate's `best_device()` uses to pick the right backend.
//
// NOTE: We emit bare cfg keys, NOT `feature:cuda`/`feature:metal`, because
// rustc's `--cfg` flag does not accept the `feature:key` form from build
// scripts. The crate's Cargo features (`cuda`, `metal`) remain available
// for users who want to force a specific backend, but they are NOT enabled
// here — the build script only signals which backend to use.

fn main() {
    tauri_build::build();

    // --- CUDA detection (Linux + Windows) ---
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    {
        let has_nvcc = std::process::Command::new("nvcc")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if has_nvcc {
            println!("cargo:rustc-cfg=cuda");
            println!("cargo:warning=Qwen3-ASR CUDA toolkit detected; enabling GPU acceleration");
        } else {
            println!("cargo:warning=Qwen3-ASR CUDA not detected; using CPU fallback");
        }
    }

    // Re-run the build script if the CUDA toolkit is installed/uninstalled
    // so the cfg reflects the current environment.
    println!("cargo:rerun-if-env-changed=CUDA_HOME");
    println!("cargo:rerun-if-env-changed=CUDA_PATH");
    println!("cargo:rerun-if-env-changed=CUDA_TOOLKIT_ROOT_DIR");
    println!("cargo:rerun-if-env-changed=CUDNN_LIB");
    // Force re-run when the build script itself changes so stale cached
    // cfg flags (e.g. the old `feature:metal` form) are never reused.
    println!("cargo:rerun-if-changed=build.rs");

    // --- macOS Configuration ---
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "macos" {
        // Embed runtime search paths into the Mach-O binary so dyld can locate
        // bundled dynamic libraries (such as libonnxruntime) both in the .app
        // bundle (Contents/Frameworks) and in development/direct binary execution.
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Frameworks");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path/../Frameworks");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");

        // Enable Metal acceleration for Qwen3-ASR (available on macOS)
        println!("cargo:rustc-cfg=metal");
        println!("cargo:warning=Qwen3-ASR Metal acceleration enabled");

        // Ensure Frameworks directory and libonnxruntime exist for bundling
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
        let frameworks_dir = std::path::Path::new(&manifest_dir).join("Frameworks");
        let dylib_file = frameworks_dir.join("libonnxruntime.1.23.2.dylib");
        if !dylib_file.exists() {
            let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
            let arch = match target_arch.as_str() {
                "x86_64" => "x86_64",
                _ => "arm64",
            };
            let url = format!(
                "https://github.com/microsoft/onnxruntime/releases/download/v1.23.2/onnxruntime-osx-{}-1.23.2.tgz",
                arch
            );
            println!("cargo:warning=libonnxruntime.1.23.2.dylib missing in Frameworks/; fetching {} for macOS bundling...", arch);
            let _ = std::fs::create_dir_all(&frameworks_dir);
            let tarball = frameworks_dir.join("ort_temp.tgz");
            let curl_status = std::process::Command::new("curl")
                .args(["-sSL", &url, "-o", tarball.to_str().unwrap_or("ort_temp.tgz")])
                .status();
            if curl_status.map(|s| s.success()).unwrap_or(false) {
                let _ = std::process::Command::new("tar")
                    .args([
                        "-xzf",
                        tarball.to_str().unwrap_or("ort_temp.tgz"),
                        "--strip-components=2",
                        "-C",
                        frameworks_dir.to_str().unwrap_or("Frameworks"),
                        &format!("onnxruntime-osx-{}-1.23.2/lib/libonnxruntime.1.23.2.dylib", arch),
                    ])
                    .status();
                let _ = std::fs::remove_file(&tarball);
            }
        }
    }
}
