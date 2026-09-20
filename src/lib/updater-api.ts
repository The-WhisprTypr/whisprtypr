/**
 * Auto-updater API for Whisprtypr
 * Handles checking for updates and installing them from GitHub releases
 */

import { relaunch } from "@tauri-apps/plugin-process";
import { check } from "@tauri-apps/plugin-updater";
import { logger } from "./logger";

const MINIMUM_SAFE_UPDATE_VERSION = "1.0.1";
const UPDATER_CHECK_OPTIONS = { allowDowngrades: false } as const;

export interface UpdateInfo {
  version: string;
  date?: string;
  body?: string;
  currentVersion: string;
}

export interface UpdateProgress {
  downloaded: number;
  total: number | null;
}

export type UpdateStatus =
  | { status: "idle" }
  | { status: "checking" }
  | { status: "available"; info: UpdateInfo }
  | { status: "not-available"; currentVersion: string }
  | { status: "downloading"; progress: UpdateProgress }
  | { status: "ready"; info: UpdateInfo }
  | { status: "error"; message: string };

function parseVersion(version: string): number[] {
  return version
    .replace(/^v/i, "")
    .split(/[.-]/)
    .map((part) => {
      const value = Number.parseInt(part, 10);
      return Number.isFinite(value) ? value : 0;
    });
}

function compareVersions(left: string, right: string): number {
  const a = parseVersion(left);
  const b = parseVersion(right);
  const length = Math.max(a.length, b.length);

  for (let i = 0; i < length; i += 1) {
    const diff = (a[i] ?? 0) - (b[i] ?? 0);
    if (diff !== 0) return diff > 0 ? 1 : -1;
  }

  return 0;
}

function isAllowedUpdateVersion(updateVersion: string, currentVersion: string) {
  return (
    compareVersions(updateVersion, currentVersion) > 0 &&
    compareVersions(updateVersion, MINIMUM_SAFE_UPDATE_VERSION) >= 0
  );
}

/**
 * Check for available updates
 */
export async function checkForUpdates(): Promise<UpdateStatus> {
  try {
    logger.info("Checking for updates...");

    const currentVersion = await getCurrentVersion();
    const update = await check(UPDATER_CHECK_OPTIONS);

    if (!update) {
      logger.info("No updates available");
      return {
        status: "not-available",
        currentVersion,
      };
    }

    if (!isAllowedUpdateVersion(update.version, currentVersion)) {
      logger.warn("Rejected unsafe update version", {
        currentVersion,
        updateVersion: update.version,
        minimumSafeUpdateVersion: MINIMUM_SAFE_UPDATE_VERSION,
      });
      return {
        status: "not-available",
        currentVersion,
      };
    }

    logger.info(`Update available: ${update.version}`);

    return {
      status: "available",
      info: {
        version: update.version,
        date: update.date,
        body: update.body,
        currentVersion,
      },
    };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    logger.error("Failed to check for updates", { error: message });

    // Check if this is a "no release found" error (expected for new installations)
    const isNoReleaseError =
      message.includes("fetch") ||
      message.includes("JSON") ||
      message.includes("remote") ||
      message.includes("status code") ||
      message.includes("404") ||
      message.includes("Not Found");

    if (isNoReleaseError) {
      // Return "not-available" instead of error for missing release files
      // This is expected when no update is published yet
      logger.info("No update manifest found - treating as up-to-date");
      return {
        status: "not-available",
        currentVersion: await getCurrentVersion(),
      };
    }

    // Provide user-friendly error message for real errors
    let friendlyMessage = message;
    if (message.includes("network") || message.includes("connect")) {
      friendlyMessage = "Network error. Please check your internet connection.";
    }

    return { status: "error", message: friendlyMessage };
  }
}

/**
 * Download and install update
 * @param onProgress Callback for download progress updates
 */
export async function downloadAndInstallUpdate(
  onProgress?: (progress: UpdateProgress) => void
): Promise<UpdateStatus> {
  try {
    logger.info("Starting update download...");

    const currentVersion = await getCurrentVersion();
    const update = await check(UPDATER_CHECK_OPTIONS);

    if (!update) {
      return {
        status: "not-available",
        currentVersion,
      };
    }

    if (!isAllowedUpdateVersion(update.version, currentVersion)) {
      return {
        status: "error",
        message: `Rejected update ${update.version}; current version is ${currentVersion}.`,
      };
    }

    let downloaded = 0;
    let total: number | null = null;

    // Download with progress tracking
    await update.downloadAndInstall((event) => {
      switch (event.event) {
        case "Started":
          total = event.data.contentLength ?? null;
          logger.info(`Download started, size: ${total ?? "unknown"}`);
          break;
        case "Progress":
          downloaded += event.data.chunkLength;
          onProgress?.({ downloaded, total });
          break;
        case "Finished":
          logger.info("Download finished");
          break;
      }
    });

    logger.info("Update downloaded and ready to install");

    return {
      status: "ready",
      info: {
        version: update.version,
        date: update.date,
        body: update.body,
        currentVersion,
      },
    };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    logger.error("Failed to download/install update", { error: message });
    return { status: "error", message };
  }
}

/**
 * Relaunch the application after update installation
 */
export async function relaunchApp(): Promise<void> {
  logger.info("Relaunching application...");
  await relaunch();
}

/**
 * Get the current app version
 */
export async function getCurrentVersion(): Promise<string> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<string>("get_app_version");
  } catch {
    return "unknown";
  }
}

/**
 * Format bytes to human readable string
 */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

/**
 * Format download progress as percentage
 */
export function formatProgress(progress: UpdateProgress): string {
  if (progress.total === null) {
    return formatBytes(progress.downloaded);
  }
  const percent = Math.round((progress.downloaded / progress.total) * 100);
  return `${percent}% (${formatBytes(progress.downloaded)} / ${formatBytes(
    progress.total
  )})`;
}
