// WhisprTypr - Type Definitions

// Available Whisper models for offline transcription
export interface WhisperModel {
  id: string;
  name: string;
  size: string; // e.g., "75 MB", "1.5 GB"
  sizeBytes: number;
  description: string;
  languages: string[];
  defaultLanguage?: string;
  autoDetect?: boolean;
  recommended?: boolean;
  downloaded?: boolean;
  downloadProgress?: number; // 0-100
}

// App settings
export interface VocabularyEntry {
  /** The phrase the user expects to say / what Whisper commonly outputs */
  spoken: string;
  /** The canonical text that should appear in the final output */
  written: string;
}

export interface AppSettings {
  // Hotkey configuration
  pushToTalkKey: string;
  toggleKey: string;
  hotkeyMode: "push-to-talk" | "toggle";

  // Language settings
  language: string;

  // Model settings
  selectedModelId: string;

  // UI preferences
  showRecordingIndicator: boolean;
  playAudioFeedback: boolean;
  showRecordingOverlay: boolean; // Show fullscreen wave overlay when recording

  // Post-processing
  postProcessingEnabled: boolean;
  voiceCommandsEnabled: boolean;

  // Output mode
  clipboardMode: boolean; // true = copy to clipboard, false = inject text

  // Advanced
  autoStartOnBoot: boolean;
  minimizeToTray: boolean;

  // Custom vocabulary
  customVocabulary: VocabularyEntry[];
}

// Recording state
export type RecordingStatus = "idle" | "recording" | "processing" | "error";

// Model download status
export type ModelStatus =
  | "not-downloaded"
  | "downloading"
  | "downloaded"
  | "loading"
  | "ready"
  | "error";

// License status
export type LicenseStatus =
  | "active"
  | "inactive"
  | "expired"
  | "revoked"
  | "disabled"
  | "invalid"
  | "not_activated"
  | "activation_limit";

// License data
export interface LicenseData {
  licenseKey: string | null;
  activationId: string | null;
  status: LicenseStatus;
  customerEmail: string | null;
  customerName: string | null;
  expiresAt: string | null;
  isActivated: boolean;
  lastValidatedAt: string | null;
}

// App state
export interface AppState {
  // Setup flow
  isFirstLaunch: boolean;
  setupComplete: boolean;
  currentSetupStep: number;

  // Recording
  recordingStatus: RecordingStatus;
  lastTranscription: string;
  errorMessage: string | null;

  // Model
  modelStatus: ModelStatus;
  selectedModel: WhisperModel | null;
  downloadProgress: number;

  // Settings
  settings: AppSettings;
}

// Default settings
export const DEFAULT_SETTINGS: AppSettings = {
  pushToTalkKey: "Alt+Shift+S",
  toggleKey: "Alt+Shift+D",
  hotkeyMode: "push-to-talk",
  language: "en",
  selectedModelId: "base",
  showRecordingIndicator: true,
  playAudioFeedback: true,
  showRecordingOverlay: true,
  postProcessingEnabled: true,
  voiceCommandsEnabled: false,
  clipboardMode: false,
  autoStartOnBoot: false,
  minimizeToTray: true,
  customVocabulary: [
    { spoken: "wave e", written: "WhisprTypr" },
    { spoken: "t a u r i", written: "Tauri" },
    { spoken: "next js", written: "Next.js" },
    { spoken: "rust lang", written: "Rust" },
    { spoken: "k eight s", written: "k8s" },
  ],
};

// Whisper language lists (must be defined before MODEL_CAPABILITIES)
export const PARAKEET_V3_LANGUAGES = [
  "bg",
  "hr",
  "cs",
  "da",
  "nl",
  "en",
  "et",
  "fi",
  "fr",
  "de",
  "el",
  "hu",
  "it",
  "lv",
  "lt",
  "mt",
  "pl",
  "pt",
  "ro",
  "sk",
  "sl",
  "es",
  "sv",
  "ru",
  "uk",
];

