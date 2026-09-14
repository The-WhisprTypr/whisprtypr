import { Logo } from "@/components/logo";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { UpdaterView } from "@/components/updater-view";
import { useToast } from "@/hooks/use-toast";
import {
  downloadFile,
  exportAppData,
  getStorageStats,
} from "@/lib/data-management";
import { setAutoStart } from "@/lib/preferences-api";
import { cn } from "@/lib/utils";
import { reportError } from "@/lib/voice-api";
import { useAppStore } from "@/store";
import {
  AlertCircle,
  Circle,
  Clipboard,
  Database,
  Keyboard,
  Loader2,
  Monitor,
  Power,
  RotateCcw,
  Sparkles,
  Volume2,
  Waves,
  Zap,
} from "lucide-react";
import { useEffect, useState } from "react";

import { CustomVocabularyView } from "@/components/settings/custom-vocabulary-view";
import type { VocabularyEntry } from "@/types";

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

interface SettingsViewProps {
  onClose: () => void;
}

export function SettingsView(_props: SettingsViewProps) {
  const { settings, updateSettings, resetSettings } = useAppStore();
  const { success: toastSuccess, error: toastError } = useToast();

  const [isExporting, setIsExporting] = useState(false);
  const [exportError, setExportError] = useState<string | null>(null);
  const [settingsError, setSettingsError] = useState<string | null>(null);
  const [isLoadingStats, setIsLoadingStats] = useState(true);
  const [statsError, setStatsError] = useState<string | null>(null);
  const [storageStats, setStorageStats] = useState<{
    historyCount: number;
  } | null>(null);
  const [recordingPushToTalk, setRecordingPushToTalk] = useState(false);
  const [recordingToggle, setRecordingToggle] = useState(false);
  const [recordingSecondsLeft, setRecordingSecondsLeft] = useState(0);

  const getErrorMessage = (error: unknown) =>
    error instanceof Error
      ? error.message
      : String(error || "Something went wrong");

  const loadStorageStats = async () => {
    try {
      setIsLoadingStats(true);
      setStatsError(null);
      const stats = await getStorageStats();
      setStorageStats(stats);
    } catch (error) {
      const message = getErrorMessage(error);
      setStatsError(message);
      await reportError("database", message, "warning", {
        userAction: "Load storage stats",
      }).catch(console.error);
    } finally {
      setIsLoadingStats(false);
    }
  };

  useEffect(() => {
    loadStorageStats();
  }, []);

  const handleExport = async () => {
    try {
      setIsExporting(true);
      setExportError(null);
      const data = await exportAppData();
      const filename = `WhisprTypr-backup-${new Date()
        .toISOString()
        .slice(0, 10)}.json`;
      const saved = await downloadFile(data, filename);
      if (saved) {
        toastSuccess("Export complete", "Data exported successfully");
      }
    } catch (err) {
      const message = getErrorMessage(err);
      console.error("Export failed:", err);
      setExportError(message);
      toastError("Export failed", "Failed to export data");
      await reportError("filesystem", message, "error", {
        userAction: "Export app data",
      }).catch(console.error);
    } finally {
      setIsExporting(false);
    }
  };

  const handleRecordHotkey = (type: "pushToTalk" | "toggle") => {
    if (type === "pushToTalk") {
      setRecordingPushToTalk(true);
      setRecordingToggle(false);
    } else {
      setRecordingToggle(true);
      setRecordingPushToTalk(false);
    }
    setRecordingSecondsLeft(5);

    const handleKeyDown = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();

      const parts: string[] = [];
      if (e.ctrlKey) parts.push("Ctrl");
      if (e.shiftKey) parts.push("Shift");
      if (e.altKey) parts.push("Alt");
      if (e.metaKey) parts.push("Meta");

      let key = e.key;
      if (key === " ") key = "Space";
      else if (key.length === 1) key = key.toUpperCase();
      else if (key.startsWith("Arrow")) key = key;
      else if (
        key === "Control" ||
        key === "Shift" ||
        key === "Alt" ||
        key === "Meta"
      ) {
        return;
      }

      parts.push(key);
      const hotkey = parts.join("+");

      if (type === "pushToTalk") {
        updateSettings({ pushToTalkKey: hotkey });
        setSettingsError(null);
        setRecordingPushToTalk(false);
      } else {
        updateSettings({ toggleKey: hotkey });
        setSettingsError(null);
        setRecordingToggle(false);
      }

      document.removeEventListener("keydown", handleKeyDown);
    };

    document.addEventListener("keydown", handleKeyDown);

    const interval = window.setInterval(() => {
      setRecordingSecondsLeft((prev) => {
        if (prev <= 1) {
          window.clearInterval(interval);
          return 0;
        }
        return prev - 1;
      });
    }, 1000);

    setTimeout(() => {
      window.clearInterval(interval);
      setRecordingPushToTalk(false);
      setRecordingToggle(false);
      setRecordingSecondsLeft(0);
      document.removeEventListener("keydown", handleKeyDown);
    }, 5000);
  };

  const activeCount =
    Number(settings.showRecordingIndicator) +
    Number(settings.showRecordingOverlay) +
    Number(settings.playAudioFeedback) +
    Number(settings.postProcessingEnabled) +
    Number(settings.voiceCommandsEnabled);

  return (
    <div className="flex h-full flex-col overflow-hidden bg-canvas">
      {/* ─── HEADER ─── */}
      <div className="shrink-0 border-b border-hairline">
        <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-4 sm:py-5">
          <p className="eyebrow-uppercase text-ink-mid">Settings</p>
<h1
              className="font-display font-medium text-ink mt-2 text-[clamp(1.375rem,2.2vw,1.875rem)]"
              style={{ lineHeight: 1.1, letterSpacing: '-0.02em' }}
            >
              Tune WhisprTypr to your <span className="text-primary">workflow</span>.
            </h1>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto">
        <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-6 xl:py-10 space-y-6 xl:space-y-8">
          {settingsError && (
            <div className="p-3.5 rounded-md border border-destructive/30 bg-destructive/5 flex items-center gap-2.5 text-destructive">
              <AlertCircle className="h-4 w-4 shrink-0" />
              <span className="body-sm">{settingsError}</span>
            </div>
          )}

          {/* ─── HERO STATUS BAND — Dark coffee-ink ─── */}
          <section className="hero-band-dark">
            <div className="grid grid-cols-1 @xl:grid-cols-[1.4fr_1fr] gap-6 @xl:gap-10 p-5 sm:p-7 @xl:p-10 items-start @xl:items-center">
              <div className="min-w-0">
                <p className="eyebrow-uppercase text-primary mb-3">
                  <span className="inline-flex items-center gap-2">
                    <Circle className="h-1.5 w-1.5 fill-primary text-primary" />
                    Personalized
                  </span>
                </p>
                <h2
                  className="font-display font-medium text-on-dark text-[clamp(1.375rem,2.6vw,2.125rem)]"
                  style={{ lineHeight: 1.05, letterSpacing: '-0.02em' }}
                >
                  {activeCount} of 5 power-ups <span className="text-primary">active</span>.
                </h2>
                <p className="body-md text-on-dark-soft mt-3 max-w-xl">
                  Configure hotkeys, transcription behavior, and how WhisprTypr shows up on your desktop.
                </p>
              </div>

              <div className="product-ui-card-dark w-full">
                <div className="flex items-center gap-3 mb-4">
                  <div className="icon-plate-dark">
                    <Keyboard className="h-4 w-4 text-on-dark" />
                  </div>
                  <div className="min-w-0">
                    <p className="caption-strong text-on-dark">Current hotkey</p>
                    <p className="caption text-on-dark-soft mt-1 truncate">
                      {settings.hotkeyMode === "push-to-talk" ? settings.pushToTalkKey : settings.toggleKey}
                    </p>
                  </div>
                </div>
                <div className="flex items-center justify-between gap-3 pt-4 border-t" style={{ borderColor: '#36342e' }}>
                  <span className="caption text-on-dark-muted">Mode</span>
                  <span className="caption-strong text-on-dark">
                    {settings.hotkeyMode === "push-to-talk" ? "Push to talk" : "Toggle"}
                  </span>
                </div>
              </div>
            </div>
          </section>

          {/* ─── HOTKEY SETTINGS — Cream surface ─── */}
          <section className="card-feature-cream">
            <div className="flex items-center gap-3 mb-6">
              <div className="icon-plate">
                <Keyboard className="h-4 w-4" />
              </div>
              <div className="min-w-0">
                <p className="eyebrow-uppercase text-ink-mid">Hotkey</p>
                <h3
                  className="font-display font-medium text-ink text-[clamp(1rem,1.6vw,1.25rem)] mt-1"
                  style={{ letterSpacing: '-0.015em' }}
                >
                  Recording shortcuts
                </h3>
              </div>
            </div>

            <div className="space-y-5">
              <div className="space-y-2">
                <Label className="eyebrow-uppercase text-ink-mid">Recording mode</Label>
                <Select
                  value={settings.hotkeyMode}
                  onValueChange={(value: "push-to-talk" | "toggle") =>
                    updateSettings({ hotkeyMode: value })
                  }
                >
                  <SelectTrigger
                    className="paper-input border border-hairline h-11 cursor-pointer"
                    style={{ background: '#ffffff', borderRadius: '8px' }}
                  >
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="push-to-talk">Push to talk</SelectItem>
                    <SelectItem value="toggle">Toggle</SelectItem>
                  </SelectContent>
                </Select>
                <p className="caption text-body-muted">
                  {settings.hotkeyMode === "push-to-talk"
                    ? "Hold the key to record, release to stop."
                    : "Press once to start, press again to stop."}
                </p>
              </div>

              <div className="h-px bg-hairline-soft" />

              <div className="grid grid-cols-1 min-[520px]:grid-cols-2 gap-4">
                <HotkeyCaptureField
                  label="Push to talk key"
                  value={settings.pushToTalkKey}
                  isRecording={recordingPushToTalk}
                  secondsLeft={recordingSecondsLeft}
                  onRecord={() => handleRecordHotkey("pushToTalk")}
                />
                <HotkeyCaptureField
                  label="Toggle key"
                  value={settings.toggleKey}
                  isRecording={recordingToggle}
                  secondsLeft={recordingSecondsLeft}
                  onRecord={() => handleRecordHotkey("toggle")}
                />
              </div>
            </div>
          </section>

          {/* ─── PREFERENCES — White surface, hairline bordered settings rows ─── */}
          <section className="paper-card">
            <div className="flex items-center gap-3 mb-5">
              <div className="icon-plate">
                <Sparkles className="h-4 w-4" />
              </div>
              <div className="min-w-0">
                <p className="eyebrow-uppercase text-ink-mid">Preferences</p>
                <h3
                  className="font-display font-medium text-ink text-[clamp(1rem,1.6vw,1.25rem)] mt-1"
                  style={{ letterSpacing: '-0.015em' }}
                >
                  Customize your experience
                </h3>
              </div>
            </div>

            <div className="divide-y divide-hairline-soft">
              <SettingRow
                icon={<Circle className="h-3.5 w-3.5 fill-current" />}
                iconClass={settings.showRecordingIndicator ? "bg-primary/10 text-primary" : ""}
                title="Recording indicator"
                description="Show visual feedback when recording"
                checked={settings.showRecordingIndicator}
                onChange={(checked) => updateSettings({ showRecordingIndicator: checked })}
              />
              <SettingRow
                icon={<Waves className="h-4 w-4" />}
                iconClass={settings.showRecordingOverlay ? "bg-primary/10 text-primary" : ""}
                title="Recording overlay"
                description="Show fullscreen wave animation when recording"
                checked={settings.showRecordingOverlay}
                onChange={(checked) => updateSettings({ showRecordingOverlay: checked })}
              />
              <SettingRow
                icon={<Volume2 className="h-4 w-4" />}
                iconClass={settings.playAudioFeedback ? "bg-primary/10 text-primary" : ""}
                title="Audio feedback"
                description="Play sound when recording starts/stops"
                checked={settings.playAudioFeedback}
                onChange={(checked) => updateSettings({ playAudioFeedback: checked })}
              />
              <SettingRow
                icon={<Power className="h-4 w-4" />}
                iconClass={settings.autoStartOnBoot ? "bg-primary/10 text-primary" : ""}
                title="Start on boot"
                description="Launch WhisprTypr when system starts"
                checked={settings.autoStartOnBoot}
                onChange={async (checked) => {
                  try {
                    setSettingsError(null);
                    await setAutoStart(checked);
                    updateSettings({ autoStartOnBoot: checked });
                  } catch (err) {
                    const message = getErrorMessage(err);
                    console.error("Failed to set autostart:", err);
                    setSettingsError("Could not change Start on Boot.");
                    toastError(
                      "Settings error",
                      "Failed to change autostart setting",
                    );
                    await reportError("configuration", message, "error", {
                      userAction: "Change autostart setting",
                    }).catch(console.error);
                  }
                }}
              />
              <SettingRow
                icon={<Monitor className="h-4 w-4" />}
                iconClass={settings.minimizeToTray ? "bg-primary/10 text-primary" : ""}
                title="Minimize to tray"
                description="Keep running in system tray when closed"
                checked={settings.minimizeToTray}
                onChange={(checked) => updateSettings({ minimizeToTray: checked })}
              />
              <SettingRow
                icon={<Sparkles className="h-4 w-4" />}
                iconClass={settings.postProcessingEnabled ? "bg-primary/10 text-primary" : ""}
                title="Smart text processing"
                description='Auto-format: "camel case" → camelCase'
                checked={settings.postProcessingEnabled}
                onChange={(checked) => updateSettings({ postProcessingEnabled: checked })}
              />
              <SettingRow
                icon={<Zap className="h-4 w-4" />}
                iconClass={settings.voiceCommandsEnabled ? "bg-primary/10 text-primary" : ""}
                title="Voice commands"
                description="Allow spoken editing commands like undo, paste, delete line"
                checked={settings.voiceCommandsEnabled}
                onChange={(checked) => updateSettings({ voiceCommandsEnabled: checked })}
              />
              <SettingRow
                icon={<Clipboard className="h-4 w-4" />}
                iconClass={settings.clipboardMode ? "bg-primary/10 text-primary" : ""}
                title="Clipboard mode"
                description="Copy text to clipboard instead of typing at cursor"
                checked={settings.clipboardMode}
                onChange={(checked) => updateSettings({ clipboardMode: checked })}
                isLast
              />
            </div>
          </section>

          {/* ─── CUSTOM VOCABULARY ─── */}
          <CustomVocabularyView
            entries={settings.customVocabulary ?? []}
            onChange={(entries: VocabularyEntry[]) =>
              updateSettings({ customVocabulary: entries })
            }
          />

          {/* ─── DATA MANAGEMENT — Cream surface with stats card ─── */}
          <section className="card-feature-cream">
            <div className="flex items-center gap-3 mb-5">
              <div className="icon-plate">
                <Database className="h-4 w-4" />
              </div>
              <div className="min-w-0">
                <p className="eyebrow-uppercase text-ink-mid">Data</p>
                <h3
                  className="font-display font-medium text-ink text-[clamp(1rem,1.6vw,1.25rem)] mt-1"
                  style={{ letterSpacing: '-0.015em' }}
                >
                  Manage your data
                </h3>
              </div>
            </div>

            {isLoadingStats ? (
              <div
                className="p-4 rounded-md border border-hairline mb-5 flex items-center gap-3"
                style={{ background: '#fffefb' }}
              >
                <Loader2 className="h-4 w-4 animate-spin text-body-muted" />
                <span className="body-sm text-body-muted">Loading storage stats...</span>
              </div>
            ) : statsError ? (
              <div className="p-4 rounded-md border border-destructive/30 bg-destructive/5 mb-5">
                <div className="flex items-start gap-2.5 text-destructive">
                  <AlertCircle className="h-4 w-4 mt-0.5 shrink-0" />
                  <div className="min-w-0 flex-1">
                    <p className="body-sm">Could not load storage stats.</p>
                    <button
                      className="paper-button-outline size-md mt-3 cursor-pointer"
                      style={{ borderColor: '#cf202f', color: '#cf202f' }}
                      onClick={loadStorageStats}
                    >
                      Retry
                    </button>
                  </div>
                </div>
              </div>
            ) : (
              storageStats && (
                <div
                  className="p-5 rounded-md border border-hairline mb-5 grid grid-cols-1 min-[420px]:grid-cols-[auto_1fr] gap-4 min-[420px]:gap-7 items-center"
                  style={{ background: '#fffefb' }}
                >
                  <div className="min-w-[5rem]">
                    <p className="eyebrow-uppercase text-ink-mid mb-2">In archive</p>
                    <p
                      className="font-display font-medium text-ink text-[clamp(1.875rem,3vw,2.625rem)]"
                      style={{ lineHeight: 1, letterSpacing: '-0.025em', fontVariantNumeric: 'tabular-nums' }}
                    >
                      {storageStats.historyCount.toLocaleString()}
                    </p>
                  </div>
                  <div className="min-w-0">
                    <p
                      className="font-display font-medium text-ink text-[clamp(1rem,1.4vw,1.125rem)]"
                      style={{ letterSpacing: '-0.015em' }}
                    >
                      Transcriptions in history
                    </p>
                    <p className="body-sm text-body-muted mt-1.5">
                      Export your full archive (settings + history) to a JSON file you control.
                    </p>
                  </div>
                </div>
              )
            )}

            <div className="flex flex-wrap items-center gap-3">
              <button
                className="paper-button-primary cursor-pointer disabled:cursor-not-allowed disabled:opacity-50"
                onClick={handleExport}
                disabled={isExporting}
              >
                {isExporting ? <Loader2 className="h-4 w-4 animate-spin" /> : null}
                {isExporting ? "Exporting..." : "Export data"}
              </button>
              <p className="caption text-body-muted">
                Saves to a JSON file you control.
              </p>
            </div>

            {exportError && (
              <div className="mt-4 p-3.5 rounded-md border border-destructive/30 bg-destructive/5 text-destructive flex items-start gap-2.5">
                <AlertCircle className="h-4 w-4 mt-0.5 shrink-0" />
                <span className="body-sm">{exportError}</span>
              </div>
            )}
          </section>

          {/* ─── SOFTWARE UPDATES ─── */}
          <UpdaterView />

          {/* ─── RESET — Destructive cream surface ─── */}
          <section
            className="paper-card"
            style={{ borderColor: 'rgba(207,32,47,0.25)' }}
          >
            <div className="flex items-center gap-3 mb-5">
              <div className="icon-plate shrink-0" style={{ background: 'rgba(207,32,47,0.1)', color: '#cf202f' }}>
                <RotateCcw className="h-4 w-4" />
              </div>
              <div className="min-w-0">
                <p className="eyebrow-uppercase" style={{ color: '#cf202f' }}>Danger zone</p>
                <h3
                  className="font-display font-medium text-ink text-[clamp(1rem,1.6vw,1.25rem)] mt-1"
                  style={{ letterSpacing: '-0.015em' }}
                >
                  Reset to defaults
                </h3>
                <p className="body-sm text-body-muted mt-1.5">
                  Restore every option to its original value. This can't be undone.
                </p>
              </div>
            </div>
            <AlertDialog>
              <AlertDialogTrigger asChild>
                <button
                  className="paper-button-outline cursor-pointer"
                  style={{ borderColor: '#cf202f', color: '#cf202f' }}
                >
                  <RotateCcw className="h-4 w-4" />
                  Reset everything
                </button>
              </AlertDialogTrigger>
              <AlertDialogContent>
                <AlertDialogHeader>
                  <AlertDialogTitle>Reset Settings?</AlertDialogTitle>
                  <AlertDialogDescription>
                    This will restore all settings to their default values. This
                    action cannot be undone.
                  </AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                  <AlertDialogCancel className="paper-button-secondary">
                    Cancel
                  </AlertDialogCancel>
                  <AlertDialogAction
                    onClick={() => {
                      try {
                        resetSettings();
                        setSettingsError(null);
                        toastSuccess?.("Settings reset to defaults");
                      } catch (e) {
                        const message = getErrorMessage(e);
                        setSettingsError("Could not reset settings.");
                        toastError?.("Failed to reset settings");
                        reportError("configuration", message, "error", {
                          userAction: "Reset settings",
                        }).catch(console.error);
                      }
                    }}
                    className="text-white bg-destructive text-destructive-foreground hover:bg-destructive/90"
                  >
                    Reset
                  </AlertDialogAction>
                </AlertDialogFooter>
              </AlertDialogContent>
            </AlertDialog>
          </section>

          {/* ─── APP INFO FOOTER — Dark coffee-ink band ─── */}
          <section className="hero-band-dark">
            <div className="flex flex-col items-center text-center p-8 sm:p-10 gap-3">
              <Logo size="sm" />
              <p className="caption text-on-dark-soft">
                Wave your voice into text at your cursor.
              </p>
              <div className="h-px w-12 my-1" style={{ background: '#36342e' }} />
              <p className="caption text-on-dark-muted">
                © {new Date().getFullYear()} Johuniq · All rights reserved
              </p>
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}

