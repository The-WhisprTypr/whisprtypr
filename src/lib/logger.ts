/**
 * Production-grade logging utility for WhisprTypr
 * Provides structured logging with levels, timestamps, and context
 */

type LogLevel = "debug" | "info" | "warn" | "error";

interface LogEntry {
  level: LogLevel;
  message: string;
  timestamp: string;
  context?: Record<string, unknown>;
}

const LOG_COLORS = {
  debug: "#6B7280",
  info: "#3B82F6",
  warn: "#F59E0B",
  error: "#EF4444",
} as const;

class Logger {
  private isDev = import.meta.env.DEV;
  private logHistory: LogEntry[] = [];
  private maxHistorySize = 1000;

  private createEntry(
    level: LogLevel,
    message: string,
    context?: Record<string, unknown>
  ): LogEntry {
    return {
      level,
      message,
      timestamp: new Date().toISOString(),
      context,
    };
  }

  private log(
    level: LogLevel,
    message: string,
    context?: Record<string, unknown>
  ) {
    const entry = this.createEntry(level, message, context);

    // Store in history
    this.logHistory.push(entry);
    if (this.logHistory.length > this.maxHistorySize) {
      this.logHistory.shift();
    }

    // Console output in dev mode
    if (this.isDev || level === "error" || level === "warn") {
      const color = LOG_COLORS[level];
      const prefix = `%c[${level.toUpperCase()}]`;
      const style = `color: ${color}; font-weight: bold;`;

      if (context) {
        console[level === "debug" ? "log" : level](
          prefix,
          style,
          message,
          context
        );
      } else {
        console[level === "debug" ? "log" : level](prefix, style, message);
      }
    }
  }

  debug(message: string, context?: Record<string, unknown>) {
    this.log("debug", message, context);
  }

  info(message: string, context?: Record<string, unknown>) {
    this.log("info", message, context);
  }

  warn(message: string, context?: Record<string, unknown>) {
    this.log("warn", message, context);
  }

  error(message: string, error?: unknown, context?: Record<string, unknown>) {
    const errorContext = {
      ...context,
      error:
        error instanceof Error
          ? {
              name: error.name,
              message: error.message,
              stack: error.stack,
            }
          : error,
    };
    this.log("error", message, errorContext);
  }

  /**
   * Get recent log entries for debugging
   */
  getHistory(count = 100): LogEntry[] {
    return this.logHistory.slice(-count);
  }

  /**
   * Clear log history
   */
  clearHistory() {
    this.logHistory = [];
  }

  /**
   * Export logs for debugging/support
   */
  exportLogs(): string {
    return JSON.stringify(this.logHistory, null, 2);
  }
}

export const logger = new Logger();

// Convenience exports for common patterns
export const logRecording = (
  action: string,
  details?: Record<string, unknown>
) =>
  logger.info(`Recording: ${action}`, { component: "recording", ...details });

export const logTranscription = (
  action: string,
  details?: Record<string, unknown>
) =>
  logger.info(`Transcription: ${action}`, {
    component: "transcription",
    ...details,
  });

export const logHotkey = (action: string, details?: Record<string, unknown>) =>
  logger.debug(`Hotkey: ${action}`, { component: "hotkey", ...details });

export const logError = (context: string, error: unknown) =>
  logger.error(`Error in ${context}`, error);
