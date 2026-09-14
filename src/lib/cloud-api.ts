import { invoke } from "@tauri-apps/api/core";
import type { CloudProviderInfo } from "@/types";

/**
 * Get all configured and available cloud providers
 */
export async function getCloudProviders(): Promise<CloudProviderInfo[]> {
  try {
    return await invoke<CloudProviderInfo[]>("get_cloud_providers");
  } catch (error) {
    console.error("Failed to get cloud providers:", error);
    return [];
  }
}

/**
 * Save an API key and optional settings for a cloud provider
 */
export async function saveCloudProvider(
  provider: string,
  apiKey: string,
  baseUrl?: string | null,
  customModel?: string | null
): Promise<void> {
  await invoke("save_cloud_provider", {
    provider,
    apiKey: apiKey.trim(),
    baseUrl: baseUrl?.trim() || null,
    customModel: customModel?.trim() || null,
  });
}

/**
 * Delete a cloud provider's API key
 */
export async function deleteCloudProvider(provider: string): Promise<void> {
  await invoke("delete_cloud_provider", { provider });
}

/**
 * Test connectivity to a cloud provider with an API key
 */
export async function testCloudConnection(
  provider: string,
  apiKey?: string | null,
  baseUrl?: string | null
): Promise<string> {
  return await invoke<string>("test_cloud_connection", {
    provider,
    apiKey: apiKey?.trim() || null,
    baseUrl: baseUrl?.trim() || null,
  });
}