interface SettingRowProps {
  icon: React.ReactNode;
  iconClass?: string;
  title: string;
  description: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  isLast?: boolean;
}

function SettingRow({
  icon,
  iconClass = "",
  title,
  description,
  checked,
  onChange,
}: SettingRowProps) {
  return (
    <div className="grid grid-cols-[auto_1fr_auto] gap-3 sm:gap-4 items-center py-3.5 first:pt-0 last:pb-0">
      <div className={cn("icon-plate", iconClass)}>{icon}</div>
      <div className="min-w-0">
        <Label className="body-sm-strong text-ink cursor-pointer">{title}</Label>
        <p className="caption text-body-muted mt-0.5">{description}</p>
      </div>
      <Switch checked={checked} onCheckedChange={onChange} className="shrink-0" />
    </div>
  );
}

interface HotkeyCaptureFieldProps {
  label: string;
  value: string;
  isRecording: boolean;
  secondsLeft: number;
  onRecord: () => void;
}

function HotkeyCaptureField({
  label,
  value,
  isRecording,
  secondsLeft,
  onRecord,
}: HotkeyCaptureFieldProps) {
  return (
    <div className="space-y-2">
      <Label className="eyebrow-uppercase text-ink-mid">{label}</Label>
      <div className="flex items-stretch gap-2">
        <div
          className={cn(
            "relative flex-1 min-w-0 rounded-md transition-all flex items-center justify-between gap-2 px-3.5 py-2.5 overflow-hidden",
            isRecording ? "border-2 border-primary" : "border border-hairline"
          )}
          style={{ background: '#ffffff' }}
        >
          {/* Pulsing recording background */}
          {isRecording && (
            <div
              aria-hidden
              className="absolute inset-0 pointer-events-none"
              style={{
                background:
                  "linear-gradient(90deg, rgba(255,79,0,0.06), rgba(255,79,0,0.12), rgba(255,79,0,0.06))",
                animation: "shimmer 1.6s linear infinite",
                backgroundSize: "200% 100%",
              }}
            />
          )}

          {isRecording ? (
            <span className="relative z-10 flex items-center gap-2 min-w-0">
              <span className="relative flex h-2 w-2 shrink-0">
                <span className="absolute inset-0 rounded-full bg-primary opacity-60 animate-ping" />
                <span className="relative h-2 w-2 rounded-full bg-primary" />
              </span>
              <span className="body-sm-strong text-primary truncate">
                Press any key...
              </span>
              <span className="caption text-body-muted shrink-0 ml-auto tabular-nums">
                {secondsLeft}s
              </span>
            </span>
          ) : (
            <code
              className="relative z-10 font-mono text-ink truncate body-sm-strong"
              style={{ letterSpacing: '0.01em' }}
            >
              {value}
            </code>
          )}
        </div>

        <button
          className={cn(
            "shrink-0 px-4 py-2.5 text-sm rounded-md transition-colors cursor-pointer disabled:cursor-not-allowed",
            isRecording
              ? "border border-hairline bg-canvas-soft text-body-muted"
              : "border border-ink bg-canvas text-ink hover:bg-canvas-soft"
          )}
          onClick={onRecord}
          disabled={isRecording}
        >
          {isRecording ? "Listening" : "Change"}
        </button>
      </div>
    </div>
  );
}