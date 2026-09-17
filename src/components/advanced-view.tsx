import {
    Activity,
    AlertTriangle,
    Circle,
    RefreshCw,
    RotateCcw,
    Trash2,
} from "@/components/icons";
import { Switch } from "@/components/ui/switch";
import { UpdaterView } from "@/components/updater-view";
import { useToast } from "@/hooks/use-toast";
import { factoryReset } from "@/lib/data-management";
import { useAppStore } from "@/store";
import { useState } from "react";

import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogTrigger,
} from "@/components/ui/alert-dialog";

interface AdvancedViewProps {
  onClose: () => void;
}

export function AdvancedView(_props: AdvancedViewProps) {
  const { settings, updateSettings, resetSettings, setSetupComplete } =
    useAppStore();
  const { success: toastSuccess, error: toastError } = useToast();

  const [isResettingOnboarding, setIsResettingOnboarding] = useState(false);
  const [isResettingApp, setIsResettingApp] = useState(false);
  const [resetError, setResetError] = useState<string | null>(null);

  const handleDiagnosticsChange = (enabled: boolean) => {
    updateSettings({ diagnosticsEnabled: enabled });
    if (!enabled) {
      toastSuccess("Error logging disabled", "No errors will be collected.");
    } else {
      toastSuccess("Error logging enabled", "Anonymous error data will be collected.");
    }
  };

  const handleResetOnboarding = () => {
    try {
      setIsResettingOnboarding(true);
      setResetError(null);
      setSetupComplete(false);
      toastSuccess?.("Setup will run again", "Restart the app to see the setup wizard.");
    } catch (e) {
      setResetError("Could not reset onboarding.");
      toastError?.("Failed to reset onboarding");
    } finally {
      setIsResettingOnboarding(false);
    }
  };

  const handleResetAppData = async () => {
    try {
      setIsResettingApp(true);
      setResetError(null);
      await factoryReset();
      resetSettings();
      setSetupComplete(false);
      toastSuccess?.("App reset to initial state", "Restart the app to complete the reset.");
    } catch (e) {
      setResetError("Could not reset app data.");
      toastError?.("Failed to reset app data");
    } finally {
      setIsResettingApp(false);
    }
  };

  return (
    <div className="flex h-full flex-col overflow-hidden bg-canvas">
      {/* ─── HEADER ─── */}
      <div className="shrink-0 border-b border-hairline">
        <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-3 sm:py-4">
          <p className="eyebrow-uppercase text-ink-mid">Advanced</p>
          <h1
            className="display-sm text-ink mt-1"
          >
            Permissions, diagnostics, and <span className="text-primary">resetting</span>.
          </h1>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto">
        <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-4 xl:py-5 space-y-4 xl:space-y-5">
          {resetError && (
            <div className="p-3 rounded-md border border-destructive/30 bg-destructive/5 flex items-center gap-2.5 text-destructive">
              <AlertTriangle className="h-4 w-4 shrink-0" />
              <span className="body-sm">{resetError}</span>
            </div>
          )}

          {/* ─── DIAGNOSTICS ─── */}
          <section className="paper-card">
            <div className="flex items-center gap-2.5 mb-3">
              <div className="icon-plate">
                <Activity className="h-3.5 w-3.5" />
              </div>
              <div className="min-w-0 flex-1">
                <p className="eyebrow-uppercase text-ink-mid">Diagnostics</p>
                <h3 className="title-md text-ink mt-0.5">
                  Help improve Whisprtypr with anonymous diagnostics
                </h3>
              </div>
              <Switch
                checked={settings.diagnosticsEnabled}
                onCheckedChange={handleDiagnosticsChange}
                className="shrink-0"
              />
            </div>
            <p className="body-sm text-body-muted leading-relaxed">
              On by default — turn it off anytime. Sends only anonymous crash and
              error reports to help us fix bugs, never your audio, transcripts, or
              personal data.
            </p>
          </section>

          {/* ─── RESET OPTIONS ─── */}
          <section className="paper-card">
            <div className="flex items-center gap-2.5 mb-3">
              <div
                className="icon-plate shrink-0"
                style={{ background: "rgba(207,32,47,0.1)", color: "#cf202f" }}
              >
                <RotateCcw className="h-3.5 w-3.5" />
              </div>
              <div className="min-w-0 flex-1">
                <p className="eyebrow-uppercase" style={{ color: "#cf202f" }}>
                  Reset options
                </p>
                <h3 className="title-md text-ink mt-0.5">
                  Re-run setup or wipe Whisprtypr back to a clean state.
                </h3>
              </div>
            </div>

            <div className="divide-y divide-hairline-soft">
              {/* Reset Onboarding */}
              <div className="grid grid-cols-[auto_1fr_auto] gap-2.5 sm:gap-3 items-center py-3 first:pt-0">
                <div
                  className="icon-plate shrink-0"
                  style={{ background: "rgba(255,79,0,0.1)", color: "#ff4f00" }}
                >
                  <RefreshCw className="h-3.5 w-3.5" />
                </div>
                <div className="min-w-0">
                  <p className="body-sm-strong text-ink">Reset onboarding</p>
                  <p className="caption text-body-muted mt-0.5">
                    Re-run the initial setup wizard
                  </p>
                </div>
                <AlertDialog>
                  <AlertDialogTrigger asChild>
                    <button
                      className="paper-button-outline size-sm shrink-0 cursor-pointer"
                      disabled={isResettingOnboarding}
                    >
                      Reset
                    </button>
                  </AlertDialogTrigger>
                  <AlertDialogContent
                    className="bg-canvas border-hairline rounded-md p-0 gap-0 max-w-[420px] overflow-hidden"
                    style={{
                      boxShadow:
                        "0 24px 64px -16px rgba(32,21,21,0.32), 0 4px 12px -4px rgba(32,21,21,0.12)",
                    }}
                  >
                    <div className="px-5 pt-5 pb-4 border-b border-hairline">
                      <div className="flex items-start gap-2.5">
                        <div
                          className="h-8 w-8 rounded-md flex items-center justify-center shrink-0"
                          style={{
                            background: "rgba(255,79,0,0.1)",
                            color: "#ff4f00",
                          }}
                        >
                          <RefreshCw className="h-4 w-4" />
                        </div>
                        <div className="min-w-0">
                          <p className="display-xs text-ink">
                            Re-run the setup wizard?
                          </p>
                          <p className="body-sm text-body-muted mt-1">
                            The next time Whisprtypr launches, you'll go through the
                            setup wizard again. Your history, settings, and
                            downloaded models stay intact.
                          </p>
                        </div>
                      </div>
                    </div>
                    <div className="px-5 py-4 flex flex-col-reverse sm:flex-row sm:justify-end gap-2">
                      <AlertDialogCancel
                        className="paper-button-outline cursor-pointer"
                        style={{ margin: 0 }}
                      >
                        Cancel
                      </AlertDialogCancel>
                      <AlertDialogAction
                        onClick={handleResetOnboarding}
                        className="paper-button-primary cursor-pointer"
                      >
                        Re-run setup
                      </AlertDialogAction>
                    </div>
                  </AlertDialogContent>
                </AlertDialog>
              </div>

              {/* Reset App Data */}
              <div className="grid grid-cols-[auto_1fr_auto] gap-2.5 sm:gap-3 items-start py-3 last:pb-0">
                <div
                  className="icon-plate shrink-0 mt-0.5"
                  style={{ background: "rgba(207,32,47,0.1)", color: "#cf202f" }}
                >
                  <Trash2 className="h-3.5 w-3.5" />
                </div>
                <div className="min-w-0">
                  <p className="body-sm-strong text-ink">Reset app data</p>
                  <p className="caption text-body-muted mt-0.5">
                    Completely reset Whisprtypr to its initial state
                  </p>
                  <ul className="mt-1.5 space-y-0.5 caption text-body-muted">
                    <li className="flex items-center gap-1.5">
                      <Circle
                        className="h-1 w-1 fill-body-mid text-body-mid shrink-0"
                        style={{ color: "#cf202f" }}
                      />
                      Delete all transcription history
                    </li>
                    <li className="flex items-center gap-1.5">
                      <Circle
                        className="h-1 w-1 fill-body-mid text-body-mid shrink-0"
                        style={{ color: "#cf202f" }}
                      />
                      Remove all downloaded models
                    </li>
                    <li className="flex items-center gap-1.5">
                      <Circle
                        className="h-1 w-1 fill-body-mid text-body-mid shrink-0"
                        style={{ color: "#cf202f" }}
                      />
                      Clear all settings and preferences
                    </li>
                    <li className="flex items-center gap-1.5">
                      <Circle
                        className="h-1 w-1 fill-body-mid text-body-mid shrink-0"
                        style={{ color: "#cf202f" }}
                      />
                      Reset system permissions
                    </li>
                  </ul>
                </div>
                <AlertDialog>
                  <AlertDialogTrigger asChild>
                    <button
                      className="paper-button-outline size-sm shrink-0 cursor-pointer"
                      style={{ borderColor: "#cf202f", color: "#cf202f" }}
                      disabled={isResettingApp}
                    >
                      {isResettingApp ? (
                        <span className="caption-strong">Resetting...</span>
                      ) : (
                        "Reset"
                      )}
                    </button>
                  </AlertDialogTrigger>
                  <AlertDialogContent
                    className="bg-canvas border-hairline rounded-md p-0 gap-0 max-w-[440px] overflow-hidden"
                    style={{
                      boxShadow:
                        "0 24px 64px -16px rgba(32,21,21,0.32), 0 4px 12px -4px rgba(32,21,21,0.12)",
                    }}
                  >
                    <div
                      className="px-5 pt-5 pb-4 border-b border-hairline"
                      style={{ background: "rgba(207,32,47,0.04)" }}
                    >
                      <div className="flex items-start gap-2.5">
                        <div
                          className="h-8 w-8 rounded-md flex items-center justify-center shrink-0"
                          style={{
                            background: "rgba(207,32,47,0.1)",
                            color: "#cf202f",
                          }}
                        >
                          <AlertTriangle className="h-4 w-4" />
                        </div>
                        <div className="min-w-0">
                          <p className="display-xs text-ink">
                            Reset everything?
                          </p>
                          <p className="body-sm text-body-muted mt-1">
                            This deletes all transcription history, removes all
                            downloaded models, clears all settings, and resets
                            system permissions. Your license stays intact. This
                            cannot be undone.
                          </p>
                        </div>
                      </div>
                    </div>
                    <div className="px-5 py-4 flex flex-col-reverse sm:flex-row sm:justify-end gap-2">
                      <AlertDialogCancel
                        className="paper-button-outline cursor-pointer"
                        style={{ margin: 0 }}
                      >
                        Cancel
                      </AlertDialogCancel>
                      <AlertDialogAction
                        onClick={handleResetAppData}
                        className="paper-button-primary cursor-pointer"
                        style={{
                          background: "#cf202f",
                          borderColor: "#cf202f",
                          color: "#fffefb",
                        }}
                      >
                        Reset everything
                      </AlertDialogAction>
                    </div>
                  </AlertDialogContent>
                </AlertDialog>
              </div>
            </div>
          </section>

          {/* ─── SOFTWARE UPDATES ─── */}
          <UpdaterView />
        </div>
      </div>
    </div>
  );
}