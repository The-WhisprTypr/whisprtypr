// Whisprtypr - Type Definitions

export type CloudProviderId = "groq" | "openai" | "deepgram" | "mistral" | "custom";

export interface CloudProviderInfo {
  id: CloudProviderId;
  name: string;
  configured: boolean;
  masked_key: string;
  base_url?: string | null;
  custom_model?: string | null;
}

// ==================== AI Formatting Providers (BYOK) ====================

export type AiFormattingProviderId = "gemini" | "anthropic" | "openai" | "deepseek" | "custom";

export interface AiFormattingProviderInfo {
  id: AiFormattingProviderId;
  name: string;
  configured: boolean;
  masked_key: string;
  base_url?: string | null;
  custom_model?: string | null;
}

// ==================== AI Formatting Styles ====================

export type AiFormattingStyle =
  | "personal"
  | "clean"
  | "writing"
  | "notes"
  | "email"
  | "code"
  | "social"
  | "academic"
  | "business"
  | "transcript"
  | "legal"
  | "meeting"
  | "journaling";

export interface AiFormattingStyleInfo {
  id: AiFormattingStyle;
  name: string;
  description: string;
  icon?: React.ElementType;
  prompt: string;
}

export const AI_FORMATTING_STYLES: AiFormattingStyleInfo[] = [
  {
    id: "personal",
    name: "Personal Dictation",
    description: "Light cleanup — fixes obvious errors and punctuation while preserving your natural conversational tone.",
    prompt:
      "You are a helpful dictation assistant. The user has spoken the following text which was transcribed from voice. Your job is to lightly clean up obvious transcription errors, punctuation, and filler words while preserving the speaker's casual, conversational tone and personality. Do not over-edit or change the speaker's voice. Return only the formatted text, with no preamble or explanation.",
  },
  {
    id: "clean",
    name: "Clean Dictation",
    description: "Polished prose — removes filler words, fixes grammar, and produces crisp, professional sentences.",
    prompt:
      "You are a professional dictation editor. The user has spoken the following text which was transcribed from voice. Your job is to produce clean, professional prose: fix grammar, remove filler words (um, uh, like, you know), add proper punctuation, ensure proper sentence structure, and create clean paragraph breaks. Return only the formatted text, with no preamble or explanation.",
  },
  {
    id: "writing",
    name: "Writing",
    description: "Treat dictation as a draft article — polish into structured, engaging written content.",
    prompt:
      "You are an editor helping someone turn spoken dictation into polished written content. Format the following transcribed text as a well-structured article or blog post. Use proper paragraphs, fix grammar and flow, and polish the prose to sound professional and engaging. Return only the formatted text, with no preamble or explanation.",
  },
  {
    id: "notes",
    name: "Notes",
    description: "Extract key points into concise, scannable bullet lists or short phrases.",
    prompt:
      "You are a note-taking assistant. Extract the key points and important information from the following transcribed dictation. Format as concise bullet points or short phrases, capturing the essential information. Remove filler words and redundant phrasing. Return only the formatted notes, with no preamble or explanation.",
  },
  {
    id: "email",
    name: "Email",
    description: "Format dictation as a polished business email with greeting, body, and sign-off.",
    prompt:
      "You are a professional email assistant. Format the following transcribed dictation as a polished business email. Add an appropriate greeting, structure the body with clear paragraphs, and include a professional sign-off. Ensure the tone is appropriate and professional. Return only the formatted email text, with no preamble or explanation.",
  },
  {
    id: "code",
    name: "Code",
    description: "Convert spoken programming terms into clean, properly formatted code.",
    prompt:
      "You are a coding assistant. The following text was transcribed from voice and contains spoken programming terms, code snippets, and technical instructions. Convert spoken descriptions of code into clean, properly formatted code. Apply appropriate casing (camelCase, PascalCase, snake_case), insert code symbols (brackets, braces, operators) that were spoken as words, and organize into logical blocks. Preserve any literal code. Return only the formatted code, with no preamble or explanation.",
  },
  {
    id: "social",
    name: "Social",
    description: "Casual, social-media-friendly text with short paragraphs and engaging tone.",
    prompt:
      "You are a social media assistant. The following text was transcribed from casual voice dictation. Format it for social media: keep a conversational and engaging tone, use short paragraphs or sentences, add appropriate line breaks, and make it easy to read. Return only the formatted text, with no preamble or explanation.",
  },
  {
    id: "academic",
    name: "Academic",
    description: "Formal academic tone with precise language and structured paragraphs.",
    prompt:
      "You are an academic writing assistant. The following text was transcribed from voice. Rewrite it in a formal academic style: use precise language, proper sentence structure, formal tone, and structured paragraphs. Remove colloquialisms and filler words. Return only the formatted text, with no preamble or explanation.",
  },
  {
    id: "business",
    name: "Business",
    description: "Professional business document with clear structure and concise language.",
    prompt:
      "You are a business communication assistant. The following text was transcribed from voice. Format it as a professional business document: use clear, concise language, structured paragraphs, bullet points where appropriate, and a professional tone. Return only the formatted text, with no preamble or explanation.",
  },
  {
    id: "transcript",
    name: "Transcript",
    description: "Clean verbatim transcript — preserves original meaning with proper punctuation and paragraphing.",
    prompt:
      "You are a transcription editor. The following text was transcribed from voice. Create a clean transcript-style output: preserve the original meaning and content, but fix obvious transcription errors, add proper punctuation, and organize into readable paragraphs. Do not change the tone or style of the original speech. Return only the formatted text, with no preamble or explanation.",
  },
  {
    id: "legal",
    name: "Legal",
    description: "Formal legal language with precise terminology and structured clauses.",
    prompt:
      "You are a legal transcription assistant. The following text was transcribed from voice. Rewrite it in a formal legal style: use precise terminology, structured paragraphs, and proper legal phrasing. Maintain the original meaning while ensuring legal accuracy. Return only the formatted text, with no preamble or explanation.",
  },
  {
    id: "meeting",
    name: "Meeting Minutes",
    description: "Structured meeting notes with action items and decisions highlighted.",
    prompt:
      "You are a meeting minutes assistant. The following text was transcribed from a meeting recording. Format it as professional meeting minutes: include a brief summary, list key discussion points, highlight decisions made, and extract action items with responsible parties and deadlines. Use clear headings and bullet points. Return only the formatted text, with no preamble or explanation.",
  },
  {
    id: "journaling",
    name: "Journaling",
    description: "Thoughtful, reflective journal entry with natural flow and personal tone.",
    prompt:
      "You are a journaling assistant. The following text was transcribed from a personal voice journal entry. Format it as a thoughtful, well-structured journal entry: preserve the personal, reflective tone, organize thoughts into coherent paragraphs, add proper punctuation, and maintain the authentic voice of the writer. Return only the formatted text, with no preamble or explanation.",
  },
];

