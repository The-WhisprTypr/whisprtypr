import { invoke } from "@tauri-apps/api/core";
import type { AiFormattingProviderInfo } from "@/types";

export async function getAiFormattingProviders(): Promise<AiFormattingProviderInfo[]> {
  try {
    return await invoke<AiFormattingProviderInfo[]>("get_ai_formatting_providers");
  } catch (error) {
    console.error("Failed to get AI formatting providers:", error);
    return [];
  }
}

export async function saveAiFormattingProvider(
  provider: string,
  apiKey: string,
  baseUrl?: string | null,
  customModel?: string | null,
): Promise<void> {
  await invoke("save_ai_formatting_provider", {
    provider,
    apiKey: apiKey.trim(),
    baseUrl: baseUrl?.trim() || null,
    customModel: customModel?.trim() || null,
  });
}

export async function deleteAiFormattingProvider(provider: string): Promise<void> {
  await invoke("delete_ai_formatting_provider", { provider });
}

export async function testAiFormattingConnection(
  provider: string,
  apiKey?: string | null,
  baseUrl?: string | null,
  model?: string | null,
): Promise<string> {
  return await invoke<string>("test_ai_formatting_connection", {
    provider,
    apiKey: apiKey?.trim() || null,
    baseUrl: baseUrl?.trim() || null,
    model: model?.trim() || null,
  });
}

export async function formatTextWithAi(
  text: string,
  style: string,
  provider?: string | null,
  model?: string | null,
): Promise<string> {
  return await invoke<string>("format_text_with_ai", {
    text,
    style,
    provider: provider || null,
    model: model || null,
  });
}
