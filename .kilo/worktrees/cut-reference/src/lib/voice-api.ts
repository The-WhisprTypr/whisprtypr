/**
 * Voice API - Recording, Transcription, and Text Injection
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";

// ============================================
// Types
// ============================================

export interface DownloadProgress {
  model_id: string;
  bytes_downloaded: number;
  total_bytes: number;
  percentage: number;
}

export interface AudioInputDevice {
  name: string;
  is_default: boolean;
}

export interface AudioOutputDevice {
  name: string;
  is_default: boolean;
}

export type AudioCaptureSource = "mic" | "system" | "both";

// ============================================
// Recording API
// ============================================

export async function getAudioInputDevices(): Promise<AudioInputDevice[]> {
  return await invoke<AudioInputDevice[]>("get_audio_input_devices");
}

export async function getAudioOutputDevices(): Promise<AudioOutputDevice[]> {
  return await invoke<AudioOutputDevice[]>("get_audio_output_devices");
}

export async function setAudioInputDevice(
  deviceName: string | null
): Promise<void> {
  await invoke("set_audio_input_device", { deviceName });
}

export async function setAudioCaptureConfig(
  captureSource: AudioCaptureSource,
  inputDeviceName: string | null,
  outputDeviceName: string | null
): Promise<void> {
  await invoke("set_audio_capture_config", {
    captureSource,
    inputDeviceName,
    outputDeviceName,
  });
}

export async function startRecording(): Promise<void> {
  await invoke("start_recording");
}

export async function stopRecording(): Promise<number[]> {
  return await invoke<number[]>("stop_recording");
}

export async function cancelRecording(): Promise<void> {
  await invoke("cancel_recording");
}

export async function saveTempAudio(audioSamples: number[]): Promise<string> {
  return await invoke<string>("save_temp_audio", { samples: audioSamples });
}

export async function isRecording(): Promise<boolean> {
  return await invoke<boolean>("is_recording");
}

// ============================================
// Recording Overlay API
// ============================================

export async function showRecordingOverlay(): Promise<void> {
  await invoke("show_recording_overlay");
}

export async function hideRecordingOverlay(): Promise<void> {
  await invoke("hide_recording_overlay");
}

// ============================================
// Transcription API
// ============================================

export async function loadModel(
  modelId: string,
  language: string = "en"
): Promise<void> {
  await invoke("load_model", { modelId, language });
}

export async function transcribeAudio(audioSamples: number[]): Promise<string> {
  return await invoke<string>("transcribe_audio", { audioSamples });
}

export async function recordAndTranscribe(): Promise<string> {
  return await invoke<string>("record_and_transcribe");
}

export async function transcribeFile(
  filePath: string,
  enablePostProcessing: boolean = true,
  enableVoiceCommands: boolean = false
): Promise<string> {
  let text = await invoke<string>("transcribe_file", { filePath });
  if (text && enablePostProcessing) {
    text = await postProcessText(text);
  } else if (text && enableVoiceCommands) {
    text = await extractVoiceCommands(text);
  }

  if (text) {
    text = enableVoiceCommands
      ? await processVoiceCommands(text)
      : stripVoiceCommandTokens(text);
  }

  return text;
}

// ============================================
// Model Download API
// ============================================

export async function downloadModel(modelId: string): Promise<string> {
  return await invoke<string>("download_model", { modelId });
}

export async function cancelModelDownload(modelId: string): Promise<boolean> {
  return await invoke<boolean>("cancel_model_download", { modelId });
}

export async function deleteModel(modelId: string): Promise<void> {
  await invoke("delete_model", { modelId });
}

export async function isModelDownloaded(modelId: string): Promise<boolean> {
  try {
    return await invoke<boolean>("is_model_downloaded", { modelId });
  } catch (error) {
    console.error("Error checking if model is downloaded:", error);
    return false;
  }
}

export async function getDownloadedModels(): Promise<string[]> {
  return await invoke<string[]>("get_downloaded_models");
}

export async function getModelPath(modelId: string): Promise<string> {
  try {
    return await invoke<string>("get_model_path", { modelId });
  } catch (error) {
    console.error("Error getting model path:", error);
    throw error;
  }
}

export async function getModelsDir(): Promise<string> {
  return await invoke<string>("get_models_dir");
}

// ============================================
// Download Progress Listener
// ============================================

export async function onDownloadProgress(
  callback: (progress: DownloadProgress) => void
): Promise<UnlistenFn> {
  return await listen<DownloadProgress>("download-progress", (event) => {
    callback(event.payload);
  });
}

// ============================================
// Post-Processing API
// ============================================

export async function postProcessText(text: string): Promise<string> {
  return await invoke<string>("post_process_text", { text });
}

export async function extractVoiceCommands(text: string): Promise<string> {
  return await invoke<string>("extract_voice_commands", { text });
}

// ============================================
// Text Injection API
// ============================================

export async function injectText(text: string): Promise<void> {
  await invoke("inject_text", { text });
}

// ============================================
// High-Level Voice-to-Text Function
// ============================================

export interface VoiceToTextOptions {
  onRecordingStart?: () => void;
  onRecordingStop?: () => void;
  onTranscriptionStart?: () => void;
  onTranscriptionComplete?: (text: string) => void;
  onError?: (error: string) => void;
  injectToActiveWindow?: boolean;
  enablePostProcessing?: boolean;
  enableVoiceCommands?: boolean;
}

/**
 * Complete voice-to-text flow:
 * 1. Stop recording
 * 2. Transcribe audio
 * 3. Post-process text (if enabled)
 * 4. Optionally inject text to active cursor
 */