export const WHISPER_MULTILINGUAL_LANGUAGES = [
  "en",
  "zh",
  "de",
  "es",
  "ru",
  "ko",
  "fr",
  "ja",
  "pt",
  "tr",
  "pl",
  "ca",
  "nl",
  "ar",
  "sv",
  "it",
  "id",
  "hi",
  "fi",
  "vi",
  "he",
  "uk",
  "el",
  "ms",
  "cs",
  "ro",
  "da",
  "hu",
  "ta",
  "no",
  "th",
  "ur",
  "hr",
  "bg",
  "lt",
  "la",
  "mi",
  "ml",
  "cy",
  "sk",
  "te",
  "fa",
  "lv",
  "bn",
  "sr",
  "az",
  "sl",
  "kn",
  "et",
  "mk",
  "br",
  "eu",
  "is",
  "hy",
  "ne",
  "mn",
  "bs",
  "kk",
  "sq",
  "sw",
  "gl",
  "mr",
  "pa",
  "si",
  "km",
  "sn",
  "yo",
  "so",
  "af",
  "oc",
  "ka",
  "be",
  "tg",
  "sd",
  "gu",
  "am",
  "yi",
  "lo",
  "uz",
  "fo",
  "ht",
  "ps",
  "tk",
  "nn",
  "mt",
  "sa",
  "lb",
  "my",
  "bo",
  "tl",
  "mg",
  "as",
  "tt",
  "haw",
  "ln",
  "ha",
  "ba",
  "jw",
];

export const QWEN3_ASR_LANGUAGES = [
  "zh",
  "en",
  "yue",
  "ar",
  "de",
  "fr",
  "es",
  "pt",
  "id",
  "it",
  "ko",
  "ru",
  "th",
  "vi",
  "ja",
  "tr",
  "hi",
  "ms",
  "nl",
  "sv",
  "da",
  "fi",
  "pl",
  "cs",
  "fil",
  "fa",
  "el",
  "hu",
  "mk",
  "ro",
];

export const LANGUAGE_NAMES: Record<string, string> = {
  auto: "Auto detect",
  af: "Afrikaans",
  am: "Amharic",
  ar: "Arabic",
  as: "Assamese",
  az: "Azerbaijani",
  ba: "Bashkir",
  be: "Belarusian",
  bg: "Bulgarian",
  bn: "Bengali",
  bo: "Tibetan",
  br: "Breton",
  bs: "Bosnian",
  ca: "Catalan",
  cs: "Czech",
  cy: "Welsh",
  da: "Danish",
  de: "German",
  el: "Greek",
  en: "English",
  es: "Spanish",
  et: "Estonian",
  eu: "Basque",
  fa: "Persian",
  fi: "Finnish",
  fil: "Filipino",
  fo: "Faroese",
  fr: "French",
  gl: "Galician",
  gu: "Gujarati",
  ha: "Hausa",
  haw: "Hawaiian",
  he: "Hebrew",
  hi: "Hindi",
  hr: "Croatian",
  ht: "Haitian Creole",
  hu: "Hungarian",
  hy: "Armenian",
  id: "Indonesian",
  is: "Icelandic",
  it: "Italian",
  ja: "Japanese",
  jw: "Javanese",
  ka: "Georgian",
  kk: "Kazakh",
  km: "Khmer",
  kn: "Kannada",
  ko: "Korean",
  la: "Latin",
  lb: "Luxembourgish",
  ln: "Lingala",
  lo: "Lao",
  lt: "Lithuanian",
  lv: "Latvian",
  mg: "Malagasy",
  mi: "Maori",
  mk: "Macedonian",
  ml: "Malayalam",
  mn: "Mongolian",
  mr: "Marathi",
  ms: "Malay",
  mt: "Maltese",
  my: "Myanmar",
  ne: "Nepali",
  nl: "Dutch",
  nn: "Nynorsk",
  no: "Norwegian",
  oc: "Occitan",
  pa: "Punjabi",
  pl: "Polish",
  ps: "Pashto",
  pt: "Portuguese",
  ro: "Romanian",
  ru: "Russian",
  sa: "Sanskrit",
  sd: "Sindhi",
  si: "Sinhala",
  sk: "Slovak",
  sl: "Slovenian",
  sn: "Shona",
  so: "Somali",
  sq: "Albanian",
  sr: "Serbian",
  su: "Sundanese",
  sv: "Swedish",
  sw: "Swahili",
  ta: "Tamil",
  te: "Telugu",
  tg: "Tajik",
  th: "Thai",
  tk: "Turkmen",
  tl: "Tagalog",
  tr: "Turkish",
  tt: "Tatar",
  uk: "Ukrainian",
  ur: "Urdu",
  uz: "Uzbek",
  vi: "Vietnamese",
  yue: "Cantonese",
  yi: "Yiddish",
  yo: "Yoruba",
  zh: "Chinese",
};

