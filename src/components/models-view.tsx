import { CloudModelsTab } from "@/components/cloud-models-tab";
import {
    AlertCircle,
    ArrowRight,
    Check,
    Circle,
    Cpu,
    Gauge,
    Globe,
    HardDrive,
    Loader2,
    Star,
    Trash2,
    X,
    Zap,
} from "@/components/icons";
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
    LANGUAGE_NAMES,
    type ModelBadgeCategory,
    type WhisperModel,
} from "@/types";
import { useEffect, useMemo, useState } from "react";

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

  const [activeTab, setActiveTab] = useState<"local" | "cloud">(() => {
    return selectedModel?.isCloud ? "cloud" : "local";
  });

  const localModels = useMemo(
    () => availableModels.filter((m) => !m.isCloud),
    [availableModels]
  );
  const cloudModels = useMemo(
    () => availableModels.filter((m) => m.isCloud),
    [availableModels]
  );

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
    if (category === "fast" || category === "ultra-fast" || category === "cloud") {
      return <Zap className="h-3 w-3" />;
    }
    return <HardDrive className="h-3 w-3" />;
  };

  const categoryLabel: Record<ModelBadgeCategory, string> = {
    recommended: "Recommended",
    accurate: "Accurate",
    fast: "Fast",
    compact: "Small",
    cloud: "Cloud BYOK",
    "ultra-fast": "Ultra-fast",
  };

  const categoryAccent: Record<ModelBadgeCategory, string> = {
    recommended: "bg-primary/10 text-primary",
    accurate: "bg-canvas-soft text-ink",
    fast: "bg-canvas-soft text-ink",
    compact: "bg-canvas-soft text-ink",
    cloud: "bg-primary/10 text-primary",
    "ultra-fast": "bg-primary/10 text-primary",
  };

  const isBusy = Boolean(downloadingModelId || deletingModelId);
  const hasModels = availableModels.length > 0;
  const downloadedCount = localModels.filter((m) => m.downloaded).length;
  const currentLanguageName =
    languageOptions.find((l) => l.code === selectedLanguage)?.name ??
    (selectedLanguage === "auto" ? "Auto detect" : selectedLanguage.toUpperCase());

  return (
    <div className="flex h-full flex-col overflow-hidden bg-canvas">
      {/* ─── HEADER ─── */}
      <div className="shrink-0 border-b border-hairline">
        <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-3 sm:py-4 flex items-center justify-between gap-4 flex-wrap">
          <div>
            <p className="eyebrow-uppercase text-ink-mid">Models</p>
            <h1 className="display-sm text-ink mt-0.5">
              Choose your <span className="text-primary">engine</span>.
            </h1>
          </div>

          {/* Segmented Tab Controls - Zapier styled */}
          <div className="flex items-center gap-1 p-1 rounded-xl bg-canvas-soft border border-hairline">
            <button
              onClick={() => setActiveTab("local")}
              className={cn(
                "caption-strong px-3.5 py-1.5 rounded-lg transition-all cursor-pointer flex items-center gap-1.5 text-xs",
                activeTab === "local"
                  ? "bg-ink text-on-dark shadow-sm"
                  : "text-body-muted hover:text-ink hover:bg-canvas"
              )}
            >
              <HardDrive className="h-3.5 w-3.5" />
              Local Models
              <span
                className={cn(
                  "caption px-1.5 py-0.2 rounded-full text-[10px] ml-0.5",
                  activeTab === "local"
                    ? "bg-white/20 text-white"
                    : "bg-black/5 text-body-muted"
                )}
              >
                {localModels.length}
              </span>
            </button>
            <button
              onClick={() => setActiveTab("cloud")}
              className={cn(
                "caption-strong px-3.5 py-1.5 rounded-lg transition-all cursor-pointer flex items-center gap-1.5 text-xs",
                activeTab === "cloud"
                  ? "bg-primary text-primary-foreground shadow-sm"
                  : "text-body-muted hover:text-ink hover:bg-canvas"
              )}
            >
              <Zap className="h-3.5 w-3.5" />
              Cloud Models (BYOK)
              <span
                className={cn(
                  "caption-strong px-1.5 py-0.2 rounded-full text-[10px] ml-0.5",
                  activeTab === "cloud"
                    ? "bg-black/20 text-white"
                    : "bg-primary/10 text-primary"
                )}
              >
                BYOK
              </span>
            </button>
          </div>
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
            <div className="paper-card p-6 flex flex-col items-center text-center max-w-sm">
              <div className="icon-plate-orange mb-4">
                <AlertCircle className="h-4 w-4" />
              </div>
              <h3 className="title-md text-ink">
                Models unavailable
              </h3>
              <p className="body-sm text-body-muted mt-1.5">
                {pageError || "The model list could not be loaded."}
              </p>
              <button
                className="paper-button-primary mt-4 cursor-pointer disabled:cursor-not-allowed disabled:opacity-50"
                onClick={handleRetryLoad}
                disabled={isRetrying}
              >
                {isRetrying ? <Loader2 className="h-4 w-4 animate-spin" /> : null}
                {isRetrying ? "Retrying..." : "Retry"}
              </button>
            </div>
          </div>
        ) : activeTab === "cloud" ? (
          <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-4 xl:py-5">
            <CloudModelsTab
              cloudModels={cloudModels}
              selectedModel={selectedModel}
              onSelectModel={handleSelectModel}
              isBusy={isBusy}
            />
          </div>
        ) : (
          <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-4 xl:py-5 space-y-4 xl:space-y-5">

            {/* ─── HERO STATUS BAND — Dark coffee ink ─── */}
            <section className="hero-band-dark">
              <div className="grid grid-cols-1 @xl:grid-cols-[1.4fr_1fr] gap-4 @xl:gap-5 p-4 sm:p-5 @xl:p-6 items-start @xl:items-center">
                <div className="min-w-0">
                  <p className="eyebrow-uppercase text-primary mb-2">
                    <span className="inline-flex items-center gap-2">
                      <Circle className="h-1.5 w-1.5 fill-primary text-primary" />
                      Local transcription
                    </span>
                  </p>
<h2
                    className="display-md text-on-dark"
                  >
                    {downloadedCount > 0
                      ? <>You have <span className="text-primary">{downloadedCount}</span> model{downloadedCount === 1 ? "" : "s"} ready.</>
                      : <>Download your first model to start.</>}
                  </h2>
                  <p className="body-sm text-on-dark-soft mt-2 max-w-xl">
                    {selectedModel
                      ? `Currently using ${selectedModel.name}. Download more below to compare speed, accuracy, and size.`
                      : "Pick a model that fits your machine. Smaller models load faster; larger ones transcribe more accurately."}
                  </p>
                </div>

                {/* Right: language picker card */}
                {selectedModel ? (
                  <div className="product-ui-card-dark w-full">
                    <div className="flex items-center gap-2.5 mb-3">
                      <div className="icon-plate-dark">
                        <Globe className="h-3.5 w-3.5 text-on-dark" />
                      </div>
                      <div className="min-w-0 flex-1">
                        <p className="caption-strong text-on-dark">Spoken language</p>
                        <p className="caption text-on-dark-soft mt-0.5 truncate">{currentLanguageName}</p>
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
                          className="border-0 h-9 text-on-dark cursor-pointer"
                          style={{ background: '#14100e', borderRadius: '8px' }}
                        >
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          {languageOptions.map((language) => (
                            <SelectItem key={language.code} value={language.code}>
                              <div className="flex items-center gap-2">
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
                    <div className="flex items-center gap-2.5">
                      <div className="icon-plate-dark">
                        <Cpu className="h-3.5 w-3.5 text-on-dark" />
                      </div>
                      <div className="min-w-0">
                        <p className="caption-strong text-on-dark">No active model</p>
                        <p className="caption text-on-dark-soft mt-0.5">Download one below to begin.</p>
                      </div>
                    </div>
                  </div>
                )}
              </div>
            </section>

            {/* ─── MODELS LIST ─── */}
            <section className="flex flex-col gap-3 sm:gap-4">
              <div className="flex items-end justify-between gap-2 flex-wrap">
                <div className="min-w-0">
                  <p className="eyebrow-uppercase text-ink-mid">Available</p>
                  <h2
                    className="display-xs text-ink mt-1"
                  >
                    All models
                  </h2>
                </div>
                <p className="caption text-body-muted">
                  {localModels.length} total · {downloadedCount} downloaded
                </p>
              </div>

              <div className="grid grid-cols-1 min-[640px]:grid-cols-2 @3xl:grid-cols-2 gap-3 sm:gap-4">
                {localModels.map((model) => {
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
                        <div className="absolute left-0 top-4 bottom-4 w-1 bg-primary rounded-r-full" />
                      )}

                      <div className="flex items-start justify-between gap-2.5 sm:gap-3">
                        <div className="min-w-0 flex-1">
                          <div className="flex items-center gap-2 flex-wrap">
                            <h3
                              className="title-sm text-ink"
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
                                <Check className="h-2.5 w-2.5" />
                                Ready
                              </span>
                            )}
                          </div>

                          {categories.length > 0 && (
                            <div className="flex items-center gap-1.5 mt-1.5 flex-wrap">
                              {categories.map((category) => (
                                <span
                                  key={category}
                                  className={cn(
                                    "inline-flex items-center gap-1 caption px-1.5 py-0.5 rounded-full",
                                    categoryAccent[category],
                                  )}
                                >
                                  {categoryIcon(category)}
                                  {categoryLabel[category]}
                                </span>
                              ))}
                            </div>
                          )}

                          <p className="body-sm text-body-muted mt-1.5 leading-relaxed">
                            {model.description}
                          </p>

                          <div className="flex items-center gap-2.5 mt-2 caption text-body-muted flex-wrap">
                            <span className="inline-flex items-center gap-1">
                              <HardDrive className="h-2.5 w-2.5" />
                              {model.size}
                            </span>
                            <span className="h-1 w-1 rounded-full bg-body-mid shrink-0" />
                            <span className="inline-flex items-center gap-1">
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

                        <div className="flex items-center gap-1.5 shrink-0">
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
                                <div className="flex h-8 w-8 items-center justify-center rounded-md bg-primary text-on-dark">
                                  <Check className="h-3.5 w-3.5" />
                                </div>
                              )}
                              <AlertDialog>
                                <AlertDialogTrigger asChild>
                                  <button
                                    className="flex h-8 w-8 items-center justify-center rounded-md border border-hairline text-body-muted hover:border-destructive hover:text-destructive transition-colors"
                                    onClick={(event) => event.stopPropagation()}
                                    disabled={isDeleting || isBusy}
                                    aria-label={`Delete ${model.name}`}
                                  >
                                    <Trash2 className="h-3 w-3" />
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
                                    <Loader2 className="h-2.5 w-2.5 animate-spin" />
                                    {Math.floor(downloadProgress)}%
                                  </>
                                ) : (
                                  "Download"
                                )}
                              </button>
                              {isDownloading && (
                                <button
                                  className="flex h-8 w-8 items-center justify-center rounded-md border border-hairline text-body-muted hover:border-destructive hover:text-destructive transition-colors"
                                  onClick={(event) => {
                                    event.stopPropagation();
                                    handleCancelDownload(model);
                                  }}
                                  disabled={isCanceling}
                                  title="Cancel download"
                                  aria-label="Cancel download"
                                >
                                  {isCanceling ? (
                                    <Loader2 className="h-3 w-3 animate-spin" />
                                  ) : (
                                    <X className="h-3 w-3" />
                                  )}
                                </button>
                              )}
                            </div>
                          )}
                        </div>
                      </div>

                      {/* Download progress */}
                      {isDownloading && (
                        <div className="mt-3">
                          <div className="h-1 bg-canvas-soft rounded-full overflow-hidden">
                            <div
                              className="h-full bg-primary transition-all duration-300 rounded-full"
                              style={{ width: `${downloadProgress}%` }}
                            />
                          </div>
                          <div className="flex items-center justify-between mt-1.5">
                            <p className="caption text-body-muted">
                              {isCanceling
                                ? "Canceling download..."
                                : "Keep Whisprtypr open while this downloads."}
                            </p>
                            <p className="caption-strong text-primary">
                              {Math.floor(downloadProgress)}%
                            </p>
                          </div>
                        </div>
                      )}

                      {/* Row errors */}
                      {rowErrors[model.id] && (
                        <div className="mt-3 p-2.5 rounded-md border border-destructive/30 bg-destructive/5">
                          <div className="flex items-start gap-2">
                            <AlertCircle className="h-3.5 w-3.5 text-destructive mt-0.5 flex-shrink-0" />
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
              <div className="grid grid-cols-1 @xl:grid-cols-[auto_1fr_auto] gap-3 @xl:gap-5 items-start @xl:items-center">
                <div className="icon-plate-orange shrink-0">
                  <HardDrive className="h-4 w-4" />
                </div>
                <div className="min-w-0">
                  <p className="eyebrow-uppercase text-ink-mid mb-1">Local-only</p>
                  <h3
                    className="title-md text-ink"
                  >
                    Models live on your device.
                  </h3>
                  <p className="body-sm text-body-muted mt-1.5 max-w-xl">
                    Downloaded models run entirely offline. Audio never leaves your machine while transcribing.
                  </p>
                </div>
                <ArrowRight className="h-4 w-4 text-body-muted hidden @xl:block" />
              </div>
            </section>
          </div>
        )}
      </div>
    </div>
  );
}