export async function completeVoiceToText(
  options: VoiceToTextOptions = {},
  _selectedModelId?: string
): Promise<string | null> {
  try {
    options.onRecordingStop?.();
    options.onTranscriptionStart?.();

    let text = "";

    text = await recordAndTranscribe();

    if (text && options.enablePostProcessing) {
      text = await postProcessText(text);
    } else if (text && options.enableVoiceCommands) {
      text = await extractVoiceCommands(text);
    }

    if (text) {
      text = options.enableVoiceCommands
        ? await processVoiceCommands(text)
        : stripVoiceCommandTokens(text);
    }

    options.onTranscriptionComplete?.(text);

    // Inject text if requested
    if (options.injectToActiveWindow && text) {
      await injectText(text);
    }

    return text;
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    options.onError?.(errorMessage);
    return null;
  }
}

/**
 * Start a voice recording session
 */
export async function startVoiceRecording(
  onStart?: () => void
): Promise<boolean> {
  try {
    await startRecording();
    onStart?.();
    return true;
  } catch (error) {
    console.error("Failed to start recording:", error);
    return false;
  }
}

/**
 * Stop recording and get transcription (with optional post-processing)
 */
export async function stopAndTranscribe(
  enablePostProcessing: boolean = true
): Promise<string | null> {
  try {
    let text = await recordAndTranscribe();
    if (enablePostProcessing && text) {
      text = await postProcessText(text);
    }
    return text;
  } catch (error) {
    console.error("Failed to transcribe:", error);
    return null;
  }
}