export const AI_FORMATTING_STYLE_LABELS: Record<AiFormattingStyle, string> = {
  personal: "Personal Dictation",
  clean: "Clean Dictation",
  writing: "Writing",
  notes: "Notes",
  email: "Email",
  code: "Code",
  social: "Social",
  academic: "Academic",
  business: "Business",
  transcript: "Transcript",
  legal: "Legal",
  meeting: "Meeting Minutes",
  journaling: "Journaling",
};

// Available Whisper models for offline transcription & BYOK cloud models
export interface WhisperModel {
  id: string;
  name: string;
  size: string; // e.g., "75 MB", "1.5 GB", "Cloud API"
  sizeBytes: number;
  description: string;
  languages: string[];
  defaultLanguage?: string;
  autoDetect?: boolean;
  recommended?: boolean;
  downloaded?: boolean;
  downloadProgress?: number; // 0-100
  isCloud?: boolean;
  provider?: CloudProviderId;
  latencyEstimate?: string;
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
  playAudioFeedback: boolean;
  showRecordingOverlay: boolean; // Show recording overlay when recording
  recordingOverlayPosition: "top-left" | "top-center" | "top-right" | "bottom-left" | "bottom-center" | "bottom-right";

  // Post-processing
  postProcessingEnabled: boolean;
  voiceCommandsEnabled: boolean;

  // Output mode
  clipboardMode: boolean; // true = copy to clipboard, false = inject text

  // Translation
  translationEnabled: boolean;
  translationHotkey: string;
  translationSourceLanguage: string;
  translationTargetLanguage: string;
  translationApiKey: string;

  // AI Formatting (BYOK)
  aiFormattingEnabled: boolean;
  aiFormattingProviderId: AiFormattingProviderId;
  aiFormattingStyle: AiFormattingStyle;
  aiFormattingModel: string;

  // Advanced
  autoStartOnBoot: boolean;
  minimizeToTray: boolean;

  // Diagnostics
  diagnosticsEnabled: boolean;

  // Updates
  autoCheckForUpdates: boolean;

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
  playAudioFeedback: true,
  showRecordingOverlay: true,
  recordingOverlayPosition: "top-center",
  postProcessingEnabled: true,
  voiceCommandsEnabled: false,
  clipboardMode: false,
  translationEnabled: false,
  translationHotkey: "Alt+Shift+T",
  translationSourceLanguage: "en",
  translationTargetLanguage: "es",
  translationApiKey: "",

  // AI Formatting (BYOK)
  aiFormattingEnabled: false,
  aiFormattingProviderId: "openai",
  aiFormattingStyle: "clean",
  aiFormattingModel: "gpt-4o-mini",

