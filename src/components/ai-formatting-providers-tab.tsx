import { useState } from "react";
import {
  AlertCircle,
  Check,
  Circle,
  ExternalLink,
  Key,
  Loader2,
  ShieldCheck,
  Sparkles,
  Trash2,
} from "@/components/icons";
import { useToast } from "@/hooks/use-toast";
import { cn } from "@/lib/utils";
import {
  deleteAiFormattingProvider,
  saveAiFormattingProvider,
  testAiFormattingConnection,
} from "@/lib/ai-formatting-api";
import { useAppStore } from "@/store";
import type {
  AiFormattingProviderId,
  AiFormattingProviderInfo,
} from "@/types";
import { AI_FORMATTING_STYLES, type AiFormattingStyle } from "@/types";

interface ProviderMeta {
  id: AiFormattingProviderId;
  name: string;
  badge: string;
  description: string;
  placeholder: string;
  keyUrl: string;
  defaultModel: string;
  recommended?: boolean;
}

const PROVIDERS_META: ProviderMeta[] = [
  {
    id: "openai",
    name: "OpenAI",
    badge: "GPT-4o · o1",
    description:
      "OpenAI's powerful language models for high-quality text formatting and rewriting.",
    placeholder: "sk-...",
    keyUrl: "https://platform.openai.com/api-keys",
    defaultModel: "gpt-4o-mini",
    recommended: true,
  },
  {
    id: "gemini",
    name: "Gemini",
    badge: "Gemini 2.0",
    description:
      "Google's multimodal models with strong reasoning and multilingual formatting capabilities.",
    placeholder: "API key...",
    keyUrl: "https://aistudio.google.com/apikey",
    defaultModel: "gemini-1.5-flash",
    recommended: true,
  },
  {
    id: "anthropic",
    name: "Anthropic",
    badge: "Claude 3.5/4",
    description:
      "Constitutional AI with excellent writing, editing, and formatting quality.",
    placeholder: "sk-ant-...",
    keyUrl: "https://console.anthropic.com/settings/keys",
    defaultModel: "claude-3-5-sonnet-20241022",
    recommended: true,
  },
  {
    id: "deepseek",
    name: "DeepSeek",
    badge: "DeepSeek-V3/Chat",
    description:
      "High-performance open models at competitive pricing.",
    placeholder: "sk-...",
    keyUrl: "https://platform.deepseek.com/api-keys",
    defaultModel: "deepseek-chat",
  },
  {
    id: "custom",
    name: "Custom Endpoint",
    badge: "OpenAI-compatible",
    description:
      "Connect to any self-hosted or OpenAI-compatible LLM endpoint (e.g., Ollama, vLLM, Localhost).",
    placeholder: "Optional API Key...",
    keyUrl: "",
    defaultModel: "gpt-4o-mini",
  },
];

interface AiFormattingProvidersTabProps {
  selectedStyle: AiFormattingStyle;
}