// Voice command control sequences
const VOICE_COMMANDS: Record<string, () => Promise<void>> = {
  // Editing commands
  "[[DELETE_LAST]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "backspace_word" });
  },
  "[[UNDO]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "undo" });
  },
  "[[REDO]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "redo" });
  },
  "[[SELECT_ALL]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "select_all" });
  },
  "[[COPY]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "copy" });
  },
  "[[CUT]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "cut" });
  },
  "[[PASTE]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "paste" });
  },
  "[[BACKSPACE]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "backspace" });
  },
  "[[DELETE_FORWARD]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "delete" });
  },
  "[[DELETE_WORD]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "delete_word" });
  },
  "[[DELETE_LINE]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "delete_line" });
  },
  "[[ENTER]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "enter" });
  },
  "[[TAB]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "tab" });
  },
  "[[ESCAPE]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "escape" });
  },
  "[[PAGE_UP]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "page_up" });
  },
  "[[PAGE_DOWN]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "page_down" });
  },
  // Cursor movement commands
  "[[LEFT]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "left" });
  },
  "[[RIGHT]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "right" });
  },
  "[[UP]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "up" });
  },
  "[[DOWN]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "down" });
  },
  "[[HOME]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "home" });
  },
  "[[END]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "end" });
  },
  "[[WORD_LEFT]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "word_left" });
  },
  "[[WORD_RIGHT]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "word_right" });
  },
  "[[SELECT_LEFT]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "select_left" });
  },
  "[[SELECT_RIGHT]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "select_right" });
  },
  "[[SELECT_UP]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "select_up" });
  },
  "[[SELECT_DOWN]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "select_down" });
  },
  "[[SELECT_WORD_LEFT]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "select_word_left" });
  },
  "[[SELECT_WORD_RIGHT]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "select_word_right" });
  },
  "[[SELECT_TO_START]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "select_to_start" });
  },
  "[[SELECT_TO_END]]": async () => {
    await invoke("execute_keyboard_shortcut", { shortcut: "select_to_end" });
  },
};

function stripVoiceCommandTokens(text: string): string {
  let result = text;

  for (const command of Object.keys(VOICE_COMMANDS)) {
    result = result.split(command).join("");
  }

  return result.trim();
}

/**
 * Process voice commands in text and execute them
 * Returns the text with commands removed, and executes the commands
 */
async function processVoiceCommands(text: string): Promise<string> {
  let result = text;

  for (const [command, action] of Object.entries(VOICE_COMMANDS)) {
    if (result.includes(command)) {
      // Execute the command
      try {
        await action();
      } catch (error) {
        console.error(`Failed to execute voice command ${command}:`, error);
      }
      // Remove the command from text
      result = result.split(command).join("").trim();
    }
  }

  return result;
}

/**
 * Stop recording, transcribe, post-process, and inject text or copy to clipboard
 */
export async function stopTranscribeAndInject(
  enablePostProcessing: boolean = true,
  clipboardMode: boolean = false,
  selectedModelId?: string,
  enableVoiceCommands: boolean = false
): Promise<string | null> {
  try {
    let text = await completeVoiceToText({
      enablePostProcessing,
      enableVoiceCommands,
      injectToActiveWindow: !clipboardMode,
    }, selectedModelId);

    if (text && text.trim()) {
      if (clipboardMode) {
        await writeText(text);
      }
      // Note: injectText is already handled by completeVoiceToText if injectToActiveWindow is true
    }
    return text;
  } catch (error) {
    console.error("Failed to transcribe and inject:", error);
    return null;
  }
}

// ============================================
// Hotkey API
// ============================================

export async function registerHotkey(hotkey: string): Promise<void> {
  await invoke("register_hotkey", { hotkey });
}

export async function unregisterHotkeys(): Promise<void> {
  await invoke("unregister_hotkeys");
}

export async function onHotkeyPressed(
  callback: () => void
): Promise<UnlistenFn> {
  return await listen("hotkey-pressed", () => {
    callback();
  });
}

export async function onHotkeyReleased(
  callback: () => void
): Promise<UnlistenFn> {
  return await listen("hotkey-released", () => {
    callback();
  });
}

// ============================================
// Tray Events
// ============================================

export async function onTrayStartRecording(
  callback: () => void
): Promise<UnlistenFn> {
  return await listen("tray-start-recording", () => {
    callback();
  });
}

export async function onTrayStopRecording(
  callback: () => void
): Promise<UnlistenFn> {
  return await listen("tray-stop-recording", () => {
    callback();
  });
}

export type TrayNavigationTarget =
  | "main"
  | "transcribe"
  | "history"
  | "models"
  | "settings"
  | "help";

