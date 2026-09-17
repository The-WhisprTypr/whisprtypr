/**
 * User-friendly error messages for Whisprtypr
 * Maps technical errors to actionable user messages
 */

export interface UserError {
  title: string;
  message: string;
  action?: string;
  actionLabel?: string;
  recoverable: boolean;
}

const ERROR_MAP: Record<string, UserError> = {
  // Recording errors
  "No input device available": {
    title: "No Microphone Found",
    message: "Please connect a microphone and try again.",
    recoverable: true,
  },
  "Already recording": {
    title: "Already Recording",
    message: "A recording is already in progress.",
    recoverable: true,
  },
  "No audio recorded": {
    title: "No Audio Captured",
    message: "No audio was captured. Please speak and try again.",
    recoverable: true,
  },
  "Failed to start recording": {
    title: "Recording Failed",
    message:
      "Could not start recording. Please check your microphone permissions.",
    recoverable: true,
  },

  // Transcription errors
  "No model loaded": {
    title: "Model Not Loaded",
    message: "The AI model is not loaded. Please wait or restart the app.",
    recoverable: true,
  },
  "Transcription failed": {
    title: "Transcription Failed",
    message: "Could not transcribe the audio. Please try again.",
    recoverable: true,
  },
  "Model file not found": {
    title: "Model Missing",
    message:
      "The AI model file is missing. Please download it again in Settings.",
    action: "settings",
    actionLabel: "Go to Settings",
    recoverable: true,
  },

  // Download errors
  "Download failed": {
    title: "Download Failed",
    message:
      "Could not download the model. Please check your internet connection.",
    recoverable: true,
  },
  "Network error": {
    title: "Network Error",
    message: "Please check your internet connection and try again.",
    recoverable: true,
  },

  // Hotkey errors
  "Invalid hotkey": {
    title: "Invalid Hotkey",
    message:
      "The hotkey combination is not valid. Please choose a different one.",
    recoverable: true,
  },
  "Hotkey already in use": {
    title: "Hotkey Conflict",
    message: "This hotkey is already used by another application.",
    recoverable: true,
  },

  // Text injection errors
  "Failed to inject text": {
    title: "Could Not Insert Text",
    message:
      "Could not insert text at cursor. Please make sure a text field is focused.",
    recoverable: true,
  },

  // Database errors
  "Database error": {
    title: "Storage Error",
    message:
      "Could not save data. The app will continue working but changes may not persist.",
    recoverable: true,
  },

  // License errors
  "License error": {
    title: "License Error",
    message: "License verification failed. Please try again.",
    recoverable: true,
  },
  "No active license": {
    title: "No Active License",
    message: "No active license was found. Please activate your license key.",
    recoverable: true,
  },
  "license server": {
    title: "License Error",
    message:
      "This license could not be verified. Please contact support if you think this is a mistake.",
    recoverable: true,
  },

  // File errors
  "File not found": {
    title: "File Not Found",
    message: "The selected file could not be found.",
    recoverable: true,
  },
  "Invalid WAV file": {
    title: "Invalid Audio File",
    message: "The file format is not supported. Please use a WAV file.",
    recoverable: true,
  },
};

/**
 * Parse an error and return a user-friendly message
 */
export function parseError(error: unknown): UserError {
  let errorString = error instanceof Error ? error.message : String(error);
  if (error && typeof error === "object" && !(error instanceof Error)) {
    const wrapped = error as { payload?: unknown; message?: unknown };
    if (typeof wrapped.payload === "string") {
      errorString = wrapped.payload;
    } else if (typeof wrapped.message === "string") {
      errorString = wrapped.message;
    }
  }

  const lower = errorString.toLowerCase();

  if (
    lower.includes("{\"error\"") ||
    lower.includes("badrequest") ||
    lower.includes("http 400") ||
    lower.includes("http 401") ||
    lower.includes("http 403") ||
    lower.includes("http 404") ||
    lower.includes("http 422") ||
    lower.includes("0 more usages")
  ) {
    return {
      title: "License Error",
      message:
        "This license could not be verified. Please contact support if you think this is a mistake.",
      recoverable: true,
    };
  }

  // Check for exact matches
  if (ERROR_MAP[errorString]) {
    return ERROR_MAP[errorString];
  }

  // Check for partial matches
  for (const [key, value] of Object.entries(ERROR_MAP)) {
    if (errorString.toLowerCase().includes(key.toLowerCase())) {
      return value;
    }
  }

  // Default error
  return {
    title: "Something Went Wrong",
    message: errorString || "An unexpected error occurred. Please try again.",
    recoverable: true,
  };
}

/**
 * Get a simple error message string
 */
export function getErrorMessage(error: unknown): string {
  const userError = parseError(error);
  return userError.message;
}

/**
 * Check if an error is recoverable (user can retry)
 */
export function isRecoverableError(error: unknown): boolean {
  return parseError(error).recoverable;
}
