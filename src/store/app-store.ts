import {
  dbGetAppState,
  dbGetModels,
  dbGetSettings,
  dbModelsToFrontend,
  dbSetCurrentSetupStep,
  dbSetModelDownloaded,
  dbSetSelectedModel,
  dbSetSetupComplete,
  dbSettingsToFrontend,
  dbUpdateSettings,
  frontendSettingsToDb,
} from "@/lib/database-api";
import { getCloudProviders } from "@/lib/cloud-api";
import { getAiFormattingProviders } from "@/lib/ai-formatting-api";
import type {
  AppSettings,
  AppState,
  CloudProviderInfo,
  AiFormattingProviderInfo,
  ModelStatus,
  RecordingStatus,
  WhisperModel,
} from "@/types";
import { CLOUD_MODELS, DEFAULT_SETTINGS } from "@/types";
import { create } from "zustand";

function buildMergedModels(
  localModels: WhisperModel[],
  providers: CloudProviderInfo[]
): WhisperModel[] {
  const providerMap = new Map<string, boolean>();
  for (const p of providers) {
    providerMap.set(p.id, p.configured);
  }

  const cloud = CLOUD_MODELS.map((m) => {
    const isConfigured = Boolean(m.provider && providerMap.get(m.provider));
    return {
      ...m,
      downloaded: isConfigured,
    };
  });

  return [...localModels, ...cloud];
}

interface AppStore extends AppState {
  // Initialization
  isInitialized: boolean;
  initializeFromDb: () => Promise<void>;

  // Setup actions
  setSetupComplete: (complete: boolean) => void;
  setCurrentSetupStep: (step: number) => void;
  nextSetupStep: () => void;
  prevSetupStep: () => void;

  // Recording actions
  setRecordingStatus: (status: RecordingStatus) => void;
  setLastTranscription: (text: string) => void;
  setErrorMessage: (message: string | null) => void;

  // Model actions
  setModelStatus: (status: ModelStatus) => void;
  setSelectedModel: (model: WhisperModel | null) => void;
  setDownloadProgress: (progress: number) => void;
  setAvailableModels: (models: WhisperModel[]) => void;
  markModelDownloaded: (modelId: string, path?: string) => void;
  setModelReady: (ready: boolean) => void;
  modelReady: boolean;

  // Cloud providers
  cloudProviders: CloudProviderInfo[];
  refreshCloudProviders: () => Promise<void>;

  // AI formatting providers (BYOK)
  aiFormattingProviders: AiFormattingProviderInfo[];
  refreshAiFormattingProviders: () => Promise<void>;

  // Settings actions
  updateSettings: (settings: Partial<AppSettings>) => void;
  resetSettings: () => void;

  // Available models (Local + Cloud BYOK)
  availableModels: WhisperModel[];

  // Utility
  reset: () => void;
}

const initialState: AppState & {
  availableModels: WhisperModel[];
  cloudProviders: CloudProviderInfo[];
  aiFormattingProviders: AiFormattingProviderInfo[];
  isInitialized: boolean;
  modelReady: boolean;
} = {
  isInitialized: false,
  isFirstLaunch: true,
  setupComplete: false,
  currentSetupStep: 0,
  recordingStatus: "idle",
  lastTranscription: "",
  errorMessage: null,
  modelStatus: "not-downloaded",
  modelReady: false,
  selectedModel: null,
  downloadProgress: 0,
  settings: DEFAULT_SETTINGS,
  availableModels: [],
  cloudProviders: [],
  aiFormattingProviders: [],
};

