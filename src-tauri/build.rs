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

    // --- Metal detection (macOS) ---
    // Metal is part of the macOS SDK and is always available, so we
    // unconditionally enable it on macOS.
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-cfg=metal");
        println!("cargo:warning=Qwen3-ASR Metal acceleration enabled");
    }
}
