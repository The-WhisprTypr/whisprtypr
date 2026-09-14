/**
 * Tauri API wrappers for WhisprTypr
 * These functions provide a clean interface to communicate with the Rust backend
 */

import type { AppSettings, WhisperModel } from "@/types";
import { invoke } from "@tauri-apps/api/core";
import { readText, writeText } from "@tauri-apps/plugin-clipboard-manager";
import {
  isRegistered,
  register,
  unregister,
} from "@tauri-apps/plugin-global-shortcut";
import { load, Store } from "@tauri-apps/plugin-store";

// ============================================
// Store (Persistent Settings)
// ============================================

let store: Store | null = null;

async function getStore(): Promise<Store> {
  if (!store) {
    store = await load("settings.json", { autoSave: true, defaults: {} });
  }
  return store;
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  const s = await getStore();
  await s.set("settings", settings);
}

export async function loadSettings(): Promise<AppSettings | null> {
  const s = await getStore();
  const result = await s.get<AppSettings>("settings");
  return result ?? null;
}

export async function saveModelInfo(model: WhisperModel): Promise<void> {
  const s = await getStore();
  await s.set("selectedModel", model);
}

export async function loadModelInfo(): Promise<WhisperModel | null> {
  const s = await getStore();
  const result = await s.get<WhisperModel>("selectedModel");
  return result ?? null;
}

// ============================================
// Global Shortcuts
// ============================================

export async function registerHotkey(
  shortcut: string,
  callback: () => void
): Promise<boolean> {
  try {
    const alreadyRegistered = await isRegistered(shortcut);
    if (alreadyRegistered) {
      await unregister(shortcut);
    }
    await register(shortcut, callback);
    return true;
  } catch (error) {
    console.error("Failed to register hotkey:", error);
    return false;
  }
}

export async function unregisterHotkey(shortcut: string): Promise<boolean> {
  try {
    await unregister(shortcut);
    return true;
  } catch (error) {
    console.error("Failed to unregister hotkey:", error);
    return false;
  }
}

export async function isHotkeyRegistered(shortcut: string): Promise<boolean> {
  try {
    return await isRegistered(shortcut);
  } catch (error) {
    console.error("Failed to check hotkey registration:", error);
    return false;
  }
}

// ============================================
// Clipboard
// ============================================

export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    await writeText(text);
    return true;
  } catch (error) {
    console.error("Failed to copy to clipboard:", error);
    return false;
  }
}

export async function readFromClipboard(): Promise<string | null> {
  try {
    return await readText();
  } catch (error) {
    console.error("Failed to read from clipboard:", error);
    return null;
  }
}

// ============================================
// Audio Recording (Rust Backend)
// ============================================

export async function startRecording(): Promise<boolean> {
  try {
    await invoke("start_recording");
    return true;
  } catch (error) {
    console.error("Failed to start recording:", error);
    return false;
  }
}

export async function stopRecording(): Promise<string | null> {
  try {
    const transcription = await invoke<string>("stop_recording");
    return transcription;
  } catch (error) {
    console.error("Failed to stop recording:", error);
    return null;
  }
}

export async function cancelRecording(): Promise<void> {
  try {
    await invoke("cancel_recording");
  } catch (error) {
    console.error("Failed to cancel recording:", error);
  }
}

// ============================================
// AI Model Management
// ============================================

export interface ModelDownloadProgress {
  modelId: string;
  bytesDownloaded: number;
  totalBytes: number;
  percentage: number;
}

export async function getAvailableModels(): Promise<WhisperModel[]> {
  try {
    return await invoke<WhisperModel[]>("get_available_models");
  } catch (error) {
    console.error("Failed to get available models:", error);
    return [];
  }
}

export async function getDownloadedModels(): Promise<string[]> {
  try {
    return await invoke<string[]>("get_downloaded_models");
  } catch (error) {
    console.error("Failed to get downloaded models:", error);
    return [];
  }
}

export async function downloadModel(modelId: string): Promise<boolean> {
  try {
    await invoke("download_model", { modelId });
    return true;
  } catch (error) {
    console.error("Failed to download model:", error);
    return false;
  }
}

export async function deleteModel(modelId: string): Promise<boolean> {
  try {
    await invoke("delete_model", { modelId });
    return true;
  } catch (error) {
    console.error("Failed to delete model:", error);
    return false;
  }
}

export async function loadModel(modelId: string): Promise<boolean> {
  try {
    await invoke("load_model", { modelId });
    return true;
  } catch (error) {
    console.error("Failed to load model:", error);
    return false;
  }
}

export async function getModelStatus(modelId: string): Promise<string> {
  try {
    return await invoke<string>("get_model_status", { modelId });
  } catch (error) {
    console.error("Failed to get model status:", error);
    return "error";
  }
}

// ============================================
// Text Injection
// ============================================

export async function injectText(text: string): Promise<boolean> {
  try {
    await invoke("inject_text", { text });
    return true;
  } catch (error) {
    console.error("Failed to inject text:", error);
    return false;
  }
}

// ============================================
// System
// ============================================

export async function getAppVersion(): Promise<string> {
  try {
    return await invoke<string>("get_app_version");
  } catch (error) {
    console.error("Failed to get app version:", error);
    return "0.0.0";
  }
}

export async function getModelsDirectory(): Promise<string> {
  try {
    return await invoke<string>("get_models_directory");
  } catch (error) {
    console.error("Failed to get models directory:", error);
    return "";
  }
}

export async function checkMicrophonePermission(): Promise<boolean> {
  try {
    return await invoke<boolean>("check_microphone_permission");
  } catch (error) {
    console.error("Failed to check microphone permission:", error);
    return false;
  }
}

export async function requestMicrophonePermission(): Promise<boolean> {
  try {
    return await invoke<boolean>("request_microphone_permission");
  } catch (error) {
    console.error("Failed to request microphone permission:", error);
    return false;
  }
}