export async function onTrayNavigate(
  callback: (target: TrayNavigationTarget) => void
): Promise<UnlistenFn> {
  return await listen<TrayNavigationTarget>("tray-navigate", (event) => {
    callback(event.payload);
  });
}

// ============================================
// History API
// ============================================

export interface TranscriptionHistoryItem {
  id: number;
  text: string;
  model_id: string;
  language: string;
  duration_ms: number;
  created_at: string;
}

export async function getTranscriptionHistory(
  limit?: number,
  offset?: number,
  search?: string
): Promise<TranscriptionHistoryItem[]> {
  return await invoke<TranscriptionHistoryItem[]>("get_transcription_history", {
    limit,
    offset,
    search: search?.trim() || null,
  });
}

export async function getTranscriptionHistoryCount(search?: string): Promise<number> {
  return await invoke<number>("get_transcription_history_count", {
    search: search?.trim() || null,
  });
}

export async function addTranscription(
  text: string,
  modelId: string,
  language: string,
  durationMs: number
): Promise<number> {
  return await invoke<number>("add_transcription", {
    text,
    modelId,
    language,
    durationMs,
  });
}

export async function clearTranscriptionHistory(): Promise<void> {
  await invoke("clear_transcription_history");
}

export async function deleteTranscriptionItem(id: number): Promise<void> {
  await invoke("delete_transcription", { id });
}

// ============================================
// Error Reporting API
// ============================================

export type ErrorSeverity =
  | "debug"
  | "info"
  | "warning"
  | "error"
  | "critical"
  | "fatal";

export type ErrorCategory =
  | "transcription"
  | "audio"
  | "model"
  | "database"
  | "network"
  | "filesystem"
  | "license"
  | "ui"
  | "system"
  | "configuration"
  | "unknown";

export interface ErrorReport {
  id: string;
  timestamp: string;
  severity: ErrorSeverity;
  category: ErrorCategory;
  message: string;
  details?: string;
  stack_trace?: string;
  user_action?: string;
  context?: Record<string, string>;
  occurrence_count: number;
  app_version: string;
  os_info: string;
}

export interface ErrorStats {
  total_errors: number;
  by_category: Record<string, number>;
  by_severity: Record<string, number>;
}

/**
 * Report an error to the error reporting system
 */
export async function reportError(
  category: ErrorCategory,
  message: string,
  severity: ErrorSeverity = "error",
  options?: {
    stackTrace?: string;
    userAction?: string;
    context?: Record<string, string>;
  }
): Promise<void> {
  await invoke("report_error", {
    category,
    message,
    severity,
    stackTrace: options?.stackTrace,
    userAction: options?.userAction,
    context: options?.context,
  });
}

/**
 * Get recent error reports
 */
export async function getErrorReports(limit?: number): Promise<ErrorReport[]> {
  return await invoke<ErrorReport[]>("get_error_reports", { limit });
}

/**
 * Get error statistics
 */
export async function getErrorStats(): Promise<ErrorStats> {
  return await invoke<ErrorStats>("get_error_stats");
}

/**
 * Export error reports to a file and get the content
 */
export async function exportErrorReports(
  format: "json" | "markdown" = "json"
): Promise<string> {
  return await invoke<string>("export_error_reports", { format });
}

/**
 * Clear all error reports from memory
 */
export async function clearErrorReports(): Promise<void> {
  await invoke("clear_error_reports");
}

/**
 * Load error reports from disk
 */
export async function loadErrorReports(): Promise<number> {
  return await invoke<number>("load_error_reports");
}

/**
 * Helper to capture and report errors from async operations
 */
export async function withErrorReporting<T>(
  operation: () => Promise<T>,
  category: ErrorCategory,
  userAction?: string
): Promise<T> {
  try {
    return await operation();
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    const stackTrace = error instanceof Error ? error.stack : undefined;

    await reportError(category, errorMessage, "error", {
      stackTrace,
      userAction,
    });

    throw error;
  }
}
