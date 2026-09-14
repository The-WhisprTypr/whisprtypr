/**
 * Data management utilities for WhisprTypr
 * Export, import, backup, and cleanup functions
 */

import { invoke } from "@tauri-apps/api/core";
import {
  clearTranscriptionHistory,
  getTranscriptionHistory,
  type TranscriptionHistoryItem,
} from "./voice-api";

export interface ExportData {
  version: string;
  exportedAt: string;
  history: TranscriptionHistoryItem[];
}

/**
 * Export history data as JSON
 */
export async function exportAppData(): Promise<string> {
  const history = await getTranscriptionHistory(10000); // Get all history

  const exportData: ExportData = {
    version: "1.0.0",
    exportedAt: new Date().toISOString(),
    history,
  };

  return JSON.stringify(exportData, null, 2);
}

/**
 * Import app data from JSON
 */
export async function importAppData(jsonData: string): Promise<{
  success: boolean;
  historyCount: number;
  error?: string;
}> {
  try {
    const data: ExportData = JSON.parse(jsonData);

    // Validate version
    if (!data.version) {
      return {
        success: false,
        historyCount: 0,
        error: "Invalid export file format",
      };
    }

    let historyCount = 0;

    // Import history
    if (data.history && Array.isArray(data.history)) {
      for (const item of data.history) {
        try {
          await invoke("add_transcription", {
            text: item.text,
            model_id: item.model_id,
            language: item.language,
            duration_ms: item.duration_ms,
          });
          historyCount++;
        } catch {
          // Skip items that fail to import
        }
      }
    }

    return { success: true, historyCount };
  } catch (error) {
    return {
      success: false,
      historyCount: 0,
      error:
        error instanceof Error ? error.message : "Failed to parse import file",
    };
  }
}

/**
 * Get storage statistics
 */
export async function getStorageStats(): Promise<{
  historyCount: number;
  oldestEntry?: string;
  newestEntry?: string;
}> {
  const history = await getTranscriptionHistory(10000);

  if (history.length === 0) {
    return { historyCount: 0 };
  }

  // Sort by date
  const sorted = [...history].sort(
    (a, b) =>
      new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
  );

  return {
    historyCount: history.length,
    oldestEntry: sorted[0]?.created_at,
    newestEntry: sorted[sorted.length - 1]?.created_at,
  };
}

/**
 * Clear history older than specified days
 */
export async function clearOldHistory(daysToKeep: number): Promise<number> {
  const history = await getTranscriptionHistory(10000);
  const cutoffDate = new Date();
  cutoffDate.setDate(cutoffDate.getDate() - daysToKeep);

  let deletedCount = 0;
  for (const item of history) {
    const itemDate = new Date(item.created_at);
    if (itemDate < cutoffDate) {
      try {
        await invoke("delete_transcription", { id: item.id });
        deletedCount++;
      } catch {
        // Continue on error
      }
    }
  }

  return deletedCount;
}

/**
 * Clear all app data (factory reset)
 */
export async function factoryReset(): Promise<void> {
  // Clear history
  await clearTranscriptionHistory();

  // Reset settings will be handled by the store
}

/**
 * Download data as file using Tauri's dialog for save location
 */
export async function downloadFile(
  content: string,
  filename: string,
  _mimeType: string = "application/json"
): Promise<boolean> {
  try {
    // Use dynamic import to avoid issues
    const { save } = await import("@tauri-apps/plugin-dialog");

    // Open save dialog
    const isMarkdown =
      _mimeType === "text/markdown" ||
      filename.toLowerCase().endsWith(".md") ||
      filename.toLowerCase().endsWith(".markdown");

    const filePath = await save({
      defaultPath: filename,
      filters: [
        {
          name: isMarkdown ? "Markdown" : "JSON",
          extensions: isMarkdown ? ["md", "markdown"] : ["json"],
        },
      ],
    });

    if (filePath) {
      await invoke("save_export_file", { path: filePath, content });
      return true;
    }
    return false;
  } catch (error) {
    console.error("Failed to save file:", error);
    // Fallback to browser download
    const blob = new Blob([content], { type: _mimeType });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    return true;
  }
}
