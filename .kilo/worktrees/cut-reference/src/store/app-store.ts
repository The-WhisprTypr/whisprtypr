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
import type {
  AppSettings,
  AppState,
  ModelStatus,
  RecordingStatus,
  WhisperModel,
} from "@/types";
import { DEFAULT_SETTINGS } from "@/types";
import { create } from "zustand";

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

  // Settings actions
  updateSettings: (settings: Partial<AppSettings>) => void;
  resetSettings: () => void;

  // Available models from DB
  availableModels: WhisperModel[];

  // Utility
  reset: () => void;
}

const initialState: AppState & {
  availableModels: WhisperModel[];
  isInitialized: boolean;
} = {
  isInitialized: false,
  isFirstLaunch: true,
  setupComplete: false,
  currentSetupStep: 0,
  recordingStatus: "idle",
  lastTranscription: "",
  errorMessage: null,
  modelStatus: "not-downloaded",
  selectedModel: null,
  downloadProgress: 0,
  settings: DEFAULT_SETTINGS,
  availableModels: [],
};

export const useAppStore = create<AppStore>()((set, get) => ({
  ...initialState,

  // Initialize from SQLite database
  initializeFromDb: async () => {
    console.log("[Store] Starting initialization from database...");
    try {
      // Load app state from database
      console.log("[Store] Fetching app state...");
      const dbState = await dbGetAppState();
      console.log("[Store] App state:", dbState);
      
      console.log("[Store] Fetching settings...");
      const dbSettings = await dbGetSettings();
      console.log("[Store] Settings:", dbSettings);
      
      console.log("[Store] Fetching models...");
      const dbModels = await dbGetModels();
      console.log("[Store] Models:", dbModels);

      const settings = dbSettingsToFrontend(dbSettings);
      const models = dbModelsToFrontend(dbModels);

      // Find selected model
      const selectedModel = dbState.selected_model_id
        ? models.find((m) => m.id === dbState.selected_model_id) || null
        : null;

      // Determine model status
      let modelStatus: ModelStatus = "not-downloaded";
      if (selectedModel?.downloaded) {
        modelStatus = "downloaded";
      }

      console.log("[Store] Setting state with:", {
        isFirstLaunch: dbState.is_first_launch,
        setupComplete: dbState.setup_complete,
        currentSetupStep: dbState.current_setup_step,
        selectedModel,
        modelStatus,
        availableModels: models.length,
      });

      set({
        isInitialized: true,
        isFirstLaunch: dbState.is_first_launch,
        setupComplete: dbState.setup_complete,
        currentSetupStep: dbState.current_setup_step,
        selectedModel,
        modelStatus,
        settings,
        availableModels: models,
      });
      
      console.log("[Store] Initialization complete!");
    } catch (error) {
      console.error("Failed to initialize from database:", error);
      // Keep default state if DB fails (for dev mode in browser)
      set({ isInitialized: true });
    }
  },

  // Setup actions
  setSetupComplete: (complete) => {
    set({ setupComplete: complete, isFirstLaunch: !complete });
    // Sync to database
    dbSetSetupComplete(complete).catch(console.error);
  },

  setCurrentSetupStep: (step) => {
    set({ currentSetupStep: step });
    // Sync to database
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
    set({
      selectedModel: model,
      settings: { ...get().settings, selectedModelId: model?.id || "" },
    });
    // Sync to database
    dbSetSelectedModel(model?.id || null).catch(console.error);
  },

  setDownloadProgress: (progress) => set({ downloadProgress: progress }),

  setAvailableModels: (models) => set({ availableModels: models }),

  markModelDownloaded: (modelId, path) => {
    const models = get().availableModels.map((m) =>
      m.id === modelId ? { ...m, downloaded: true } : m
    );
    set({ availableModels: models });

    // Update selected model if it's the one being downloaded
    const selectedModel = get().selectedModel;
    if (selectedModel?.id === modelId) {
      set({
        selectedModel: { ...selectedModel, downloaded: true },
        modelStatus: "downloaded",
      });
    }

    // Sync to database
    dbSetModelDownloaded(modelId, true, path).catch(console.error);
  },

  // Settings actions
  updateSettings: (newSettings) => {
    const updatedSettings = { ...get().settings, ...newSettings };
    set({ settings: updatedSettings });
    // Sync to database
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
export const useIsInitialized = () =>
  useAppStore((state) => state.isInitialized);
