import { useToast } from "@/hooks/use-toast";
import { cn, openUrl } from "@/lib/utils";
import { reportError } from "@/lib/voice-api";
import {
  checkForUpdates,
  downloadAndInstallUpdate,
  formatProgress,
  getCurrentVersion,
  relaunchApp,
  type UpdateProgress,
  type UpdateStatus,
} from "@/lib/updater-api";
import {
  AlertCircle,
  Check,
  Circle,
  Loader2,
  RefreshCw,
  Sparkles,
} from "@/components/icons";
import { useCallback, useEffect, useState } from "react";

export function UpdaterView() {
  const [status, setStatus] = useState<UpdateStatus>({ status: "idle" });
  const [progress, setProgress] = useState<UpdateProgress | null>(null);
  const [currentVersion, setCurrentVersion] = useState<string>("");
  const [versionError, setVersionError] = useState<string | null>(null);
  const { success: toastSuccess, error: toastError } = useToast();

  useEffect(() => {
    getCurrentVersion()
      .then(setCurrentVersion)
      .catch((error) => {
        const message =
          error instanceof Error ? error.message : "Could not read app version";
        setVersionError(message);
        reportError("system", message, "warning", {
          userAction: "Read app version",
        }).catch(console.error);
      });
  }, []);

  const handleCheckForUpdates = useCallback(async () => {
    setStatus({ status: "checking" });
    setProgress(null);
    try {
      const result = await checkForUpdates();
      setStatus(result);

      if (result.status === "available") {
        toastSuccess(`Update ${result.info.version} available!`);
      } else if (result.status === "not-available") {
        toastSuccess("You're running the latest version");
      } else if (result.status === "error") {
        toastError(result.message);
        await reportError("network", result.message, "error", {
          userAction: "Check for updates",
        }).catch(console.error);
      }
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "Failed to check for updates";
      setStatus({ status: "error", message });
      toastError(message);
      await reportError("network", message, "error", {
        userAction: "Check for updates",
      }).catch(console.error);
    }
  }, [toastSuccess, toastError]);

  const handleDownloadAndInstall = useCallback(async () => {
    setStatus({
      status: "downloading",
      progress: { downloaded: 0, total: null },
    });
    setProgress({ downloaded: 0, total: null });

    try {
      const result = await downloadAndInstallUpdate((p) => {
        setProgress(p);
        setStatus({ status: "downloading", progress: p });
      });

      setStatus(result);

      if (result.status === "ready") {
        toastSuccess("Update downloaded! Click 'Restart' to apply.");
      } else if (result.status === "error") {
        toastError(result.message);
        await reportError("network", result.message, "error", {
          userAction: "Download and install update",
        }).catch(console.error);
      }
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "Failed to download update";
      setStatus({ status: "error", message });
      toastError(message);
      await reportError("network", message, "error", {
        userAction: "Download and install update",
      }).catch(console.error);
    }
  }, [toastSuccess, toastError]);

  const handleRelaunch = useCallback(async () => {
    try {
      await relaunchApp();
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "Failed to restart application";
      toastError(message);
      await reportError("system", message, "error", {
        userAction: "Restart to apply update",
      }).catch(console.error);
    }
  }, [toastError]);

  const isUpdateAvailable = status.status === "available";
  const isReady = status.status === "ready";
  const isDownloading = status.status === "downloading";
  const isError = status.status === "error";

  return (
    <section className="card-feature-cream relative overflow-hidden">
      {/* Decorative orange glow when update is available */}
      {isUpdateAvailable && (
        <div
          aria-hidden
          className="absolute -top-16 -right-16 h-56 w-56 pointer-events-none"
          style={{
            background:
              "radial-gradient(circle, rgba(255,79,0,0.18), transparent 70%)",
          }}
        />
      )}

      <div className="relative">
        {/* Header */}
        <div className="flex items-start gap-3 mb-5">
          <div
            className={cn(
              "icon-plate shrink-0",
              isUpdateAvailable && "!bg-primary/10 !text-primary",
              isReady && "!bg-primary/10 !text-primary",
            )}
          >
            {isUpdateAvailable || isReady ? (
              <Sparkles className="h-4 w-4" />
            ) : (
              <RefreshCw className="h-4 w-4" />
            )}
          </div>
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2 flex-wrap">
              <p className="eyebrow-uppercase text-ink-mid">Updates</p>
              {isUpdateAvailable && (
                <span
                  className="inline-flex items-center gap-1.5 caption-strong px-2 py-0.5 rounded-full"
                  style={{ background: "rgba(255,79,0,0.1)", color: "#ff4f00" }}
                >
                  <Circle className="h-1.5 w-1.5 fill-primary text-primary" />
                  New version
                </span>
              )}
              {isReady && (
                <span
                  className="inline-flex items-center gap-1.5 caption-strong px-2 py-0.5 rounded-full"
                  style={{ background: "rgba(255,79,0,0.1)", color: "#ff4f00" }}
                >
                  <Check className="h-3 w-3" />
                  Ready to install
                </span>
              )}
              {status.status === "not-available" && (
                <span className="inline-flex items-center gap-1.5 caption-strong px-2 py-0.5 rounded-full bg-canvas text-ink">
                  <Check className="h-3 w-3" />
                  Up to date
                </span>
              )}
            </div>
            <h3
              className="title-lg text-ink mt-1"
            >
              Software updates
            </h3>
            <p className="body-sm text-body-muted mt-1.5">
              {isUpdateAvailable
                ? `Version ${status.info.version} is ready for you.`
                : isReady
                  ? "Update downloaded — restart WhisprTypr to apply it."
                  : "Keep WhisprTypr current with the latest fixes and features."}
            </p>
          </div>
          <span
            className="caption-strong px-2.5 py-1 rounded-md font-mono shrink-0 hidden sm:inline-block"
            style={{
              background: "#fffefb",
              border: "1px solid #e8e2d6",
              color: "#605d52",
              letterSpacing: "0.01em",
            }}
            title={versionError ?? undefined}
          >
            {currentVersion ? `v${currentVersion}` : versionError ? "v?" : "..."}
          </span>
        </div>

        {/* Version row (mobile fallback) */}
        <div className="sm:hidden mb-4 flex items-center gap-2">
          <span className="caption-strong text-ink-mid">Installed</span>
          <span
            className="caption-strong px-2 py-0.5 rounded-md font-mono"
            style={{
              background: "#fffefb",
              border: "1px solid #e8e2d6",
              color: "#605d52",
            }}
          >
            {currentVersion ? `v${currentVersion}` : versionError ? "v?" : "..."}
          </span>
        </div>

        {/* Error state */}
        {isError && (
          <div className="mb-5 p-3.5 rounded-md border border-destructive/30 bg-destructive/5 flex items-start gap-2.5 text-destructive">
            <AlertCircle className="h-4 w-4 mt-0.5 shrink-0" />
            <span className="body-sm">{status.message}</span>
          </div>
        )}

        {/* Download progress */}
        {isDownloading && progress && (
          <div className="mb-5">
            <div className="flex items-center justify-between mb-2">
              <span className="caption-strong text-ink-mid">
                {progress.total
                  ? formatProgress(progress)
                  : "Downloading..."}
              </span>
              <span
                className="caption-strong text-primary tabular-nums"
                style={{ fontVariantNumeric: "tabular-nums" }}
              >
                {progress.total
                  ? `${Math.round((progress.downloaded / progress.total) * 100)}%`
                  : "—"}
              </span>
            </div>
            <div className="h-1.5 bg-canvas-soft rounded-full overflow-hidden">
              <div
                className="h-full bg-primary transition-all duration-300 rounded-full"
                style={{
                  width: progress.total
                    ? `${(progress.downloaded / progress.total) * 100}%`
                    : "50%",
                }}
              />
            </div>
            <p className="caption text-body-muted mt-2">
              Keep WhisprTypr open while the update downloads.
            </p>
          </div>
        )}

        {/* Release notes */}
        {(isUpdateAvailable || isReady) && status.info.body && (
          <div
            className="mb-5 p-4 rounded-md border border-hairline"
            style={{ background: "#fffefb" }}
          >
            <p className="caption-strong text-ink-mid mb-2">What's new</p>
            <p className="body-sm text-ink leading-relaxed whitespace-pre-wrap line-clamp-5">
              {status.info.body}
            </p>
          </div>
        )}

        {/* Action row */}
        <div className="flex flex-wrap items-center gap-3">
          {status.status === "idle" ||
          status.status === "not-available" ||
          status.status === "error" ? (
            <button
              onClick={handleCheckForUpdates}
              className="paper-button-primary size-md cursor-pointer"
            >
              {isError ? "Try again" : "Check for updates"}
            </button>
          ) : null}

          {status.status === "checking" && (
            <button
              disabled
              className="paper-button-secondary size-md cursor-not-allowed"
            >
              <Loader2 className="h-3.5 w-3.5 animate-spin" />
              Checking for updates...
            </button>
          )}

          {isUpdateAvailable && (
            <button
              onClick={handleDownloadAndInstall}
              className="paper-button-primary size-md cursor-pointer"
            >
              Download & install
            </button>
          )}

          {isReady && (
            <button
              onClick={handleRelaunch}
              className="paper-button-primary size-md cursor-pointer"
            >
              Restart now
            </button>
          )}

          {/* Secondary action: open releases on GitHub */}
          <button
            onClick={() =>
              openUrl("https://github.com/johuniq/whisprtypr/releases/latest")
            }
            className="paper-button-outline size-md cursor-pointer"
          >
            View release notes
          </button>
        </div>
      </div>
    </section>
  );
}