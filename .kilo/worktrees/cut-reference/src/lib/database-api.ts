/**
 * Database API wrappers for WhisprTypr
 * These functions provide a clean interface to interact with SQLite database via Tauri backend
 */

import { invoke } from "@tauri-apps/api/core";

// ============================================
// Types (matching Rust structs)
// ============================================

export interface DbVocabularyEntry {
  spoken: string;
  written: string;
}

export interface DbAppSettings {
  push_to_talk_key: string;
  toggle_key: string;
  hotkey_mode: string;
  language: string;
  selected_model_id: string;
  show_recording_indicator: boolean;
  show_recording_overlay: boolean;
  play_audio_feedback: boolean;
  auto_start_on_boot: boolean;
  minimize_to_tray: boolean;
  post_processing_enabled: boolean;
  voice_commands_enabled: boolean;
  clipboard_mode: boolean;
  custom_vocabulary: DbVocabularyEntry[];
}

export interface DbAppState {
  is_first_launch: boolean;
  setup_complete: boolean;
  current_setup_step: number;
  selected_model_id: string | null;
}

export interface DbWhisperModel {
  id: string;
  name: string;
  size: string;
  size_bytes: number;
  description: string;
  languages: string; // JSON array as string
  downloaded: boolean;
  download_path: string | null;
}

export interface DbTranscriptionHistory {
  id: number;
  text: string;
  model_id: string;
  language: string;
  duration_ms: number;
  created_at: string;
}

// ============================================
// Settings API
// ============================================

export async function dbGetSettings(): Promise<DbAppSettings> {
  return await invoke<DbAppSettings>("get_settings");
}

export async function dbUpdateSettings(settings: DbAppSettings): Promise<void> {
  await invoke("update_settings", { settings });
}

export async function dbUpdateSetting(
  key: string,
  value: string
): Promise<void> {
  await invoke("update_setting", { key, value });
}

// ============================================
// App State API
// ============================================

export async function dbGetAppState(): Promise<DbAppState> {
  return await invoke<DbAppState>("get_app_state");
}

export async function dbUpdateAppState(state: DbAppState): Promise<void> {
  await invoke("update_app_state", { state });
}

export async function dbSetSetupComplete(complete: boolean): Promise<void> {
  await invoke("set_setup_complete", { complete });
}

export async function dbSetCurrentSetupStep(step: number): Promise<void> {
  await invoke("set_current_setup_step", { step });
}

// ============================================
// Model API
// ============================================

export async function dbGetModels(): Promise<DbWhisperModel[]> {
  return await invoke<DbWhisperModel[]>("get_models");
}

export async function dbGetModel(id: string): Promise<DbWhisperModel | null> {
  return await invoke<DbWhisperModel | null>("get_model", { id });
}

export async function dbSetModelDownloaded(
  id: string,
  downloaded: boolean,
  path?: string
): Promise<void> {
  await invoke("set_model_downloaded", { id, downloaded, path: path ?? null });
}

export async function dbSetSelectedModel(
  modelId: string | null
): Promise<void> {
  await invoke("set_selected_model", { modelId });
}

// ============================================
// Transcription History API
// ============================================

export async function dbAddTranscription(
  text: string,
  modelId: string,
  language: string,
  durationMs: number
): Promise<number> {
  // The backend `add_transcription` command expects snake_case parameter names
  // matching the Rust function signature: (text, model_id, language, duration_ms)
  return await invoke<number>("add_transcription", {
    text,
    model_id: modelId,
    language,
    duration_ms: durationMs,
  });
}

export async function dbGetTranscriptionHistory(
  limit?: number,
  offset?: number,
  search?: string
): Promise<DbTranscriptionHistory[]> {
  return await invoke<DbTranscriptionHistory[]>("get_transcription_history", {
    limit: limit ?? null,
    offset: offset ?? null,
    search: search?.trim() || null,
  });
}

export async function dbClearTranscriptionHistory(): Promise<void> {
  await invoke("clear_transcription_history");
}