  autoStartOnBoot: false,
  minimizeToTray: true,
  diagnosticsEnabled: true,
  autoCheckForUpdates: false,
  customVocabulary: [
    { spoken: "wave e", written: "Whisprtypr" },
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

export type ModelBadgeCategory = "recommended" | "accurate" | "fast" | "compact" | "cloud" | "ultra-fast";

export function getModelCategories(model: WhisperModel): ModelBadgeCategory[] {
  const categories: ModelBadgeCategory[] = [];

  if (model.isCloud) {
    categories.push("cloud");
  }

  if (model.recommended) {
    categories.push("recommended");
  }

  if (model.provider === "groq" || model.provider === "deepgram") {
    categories.push("ultra-fast");
  }

  if (
    model.id.startsWith("qwen3-asr-") ||
    model.id.includes("large") ||
    model.id === "medium" ||
    model.id === "medium.en" ||
    model.id.includes("nova")
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

  if (!model.isCloud && model.sizeBytes <= 200 * 1024 * 1024) {
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

export const CLOUD_MODELS: WhisperModel[] = [
  {
    id: "cloud:groq:whisper-large-v3",
    name: "Groq Whisper Large v3",
    size: "Cloud API",
    sizeBytes: 0,
    description:
      "State-of-the-art Whisper accuracy with sub-second (~350ms) inference powered by Groq LPUs.",
    languages: WHISPER_MULTILINGUAL_LANGUAGES,
    defaultLanguage: "en",
    autoDetect: true,
    recommended: true,
    isCloud: true,
    provider: "groq",
    latencyEstimate: "~350ms",
  },
  {
    id: "cloud:groq:whisper-large-v3-turbo",
    name: "Groq Whisper Large v3 Turbo",
    size: "Cloud API",
    sizeBytes: 0,
    description:
      "Turbocharged Whisper engine optimized for low-latency multilingual speech recognition.",
    languages: WHISPER_MULTILINGUAL_LANGUAGES,
    defaultLanguage: "en",
    autoDetect: true,
    isCloud: true,
    provider: "groq",
    latencyEstimate: "~250ms",
  },
  {
    id: "cloud:groq:distil-whisper-large-v3-en",
    name: "Groq Distil-Whisper English",
    size: "Cloud API",
    sizeBytes: 0,
    description:
      "Ultra-fast English dictation with near-zero latency and high conversational accuracy.",
    languages: ["en"],
    defaultLanguage: "en",
    autoDetect: false,
    isCloud: true,
    provider: "groq",
    latencyEstimate: "~180ms",
  },
  {
    id: "cloud:openai:whisper-1",
    name: "OpenAI Whisper-1",
    size: "Cloud API",
    sizeBytes: 0,
    description:
      "Official OpenAI Whisper model. Industry gold standard across 90+ spoken languages.",
    languages: WHISPER_MULTILINGUAL_LANGUAGES,
    defaultLanguage: "en",
    autoDetect: true,
    isCloud: true,
    provider: "openai",
    latencyEstimate: "~1.2s",
  },
  {
    id: "cloud:deepgram:nova-3",
    name: "Deepgram Nova-3",
    size: "Cloud API",
    sizeBytes: 0,
    description:
      "Deepgram's flagship conversational speech engine with high speed and smart formatting.",
    languages: WHISPER_MULTILINGUAL_LANGUAGES,
    defaultLanguage: "en",
    autoDetect: true,
    recommended: true,
    isCloud: true,
    provider: "deepgram",
    latencyEstimate: "~300ms",
  },
  {
    id: "cloud:deepgram:nova-2",
    name: "Deepgram Nova-2",
    size: "Cloud API",
    sizeBytes: 0,
    description:
      "Enterprise-grade speech recognition with deep domain and multilingual comprehension.",
    languages: WHISPER_MULTILINGUAL_LANGUAGES,
    defaultLanguage: "en",
    autoDetect: true,
    isCloud: true,
    provider: "deepgram",
    latencyEstimate: "~450ms",
  },
  {
    id: "cloud:mistral:voxtral-mini",
    name: "Mistral Voxtral",
    size: "Cloud API",
    sizeBytes: 0,
    description:
      "Mistral speech recognition model with strong multi-language understanding.",
    languages: WHISPER_MULTILINGUAL_LANGUAGES,
    defaultLanguage: "en",
    autoDetect: true,
    isCloud: true,
    provider: "mistral",
    latencyEstimate: "~600ms",
  },
  {
    id: "cloud:custom:custom-model",
    name: "Custom Endpoint",
    size: "Cloud API",
    sizeBytes: 0,
    description:
      "Connect to any self-hosted or OpenAI-compatible transcription server (e.g. LocalAI, vLLM, RunPod).",
    languages: WHISPER_MULTILINGUAL_LANGUAGES,
    defaultLanguage: "en",
    autoDetect: true,
    isCloud: true,
    provider: "custom",
    latencyEstimate: "Custom",
  },
];

export const ALL_LOCAL_MODELS = [
  ...WHISPER_MODELS,
  ...PARAKEET_MODELS,
  ...QWEN3_ASR_MODELS,
];

export const ALL_MODELS = [
  ...ALL_LOCAL_MODELS,
  ...CLOUD_MODELS,
];

