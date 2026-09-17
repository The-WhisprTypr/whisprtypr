# SOLID Refactoring Plan — src-tauri

## Current State Analysis

The `src-tauri/src/` codebase has significant SOLID violations that make the code
hard to maintain, test, and extend. This document catalogs the violations and
details the refactoring plan.

---

## SOLID Violations Catalog

### 1. Single Responsibility Principle (SRP) Violations

| Module | Lines | Problem |
|--------|-------|---------|
| `lib.rs` | 809 ▼ (was 3,879) | **God module** — dramatically reduced. All 80 commands extracted to `src/commands/`. Now contains types, state wrappers, shared helpers, app setup, tray, and invoke_handler. |
| `database/` (was database.rs) | 1,251 (was 1,242) | **God module** — now split into 9 modules under `src/database/` |
| `license/` (was license.rs) | 1,588 (was 1,571) | **God module** — split into 5 modules under `src/license/` |
| `post_process/` (was post_process.rs) | 1,230+ (was 1,351) | **God module** — split into 3 modules under `src/post_process/` |
| `transcription.rs` | 558 | Transcriber implementations + model download URL logic + file listing functions (`get_parakeet_files`, `get_qwen3_asr_files`, `get_model_filename`). |
| `cloud_transcription.rs` | 778 | Provider connection tests, actual transcription calls, API key encryption/decryption, masking — all in one module. |
| `downloader.rs` | 542 | Download logic, model validation, Qwen3 tokenizer generation, directory size estimation. |
| `error_reporting.rs` | 700 | Error reporting, crash reports, log file management, export to JSON/Markdown, persistence. |

### 2. Open/Closed Principle (OCP) Violations

**Cloud provider handling** — Adding a new cloud provider requires modifying:
- `cloud_transcription.rs::test_provider_connection()` — match block
- `cloud_transcription.rs::transcribe_with_cloud()` — match block
- `lib.rs` — `get_cloud_providers()` provider list, `save_cloud_provider()` validation
- Multiple other locations

**AI formatting provider handling** — Same pattern as cloud providers:
- `ai_formatting.rs::test_ai_provider_connection()` — match block
- `ai_formatting.rs::format_text_with_ai()` — match block
- `lib.rs` — provider lists duplicated

**Model language support** — `is_model_language_supported()` in lib.rs has a growing match block for every model.

### 3. Dependency Inversion Principle (DIP) Violations

- `lib.rs` (commands) directly imports and calls into every module — no abstraction boundaries between the command layer and implementation modules.
- Cloud provider connection logic is tightly coupled to reqwest directly — cannot test without network.
- Database operations are called directly from commands — no repository interface.

### 4. Interface Segregation Principle (ISP) Violations

- `AppSettings` struct has 40+ fields used by different parts of the app — commands that only need a few fields still receive the whole struct.

### 5. Duplication (DRY Violations)

- Provider connection test logic is nearly identical between `cloud_transcription.rs` and `ai_formatting.rs`.
- API key encryption/decryption logic appears in both `license.rs` and `cloud_transcription.rs`.
- Provider info mapping (id, name, masked_key, etc.) is duplicated in `lib.rs` for both cloud providers and AI formatting providers.
- Model ID whitelist (`VALID_MODEL_IDS`) is duplicated across multiple commands in lib.rs.

---

## Refactoring Plan

### Phase 1: Extract Utility Module (SRP) ✅ COMPLETE
**Impact**: Reduced lib.rs from 3,879 → 3,581 lines (−298 lines of duplicates removed)
- Created `src/utils.rs` with extracted functions:
  - `sanitize_text`, `canonicalize_existing_file_path`, `validate_export_path`
  - `is_valid_language_code`, `is_model_language_supported`
  - `is_youtube_url`, `sanitize_url`
  - `read_audio_file`, `interleaved_to_mono`, `resample_audio`, `append_audio_samples_with_limit`
- Re-exported via `pub use utils::*;` in `lib.rs`
- All 6 duplicate function definitions removed from `lib.rs`

### Phase 2: Provider Abstraction (OCP + DIP) ✅ COMPLETE
**Impact**: New providers can be added by implementing a trait, no match blocks to modify
- Created `src/providers/` module:
  - `cloud.rs` — `CloudProvider` trait, `CloudProviderRegistry`, `CloudProviderInfo`
  - `ai_formatting.rs` — `AiFormattingProvider` trait, `AiFormattingProviderRegistry`, `AiFormattingProviderInfo`
  - `registry.rs` — `default_cloud_providers()`, `default_ai_formatting_providers()` helper functions
  - `mod.rs` — Shared constants and provider ID lists

