/**
 * Preferences API - Handles system-level preferences like autostart and minimize to tray
 */

import { getCurrentWindow } from "@tauri-apps/api/window";
import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";

/**
 * Enable or disable autostart on boot
 * Note: Launch args (--minimized) are configured in the Rust backend
 * during plugin initialization (see src-tauri/src/lib.rs)
 */
export async function setAutoStart(enabled: boolean): Promise<void> {
  try {
    if (enabled) {
      await enable();
    } else {
      await disable();
    }

    // Verify the change took effect
    const now = await isEnabled();
    if (now !== enabled) {
      console.warn(
        `Autostart toggle did not match expected state (expected=${enabled}, got=${now})`
      );
    }
  } catch (error) {
    console.error("Failed to set autostart:", error);
    throw error;
  }
}

/**
 * Check if autostart is enabled
 * Note: Launch args (--minimized) are configured in the Rust backend
 * during plugin initialization (see src-tauri/src/lib.rs)
 */
export async function getAutoStartEnabled(): Promise<boolean> {
  try {
    return await isEnabled();
  } catch (error) {
    console.error("Failed to check autostart status:", error);
    return false;
  }
}

/**
 * Minimize window to system tray instead of closing
 */
export async function minimizeToTray(): Promise<void> {
  try {
    const window = getCurrentWindow();
    await window.hide();
  } catch (error) {
    console.error("Failed to minimize to tray:", error);
    throw error;
  }
}

/**
 * Play audio feedback sound
 * Uses Web Audio API to generate simple beep sounds
 */
export function playFeedbackSound(type: "start" | "stop"): void {
  try {
    const audioContext = new AudioContext();
    const oscillator = audioContext.createOscillator();
    const gainNode = audioContext.createGain();

    oscillator.connect(gainNode);
    gainNode.connect(audioContext.destination);

    // Different frequencies for start/stop
    if (type === "start") {
      oscillator.frequency.value = 880; // A5 - higher pitch for start
      gainNode.gain.value = 0.1;
    } else {
      oscillator.frequency.value = 440; // A4 - lower pitch for stop
      gainNode.gain.value = 0.1;
    }

    oscillator.type = "sine";
    oscillator.start();

    // Short beep duration
    setTimeout(() => {
      gainNode.gain.exponentialRampToValueAtTime(
        0.001,
        audioContext.currentTime + 0.1
      );
      setTimeout(() => {
        oscillator.stop();
        audioContext.close();
      }, 100);
    }, 80);
  } catch (error) {
    console.error("Failed to play feedback sound:", error);
  }
}
