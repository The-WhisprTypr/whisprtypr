import { openUrl as tauriOpenUrl } from "@tauri-apps/plugin-opener";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/**
 * Open a URL in the default browser
 * Works in both Tauri and browser environments
 */
export async function openUrl(url: string): Promise<void> {
  try {
    // Use Tauri opener plugin
    await tauriOpenUrl(url);
  } catch {
    // Fallback to window.open for browser dev mode
    window.open(url, "_blank");
  }
}
