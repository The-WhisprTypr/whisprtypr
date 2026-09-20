// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Compatibility shim for macOS: Objective-C/C++ code compiled with Clang availability checks
// (e.g., whisper-rs-sys / ggml-metal using @available) emits references to ___isPlatformVersionAtLeast.
// Because rustc links with -nodefaultlibs, libclang_rt.osx.a is not automatically passed to ld.
#[cfg(target_os = "macos")]
#[no_mangle]
pub unsafe extern "C" fn ___isPlatformVersionAtLeast(
    platform: u32,
    major: u32,
    minor: u32,
    subminor: u32,
) -> i32 {
    #[repr(C)]
    struct DyldBuildVersion {
        platform: u32,
        version: u32,
    }

    extern "C" {
        fn _availability_version_check(count: u32, versions: *const DyldBuildVersion) -> i32;
    }

    let ver = DyldBuildVersion {
        platform,
        version: (major << 16) | (minor << 8) | subminor,
    };
    _availability_version_check(1, &ver)
}

fn main() {
    whisprtypr_lib::run()
}
