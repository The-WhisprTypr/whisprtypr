// Build script for the vendored qwen3-asr crate.
//
// GPU acceleration is provided by Candle's `cuda` and `metal` Cargo
// features, which pull in platform toolchains (nvcc / the macOS SDK).
// Enabling them unconditionally breaks the build on machines without the
// toolkit, so we detect the toolchain here and emit cfg flags that the
// crate's `best_device()` uses to pick the right backend.
//
// NOTE: We emit bare cfg keys (`metal`, `cuda`) rather than
// `feature:metal` / `feature:cuda` because rustc's `--cfg` flag does not
// accept the `feature:key` form from build scripts. The crate's Cargo
// features (`cuda`, `metal`) remain available for users who want to force
// a specific backend — when they're enabled we also set the matching cfg
// flag so `best_device()` picks the right backend.

fn main() {
    // --- CUDA detection (Linux + Windows) ---
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    {
        use std::process::Command;

        // Allow users to force CUDA via the Cargo feature even when nvcc
        // isn't on PATH (e.g. when the toolkit is installed in a
        // non-standard location).
        let force_cuda = std::env::var("CARGO_FEATURE_CUDA").is_ok();
        let has_nvcc = force_cuda
            || Command::new("nvcc")
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

        if has_nvcc {
            println!("cargo:rustc-cfg=cuda");
            println!("cargo:warning=qwen3-asr: CUDA toolkit detected; enabling GPU acceleration");
        } else {
            println!("cargo:warning=qwen3-asr: CUDA toolkit not found; using CPU fallback");
        }
    }

    // --- Metal detection (macOS) ---
    // Metal is part of the macOS SDK and is always available, so we
    // unconditionally enable it on macOS. Users on other platforms who
    // force the `metal` Cargo feature will also get the cfg flag.
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-cfg=metal");
        println!("cargo:warning=qwen3-asr: Metal acceleration enabled");
    }

    // Re-run if the toolkit is installed/uninstalled.
    println!("cargo:rerun-if-env-changed=CUDA_HOME");
    println!("cargo:rerun-if-env-changed=CUDA_PATH");
    println!("cargo:rerun-if-env-changed=CUDA_TOOLKIT_ROOT_DIR");
    // Force re-run when the build script itself changes so stale cached
    // cfg flags (e.g. the old `feature:metal` form) are never reused.
    println!("cargo:rerun-if-changed=build.rs");
}