export const useAppStore = create<AppStore>()((set, get) => ({
  ...initialState,

  // Initialize from SQLite database
  initializeFromDb: async () => {
    console.log("[Store] Starting initialization from database...");
    try {
      const [dbState, dbSettings, dbModels, providers, aiProviders] = await Promise.all([
        dbGetAppState(),
        dbGetSettings(),
        dbGetModels(),
        getCloudProviders(),
        getAiFormattingProviders(),
      ]);

      const settings = dbSettingsToFrontend(dbSettings);
      const localModels = dbModelsToFrontend(dbModels);
      const mergedModels = buildMergedModels(localModels, providers);

      // Find selected model (either local or cloud)
      const selectedModel = dbState.selected_model_id
        ? mergedModels.find((m) => m.id === dbState.selected_model_id) || null
        : null;

      // Determine model status
      let modelStatus: ModelStatus = "not-downloaded";
      if (selectedModel?.downloaded) {
        modelStatus = "downloaded";
      }

      set({
        isInitialized: true,
        isFirstLaunch: dbState.is_first_launch,
        setupComplete: dbState.setup_complete,
        currentSetupStep: dbState.current_setup_step,
        selectedModel,
        modelStatus,
        modelReady: Boolean(selectedModel?.downloaded),
        settings: settings,
        availableModels: mergedModels,
        cloudProviders: providers,
        aiFormattingProviders: aiProviders,
      });

      console.log("[Store] Initialization complete!");
    } catch (error) {
      console.error("Failed to initialize from database:", error);
      set({ isInitialized: true });
    }
  },

  // Setup actions
  setSetupComplete: (complete) => {
    set({ setupComplete: complete, isFirstLaunch: !complete });
    dbSetSetupComplete(complete).catch(console.error);
  },

  setCurrentSetupStep: (step) => {
    set({ currentSetupStep: step });
    dbSetCurrentSetupStep(step).catch(console.error);
  },

  nextSetupStep: () => {
    const newStep = get().currentSetupStep + 1;
    set({ currentSetupStep: newStep });
    dbSetCurrentSetupStep(newStep).catch(console.error);
  },

  prevSetupStep: () => {
    const newStep = Math.max(0, get().currentSetupStep - 1);
    set({ currentSetupStep: newStep });
    dbSetCurrentSetupStep(newStep).catch(console.error);
  },

  // Recording actions
  setRecordingStatus: (status) => set({ recordingStatus: status }),
  setLastTranscription: (text) => set({ lastTranscription: text }),
  setErrorMessage: (message) => set({ errorMessage: message }),

  // Model actions
  setModelStatus: (status) => set({ modelStatus: status }),

  setSelectedModel: (model) => {
    const isReady = Boolean(model?.downloaded);
    set({
      selectedModel: model,
      modelReady: isReady,
      modelStatus: model?.downloaded ? "downloaded" : "not-downloaded",
      settings: { ...get().settings, selectedModelId: model?.id || "" },
    });
    dbSetSelectedModel(model?.id || null).catch(console.error);
  },

  setDownloadProgress: (progress) => set({ downloadProgress: progress }),

  setAvailableModels: (models) => set({ availableModels: models }),

  markModelDownloaded: (modelId, path) => {
    const models = get().availableModels.map((m) =>
      m.id === modelId ? { ...m, downloaded: true } : m
    );
    set({ availableModels: models });

    const selectedModel = get().selectedModel;
    if (selectedModel?.id === modelId) {
      set({
        selectedModel: { ...selectedModel, downloaded: true },
        modelStatus: "downloaded",
        modelReady: true,
      });
    }

    dbSetModelDownloaded(modelId, true, path).catch(console.error);
  },

  setModelReady: (ready) => set({ modelReady: ready }),

   // Cloud providers refresh
  refreshCloudProviders: async () => {
    try {
      const providers = await getCloudProviders();
      const currentModels = get().availableModels;
      const localModels = currentModels.filter((m) => !m.isCloud);
      const mergedModels = buildMergedModels(localModels, providers);

      const currentSelected = get().selectedModel;
      const updatedSelected = currentSelected
        ? mergedModels.find((m) => m.id === currentSelected.id) || currentSelected
        : null;

      set({
        cloudProviders: providers,
        availableModels: mergedModels,
        selectedModel: updatedSelected,
        modelStatus: updatedSelected?.downloaded ? "downloaded" : "not-downloaded",
        modelReady: Boolean(updatedSelected?.downloaded),
      });
    } catch (err) {
      console.error("Failed to refresh cloud providers:", err);
    }
  },

  // AI formatting providers refresh
  refreshAiFormattingProviders: async () => {
    try {
      const providers = await getAiFormattingProviders();
      set({ aiFormattingProviders: providers });
    } catch (err) {
      console.error("Failed to refresh AI formatting providers:", err);
    }
  },

  // Settings actions
  updateSettings: (newSettings) => {
    const updatedSettings = { ...get().settings, ...newSettings };
    set({ settings: updatedSettings });
    dbUpdateSettings(frontendSettingsToDb(updatedSettings)).catch(
      console.error
    );
  },

  resetSettings: () => {
    set({ settings: DEFAULT_SETTINGS });
    dbUpdateSettings(frontendSettingsToDb(DEFAULT_SETTINGS)).catch(
      console.error
    );
  },

  // Utility
  reset: () => set(initialState),
}));

// Selector hooks for common use cases
export const useRecordingStatus = () =>
  useAppStore((state) => state.recordingStatus);
export const useSettings = () => useAppStore((state) => state.settings);
export const useModelStatus = () => useAppStore((state) => state.modelStatus);
export const useSetupState = () =>
  useAppStore((state) => ({
    isFirstLaunch: state.isFirstLaunch,
    setupComplete: state.setupComplete,
    currentSetupStep: state.currentSetupStep,
  }));
export const useAvailableModels = () =>
  useAppStore((state) => state.availableModels);
export const useCloudProviders = () =>
  useAppStore((state) => state.cloudProviders);
export const useAiFormattingProviders = () =>
  useAppStore((state) => state.aiFormattingProviders);
export const useIsInitialized = () =>
  useAppStore((state) => state.isInitialized);
