import { cn } from "@/lib/utils";
import {
  downloadModel,
  isModelDownloaded,
  onDownloadProgress,
  type DownloadProgress,
} from "@/lib/voice-api";
import { useAppStore, useAvailableModels } from "@/store";
import {
  ALL_MODELS,
  getDefaultLanguageForModel,
  getModelLanguageLabel,
  isLanguageSupportedByModel,
  type WhisperModel,
} from "@/types";
import { Check, Cpu, HardDrive, Loader2, Sparkles, Star } from "@/components/icons";
import { useEffect, useMemo, useState } from "react";

interface ModelSelectStepProps {
  onNext: () => void;
  onBack: () => void;
}

const STEPS_TOTAL = 4;
const STEP_INDEX = 3;
const ONBOARDING_RECOMMENDED_MODEL_ID = "parakeet-v3";

export function ModelSelectStep({ onNext, onBack }: ModelSelectStepProps) {
  const {
    selectedModel,
    setSelectedModel,
    settings,
    updateSettings,
    downloadProgress,
    setDownloadProgress,
    modelStatus,
    setModelStatus,
    markModelDownloaded,
  } = useAppStore();

  const dbModels = useAvailableModels();
  const sourceModels: WhisperModel[] =
    dbModels.length > 0 ? dbModels : ALL_MODELS;
  const models = useMemo(
    () =>
      [...sourceModels].sort((a, b) => {
        if (a.id === ONBOARDING_RECOMMENDED_MODEL_ID) return -1;
        if (b.id === ONBOARDING_RECOMMENDED_MODEL_ID) return 1;
        return 0;
      }),
    [sourceModels]
  );

  const [downloadingModelId, setDownloadingModelId] = useState<string | null>(null);
  const [downloadError, setDownloadError] = useState<string | null>(null);

  useEffect(() => {
    if (selectedModel || models.length === 0) return;
    const defaultModel =
      models.find((model) => model.id === ONBOARDING_RECOMMENDED_MODEL_ID) || models[0];
    setSelectedModel(defaultModel);
    if (!isLanguageSupportedByModel(defaultModel, settings.language)) {
      updateSettings({ language: getDefaultLanguageForModel(defaultModel) });
    }
    if (defaultModel.isCloud) {
      setModelStatus("downloaded");
    } else {
      isModelDownloaded(defaultModel.id)
        .then((downloaded) => setModelStatus(downloaded ? "downloaded" : "not-downloaded"))
        .catch(console.error);
    }
  }, [selectedModel, models, settings.language, setSelectedModel, updateSettings, setModelStatus]);

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    onDownloadProgress((progress: DownloadProgress) => {
      if (progress.model_id === downloadingModelId) {
        setDownloadProgress(progress.percentage);
      }
    }).then((unlistenFn) => {
      unlisten = unlistenFn;
    });
    return () => {
      if (unlisten) unlisten();
    };
  }, [downloadingModelId, setDownloadProgress]);

  useEffect(() => {
    if (!selectedModel || selectedModel.isCloud) return;
    isModelDownloaded(selectedModel.id)
      .then((downloaded) => {
        if (downloaded) setModelStatus("downloaded");
      })
      .catch(console.error);
  }, [selectedModel, setModelStatus]);

  const handleSelectModel = (modelId: string) => {
    if (modelStatus === "downloading") return;
    const model = models.find((m) => m.id === modelId);
    if (model) {
      setSelectedModel(model);
      if (!isLanguageSupportedByModel(model, settings.language)) {
        updateSettings({ language: getDefaultLanguageForModel(model) });
      }
      setDownloadError(null);
      if (model.isCloud) {
        setModelStatus("downloaded");
        return;
      }
      isModelDownloaded(modelId)
        .then((downloaded) =>
          setModelStatus(downloaded ? "downloaded" : "not-downloaded")
        )
        .catch(console.error);
    }
  };

  const handleDownload = async () => {
    if (!selectedModel) return;
    setDownloadingModelId(selectedModel.id);
    setModelStatus("downloading");
    setDownloadProgress(0);
    setDownloadError(null);

    try {
      const modelPath = await downloadModel(selectedModel.id);
      setDownloadProgress(100);
      setModelStatus("downloaded");
      setDownloadingModelId(null);
      markModelDownloaded(selectedModel.id, modelPath);
    } catch (error) {
      console.error("Download failed:", error);
      setModelStatus("error");
      setDownloadingModelId(null);
      setDownloadError(error instanceof Error ? error.message : "Download failed");
    }
  };

  const isDownloading = modelStatus === "downloading";
  const isDownloaded =
    modelStatus === "downloaded" ||
    modelStatus === "ready" ||
    selectedModel?.downloaded ||
    selectedModel?.isCloud;
  const canContinue = selectedModel && isDownloaded;

  return (
    <div className="flex h-full flex-col overflow-hidden bg-canvas">
      <div className="flex-1 overflow-y-auto">
        <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-6 xl:py-10 space-y-6">
          {/* HERO — Dark band */}
          <section className="hero-band-dark">
            <div className="flex flex-col items-center text-center gap-4 p-8 sm:p-10">
              <p className="text-[11px] font-medium tracking-[0.1em] uppercase leading-none text-primary">
                <span className="inline-flex items-center gap-2">
                  <span className="h-1 w-1 rounded-full bg-primary" />
                  Step {STEP_INDEX} of {STEPS_TOTAL}
                </span>
              </p>
              <h2 className="text-[28px] font-medium leading-tight tracking-tight text-on-dark">
                Choose your <span className="text-primary">engine</span>.
              </h2>
              <p className="text-sm text-on-dark-soft max-w-md leading-relaxed">
                Larger models are more accurate but need more storage. Pick one to get started.
              </p>
            </div>
          </section>

          {/* MODEL CARDS */}
          <section className="card-feature-cream">
            <div className="flex items-center gap-3 mb-5">
              <div className="flex items-center justify-center size-8 rounded-full bg-primary/12 text-primary">
                <Sparkles className="h-4 w-4" />
              </div>
              <div className="min-w-0 flex-1">
                <p className="text-[11px] font-medium tracking-[0.1em] uppercase leading-none text-ink-mid">Models</p>
                <h3 className="text-[15px] font-semibold tracking-tight text-ink mt-1">
                  {models.length} available
                </h3>
              </div>
            </div>

            <div className="space-y-3">
              {models.map((model) => {
                const isSelected = selectedModel?.id === model.id;
                const isThisDownloading = downloadingModelId === model.id;
                const isRecommended = model.id === ONBOARDING_RECOMMENDED_MODEL_ID;

                return (
                  <button
                    key={model.id}
                    type="button"
                    onClick={() => handleSelectModel(model.id)}
                    disabled={isDownloading && !isThisDownloading}
                    className={cn(
                      "w-full text-left rounded-md border p-4 transition-all cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed",
                      isSelected
                        ? "border-primary bg-canvas"
                        : "border-hairline bg-canvas hover:border-ink",
                    )}
                    style={isSelected ? { boxShadow: "0 0 0 1px #ff4f00" } : undefined}
                  >
                    <div className="flex items-start gap-3 sm:gap-4">
                      <div
                        className={cn(
                          "flex h-10 w-10 items-center justify-center rounded-md shrink-0",
                          isSelected
                            ? "bg-primary text-on-dark"
                            : "bg-canvas-soft text-ink",
                        )}
                      >
                        <Cpu className="h-4 w-4" />
                      </div>
                      <div className="min-w-0 flex-1">
                        <div className="flex items-center gap-2 flex-wrap">
                          <span className="text-xs font-semibold text-ink">
                            {model.name}
                          </span>
                          {model.isCloud && (
                            <span className="text-[11px] font-medium uppercase tracking-[0.08em] px-2 py-0.5 rounded-full bg-primary/10 text-primary">
                              Cloud
                            </span>
                          )}
                          {isSelected && (
                            <span
                              className="text-[11px] font-medium uppercase tracking-[0.08em] px-2 py-0.5 rounded-full"
                              style={{ background: "rgba(255,79,0,0.1)", color: "#ff4f00" }}
                            >
                              Selected
                            </span>
                          )}
                          {isRecommended && !isSelected && (
                            <span className="text-[11px] font-medium uppercase tracking-[0.08em] text-ink bg-canvas-soft px-2 py-0.5 rounded-full inline-flex items-center gap-1">
                              <Star className="h-3 w-3" />
                              Recommended
                            </span>
                          )}
                        </div>
                        <p className="text-xs text-body-muted mt-1.5">
                          {model.description}
                        </p>
                        <div className="flex items-center gap-2 mt-2.5 flex-wrap">
                          <span className="inline-flex items-center gap-1.5 text-[11px] px-2 py-1 rounded-md bg-canvas-soft text-body">
                            <HardDrive className="h-3 w-3" />
                            {model.size}
                          </span>
                          <span className="text-[11px] px-2 py-1 rounded-md bg-canvas-soft text-body">
                            {getModelLanguageLabel(model)}
                          </span>
                        </div>

                        {isThisDownloading && (
                          <div className="mt-3 space-y-1.5">
                            <div className="h-1.5 bg-canvas-soft rounded-full overflow-hidden">
                              <div
                                className="h-full bg-primary transition-all duration-300 rounded-full"
                                style={{ width: `${downloadProgress}%` }}
                              />
                            </div>
                            <div className="flex items-center justify-between">
                              <p className="text-[11px] text-body-muted">Downloading...</p>
                              <p className="text-[11px] font-medium uppercase tracking-[0.08em] text-primary tabular-nums">
                                {Math.round(downloadProgress)}%
                              </p>
                            </div>
                          </div>
                        )}

                        {model.downloaded && !isThisDownloading && (
                          <div className="mt-2.5 flex items-center gap-1.5 text-[11px] font-medium uppercase tracking-[0.08em] text-primary">
                            <Check className="h-3 w-3" />
                            Downloaded
                          </div>
                        )}
                      </div>
                    </div>
                  </button>
                );
              })}
            </div>

            {downloadError && (
              <div
                className="mt-4 p-3.5 rounded-md border flex items-start gap-2.5 border-[#cf202f]/30 bg-[#cf202f]/5 text-[#cf202f]"
              >
                <p className="text-xs">{downloadError}</p>
              </div>
            )}
          </section>
        </div>
      </div>

      {/* STICKY FOOTER */}
      <div className="shrink-0 border-t border-hairline bg-canvas-soft">
        <div className="max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-4 flex items-center justify-between gap-3 flex-wrap">
          <button
            onClick={onBack}
            disabled={isDownloading}
            className="paper-button-outline cursor-pointer disabled:opacity-50"
          >
            Back
          </button>

          {!isDownloaded ? (
            selectedModel?.isCloud ? (
              <button
                onClick={onNext}
                disabled={!selectedModel}
                className="paper-button-primary cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Continue
              </button>
            ) : (
              <button
                onClick={handleDownload}
                disabled={!selectedModel || isDownloading}
                className="paper-button-primary cursor-pointer disabled:cursor-not-allowed disabled:opacity-50"
              >
                {isDownloading ? <Loader2 className="h-4 w-4 animate-spin" /> : null}
                {isDownloading
                  ? `Downloading ${Math.round(downloadProgress)}%`
                  : `Download ${selectedModel?.size ?? ""}`.trim()}
              </button>
            )
          ) : (
            <button
              onClick={onNext}
              disabled={!canContinue}
              className="paper-button-primary cursor-pointer disabled:cursor-not-allowed disabled:opacity-50"
            >
              Continue
            </button>
          )}
        </div>
      </div>
    </div>
  );
}