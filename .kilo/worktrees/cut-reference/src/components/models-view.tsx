import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useToast } from "@/hooks/use-toast";
import { cn } from "@/lib/utils";
import {
  cancelModelDownload,
  deleteModel,
  downloadModel,
  onDownloadProgress,
  reportError,
} from "@/lib/voice-api";
import { useAppStore, useAvailableModels, useIsInitialized } from "@/store";
import {
  getDefaultLanguageForModel,
  getModelCategories,
  getModelLanguageLabel,
  getModelLanguageOptions,
  isLanguageSupportedByModel,
  type ModelBadgeCategory,
  type WhisperModel,
} from "@/types";
import { LANGUAGE_NAMES } from "@/types";
import {
  AlertCircle,
  ArrowRight,
  Check,
  Cpu,
  Gauge,
  Globe,
  HardDrive,
  Loader2,
  Star,
  Trash2,
  X,
  Zap,
  Circle,
} from "lucide-react";
import { useEffect, useState } from "react";

interface ModelsViewProps {
  onClose: () => void;
}

export function ModelsView(_props: ModelsViewProps) {
  const {
    initializeFromDb,
    selectedModel,
    setSelectedModel,
    settings,
    updateSettings,
    markModelDownloaded,
    setAvailableModels,
  } = useAppStore();
  const availableModels = useAvailableModels();
  const isInitialized = useIsInitialized();
  const { success: toastSuccess, error: toastError } = useToast();

  const [downloadingModelId, setDownloadingModelId] = useState<string | null>(
    null
  );
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [deletingModelId, setDeletingModelId] = useState<string | null>(null);
  const [cancelingModelId, setCancelingModelId] = useState<string | null>(null);
  const [rowErrors, setRowErrors] = useState<Record<string, string>>({});
  const [pageError, setPageError] = useState<string | null>(null);
  const [isRetrying, setIsRetrying] = useState(false);
  const languageOptions = selectedModel
    ? getModelLanguageOptions(selectedModel)
    : [];
  const selectedLanguage =
    selectedModel && isLanguageSupportedByModel(selectedModel, settings.language)
      ? settings.language
      : languageOptions[0]?.code ?? "en";

  useEffect(() => {
    if (
      selectedModel &&
      !isLanguageSupportedByModel(selectedModel, settings.language)
    ) {
      updateSettings({ language: getDefaultLanguageForModel(selectedModel) });
    }
  }, [selectedModel, settings.language, updateSettings]);

  useEffect(() => {
    const unsubscribe = onDownloadProgress((progress) => {
      if (progress.model_id === downloadingModelId) {
        setDownloadProgress(Math.min(100, Math.max(0, progress.percentage)));
      }
    });

    return () => {
      unsubscribe
        .then((fn) => fn())
        .catch((err) => {
          console.error("Failed to unsubscribe download listener:", err);
        });
    };
  }, [downloadingModelId]);

  const getErrorMessage = (error: unknown) => {
    if (error instanceof Error) return error.message;
    if (typeof error === "string") return error;
    return "Something went wrong. Please try again.";
  };

  const setRowError = (modelId: string, message: string | null) => {
    setRowErrors((current) => {
      const next = { ...current };
      if (message) {
        next[modelId] = message;
      } else {
        delete next[modelId];
      }
      return next;
    });
  };

  const handleRetryLoad = async () => {
    try {
      setIsRetrying(true);
      setPageError(null);
      await initializeFromDb();
    } catch (err) {
      const message = getErrorMessage(err);
      setPageError(message);
      await reportError("model", message, "error", {
        userAction: "Retry loading model list",
      }).catch(console.error);
    } finally {
      setIsRetrying(false);
    }
  };

  const handleDownloadModel = async (model: WhisperModel) => {
    if (downloadingModelId || deletingModelId) return;

    try {
      setRowError(model.id, null);
      setDownloadingModelId(model.id);
      setCancelingModelId(null);
      setDownloadProgress(0);

      const modelPath = await downloadModel(model.id);
      markModelDownloaded(model.id, modelPath);
      if (!selectedModel) {
        setSelectedModel({ ...model, downloaded: true });
        updateSettings({ language: getDefaultLanguageForModel(model) });
      }
      toastSuccess("Model downloaded", `${model.name} is ready to use`);
    } catch (err) {
      const message = getErrorMessage(err);
      console.error("Download failed:", err);
      if (message.toLowerCase().includes("cancelled")) {
        toastSuccess("Download canceled", `${model.name} was not installed`);
        return;
      }
      setRowError(model.id, message);
      toastError("Download failed", `Failed to download ${model.name} model`);
      await reportError("model", message, "error", {
        userAction: `Download model: ${model.id}`,
        context: { modelId: model.id },
      }).catch(console.error);
    } finally {
      setDownloadingModelId(null);
      setCancelingModelId(null);
    }
  };

  const handleCancelDownload = async (model: WhisperModel) => {
    if (downloadingModelId !== model.id || cancelingModelId) return;

    try {
      setCancelingModelId(model.id);
      const canceled = await cancelModelDownload(model.id);
      if (!canceled) {
        setCancelingModelId(null);
        setRowError(model.id, "Could not cancel because the download is no longer active.");
      }
    } catch (err) {
      const message = getErrorMessage(err);
      setCancelingModelId(null);
      setRowError(model.id, message);
      await reportError("model", message, "error", {
        userAction: `Cancel model download: ${model.id}`,
        context: { modelId: model.id },
      }).catch(console.error);
    }
  };

  const handleDeleteModel = async (model: WhisperModel) => {
    if (!model.downloaded || downloadingModelId || deletingModelId) return;

    try {
      setRowError(model.id, null);
      setDeletingModelId(model.id);
      await deleteModel(model.id);

      if (selectedModel?.id === model.id) {
        setSelectedModel(null);
      }

      setAvailableModels(
        availableModels.map((availableModel) =>
          availableModel.id === model.id
            ? { ...availableModel, downloaded: false }
            : availableModel
        )
      );
      toastSuccess("Model deleted", `${model.name} has been removed`);
    } catch (err) {
      const message = getErrorMessage(err);
      console.error("Delete failed:", err);
      setRowError(model.id, message);
      toastError("Delete failed", `Failed to delete ${model.name} model`);
      await reportError("model", message, "error", {
        userAction: `Delete model: ${model.id}`,
        context: { modelId: model.id },
      }).catch(console.error);
    } finally {
      setDeletingModelId(null);
    }
  };

  const handleSelectModel = (model: WhisperModel) => {
    if (downloadingModelId || deletingModelId) return;

    if (model.downloaded) {
      setRowError(model.id, null);
      setSelectedModel(model);
      if (!isLanguageSupportedByModel(model, settings.language)) {
        updateSettings({ language: getDefaultLanguageForModel(model) });
      }
    }
  };

  const categoryIcon = (category: ModelBadgeCategory) => {
    if (category === "recommended") {
      return <Star className="h-3 w-3" />;
    }
    if (category === "accurate") {
      return <Gauge className="h-3 w-3" />;
    }
    if (category === "fast") {
      return <Zap className="h-3 w-3" />;
    }
    return <HardDrive className="h-3 w-3" />;
  };

  const categoryLabel: Record<ModelBadgeCategory, string> = {
    recommended: "Recommended",
    accurate: "Accurate",
    fast: "Fast",
    compact: "Small",
  };

  const categoryAccent: Record<ModelBadgeCategory, string> = {
    recommended: "bg-primary/10 text-primary",
    accurate: "bg-canvas-soft text-ink",
    fast: "bg-canvas-soft text-ink",
    compact: "bg-canvas-soft text-ink",
  };

  const isBusy = Boolean(downloadingModelId || deletingModelId);
  const hasModels = availableModels.length > 0;
  const downloadedCount = availableModels.filter(m => m.downloaded).length;
  const currentLanguageName =
    languageOptions.find((l) => l.code === selectedLanguage)?.name ??
    (selectedLanguage === "auto" ? "Auto detect" : selectedLanguage.toUpperCase());

  return (
    <div className="flex h-full flex-col overflow-hidden bg-canvas">
      {/* ─── HEADER — Cream band with active status ─── */}
      <div className="shrink-0 border-b border-hairline">
        <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-4 sm:py-5 flex items-center justify-between gap-4 flex-wrap">
          <div className="min-w-0">
            <p className="eyebrow-uppercase text-ink-mid">Models</p>
            <h1
              className="font-display font-medium text-ink mt-2 text-[clamp(1.375rem,2.2vw,1.875rem)]"
              style={{ lineHeight: 1.1, letterSpacing: '-0.02em' }}
            >
              Choose your engine.
            </h1>
          </div>
          {selectedModel && (
            <div className="flex items-center gap-3 min-w-0">
              <span className="eyebrow-uppercase text-ink-mid hidden sm:inline">Active</span>
              <span className="badge-pill border border-hairline">
                <Circle className="h-1.5 w-1.5 fill-primary text-primary shrink-0" />
                <span className="truncate max-w-[13.875rem]">{selectedModel.name}</span>
              </span>
            </div>
          )}
        </div>
      </div>

      <div className="flex-1 overflow-y-auto">
        {!isInitialized ? (
          <div className="flex items-center justify-center h-full p-6">
            <div className="flex flex-col items-center gap-3">
              <Loader2 className="h-8 w-8 animate-spin text-body-muted" />
              <p className="body-sm text-body-muted">Loading models...</p>
            </div>
          </div>
        ) : !hasModels || pageError ? (
          <div className="flex items-center justify-center h-full p-6">
            <div className="paper-card p-8 flex flex-col items-center text-center max-w-sm">
              <div className="icon-plate-orange mb-5">
                <AlertCircle className="h-5 w-5" />
              </div>
              <h3 className="font-display font-medium text-ink text-[1.125rem]" style={{ letterSpacing: '-0.015em' }}>
                Models unavailable
              </h3>
              <p className="body-sm text-body-muted mt-2">
                {pageError || "The model list could not be loaded."}
              </p>
              <button
                className="paper-button-primary mt-6 cursor-pointer disabled:cursor-not-allowed disabled:opacity-50"
                onClick={handleRetryLoad}
                disabled={isRetrying}
              >
                {isRetrying ? <Loader2 className="h-4 w-4 animate-spin" /> : null}
                {isRetrying ? "Retrying..." : "Retry"}
              </button>
            </div>
          </div>
        ) : (
          <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-6 xl:py-10 space-y-6 xl:space-y-8">

            {/* ─── HERO STATUS BAND — Dark coffee ink ─── */}
            <section className="hero-band-dark">
              <div className="grid grid-cols-1 @xl:grid-cols-[1.4fr_1fr] gap-5 @xl:gap-8 p-5 sm:p-7 @xl:p-10 items-start @xl:items-center">
                <div className="min-w-0">
                  <p className="eyebrow-uppercase text-primary mb-3">
                    <span className="inline-flex items-center gap-2">
                      <Circle className="h-1.5 w-1.5 fill-primary text-primary" />
                      Local transcription
                    </span>
                  </p>
<h2
                    className="font-display font-medium text-on-dark text-[clamp(1.375rem,2.4vw,2.125rem)]"
                    style={{ lineHeight: 1.05, letterSpacing: '-0.02em' }}
                  >
                    {downloadedCount > 0
                      ? <>You have <span className="text-primary">{downloadedCount}</span> model{downloadedCount === 1 ? "" : "s"} ready.</>
                      : <>Download your first model to start.</>}
                  </h2>
                  <p className="body-md text-on-dark-soft mt-3 max-w-xl">
                    {selectedModel
                      ? `Currently using ${selectedModel.name}. Download more below to compare speed, accuracy, and size.`
                      : "Pick a model that fits your machine. Smaller models load faster; larger ones transcribe more accurately."}
                  </p>
                </div>

                {/* Right: language picker card */}
                {selectedModel ? (
                  <div className="product-ui-card-dark w-full">
                    <div className="flex items-center gap-3 mb-4">
                      <div className="icon-plate-dark">
                        <Globe className="h-4 w-4 text-on-dark" />
                      </div>
                      <div className="min-w-0 flex-1">
                        <p className="caption-strong text-on-dark">Spoken language</p>
                        <p className="caption text-on-dark-soft mt-1 truncate">{currentLanguageName}</p>
                      </div>
                    </div>

                    {languageOptions.length > 0 ? (
                      <Select
                        value={selectedLanguage}
                        onValueChange={(language) => {
                          try {
                            updateSettings({ language });
                          } catch (err) {
                            const message = getErrorMessage(err);
                            setPageError(message);
                            reportError("configuration", message, "error", {
                              userAction: "Change spoken language",
                            }).catch(console.error);
                          }
                        }}
                        disabled={isBusy}
                      >
                        <SelectTrigger
                          className="border-0 h-11 text-on-dark cursor-pointer"
                          style={{ background: '#14100e', borderRadius: '8px' }}
                        >
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          {languageOptions.map((language) => (
                            <SelectItem key={language.code} value={language.code}>
                              <div className="flex items-center gap-2.5">
                                <span
                                  className="caption-strong uppercase flex h-5 w-8 items-center justify-center rounded"
                                  style={{ background: '#f8f4f0', color: '#605d52', letterSpacing: '0.05em' }}
                                >
                                  {language.code === "auto" ? "AUTO" : language.code}
                                </span>
                                <span>{language.name}</span>
                              </div>
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    ) : (
                      <p className="caption text-on-dark-soft">
                        No languages configured for {selectedModel.name}.
                      </p>
                    )}
                  </div>
                ) : (
                  <div className="product-ui-card-dark w-full">
                    <div className="flex items-center gap-3">
                      <div className="icon-plate-dark">
                        <Cpu className="h-4 w-4 text-on-dark" />
                      </div>
                      <div className="min-w-0">
                        <p className="caption-strong text-on-dark">No active model</p>
                        <p className="caption text-on-dark-soft mt-1">Download one below to begin.</p>
                      </div>
                    </div>
                  </div>
                )}
              </div>
            </section>

            {/* ─── MODELS LIST ─── */}
            <section className="flex flex-col gap-4 sm:gap-5">
              <div className="flex items-end justify-between gap-3 flex-wrap">
                <div className="min-w-0">
                  <p className="eyebrow-uppercase text-ink-mid">Available</p>
                  <h2
                    className="font-display font-medium text-ink mt-2 text-[clamp(1.125rem,1.8vw,1.5rem)]"
                    style={{ lineHeight: 1.1, letterSpacing: '-0.02em' }}
                  >
                    All models
                  </h2>
                </div>
                <p className="caption text-body-muted">
                  {availableModels.length} total · {downloadedCount} downloaded
                </p>
              </div>

              <div className="grid grid-cols-1 min-[640px]:grid-cols-2 @3xl:grid-cols-2 gap-4 sm:gap-5">
                {availableModels.map((model) => {
                  const isActive = selectedModel?.id === model.id;
                  const isDownloading = downloadingModelId === model.id;
                  const isCanceling = cancelingModelId === model.id;
                  const isDeleting = deletingModelId === model.id;
                  const categories = getModelCategories(model);

                  return (
                    <div
                      key={model.id}
                      className={cn(
                        "paper-card relative transition-all",
                        model.downloaded && "cursor-pointer",
                        isActive && "border-ink shadow-[0_8px_24px_-16px_rgba(32,21,21,0.25)]"
                      )}
                      onClick={() => handleSelectModel(model)}
                    >
                      {/* Active indicator stripe */}
                      {isActive && (
                        <div className="absolute left-0 top-5 bottom-5 w-1 bg-primary rounded-r-full" />
                      )}

                      <div className="flex items-start justify-between gap-3 sm:gap-4">
                        <div className="min-w-0 flex-1">
                          <div className="flex items-center gap-2 flex-wrap">
                            <h3
                              className="font-display font-medium text-ink text-[clamp(1rem,1.3vw,1.125rem)]"
                              style={{ letterSpacing: '-0.015em' }}
                            >
                              {model.name}
                            </h3>
                            {isActive && (
                              <span className="caption-strong text-primary bg-primary/10 px-2 py-0.5 rounded-full">
                                Active
                              </span>
                            )}
                            {model.downloaded && !isActive && (
                              <span className="caption-strong flex items-center gap-1 px-2 py-0.5 rounded-full" style={{ color: '#05b169', background: 'rgba(5,177,105,0.1)' }}>
                                <Check className="h-3 w-3" />
                                Ready
                              </span>
                            )}
                          </div>

                          {categories.length > 0 && (
                            <div className="flex items-center gap-1.5 mt-2.5 flex-wrap">
                              {categories.map((category) => (
                                <span
                                  key={category}
                                  className={cn(
                                    "inline-flex items-center gap-1 caption px-2 py-0.5 rounded-full",
                                    categoryAccent[category],
                                  )}
                                >
                                  {categoryIcon(category)}
                                  {categoryLabel[category]}
                                </span>
                              ))}
                            </div>
                          )}

                          <p className="body-sm text-body-muted mt-2.5 leading-relaxed">
                            {model.description}
                          </p>

                          <div className="flex items-center gap-3 mt-3 caption text-body-muted flex-wrap">
                            <span className="inline-flex items-center gap-1.5">
                              <HardDrive className="h-3 w-3" />
                              {model.size}
                            </span>
                            <span className="h-1 w-1 rounded-full bg-body-mid shrink-0" />
                            <span className="inline-flex items-center gap-1.5">
                              {getModelLanguageLabel(model)}
                              {isActive &&
                                selectedLanguage &&
                                languageOptions.some(
                                  (l) => l.code === selectedLanguage,
                                ) && (
                                  <span className="body-sm-strong text-ink">
                                    · {LANGUAGE_NAMES[selectedLanguage] ?? selectedLanguage}
                                  </span>
                                )}
                            </span>
                          </div>
                        </div>

                        <div className="flex items-center gap-1.5 sm:gap-2 shrink-0">
                          {model.downloaded ? (
                            <>
                              {!isActive && (
                                <button
                                  className="paper-button-outline size-sm cursor-pointer"
                                  style={{ borderColor: '#201515', color: '#201515' }}
                                  onClick={(event) => {
                                    event.stopPropagation();
                                    handleSelectModel(model);
                                  }}
                                  disabled={isBusy}
                                >
                                  Use
                                </button>
                              )}
                              {isActive && (
                                <div className="flex h-9 w-9 items-center justify-center rounded-md bg-primary text-on-dark">
                                  <Check className="h-4 w-4" />
                                </div>
                              )}
                              <AlertDialog>
                                <AlertDialogTrigger asChild>
                                  <button
                                    className="flex h-9 w-9 items-center justify-center rounded-md border border-hairline text-body-muted hover:border-destructive hover:text-destructive transition-colors"
                                    onClick={(event) => event.stopPropagation()}
                                    disabled={isDeleting || isBusy}
                                    aria-label={`Delete ${model.name}`}
                                  >
                                    {isDeleting ? (
                                      <Loader2 className="h-3.5 w-3.5 animate-spin" />
                                    ) : (
                                      <Trash2 className="h-3.5 w-3.5" />
                                    )}
                                  </button>
                                </AlertDialogTrigger>
                                <AlertDialogContent
                                  onClick={(event) => event.stopPropagation()}
                                >
                                  <AlertDialogHeader>
                                    <AlertDialogTitle>Delete model?</AlertDialogTitle>
                                    <AlertDialogDescription>
                                      This removes {model.name} from local storage.
                                      You can download it again later.
                                    </AlertDialogDescription>
                                  </AlertDialogHeader>
                                  <AlertDialogFooter>
                                    <AlertDialogCancel className="paper-button-secondary">Cancel</AlertDialogCancel>
                                    <AlertDialogAction
                                      onClick={() => handleDeleteModel(model)}
                                      disabled={isBusy}
                                      className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
                                    >
                                      Delete
                                    </AlertDialogAction>
                                  </AlertDialogFooter>
                                </AlertDialogContent>
                              </AlertDialog>
                            </>
                          ) : (
                            <div className="flex items-center gap-1.5">
                              <button
                                className="paper-button-primary size-sm cursor-pointer"
                                onClick={(event) => {
                                  event.stopPropagation();
                                  handleDownloadModel(model);
                                }}
                                disabled={downloadingModelId !== null}
                                title={
                                  downloadingModelId && !isDownloading
                                    ? "Wait for the current download to finish"
                                    : undefined
                                }
                              >
                                {isDownloading ? (
                                  <>
                                    <Loader2 className="h-3 w-3 animate-spin" />
                                    Downloading {Math.floor(downloadProgress)}%
                                  </>
                                ) : (
                                  "Download"
                                )}
                              </button>
                              {isDownloading && (
                                <button
                                  className="flex h-9 w-9 items-center justify-center rounded-md border border-hairline text-body-muted hover:border-destructive hover:text-destructive transition-colors"
                                  onClick={(event) => {
                                    event.stopPropagation();
                                    handleCancelDownload(model);
                                  }}
                                  disabled={isCanceling}
                                  title="Cancel download"
                                  aria-label="Cancel download"
                                >
                                  {isCanceling ? (
                                    <Loader2 className="h-3.5 w-3.5 animate-spin" />
                                  ) : (
                                    <X className="h-3.5 w-3.5" />
                                  )}
                                </button>
                              )}
                            </div>
                          )}
                        </div>
                      </div>

                      {/* Download progress */}
                      {isDownloading && (
                        <div className="mt-5">
                          <div className="h-1.5 bg-canvas-soft rounded-full overflow-hidden">
                            <div
                              className="h-full bg-primary transition-all duration-300 rounded-full"
                              style={{ width: `${downloadProgress}%` }}
                            />
                          </div>
                          <div className="flex items-center justify-between mt-2">
                            <p className="caption text-body-muted">
                              {isCanceling
                                ? "Canceling download..."
                                : "Keep WhisprTypr open while this downloads."}
                            </p>
                            <p className="caption-strong text-primary">
                              {Math.floor(downloadProgress)}%
                            </p>
                          </div>
                        </div>
                      )}

                      {/* Row errors */}
                      {rowErrors[model.id] && (
                        <div className="mt-5 p-3 rounded-md border border-destructive/30 bg-destructive/5">
                          <div className="flex items-start gap-2.5">
                            <AlertCircle className="h-4 w-4 text-destructive mt-0.5 flex-shrink-0" />
                            <div className="min-w-0 flex-1">
                              <p className="caption-strong text-destructive">
                                Action failed
                              </p>
                              <p className="body-sm text-destructive/80 mt-1 break-words">
                                {rowErrors[model.id]}
                              </p>
                            </div>
                          </div>
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
            </section>

            {/* ─── FOOTER NOTE — Local-only privacy ─── */}
            <section className="card-feature-cream">
              <div className="grid grid-cols-1 @xl:grid-cols-[auto_1fr_auto] gap-5 @xl:gap-8 items-start @xl:items-center">
                <div className="icon-plate-orange shrink-0">
                  <HardDrive className="h-5 w-5" />
                </div>
                <div className="min-w-0">
                  <p className="eyebrow-uppercase text-ink-mid mb-2">Local-only</p>
                  <h3
                    className="font-display font-medium text-ink text-[clamp(1rem,1.6vw,1.25rem)]"
                    style={{ letterSpacing: '-0.015em' }}
                  >
                    Models live on your device.
                  </h3>
                  <p className="body-sm text-body-muted mt-2 max-w-xl">
                    Downloaded models run entirely offline. Audio never leaves your machine while transcribing.
                  </p>
                </div>
                <ArrowRight className="h-5 w-5 text-body-muted hidden @xl:block" />
              </div>
            </section>
          </div>
        )}
      </div>
    </div>
  );
}