export function AiFormattingProvidersTab({ selectedStyle }: AiFormattingProvidersTabProps) {
  const { aiFormattingProviders, refreshAiFormattingProviders } = useAppStore();
  const { success: toastSuccess, error: toastError } = useToast();

  const [inputKeys, setInputKeys] = useState<Record<string, string>>({});
  const [baseUrls, setBaseUrls] = useState<Record<string, string>>({
    custom: "http://localhost:8000/v1",
  });
  const [customModels, setCustomModels] = useState<Record<string, string>>({});
  const [showKeys, setShowKeys] = useState<Record<string, boolean>>({});
  const [testingProvider, setTestingProvider] = useState<string | null>(null);
  const [testResults, setTestResults] = useState<
    Record<string, { ok: boolean; message: string }>
  >({});
  const [savingProvider, setSavingProvider] = useState<string | null>(null);

  const getProviderInfo = (id: AiFormattingProviderId): AiFormattingProviderInfo | undefined =>
    aiFormattingProviders.find((p) => p.id === id);

  const configuredCount = aiFormattingProviders.filter((p) => p.configured).length;

  const handleSaveKey = async (providerId: AiFormattingProviderId) => {
    const rawKey = inputKeys[providerId] ?? "";
    const info = getProviderInfo(providerId);

    if (!rawKey.trim() && !info?.configured && providerId !== "custom") {
      toastError("API Key Required", "Please enter an API key first");
      return;
    }

    try {
      setSavingProvider(providerId);
      const keyToSave = rawKey.trim();
      const baseUrl = baseUrls[providerId] ?? info?.base_url ?? null;
      const customModel = customModels[providerId] ?? info?.custom_model ?? null;

      await saveAiFormattingProvider(providerId, keyToSave, baseUrl, customModel);
      await refreshAiFormattingProviders();

      setInputKeys((prev) => ({ ...prev, [providerId]: "" }));
      toastSuccess(
        "API Key Saved",
        `${PROVIDERS_META.find((p) => p.id === providerId)?.name} is ready for AI formatting`
      );
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      toastError("Failed to save key", msg);
    } finally {
      setSavingProvider(null);
    }
  };

  const handleDeleteKey = async (providerId: AiFormattingProviderId) => {
    try {
      setSavingProvider(providerId);
      await deleteAiFormattingProvider(providerId);
      await refreshAiFormattingProviders();
      setInputKeys((prev) => ({ ...prev, [providerId]: "" }));
      setTestResults((prev) => {
        const next = { ...prev };
        delete next[providerId];
        return next;
      });
      toastSuccess(
        "Key Removed",
        `${PROVIDERS_META.find((p) => p.id === providerId)?.name} key has been removed`
      );
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      toastError("Failed to remove key", msg);
    } finally {
      setSavingProvider(null);
    }
  };

  const handleTestConnection = async (providerId: AiFormattingProviderId) => {
    const rawKey = inputKeys[providerId]?.trim();
    const info = getProviderInfo(providerId);

    if (!rawKey?.trim() && !info?.configured && providerId !== "custom") {
      toastError("API Key Required", "Please enter an API key to test");
      return;
    }

    try {
      setTestingProvider(providerId);
      setTestResults((prev) => {
        const next = { ...prev };
        delete next[providerId];
        return next;
      });

      const baseUrl = baseUrls[providerId] ?? info?.base_url ?? null;
      const model = customModels[providerId] ?? info?.custom_model ?? null;
      const resultMsg = await testAiFormattingConnection(
        providerId,
        rawKey || null,
        baseUrl || null,
        model || null
      );

      setTestResults((prev) => ({
        ...prev,
        [providerId]: { ok: true, message: resultMsg },
      }));
      toastSuccess("Connection Successful", resultMsg);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setTestResults((prev) => ({
        ...prev,
        [providerId]: { ok: false, message: msg },
      }));
      toastError("Connection Failed", msg);
    } finally {
      setTestingProvider(null);
    }
  };

  const getStyleInfo = (style: AiFormattingStyle) =>
    AI_FORMATTING_STYLES.find((s) => s.id === style) ?? AI_FORMATTING_STYLES[0];

  const styleInfo = getStyleInfo(selectedStyle);

  return (
    <div className="space-y-4 xl:space-y-5">
      {/* ─── HERO BAND ─── */}
      <section className="hero-band-dark">
        <div className="grid grid-cols-1 @xl:grid-cols-[1.4fr_1fr] gap-4 @xl:gap-5 p-4 sm:p-5 @xl:p-6 items-start @xl:items-center">
          <div className="min-w-0">
            <p className="eyebrow-uppercase text-primary mb-2">
              <span className="inline-flex items-center gap-2">
                <Circle className="h-1.5 w-1.5 fill-primary text-primary" />
                AI Formatting (BYOK)
              </span>
            </p>
            <h2 className="display-md text-on-dark">
              {configuredCount > 0 ? (
                <>
                  You have{" "}
                  <span className="text-primary">{configuredCount}</span> LLM{" "}
                  {configuredCount === 1 ? "provider" : "providers"} connected.
                </>
              ) : (
                <>Connect your LLM provider to start formatting.</>
              )}
            </h2>
            <p className="body-sm text-on-dark-soft mt-2 max-w-xl leading-relaxed">
              Bring Your Own Key connects your AI provider so your transcribed voice is reformatted with the selected style (e.g. Clean Dictation, Email, Code, Notes) before it reaches your cursor. Your API keys are encrypted at rest on this device.
            </p>
          </div>

          {/* Security & Privacy Card */}
          <div className="product-ui-card-dark w-full">
            <div className="flex items-center gap-2.5 mb-2.5">
              <div className="icon-plate-dark">
                <ShieldCheck className="h-3.5 w-3.5 text-primary" />
              </div>
              <div className="min-w-0 flex-1">
                <p className="caption-strong text-on-dark">Private & Encrypted</p>
                <p className="caption text-on-dark-soft mt-0.5">
                  AES-256-GCM hardware-derived encryption
                </p>
              </div>
            </div>
            <p className="caption text-on-dark-soft leading-relaxed">
              Text is transmitted directly to your provider's official endpoints. WhisprTypr has no middleman servers.
            </p>
          </div>
        </div>
      </section>

      {/* ─── STYLE PREVIEW ─── */}
      <section className="paper-card">
        <div className="flex items-center gap-2.5 mb-3">
          <div className="icon-plate">
            <Sparkles className="h-3.5 w-3.5 text-primary" />
          </div>
          <div className="min-w-0">
            <p className="caption-strong text-ink">Active Format Style</p>
            <h3 className="title-md text-ink mt-0.5">{styleInfo.name}</h3>
          </div>
        </div>
        <p className="body-sm text-body-muted leading-relaxed mt-1.5">
          {styleInfo.description}
        </p>
      </section>

      {/* ─── PROVIDER KEYS CONFIGURATION ─── */}
      <section className="flex flex-col gap-3 sm:gap-4">
        <div className="flex items-end justify-between gap-2 flex-wrap">
          <div className="min-w-0">
            <p className="eyebrow-uppercase text-ink-mid">Configuration</p>
            <h2 className="display-xs text-ink mt-1">Connect API Keys</h2>
          </div>
          <p className="caption text-body-muted">
            {configuredCount} of {PROVIDERS_META.length} providers configured
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-3 sm:gap-4">
          {PROVIDERS_META.map((meta) => {
            const info = getProviderInfo(meta.id);
            const isConfigured = Boolean(info?.configured);
            const isTesting = testingProvider === meta.id;
            const isSaving = savingProvider === meta.id;
            const testRes = testResults[meta.id];
            const isCustom = meta.id === "custom";
            const isShowKey = Boolean(showKeys[meta.id]);

            return (
              <div
                key={meta.id}
                id={`ai-provider-${meta.id}`}
                className={cn(
                  "paper-card relative transition-all p-4 flex flex-col justify-between",
                  isConfigured && "border-ink/20"
                )}
              >
                <div>
                  {/* Card Header */}
                  <div className="flex items-start justify-between gap-2 mb-2">
                    <div className="flex items-center gap-2">
                      <div className="flex h-7 w-7 items-center justify-center rounded-lg bg-canvas-soft border border-hairline text-ink">
                        <Key className="h-3.5 w-3.5 text-primary" />
                      </div>
                      <div>
                        <h3 className="title-sm text-ink flex items-center gap-1.5">
                          {meta.name}
                          {meta.recommended && (
                            <span className="caption-strong text-primary bg-primary/10 px-1.5 py-0.2 rounded-full text-[10px]">
                              Recommended
                            </span>
                          )}
                        </h3>
                      </div>
                    </div>

                    <div>
                      {isConfigured ? (
                        <span
                          className="caption-strong inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs"
                          style={{
                            color: "#05b169",
                            background: "rgba(5,177,105,0.1)",
                          }}
                        >
                          <Check className="h-3 w-3" />
                          Ready
                        </span>
                      ) : (
                        <span className="caption text-body-muted bg-canvas-soft px-2 py-0.5 rounded-full text-xs border border-hairline">
                          Not set
                        </span>
                      )}
                    </div>
                  </div>

                  <p className="body-sm text-body-muted mb-3 leading-relaxed">
                    {meta.description}
                  </p>

                  {/* Inputs */}
                  <div className="space-y-2.5">
                    {isCustom && (
                      <div className="space-y-2.5">
                        <div>
                          <label className="caption-strong text-ink block mb-1">
                            Base URL
                          </label>
                          <input
                            type="text"
                            value={baseUrls[meta.id] ?? info?.base_url ?? "http://localhost:8000/v1"}
                            onChange={(e) =>
                              setBaseUrls((prev) => ({
                                ...prev,
                                [meta.id]: e.target.value,
                              }))
                            }
                            placeholder="http://localhost:8000/v1"
                            className="w-full h-8 px-2.5 text-xs bg-canvas rounded-lg border border-hairline text-ink focus:outline-none focus:border-primary"
                          />
                        </div>
                        <div>
                          <label className="caption-strong text-ink block mb-1">
                            Model Identifier
                          </label>
                          <input
                            type="text"
                            value={customModels[meta.id] ?? info?.custom_model ?? meta.defaultModel}
                            onChange={(e) =>
                              setCustomModels((prev) => ({
                                ...prev,
                                [meta.id]: e.target.value,
                              }))
                            }
                            placeholder={meta.defaultModel}
                            className="w-full h-8 px-2.5 text-xs bg-canvas rounded-lg border border-hairline text-ink focus:outline-none focus:border-primary"
                          />
                        </div>
                      </div>
                    )}

                    <div>
                      <div className="flex items-center justify-between mb-1">
                        <label className="caption-strong text-ink">
                          {isCustom ? "API Key (Optional)" : "API Key"}
                        </label>
                        {meta.keyUrl && (
                          <a
                            href={meta.keyUrl}
                            target="_blank"
                            rel="noreferrer"
                            className="caption text-primary hover:underline inline-flex items-center gap-1"
                          >
                            Get key <ExternalLink className="h-2.5 w-2.5" />
                          </a>
                        )}
                      </div>

                      <div className="relative flex items-center">
                        <input
                          type={isShowKey ? "text" : "password"}
                          value={
                            inputKeys[meta.id] !== undefined
                              ? inputKeys[meta.id]
                              : isConfigured && info?.masked_key
                              ? info.masked_key
                              : ""
                          }
                          onChange={(e) =>
                            setInputKeys((prev) => ({
                              ...prev,
                              [meta.id]: e.target.value,
                            }))
                          }
                          placeholder={
                            isConfigured && info?.masked_key
                              ? info.masked_key
                              : meta.placeholder
                          }
                          className="w-full h-8 pl-2.5 pr-14 text-xs bg-canvas rounded-lg border border-hairline text-ink focus:outline-none focus:border-primary font-mono"
                        />
                        <button
                          type="button"
                          onClick={() =>
                            setShowKeys((prev) => ({
                              ...prev,
                              [meta.id]: !prev[meta.id],
                            }))
                          }
                          className="absolute right-2 caption text-[11px] text-body-muted hover:text-ink cursor-pointer"
                        >
                          {isShowKey ? "Hide" : "Show"}
                        </button>
                      </div>
                    </div>
                  </div>

                  {/* Inline Test Result Message */}
                  {testRes && (
                    <div
                      className={cn(
                        "mt-2.5 p-2 rounded-lg text-xs flex items-start gap-1.5",
                        testRes.ok
                          ? "bg-[#05b169]/10 text-[#05b169]"
                          : "bg-[#cf202f]/10 text-[#cf202f]"
                      )}
                    >
                      {testRes.ok ? (
                        <Check className="h-3.5 w-3.5 shrink-0 mt-0.5" />
                      ) : (
                        <AlertCircle className="h-3.5 w-3.5 shrink-0 mt-0.5" />
                      )}
                      <span className="leading-snug">{testRes.message}</span>
                    </div>
                  )}
                </div>

                {/* Actions */}
                <div className="flex items-center justify-between gap-2 mt-4 pt-3 border-t border-hairline">
                  <div>
                    {isConfigured && (
                      <button
                        onClick={() => handleDeleteKey(meta.id)}
                        disabled={isSaving || testingProvider !== null}
                        className="caption text-body-muted hover:text-[#cf202f] cursor-pointer inline-flex items-center gap-1 transition-colors"
                        title="Remove stored API key"
                      >
                        <Trash2 className="h-3 w-3" />
                        Clear
                      </button>
                    )}
                  </div>

                  <div className="flex items-center gap-2">
                    <button
                      onClick={() => handleTestConnection(meta.id)}
                      disabled={isTesting || isSaving || testingProvider !== null}
                      className="paper-button-outline size-sm cursor-pointer disabled:opacity-50 inline-flex items-center gap-1.5"
                    >
                      {isTesting && <Loader2 className="h-3 w-3 animate-spin" />}
                      Test
                    </button>
                    <button
                      onClick={() => handleSaveKey(meta.id)}
                      disabled={isSaving || testingProvider !== null}
                      className="paper-button-primary size-sm cursor-pointer disabled:opacity-50 inline-flex items-center gap-1.5"
                    >
                      {isSaving && <Loader2 className="h-3 w-3 animate-spin" />}
                      Save
                    </button>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </section>
    </div>
  );
}
