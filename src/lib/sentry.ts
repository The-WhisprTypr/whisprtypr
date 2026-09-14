import * as Sentry from "@sentry/react";
import { useAppStore } from "@/store";

const SENTRY_DSN = import.meta.env.VITE_SENTRY_DSN;

export function initSentry(): void {
  try {
    const settings = useAppStore.getState().settings;
    if (!settings.diagnosticsEnabled) return;
    if (!SENTRY_DSN) return;

    Sentry.init({
      dsn: SENTRY_DSN,
      release: "whisprtypr@1.0.0",
      environment: import.meta.env.MODE,
      beforeSend(event) {
        const currentSettings = useAppStore.getState().settings;
        if (!currentSettings.diagnosticsEnabled) {
          return null;
        }
        if (event.user) {
          event.user = undefined;
        }
        if (event.tags) {
          for (const key of Object.keys(event.tags)) {
            if (key.startsWith("user_") || key.startsWith("email_")) {
              delete event.tags[key];
            }
          }
        }
        return event;
      },
    });
  } catch {
    // Sentry init failed — silently ignore in production
  }
}

export function captureSentryException(
  error: Error,
  context?: Record<string, string>
): void {
  const settings = useAppStore.getState().settings;
  if (!settings.diagnosticsEnabled) return;

  Sentry.withScope((scope) => {
    if (context) {
      Object.entries(context).forEach(([key, value]) => {
        scope.setExtra(key, value);
      });
    }
    Sentry.captureException(error);
  });
}

export function captureSentryMessage(
  message: string,
  level: "error" | "warning" | "info" | "debug" = "error"
): void {
  const settings = useAppStore.getState().settings;
  if (!settings.diagnosticsEnabled) return;

  Sentry.captureMessage(message, level);
}