// Model categories for UI grouping
export type ModelCategory = "standard" | "english" | "distil" | "large";

// Model capabilities - single source of truth for language support
export interface ModelCapabilities {
  supportedLanguages: string[];
  defaultLanguage: string;
  autoDetect: boolean;
}

export const MODEL_CAPABILITIES: Record<string, ModelCapabilities> = {
  tiny: {
    supportedLanguages: WHISPER_MULTILINGUAL_LANGUAGES,
    defaultLanguage: "en",
    autoDetect: true,
  },
  base: {
    supportedLanguages: WHISPER_MULTILINGUAL_LANGUAGES,
    defaultLanguage: "en",
    autoDetect: true,
  },
  small: {
    supportedLanguages: WHISPER_MULTILINGUAL_LANGUAGES,
    defaultLanguage: "en",
    autoDetect: true,
  },
  medium: {
    supportedLanguages: WHISPER_MULTILINGUAL_LANGUAGES,
    defaultLanguage: "en",
    autoDetect: true,
  },
  "large-v3": {
    supportedLanguages: WHISPER_MULTILINGUAL_LANGUAGES,
    defaultLanguage: "en",
    autoDetect: true,
  },
  "large-v3-turbo": {
    supportedLanguages: WHISPER_MULTILINGUAL_LANGUAGES,
    defaultLanguage: "en",
    autoDetect: true,
  },
  "tiny.en": {
    supportedLanguages: ["en"],
    defaultLanguage: "en",
    autoDetect: false,
  },
  "base.en": {
    supportedLanguages: ["en"],
    defaultLanguage: "en",
    autoDetect: false,
  },
  "small.en": {
    supportedLanguages: ["en"],
    defaultLanguage: "en",
    autoDetect: false,
  },
  "medium.en": {
    supportedLanguages: ["en"],
    defaultLanguage: "en",
    autoDetect: false,
  },
  "distil-small.en": {
    supportedLanguages: ["en"],
    defaultLanguage: "en",
    autoDetect: false,
  },
  "parakeet-v3": {
    supportedLanguages: PARAKEET_V3_LANGUAGES,
    defaultLanguage: "en",
    autoDetect: true,
  },
  "parakeet-v2": {
    supportedLanguages: ["en"],
    defaultLanguage: "en",
    autoDetect: false,
  },
  "qwen3-asr-0.6b": {
    supportedLanguages: QWEN3_ASR_LANGUAGES,
    defaultLanguage: "zh",
    autoDetect: true,
  },
};

export function getModelCapabilities(
  model: Pick<WhisperModel, "id" | "languages">,
): ModelCapabilities {
  const caps = MODEL_CAPABILITIES[model.id];
  if (caps) return caps;

  const languages = model.languages.includes("multilingual")
    ? WHISPER_MULTILINGUAL_LANGUAGES
    : model.languages;

  return {
    supportedLanguages: languages,
    defaultLanguage: languages[0] ?? "en",
    autoDetect: model.languages.includes("multilingual") || languages.length > 1,
  };
}

export interface LanguageOption {
  code: string;
  name: string;
}

export function getModelLanguageLabel(
  model: Pick<WhisperModel, "id" | "languages">,
) {
  const caps = getModelCapabilities(model);
  const count = caps.supportedLanguages.length;

  if (count === 0) {
    return "No languages";
  }

  if (count === 1) {
    return LANGUAGE_NAMES[caps.supportedLanguages[0]] ?? caps.supportedLanguages[0].toUpperCase();
  }

  if (caps.autoDetect && count > 10) {
    return `${count}+ languages`;
  }

  if (count <= 3) {
    return caps.supportedLanguages.map((code) => LANGUAGE_NAMES[code] ?? code.toUpperCase()).join(", ");
  }

  return `${count} languages`;
}

export function getModelLanguageOptions(
  model: Pick<WhisperModel, "id" | "languages">,
): LanguageOption[] {
  const caps = getModelCapabilities(model);
  const options = caps.supportedLanguages.map((code) => ({
    code,
    name: LANGUAGE_NAMES[code] ?? code.toUpperCase(),
  }));

  if (caps.autoDetect) {
    return [{ code: "auto", name: "Auto detect" }, ...options];
  }

  return options;
}

export function isLanguageSupportedByModel(
  model: Pick<WhisperModel, "id" | "languages">,
  language: string,
) {
  return getModelLanguageOptions(model).some((option) => option.code === language);
}

