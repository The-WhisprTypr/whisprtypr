import {
    AlertCircle,
    Check,
    Circle,
    ExternalLink,
    HardDrive,
    Key,
    Loader2,
    ShieldCheck,
    Star,
    Trash2,
    Zap,
} from "@/components/icons";
import { useToast } from "@/hooks/use-toast";
import {
    deleteCloudProvider,
    saveCloudProvider,
    testCloudConnection,
} from "@/lib/cloud-api";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/store";
import type {
    CloudProviderId,
    CloudProviderInfo,
    WhisperModel,
} from "@/types";
import { useState } from "react";

interface CloudModelsTabProps {
  cloudModels: WhisperModel[];
  selectedModel: WhisperModel | null;
  onSelectModel: (model: WhisperModel) => void;
  isBusy: boolean;
}

interface ProviderMeta {
  id: CloudProviderId;
  name: string;
  badge: string;
  description: string;
  placeholder: string;
  keyUrl: string;
  recommended?: boolean;
}

const PROVIDERS_META: ProviderMeta[] = [
  {
    id: "groq",
    name: "Groq",
    badge: "Ultra-Fast (~350ms)",
    description: "Sub-second, high-throughput Whisper transcription powered by Groq LPUs.",
    placeholder: "gsk_...",
    keyUrl: "https://console.groq.com/keys",
    recommended: true,
  },
  {
    id: "openai",
    name: "OpenAI",
    badge: "Official Whisper-1",
    description: "Official OpenAI cloud Whisper transcription with broad multilingual accuracy.",
    placeholder: "sk-...",
    keyUrl: "https://platform.openai.com/api-keys",
  },
  {
    id: "deepgram",
    name: "Deepgram",
    badge: "Nova-3 Engine",
    description: "High-accuracy conversational speech recognition with smart formatting.",
    placeholder: "Deepgram API key...",
    keyUrl: "https://console.deepgram.com",
    recommended: true,
  },
  {
    id: "mistral",
    name: "Mistral",
    badge: "Voxtral",
    description: "Mistral AI speech models with strong European language accuracy.",
    placeholder: "Mistral API key...",
    keyUrl: "https://console.mistral.ai/api-keys",
  },
  {
    id: "custom",
    name: "Custom Endpoint",
    badge: "Self-Hosted / LocalAI",
    description: "Connect to any self-hosted Whisper API, LocalAI, vLLM, or corporate endpoint.",
    placeholder: "Optional API Key...",
    keyUrl: "",
  },
];