export async function dbDeleteTranscription(id: number): Promise<void> {
  await invoke("delete_transcription", { id });
}

// ============================================
// Utility API
// ============================================

export async function dbGetAppDataDir(): Promise<string> {
  return await invoke<string>("get_app_data_dir");
}

// ============================================
// Conversion Helpers (DB <-> Frontend types)
// ============================================

import type { AppSettings, ModelCapabilities, WhisperModel } from "@/types";
import { MODEL_CAPABILITIES, WHISPER_MULTILINGUAL_LANGUAGES } from "@/types";

export function dbSettingsToFrontend(db: DbAppSettings): AppSettings {
  return {
    pushToTalkKey: db.push_to_talk_key,
    toggleKey: db.toggle_key,
    hotkeyMode: db.hotkey_mode as "push-to-talk" | "toggle",
    language: db.language,
    selectedModelId: db.selected_model_id,
    showRecordingIndicator: db.show_recording_indicator,
    showRecordingOverlay: db.show_recording_overlay ?? true,
    playAudioFeedback: db.play_audio_feedback,
    postProcessingEnabled: db.post_processing_enabled,
    voiceCommandsEnabled: db.voice_commands_enabled ?? false,
    clipboardMode: db.clipboard_mode,
    autoStartOnBoot: db.auto_start_on_boot,
    minimizeToTray: db.minimize_to_tray,
    customVocabulary: Array.isArray(db.custom_vocabulary)
      ? db.custom_vocabulary
      : [],
  };
}

export function frontendSettingsToDb(settings: AppSettings): DbAppSettings {
  return {
    push_to_talk_key: settings.pushToTalkKey,
    toggle_key: settings.toggleKey,
    hotkey_mode: settings.hotkeyMode,
    language: settings.language,
    selected_model_id: settings.selectedModelId,
    show_recording_indicator: settings.showRecordingIndicator,
    show_recording_overlay: settings.showRecordingOverlay,
    play_audio_feedback: settings.playAudioFeedback,
    post_processing_enabled: settings.postProcessingEnabled,
    voice_commands_enabled: settings.voiceCommandsEnabled,
    clipboard_mode: settings.clipboardMode,
    auto_start_on_boot: settings.autoStartOnBoot,
    minimize_to_tray: settings.minimizeToTray,
    custom_vocabulary: (settings.customVocabulary ?? []).map((entry) => ({
      spoken: entry.spoken,
      written: entry.written,
    })),
  };
}

export function dbModelToFrontend(db: DbWhisperModel): WhisperModel {
  let languages: string[] = [];
  try {
    languages = JSON.parse(db.languages);
  } catch {
    languages = ["en"];
  }

  const capabilities: ModelCapabilities | undefined = MODEL_CAPABILITIES[db.id];

  if (capabilities) {
    languages = capabilities.supportedLanguages;
  } else if (languages.includes("multilingual")) {
    languages = WHISPER_MULTILINGUAL_LANGUAGES;
  }

  const model: WhisperModel = {
    id: db.id,
    name: db.name,
    size: db.size,
    sizeBytes: db.size_bytes,
    description: db.description,
    languages,
    downloaded: db.downloaded,
    recommended: ["base", "parakeet-v3"].includes(db.id),
  };

  if (capabilities) {
    return { ...model, ...capabilities };
  }

  return model;
}

export function dbModelsToFrontend(dbModels: DbWhisperModel[]): WhisperModel[] {
  return dbModels.map(dbModelToFrontend);
}

// ============================================
// App Info Commands
// ============================================

/**
 * Get the application version from Cargo.toml
 */
export async function getAppVersion(): Promise<string> {
  return await invoke<string>("get_app_version");
}

/**
 * Get the application name
 */
export async function getAppName(): Promise<string> {
  return await invoke<string>("get_app_name");
}

/**
 * Get app info object
 */
export async function getAppInfo(): Promise<{ name: string; version: string }> {
  const [name, version] = await Promise.all([getAppName(), getAppVersion()]);
  return { name, version };
}