export function getDefaultLanguageForModel(
  model: Pick<WhisperModel, "id" | "languages">,
) {
  const caps = getModelCapabilities(model);
  return caps.defaultLanguage;
}

export type ModelBadgeCategory = "recommended" | "accurate" | "fast" | "compact";

export function getModelCategories(model: WhisperModel): ModelBadgeCategory[] {
  const categories: ModelBadgeCategory[] = [];

  if (model.recommended) {
    categories.push("recommended");
  }

  if (
    model.id.startsWith("qwen3-asr-") ||
    model.id.includes("large") ||
    model.id === "medium" ||
    model.id === "medium.en"
  ) {
    categories.push("accurate");
  }

  if (
    model.id.includes("distil") ||
    model.id.includes("tiny") ||
    model.id.includes("base") ||
    model.id.startsWith("parakeet-")
  ) {
    categories.push("fast");
  }

  if (model.sizeBytes <= 200 * 1024 * 1024) {
    categories.push("compact");
  }

  return categories;
}

// Available transcription models
export const WHISPER_MODELS: WhisperModel[] = [
  // ========== STANDARD WHISPER (Multilingual) ==========
  {
    id: "tiny",
    name: "Whisper Tiny",
    size: "75 MB",
    sizeBytes: 75 * 1024 * 1024,
    description:
      "Fastest Whisper model. Best for quick notes and low-resource devices.",
    languages: MODEL_CAPABILITIES["tiny"].supportedLanguages,
    defaultLanguage: MODEL_CAPABILITIES["tiny"].defaultLanguage,
    autoDetect: MODEL_CAPABILITIES["tiny"].autoDetect,
  },
  {
    id: "base",
    name: "Whisper Base",
    size: "142 MB",
    sizeBytes: 142 * 1024 * 1024,
    description: "Balanced Whisper model for everyday transcription.",
    languages: MODEL_CAPABILITIES["base"].supportedLanguages,
    defaultLanguage: MODEL_CAPABILITIES["base"].defaultLanguage,
    autoDetect: MODEL_CAPABILITIES["base"].autoDetect,
  },
  {
    id: "small",
    name: "Whisper Small",
    size: "466 MB",
    sizeBytes: 466 * 1024 * 1024,
    description:
      "Improved accuracy for longer dictation, meetings, and focused writing.",
    languages: MODEL_CAPABILITIES["small"].supportedLanguages,
    defaultLanguage: MODEL_CAPABILITIES["small"].defaultLanguage,
    autoDetect: MODEL_CAPABILITIES["small"].autoDetect,
  },
  {
    id: "medium",
    name: "Whisper Medium",
    size: "1.5 GB",
    sizeBytes: 1.5 * 1024 * 1024 * 1024,
    description: "High-accuracy multilingual transcription for demanding audio.",
    languages: MODEL_CAPABILITIES["medium"].supportedLanguages,
    defaultLanguage: MODEL_CAPABILITIES["medium"].defaultLanguage,
    autoDetect: MODEL_CAPABILITIES["medium"].autoDetect,
  },

  // ========== ENGLISH-ONLY (Faster) ==========
  {
    id: "tiny.en",
    name: "Whisper Tiny English",
    size: "75 MB",
    sizeBytes: 75 * 1024 * 1024,
    description: "Fastest English-only Whisper model. Great for quick notes.",
    languages: MODEL_CAPABILITIES["tiny.en"].supportedLanguages,
    defaultLanguage: MODEL_CAPABILITIES["tiny.en"].defaultLanguage,
    autoDetect: MODEL_CAPABILITIES["tiny.en"].autoDetect,
  },
  {
    id: "base.en",
    name: "Whisper Base English",
    size: "142 MB",
    sizeBytes: 142 * 1024 * 1024,
    description: "Fast English-only Whisper model with good accuracy.",
    languages: MODEL_CAPABILITIES["base.en"].supportedLanguages,
    defaultLanguage: MODEL_CAPABILITIES["base.en"].defaultLanguage,
    autoDetect: MODEL_CAPABILITIES["base.en"].autoDetect,
  },
  {
    id: "small.en",
    name: "Whisper Small English",
    size: "466 MB",
    sizeBytes: 466 * 1024 * 1024,
    description: "Accurate English-only Whisper model for longer dictation.",
    languages: MODEL_CAPABILITIES["small.en"].supportedLanguages,
    defaultLanguage: MODEL_CAPABILITIES["small.en"].defaultLanguage,
    autoDetect: MODEL_CAPABILITIES["small.en"].autoDetect,
  },
  {
    id: "medium.en",
    name: "Whisper Medium English",
    size: "1.5 GB",
    sizeBytes: 1.5 * 1024 * 1024 * 1024,
    description: "High-accuracy English-only Whisper model.",
    languages: MODEL_CAPABILITIES["medium.en"].supportedLanguages,
    defaultLanguage: MODEL_CAPABILITIES["medium.en"].defaultLanguage,
    autoDetect: MODEL_CAPABILITIES["medium.en"].autoDetect,
  },

  // ========== DISTIL-WHISPER (Faster) ==========
  {
    id: "distil-small.en",
    name: "Distil Whisper Small English",
    size: "166 MB",
    sizeBytes: 166 * 1024 * 1024,
    description: "Fast English transcription with accuracy close to Whisper Small.",
    languages: MODEL_CAPABILITIES["distil-small.en"].supportedLanguages,
    defaultLanguage: MODEL_CAPABILITIES["distil-small.en"].defaultLanguage,
    autoDetect: MODEL_CAPABILITIES["distil-small.en"].autoDetect,
  },
  // ========== LARGE MODELS (Best Accuracy) ==========
  {
    id: "large-v3",
    name: "Whisper Large v3",
    size: "2.9 GB",
    sizeBytes: 2.9 * 1024 * 1024 * 1024,
    description: "Highest-accuracy Whisper model for professional workflows.",
    languages: MODEL_CAPABILITIES["large-v3"].supportedLanguages,
    defaultLanguage: MODEL_CAPABILITIES["large-v3"].defaultLanguage,
    autoDetect: MODEL_CAPABILITIES["large-v3"].autoDetect,
  },
  {
    id: "large-v3-turbo",
    name: "Whisper Large v3 Turbo",
    size: "1.6 GB",
    sizeBytes: 1.6 * 1024 * 1024 * 1024,
    description:
      "Fast large Whisper model with a strong speed and accuracy balance.",
    languages: MODEL_CAPABILITIES["large-v3-turbo"].supportedLanguages,
    defaultLanguage: MODEL_CAPABILITIES["large-v3-turbo"].defaultLanguage,
    autoDetect: MODEL_CAPABILITIES["large-v3-turbo"].autoDetect,
  },
];