export function CloudModelsTab({
  cloudModels,
  selectedModel,
  onSelectModel,
  isBusy,
}: CloudModelsTabProps) {
  const { cloudProviders, refreshCloudProviders } = useAppStore();
  const { success: toastSuccess, error: toastError } = useToast();

  const [inputKeys, setInputKeys] = useState<Record<string, string>>({});
  const [baseUrls, setBaseUrls] = useState<Record<string, string>>({
    custom: "http://localhost:8000/v1",
  });
  const [customModels, setCustomModels] = useState<Record<string, string>>({
    custom: "whisper-1",
  });
  const [showKeys, setShowKeys] = useState<Record<string, boolean>>({});
  const [testingProvider, setTestingProvider] = useState<string | null>(null);
  const [testResults, setTestResults] = useState<
    Record<string, { ok: boolean; message: string }>
  >({});
  const [savingProvider, setSavingProvider] = useState<string | null>(null);

  const getProviderInfo = (id: CloudProviderId): CloudProviderInfo | undefined =>
    cloudProviders.find((p) => p.id === id);

  const configuredCount = cloudProviders.filter((p) => p.configured).length;

  const handleSaveKey = async (providerId: CloudProviderId) => {
    const rawKey = inputKeys[providerId] ?? "";
    const info = getProviderInfo(providerId);

    // If no new key entered and already configured, keep existing
    if (!rawKey.trim() && !info?.configured && providerId !== "custom") {
      toastError("API Key Required", "Please enter an API key first");
      return;
    }

    try {
      setSavingProvider(providerId);
      const keyToSave = rawKey.trim();
      const baseUrl = baseUrls[providerId] ?? info?.base_url ?? null;
      const customModel = customModels[providerId] ?? info?.custom_model ?? null;

      await saveCloudProvider(providerId, keyToSave, baseUrl, customModel);
      await refreshCloudProviders();

      // Clear the input text so masked representation is used
      setInputKeys((prev) => ({ ...prev, [providerId]: "" }));
      toastSuccess(
        "API Key Saved",
        `${PROVIDERS_META.find((p) => p.id === providerId)?.name} is ready for transcription`
      );
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      toastError("Failed to save key", msg);
    } finally {
      setSavingProvider(null);
    }
  };

  const handleDeleteKey = async (providerId: CloudProviderId) => {
    try {
      setSavingProvider(providerId);
      await deleteCloudProvider(providerId);
      await refreshCloudProviders();
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

  const handleTestConnection = async (providerId: CloudProviderId) => {
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
      const resultMsg = await testCloudConnection(
        providerId,
        rawKey || null,
        baseUrl || null
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

  return (
    <div className="space-y-4 xl:space-y-5">
      {/* ─── HERO BAND ─── */}
      <section className="hero-band-dark">
        <div className="grid grid-cols-1 @xl:grid-cols-[1.4fr_1fr] gap-4 @xl:gap-5 p-4 sm:p-5 @xl:p-6 items-start @xl:items-center">
          <div className="min-w-0">
            <p className="eyebrow-uppercase text-primary mb-2">
              <span className="inline-flex items-center gap-2">
                <Circle className="h-1.5 w-1.5 fill-primary text-primary" />
                Cloud Speech-to-Text (BYOK)
              </span>
            </p>
            <h2 className="display-md text-on-dark">
              {configuredCount > 0 ? (
                <>
                  You have{" "}
                  <span className="text-primary">{configuredCount}</span> cloud{" "}
                  {configuredCount === 1 ? "provider" : "providers"} connected.
                </>
              ) : (
                <>Connect your cloud provider to start.</>
              )}
            </h2>
            <p className="body-sm text-on-dark-soft mt-2 max-w-xl leading-relaxed">
              Bring Your Own Key (BYOK) gives you blazing-fast transcription (under 400ms with Groq) with zero disk storage or GPU load. Your API keys are encrypted at rest on this device and requests are sent directly to the providers.
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
              Audio is transmitted straight to your cloud provider's official endpoints. Whisprtypr has no middleman servers.
            </p>
          </div>
        </div>
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
                id={`provider-${meta.id}`}
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
                            value={customModels[meta.id] ?? info?.custom_model ?? "whisper-1"}
                            onChange={(e) =>
                              setCustomModels((prev) => ({
                                ...prev,
                                [meta.id]: e.target.value,
                              }))
                            }
                            placeholder="whisper-1"
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
                        disabled={isSaving || isBusy}
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
                      disabled={isTesting || isSaving || isBusy}
                      className="paper-button-outline size-sm cursor-pointer disabled:opacity-50 inline-flex items-center gap-1.5"
                    >
                      {isTesting && <Loader2 className="h-3 w-3 animate-spin" />}
                      Test
                    </button>
                    <button
                      onClick={() => handleSaveKey(meta.id)}
                      disabled={isSaving || isBusy}
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

      {/* ─── CLOUD MODELS SELECTION ─── */}
      <section className="flex flex-col gap-3 sm:gap-4 mt-6">
        <div className="flex items-end justify-between gap-2 flex-wrap">
          <div className="min-w-0">
            <p className="eyebrow-uppercase text-ink-mid">Engines</p>
            <h2 className="display-xs text-ink mt-1">Available Cloud Models</h2>
          </div>
          <p className="caption text-body-muted">
            {cloudModels.length} models available via BYOK
          </p>
        </div>

        <div className="grid grid-cols-1 min-[640px]:grid-cols-2 @3xl:grid-cols-2 gap-3 sm:gap-4">
          {cloudModels.map((model) => {
            const isActive = selectedModel?.id === model.id;
            const providerInfo = model.provider ? getProviderInfo(model.provider) : undefined;
            const isProviderReady = Boolean(providerInfo?.configured);

            return (
              <div
                key={model.id}
                className={cn(
                  "paper-card relative transition-all",
                  isProviderReady && "cursor-pointer",
                  isActive && "border-ink shadow-[0_8px_24px_-16px_rgba(32,21,21,0.25)]"
                )}
                onClick={() => {
                  if (isProviderReady && !isBusy) {
                    onSelectModel(model);
                  }
                }}
              >
                {/* Active indicator stripe */}
                {isActive && (
                  <div className="absolute left-0 top-4 bottom-4 w-1 bg-primary rounded-r-full" />
                )}

                <div className="flex items-start justify-between gap-2.5 sm:gap-3">
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2 flex-wrap">
                      <h3 className="title-sm text-ink">{model.name}</h3>
                      {isActive && (
                        <span className="caption-strong text-primary bg-primary/10 px-2 py-0.5 rounded-full">
                          Active
                        </span>
                      )}
                      {isProviderReady && !isActive && (
                        <span
                          className="caption-strong flex items-center gap-1 px-2 py-0.5 rounded-full"
                          style={{
                            color: "#05b169",
                            background: "rgba(5,177,105,0.1)",
                          }}
                        >
                          <Check className="h-2.5 w-2.5" />
                          Ready
                        </span>
                      )}
                      {!isProviderReady && (
                        <span className="caption text-body-muted bg-canvas-soft border border-hairline px-2 py-0.5 rounded-full">
                          Key required
                        </span>
                      )}
                    </div>

                    {/* Badges */}
                    <div className="flex items-center gap-1.5 mt-1.5 flex-wrap">
                      <span className="inline-flex items-center gap-1 caption px-1.5 py-0.5 rounded-full bg-primary/10 text-primary">
                        <Zap className="h-3 w-3" />
                        Cloud BYOK
                      </span>
                      {model.latencyEstimate && (
                        <span className="inline-flex items-center gap-1 caption px-1.5 py-0.5 rounded-full bg-canvas-soft text-ink font-medium">
                          {model.latencyEstimate}
                        </span>
                      )}
                      {model.recommended && (
                        <span className="inline-flex items-center gap-1 caption px-1.5 py-0.5 rounded-full bg-canvas-soft text-ink">
                          <Star className="h-3 w-3" />
                          Recommended
                        </span>
                      )}
                    </div>

                    <p className="body-sm text-body-muted mt-1.5 leading-relaxed">
                      {model.description}
                    </p>

                    <div className="flex items-center gap-2.5 mt-2 caption text-body-muted flex-wrap">
                      <span className="inline-flex items-center gap-1">
                        <HardDrive className="h-2.5 w-2.5" />
                        0 MB local
                      </span>
                      <span className="h-1 w-1 rounded-full bg-body-mid shrink-0" />
                      <span>Multilingual (Auto detect)</span>
                    </div>
                  </div>

                  <div className="flex items-center gap-1.5 shrink-0">
                    {isProviderReady ? (
                      <>
                        {!isActive && (
                          <button
                            className="paper-button-outline size-sm cursor-pointer"
                            style={{ borderColor: "#201515", color: "#201515" }}
                            onClick={(event) => {
                              event.stopPropagation();
                              onSelectModel(model);
                            }}
                            disabled={isBusy}
                          >
                            Use
                          </button>
                        )}
                        {isActive && (
                          <div className="flex h-8 w-8 items-center justify-center rounded-md bg-primary text-on-dark">
                            <Check className="h-3.5 w-3.5" />
                          </div>
                        )}
                      </>
                    ) : (
                      <button
                        className="paper-button-outline size-sm cursor-pointer opacity-70 hover:opacity-100"
                        onClick={(event) => {
                          event.stopPropagation();
                          if (model.provider) {
                            const el = document.getElementById(`provider-${model.provider}`);
                            if (el) {
                              el.scrollIntoView({ behavior: "smooth", block: "center" });
                            }
                          }
                          toastError(
                            "Key Required",
                            `Configure your ${model.name.split(" ")[0]} API key above to activate.`
                          );
                        }}
                      >
                        Set key
                      </button>
                    )}
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
