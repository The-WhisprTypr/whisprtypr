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
        // dynamic libraries both in direct binary execution (@executable_path)
        // and via loader paths. Tauri automatically adds @executable_path/../Frameworks
        // via bundle.macOS.frameworks in tauri.conf.json.
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path/../Frameworks");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");

        // Link Apple Clang's compiler-rt (libclang_rt.osx.a) which provides runtime
        // helper symbols like ___isPlatformVersionAtLeast required by Clang's @available
        // checks in native dependencies (e.g. whisper-rs-sys / ggml-metal) under -nodefaultlibs.
        let clang_cmds = [
            ("xcrun", vec!["clang", "-print-file-name=libclang_rt.osx.a"]),
            ("clang", vec!["-print-file-name=libclang_rt.osx.a"]),
        ];
        let mut linked_compiler_rt = false;
        for (cmd, args) in &clang_cmds {
            if let Ok(output) = std::process::Command::new(cmd).args(args).output() {
                if output.status.success() {
                    let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    let path = std::path::Path::new(&path_str);
                    if path.is_file() && path.exists() {
                        println!("cargo:rustc-link-arg={}", path.display());
                        println!("cargo:warning=Linked compiler-rt: {}", path.display());
                        linked_compiler_rt = true;
                        break;
                    }
                }
            }
        }
        if !linked_compiler_rt {
            let candidates = [
                "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/clang",
                "/Library/Developer/CommandLineTools/usr/lib/clang",
            ];
            for candidate in &candidates {
                if let Ok(entries) = std::fs::read_dir(candidate) {
                    for entry in entries.flatten() {
                        let rt_path = entry.path().join("lib").join("darwin").join("libclang_rt.osx.a");
                        if rt_path.is_file() && rt_path.exists() {
                            println!("cargo:rustc-link-arg={}", rt_path.display());
                            println!("cargo:warning=Linked compiler-rt from fallback: {}", rt_path.display());
                            linked_compiler_rt = true;
                            break;
                        }
                    }
                }
                if linked_compiler_rt {
                    break;
                }
            }
        }

        // Enable Metal acceleration for Qwen3-ASR on Apple Silicon (aarch64)
        let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
        if target_arch == "aarch64" {
            println!("cargo:rustc-cfg=metal");
            println!("cargo:warning=Qwen3-ASR Metal acceleration enabled");
        } else {
            println!("cargo:warning=Qwen3-ASR Metal acceleration disabled on x86_64 macOS; using CPU fallback");
        }

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