### Phase 3: Extract Command Modules (SRP) ✅ COMPLETE
**Impact**: lib.rs reduced from 3,581 → 809 lines (79% reduction). Commands organized by domain; lib.rs is now an orchestrator with types, state, setup, and runtime only.

Created `src/commands/` module (13 submodules, ~96KB):

| Module | Commands | Lines |
|--------|----------|-------|
| `settings.rs` | get_settings, update_settings, update_setting, get_app_state, update_app_state, set_setup_complete, set_current_setup_step | 40 |
| `models.rs` | get_models, get_model, set_model_downloaded, set_selected_model | 84 |
| `recording.rs` | get_audio_input_devices, get_audio_output_devices, set_audio_input_device, set_audio_capture_config, start_recording, stop_recording, save_temp_audio, cancel_recording, is_recording, show_recording_overlay, hide_recording_overlay | ~200 |
| `transcription.rs` | load_model, unload_model, get_loaded_model, transcribe_audio, record_and_transcribe, record_and_translate, translate_text, transcribe_file | ~250 |
| `download.rs` | download_url_to_temp, yt_dlp_binary_name, ensure_yt_dlp, find_system_ffmpeg, extract_youtube_audio, transcribe_url, transcribe_files_batch, download_model, cancel_model_download, delete_model, is_model_downloaded, get_downloaded_models, get_model_path | 354 |
| `post_process.rs` | post_process_text, extract_voice_commands, build_processor | ~80 |
| `text_injection.rs` | inject_text, execute_keyboard_shortcut | 69 |
| `history.rs` | add_transcription, get_transcription_history, get_transcription_history_count, clear_transcription_history, delete_transcription | ~80 |
| `license.rs` | get_license, activate_license, validate_license, deactivate_license, clear_stored_license, is_license_valid, start_trial, get_trial_status, get_device_info, can_use_app | ~200 |
| `utility.rs` | get_app_data_dir, get_models_dir, register_hotkey, unregister_hotkeys, parse_hotkey, get_app_version, get_app_name, ai_formatting_style_prompt | ~100 |
| `error_reporting.rs` | report_error, get_error_reports, get_error_stats, export_error_reports, test_sentry, save_export_file, clear_error_reports, load_error_reports | ~150 |
| `providers.rs` | get_cloud_providers, save_cloud_provider, delete_cloud_provider, test_cloud_connection, get_ai_formatting_providers, save_ai_formatting_provider, delete_ai_formatting_provider, test_ai_formatting_connection, format_text_with_ai | 285 |

### Phase 4: Database Separation (SRP) ✅ COMPLETE
**Impact**: `database.rs` (1,251 lines) split into 9 focused modules under `src/database/`:

| Module | Lines | Content |
|--------|-------|---------|
| `models.rs` | 163 | All data types + Default impls |
| `schema.rs` | 459 | Database struct, `new()`, migrations, default data |
| `settings.rs` | Settings CRUD | `get_settings`, `update_settings`, `update_setting` |
| `app_state.rs` | App state CRUD | `get_app_state`, `update_app_state`, `set_setup_complete`, `set_current_setup_step` |
| `models_crud.rs` | Model CRUD | `get_models`, `get_model`, `set_model_downloaded`, `set_selected_model` |
| `history.rs` | History CRUD | Transcription history operations |
| `license.rs` | License CRUD | `get_license`, `save_license`, `clear_license` |
| `providers.rs` | Provider CRUD | Cloud + AI formatting provider operations |
| `mod.rs` | Re-exports + delegation | `impl Database` delegates 30 methods to submodules |

`Database.conn` is `pub(crate)`; all command sites still call `db.get_settings()` etc. unchanged.

**Remaining in lib.rs** (809 lines): module declarations, imports, shared types (CommandError, RateLimiter, State wrappers), helper functions (user_facing_license_error, verified_license_check, has_active_trial, license_status_to_response), `run()` with app setup/tray/invoke_handler, and test modules.

### Phase 5: License Module Split (SRP) ✅ COMPLETE
**Impact**: `license.rs` (1,571 lines) split into 6 focused modules under `src/license/`:

| Module | Lines | Content |
|--------|-------|---------|
| `models.rs` | ~310 | All types, constants, `cached_license_allows_offline` |
| `device.rs` | ~145 | Device fingerprinting, encryption, cache path helpers |
| `cache.rs` | ~65 | Cache store/load/clear |
| `api.rs` | ~100 | HTTP operations (`perform_validate`, `get_license_key`) |
| `manager.rs` | ~760 | LicenseManager orchestrator (delegates to api.rs) |
| `mod.rs` | 5 | Re-exports + module glue |