export const PARAKEET_MODELS: WhisperModel[] = [
  {
    id: "parakeet-v3",
    name: "Parakeet v3",
    size: "670 MB",
    sizeBytes: 670 * 1024 * 1024,
    description:
      "Fast multilingual Parakeet model with automatic language detection.",
    languages: MODEL_CAPABILITIES["parakeet-v3"].supportedLanguages,
    defaultLanguage: MODEL_CAPABILITIES["parakeet-v3"].defaultLanguage,
    autoDetect: MODEL_CAPABILITIES["parakeet-v3"].autoDetect,
    recommended: true,
  },
  {
    id: "parakeet-v2",
    name: "Parakeet v2",
    size: "661 MB",
    sizeBytes: 661 * 1024 * 1024,
    description: "Previous Parakeet English model with stable transcription quality.",
    languages: MODEL_CAPABILITIES["parakeet-v2"].supportedLanguages,
    defaultLanguage: MODEL_CAPABILITIES["parakeet-v2"].defaultLanguage,
    autoDetect: MODEL_CAPABILITIES["parakeet-v2"].autoDetect,
  },
];

export const QWEN3_ASR_MODELS: WhisperModel[] = [
  {
    id: "qwen3-asr-0.6b",
    name: "Qwen3-ASR 0.6B",
    size: "1.9 GB",
    sizeBytes: 1880 * 1024 * 1024,
    description:
      "Qwen3-ASR speech recognition model for accurate multilingual transcription.",
    languages: MODEL_CAPABILITIES["qwen3-asr-0.6b"].supportedLanguages,
    defaultLanguage: MODEL_CAPABILITIES["qwen3-asr-0.6b"].defaultLanguage,
    autoDetect: MODEL_CAPABILITIES["qwen3-asr-0.6b"].autoDetect,
  },
];

export const ALL_MODELS = [
  ...WHISPER_MODELS,
  ...PARAKEET_MODELS,
  ...QWEN3_ASR_MODELS,
];
