/**
 * License API - Polar.sh License Key Management with Trial Support
 */

import { invoke } from "@tauri-apps/api/core";

// ============================================
// Types
// ============================================

export interface LicenseData {
  license_key: string | null;
  display_key: string | null;
  activation_id: string | null;
  status: LicenseStatus;
  customer_email: string | null;
  customer_name: string | null;
  benefit_id: string | null;
  expires_at: string | null;
  is_activated: boolean;
  last_validated_at: string | null;
  trial_started_at: string | null;
  trial_days_remaining: number | null;
  device_id: string;
  device_label: string;
  limit_activations: number | null;
  usage: number;
  validations: number;
}

export type LicenseStatus =
  | "active"
  | "inactive"
  | "expired"
  | "revoked"
  | "disabled"
  | "invalid"
  | "not_activated"
  | "activation_limit"
  | "trial"
  | "trial_expired";

// ============================================
// License API Functions
// ============================================

/**
 * Get stored license information
 */
export async function getLicense(): Promise<LicenseData> {
  return await invoke<LicenseData>("get_license");
}

/**
 * Activate a license key on this device
 */
export async function activateLicense(
  licenseKey: string
): Promise<LicenseData> {
  return await invoke<LicenseData>("activate_license", { licenseKey });
}

/**
 * Validate the current license with Polar API
 */
export async function validateLicense(): Promise<LicenseData> {
  return await invoke<LicenseData>("validate_license");
}

/**
 * Deactivate the license from this device
 */
export async function deactivateLicense(): Promise<void> {
  await invoke("deactivate_license");
}

/**
 * Clear stored license (local only, doesn't deactivate remotely)
 */
export async function clearStoredLicense(): Promise<void> {
  await invoke("clear_stored_license");
}

/**
 * Check if current license is valid
 */
export async function isLicenseValid(): Promise<boolean> {
  return await invoke<boolean>("is_license_valid");
}

/**
 * Start a 7-day trial
 */
export async function startTrial(): Promise<LicenseData> {
  return await invoke<LicenseData>("start_trial");
}

/**
 * Get trial status information
 */
export async function getTrialStatus(): Promise<{
  isInTrial: boolean;
  daysRemaining: number;
  trialExpired: boolean;
}> {
  return await invoke("get_trial_status");
}

/**
 * Check if app can be used (has valid license or active trial)
 */
export async function canUseApp(): Promise<{
  canUse: boolean;
  reason: "licensed" | "trial" | "trial_expired" | "no_license";
  daysRemaining?: number;
}> {
  return await invoke("can_use_app");
}

/**
 * Get device information for license display
 */
export interface DeviceInfo {
  device_id: string;
  device_label: string;
  os: string;
  arch: string;
}

export async function getDeviceInfo(): Promise<DeviceInfo> {
  return await invoke<DeviceInfo>("get_device_info");
}

// ============================================
// Helper Functions
// ============================================

/**
 * Check if license status indicates an active license
 */
export function isLicenseActive(status: LicenseStatus): boolean {
  return status === "active" || status === "trial";
}

/**
 * Get human-readable license status message
 */
export function getLicenseStatusMessage(status: LicenseStatus): string {
  switch (status) {
    case "active":
      return "License is active";
    case "inactive":
      return "No license activated";
    case "trial":
      return "Trial is active";
    case "trial_expired":
      return "Trial has expired";
    case "expired":
      return "License has expired";
    case "revoked":
      return "License has been revoked";
    case "disabled":
      return "License has been disabled";
    case "invalid":
      return "Invalid license key";
    case "not_activated":
      return "License needs to be activated";
    case "activation_limit":
      return "Device activation limit reached";
    default:
      return "Unknown license status";
  }
}

/**
 * Format expiration date for display
 */
export function formatExpirationDate(expiresAt: string | null): string {
  if (!expiresAt) return "Never";

  try {
    const date = new Date(expiresAt);
    const now = new Date();

    if (date < now) {
      return "Expired";
    }

    const diffDays = Math.ceil(
      (date.getTime() - now.getTime()) / (1000 * 60 * 60 * 24)
    );

    if (diffDays <= 7) {
      return `Expires in ${diffDays} day${diffDays === 1 ? "" : "s"}`;
    }

    return `Expires on ${date.toLocaleDateString()}`;
  } catch {
    return "Unknown";
  }
}

/**
 * Mask license key for display (show only last 6 characters)
 */
export function maskLicenseKey(key: string | null): string {
  if (!key) return "****";
  if (key.length <= 6) return "****";
  return `****${key.slice(-6)}`;
}