`manager.rs` was reduced from ~895 to ~760 lines by extracting HTTP operations to `api.rs`. All public APIs preserved.

### Phase 6: Cloud Transcription Split (SRP) ✅ COMPLETE
**Impact**: `cloud_transcription.rs` (778 lines) split into 4 focused modules under `src/cloud_transcription/`:

| Module | Lines | Content |
|--------|-------|---------|
| `mod.rs` | 165 | Thin match delegators, URL transcription, speaker transcript formatting |
| `models.rs` | 22 | Types and constants (CloudProviderInfo, DeepgramWord, base URLs) |
| `util.rs` | 153 | Encoding, encryption/decryption, masking, model ID parsing, HTTP client |
| `providers/` | ~480 | groq.rs, openai.rs, deepgram.rs, mistral.rs, custom.rs (provider implementations) |

All 6 original tests preserved; 3 additional edge-case tests added (empty key, wrong key, decryption roundtrip).

### Phase 7: Post-Processing Split (SRP) ✅ COMPLETE
**Impact**: `post_process.rs` (1,351 lines) split into 3 focused modules under `src/post_process/`:

| Module | Lines | Content |
|--------|-------|---------|
| `mod.rs` | ~518 | PostProcessor struct, apply, apply_to_json, helpers, tests |
| `patterns.rs` | 172 | lazy_static regex patterns (all items `pub`) |
| `casing.rs` | 172 | to_camel_case, to_pascal_case, to_snake_case, to_kebab_case, to_constant_case |

All public APIs preserved; `apply_to_json` and `apply_to_array` still delegate to full post_process logic.

### Phase 8: Transcription Split (SRP) ✅ COMPLETE
**Impact**: `transcription.rs` (558 lines) split into 4 focused modules under `src/transcription/`:

| Module | Lines | Content |
|--------|-------|---------|
| `mod.rs` | ~80 | Transcriber enum, new(), transcribe(), warm_up(), set_language(), language(), model_id() |
| `models.rs` | ~110 | get_model_url, get_model_filename, get_parakeet_files, get_qwen3_asr_files, ParakeetFile |
| `whisper.rs` | ~95 | WhisperTranscriber (model loading, aggressive speed optimizations, segment collection) |
| `parakeet.rs` | ~70 | ParakeetTranscriber, configure_ort_acceleration, default_ort_accelerator |
| `qwen3.rs` | ~70 | Qwen3AsrTranscriber, qwen3_language_name |

All public APIs preserved (Transcriber enum, get_model_*, transcription functions).

### Phase 9: Downloader Split (SRP) ✅ COMPLETE
**Impact**: `downloader.rs` (542 lines) split into 3 focused modules under `src/downloader/`:

| Module | Lines | Content |
|--------|-------|---------|
| `mod.rs` | ~90 | ModelDownloader struct, DownloadProgress, new(), cancel tokens, get_model_path, delete_model, get_downloaded_models |
| `single.rs` | ~110 | Single file download (download_model_inner) |
| `directory.rs` | ~170 | Directory model download, validation, Qwen3 tokenizer generation, expected sizes |

All public APIs preserved (ModelDownloader, DownloadProgress, all methods).

### Phase 10: Error Reporting Split (SRP) ✅ COMPLETE
**Impact**: `error_reporting.rs` (700 lines) split into 4 focused modules under `src/error_reporting/`:

| Module | Lines | Content |
|--------|-------|---------|
| `mod.rs` | ~190 | ErrorReporter struct, global singleton, report(), panic handling, getters, export methods |
| `models.rs` | ~160 | ErrorSeverity, ErrorCategory, ErrorReport, CrashReport, ErrorStats, Display impls |
| `persistence.rs` | ~130 | write_error_to_file, write_crash_report, cleanup_old_logs, export_logs |
| `exports.rs` | ~40 | report_error, report_error_with_details, report_critical_error, get_os_info |

All public APIs preserved (ErrorReporter, all types, convenience functions).

### Phase 11: Audio Module Split (SRP) ✅ COMPLETE
**Impact**: `audio.rs` (699 lines) split into 5 focused modules under `src/audio/`:

| Module | Lines | Content |
|--------|-------|---------|
| `mod.rs` | ~200 | AudioRecorder, AudioCaptureSource, device types, all core methods, device config, recording lifecycle |
| `devices.rs` | ~80 | select_input_device, select_output_device, is_probable_loopback_input |
| `recording.rs` | ~100 | run_recording_thread, CaptureDeviceKind, build_capture_stream, build_stream_for_config |
| `processing.rs` | ~80 | mix_audio_sources, process_audio_data, resample, is_probable_loopback_input |
| `wav.rs` | ~25 | save_wav |

All public APIs preserved (AudioRecorder, device types, recording functions).

### Phase 12: AI Formatting Split (SRP) ✅ COMPLETE
**Impact**: `ai_formatting.rs` (603 lines) split into 3 focused modules under `src/ai_formatting/`:

| Module | Lines | Content |
|--------|-------|---------|
| `mod.rs` | ~100 | Constants, AiFormattingProviderInfo, default_base_url_for_provider, default_model_for_provider, tests |
| `types.rs` | ~100 | All request/response structs (ChatRequest, ChatMessage, ChatResponse, Anthropic*, Gemini*) |
| `formatting.rs` | ~350 | build_formatting_prompt, test_ai_provider_connection, format_text_with_ai (5 providers) |

All public APIs preserved (AiFormattingProviderInfo, build_formatting_prompt, format_text_with_ai, test_ai_provider_connection).

### Phase 13: Utility Module Organization (SRP) ✅ COMPLETE
**Impact**: `utils.rs` (303 lines) reorganized into 3 modules under `src/utils/`:

| Module | Lines | Content |
|--------|-------|---------|
| `mod.rs` | ~100 | Constants, re-exports, text/path utilities (sanitize_text, validate_export_path, is_model_language_supported, etc.) |
| `audio.rs` | ~100 | Audio processing (read_audio_file, append_audio_samples_with_limit, interleaved_to_mono, resample_audio) |
| `text/` | — | Text utilities group (sanitize, validate, language codes, URLs) |
| `path/` | — | Path utilities group (canonicalize, extension check, export path validation) |

SRP rationale: Audio processing functions were mixed with text sanitization and path handling; separated into focused concern groups.

### Phase 14: Translation Module Split (SRP) ✅ COMPLETE
**Impact**: `translation.rs` (73 lines) split into 2 focused modules under `src/translation/`:

| Module | Lines | Content |
|--------|-------|---------|
| `mod.rs` | ~57 | `translate()` function, API call logic |
| `types.rs` | ~21 | `TranslationRequest`, `TranslationResponse`, `ResponseData` structs |

SRP rationale: Request/response types separated from the translation HTTP logic.

### Phase 15: Text Injection Split (SRP) ✅ COMPLETE
**Impact**: `text_inject.rs` (389 lines) split into 2 focused modules under `src/text_inject/`:

| Module | Lines | Content |
|--------|-------|---------|
| `mod.rs` | ~65 | TextInjector struct, new(), inject_text(), execute_shortcut() method |
| `keyboard.rs` | ~320 | Keyboard shortcut key mapping (paste, shortcut with 30+ shortcut patterns) |

SRP rationale: Clipboard injection logic mixed with 30+ keyboard shortcut patterns; separated clipboard injection from keyboard simulation.

### Phase 16: Security Module Split (SRP) ✅ COMPLETE
**Impact**: `security.rs` (117 lines) split into 2 focused modules under `src/security/`:

| Module | Lines | Content |
|--------|-------|---------|
| `mod.rs` | ~73 | `mask_license_key`, 5 inline tests |
| `crypto.rs` | ~48 | `derive_encryption_key`, `encrypt_data`, `decrypt_data` |

SRP rationale: Key masking logic separated from cryptographic operations.

### Remaining Phases

All phases complete. Refactoring done.

---

## Implementation Order

Phase 1 (utils) → Phase 2 (providers) → Phase 3 (commands) → Phase 4 (database) → Phase 5 (license) → Phase 6 (cloud_transcription) → Phase 7 (post_process) → Phase 8 (transcription) → Phase 9 (downloader) → Phase 10 (error_reporting) → Phase 11 (audio) → Phase 12 (ai_formatting) → Phase 13 (utils) → Phase 14 (translation) → Phase 15 (text_inject) → Phase 16 (security)

Phases 1-16 complete.

Each phase is independently compilable and testable.

---

## Verification Results

| Check | Result |
|-------|--------|
| `cargo check` | ✅ Clean compile (28 warnings, all pre-existing) |
| `cargo test --lib` | ✅ 44 passed, 0 failed |
| `cargo fmt -- --check` | ✅ All files formatted |

All 16 phases complete. Translation and security modules additionally split for SRP. All checks green.
