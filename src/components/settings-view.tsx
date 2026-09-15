import {
  AlertCircle,
  Circle,
  Clipboard,
  Globe,
  Keyboard,
  Maximize2,
  Monitor,
  Power,
  RefreshCw,
  Sparkles,
  Volume2,
  Waves,
  Zap,
} from "@/components/icons";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { useToast } from "@/hooks/use-toast";
import { setAutoStart } from "@/lib/preferences-api";
import { cn } from "@/lib/utils";
import { hideRecordingOverlay, reportError } from "@/lib/voice-api";
import { useAppStore } from "@/store";
import { useEffect, useRef, useState } from "react";

import { Logo } from "@/components/logo";
import { LANGUAGE_NAMES } from "@/types";

interface SettingsViewProps {
  onClose: () => void;
}

export function SettingsView(_props: SettingsViewProps) {
  const { settings, updateSettings } = useAppStore();
  const { error: toastError } = useToast();

  const [recordingPushToTalk, setRecordingPushToTalk] = useState(false);
  const [recordingToggle, setRecordingToggle] = useState(false);
  const [recordingSecondsLeft, setRecordingSecondsLeft] = useState(0);
  const [settingsError, setSettingsError] = useState<string | null>(null);

  const getErrorMessage = (error: unknown) =>
    error instanceof Error
      ? error.message
      : String(error || "Something went wrong");

  // Ref so the keydown listener always knows which field to write to
  // without being re-bound on every state change.
  const recordingTypeRef = useRef<"pushToTalk" | "toggle" | null>(null);
  useEffect(() => {
    recordingTypeRef.current = recordingPushToTalk
      ? "pushToTalk"
      : recordingToggle
        ? "toggle"
        : null;
  }, [recordingPushToTalk, recordingToggle]);

  // Single, effect-managed keydown listener for hotkey capture. Always
  // removed on cleanup so a stale listener can never fire and overwrite
  // the wrong field.
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const type = recordingTypeRef.current;
      if (!type) return;

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
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [updateSettings]);

  // Tick the seconds-remaining counter while a capture is active
  useEffect(() => {
    if (!recordingPushToTalk && !recordingToggle) return;
    setRecordingSecondsLeft(5);
    const interval = window.setInterval(() => {
      setRecordingSecondsLeft((prev) => {
        if (prev <= 1) {
          window.clearInterval(interval);
          return 0;
        }
        return prev - 1;
      });
    }, 1000);
    return () => window.clearInterval(interval);
  }, [recordingPushToTalk, recordingToggle]);

  // 5s timeout to abort the capture if no key is pressed
  useEffect(() => {
    if (!recordingPushToTalk && !recordingToggle) return;
    const timer = window.setTimeout(() => {
      setRecordingPushToTalk(false);
      setRecordingToggle(false);
      setRecordingSecondsLeft(0);
    }, 5000);
    return () => window.clearTimeout(timer);
  }, [recordingPushToTalk, recordingToggle]);

  const handleRecordHotkey = (type: "pushToTalk" | "toggle") => {
    if (type === "pushToTalk") {
      setRecordingPushToTalk(true);
      setRecordingToggle(false);
    } else {
      setRecordingToggle(true);
      setRecordingPushToTalk(false);
    }
    setRecordingSecondsLeft(5);
  };

  const activeCount =
    Number(settings.showRecordingOverlay) +
    Number(settings.playAudioFeedback) +
    Number(settings.postProcessingEnabled) +
    Number(settings.voiceCommandsEnabled);

  return (
    <div className="flex h-full flex-col overflow-hidden bg-canvas">
      {/* ─── HEADER ─── */}
      <div className="shrink-0 border-b border-hairline">
        <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-3 sm:py-4">
          <p className="eyebrow-uppercase text-ink-mid">Settings</p>
<h1
              className="display-sm text-ink mt-1"
            >
              Tune WhisprTypr to your <span className="text-primary">workflow</span>.
            </h1>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto">
        <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-4 xl:py-5 space-y-4 xl:space-y-5">
          {settingsError && (
            <div className="p-3 rounded-md border border-destructive/30 bg-destructive/5 flex items-center gap-2.5 text-destructive">
              <AlertCircle className="h-4 w-4 shrink-0" />
              <span className="body-sm">{settingsError}</span>
            </div>
          )}

          {/* ─── HERO STATUS BAND — Dark coffee-ink ─── */}
          <section className="hero-band-dark">
            <div className="grid grid-cols-1 @xl:grid-cols-[1.4fr_1fr] gap-4 @xl:gap-6 p-4 sm:p-5 @xl:p-6 items-start @xl:items-center">
              <div className="min-w-0">
                <p className="eyebrow-uppercase text-primary mb-2">
                  <span className="inline-flex items-center gap-2">
                    <Circle className="h-1.5 w-1.5 fill-primary text-primary" />
                    Personalized
                  </span>
                </p>
                <h2
                  className="display-md text-on-dark"
                >
                  {activeCount} of 5 power-ups <span className="text-primary">active</span>.
                </h2>
                <p className="body-sm text-on-dark-soft mt-2 max-w-xl">
                  Configure hotkeys, transcription behavior, and how WhisprTypr shows up on your desktop.
                </p>
              </div>

              <div className="product-ui-card-dark w-full">
                <div className="flex items-center gap-2.5 mb-3">
                  <div className="icon-plate-dark">
                    <Keyboard className="h-3.5 w-3.5 text-on-dark" />
                  </div>
                  <div className="min-w-0">
                    <p className="caption-strong text-on-dark">Current hotkey</p>
                    <p className="caption text-on-dark-soft mt-0.5 truncate">
                      {settings.hotkeyMode === "push-to-talk" ? settings.pushToTalkKey : settings.toggleKey}
                    </p>
                  </div>
                </div>
                <div className="flex items-center justify-between gap-2 pt-3 border-t" style={{ borderColor: '#36342e' }}>
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
            <div className="flex items-center gap-2.5 mb-4">
              <div className="min-w-0">
                <p className="eyebrow-uppercase text-ink-mid">Hotkey</p>
                <h3
                  className="title-md text-ink mt-0.5"
                >
                  Recording shortcuts
                </h3>
              </div>
            </div>

            <div className="space-y-3.5">
              <div className="space-y-1.5">
                <Label className="eyebrow-uppercase text-ink-mid">Recording mode</Label>
                <Select
                  value={settings.hotkeyMode}
                  onValueChange={(value: "push-to-talk" | "toggle") =>
                    updateSettings({ hotkeyMode: value })
                  }
                >
                  <SelectTrigger
                    className="paper-input border border-hairline h-9 cursor-pointer"
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

              <div className="grid grid-cols-1 min-[520px]:grid-cols-2 gap-3">
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

          {/* ─── TRANSLATION SETTINGS — Cream surface ─── */}
          <section className="card-feature-cream">
            <div className="flex items-center gap-2.5 mb-4">
              <div className="min-w-0">
                <p className="eyebrow-uppercase text-ink-mid">Translation</p>
                <h3
                  className="title-md text-ink mt-0.5"
                >
                  Dictation translation
                </h3>
              </div>
            </div>

            <div className="space-y-3.5">
              <SettingRow
                icon={<Globe className="h-3.5 w-3.5" />}
                iconClass={settings.translationEnabled ? "bg-primary/10 text-primary" : ""}
                title="Enable translation"
                description="Dictate in one language and paste the text in another"
                checked={settings.translationEnabled}
                onChange={(checked) => updateSettings({ translationEnabled: checked })}
              />

              {settings.translationEnabled && (
                <>
                  <div className="h-px bg-hairline-soft" />

                  <HotkeyCaptureField
                    label="Translation hotkey"
                    value={settings.translationHotkey}
                    isRecording={recordingPushToTalk}
                    secondsLeft={recordingSecondsLeft}
                    onRecord={() => {
                      setRecordingPushToTalk(true);
                      setRecordingToggle(false);
                      setRecordingSecondsLeft(5);
                    }}
                  />

                  <div className="grid grid-cols-1 min-[520px]:grid-cols-2 gap-3">
                    <div className="space-y-1.5">
                      <Label className="eyebrow-uppercase text-ink-mid">Source language</Label>
                      <Select
                        value={settings.translationSourceLanguage}
                        onValueChange={(value) =>
                          updateSettings({ translationSourceLanguage: value })
                        }
                      >
                        <SelectTrigger
                          className="paper-input border border-hairline h-9 cursor-pointer"
                          style={{ background: '#ffffff', borderRadius: '8px' }}
                        >
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent className="max-h-60">
                          {Object.entries(LANGUAGE_NAMES)
                            .filter(([code]) => code !== "auto")
                            .map(([code, name]) => (
                              <SelectItem key={code} value={code}>
                                {name}
                              </SelectItem>
                            ))}
                        </SelectContent>
                      </Select>
                      <p className="caption text-body-muted">
                        Language you will speak in
                      </p>
                    </div>

                    <div className="space-y-1.5">
                      <Label className="eyebrow-uppercase text-ink-mid">Target language</Label>
                      <Select
                        value={settings.translationTargetLanguage}
                        onValueChange={(value) =>
                          updateSettings({ translationTargetLanguage: value })
                        }
                      >
                        <SelectTrigger
                          className="paper-input border border-hairline h-9 cursor-pointer"
                          style={{ background: '#ffffff', borderRadius: '8px' }}
                        >
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent className="max-h-60">
                          {Object.entries(LANGUAGE_NAMES)
                            .filter(([code]) => code !== "auto")
                            .map(([code, name]) => (
                              <SelectItem key={code} value={code}>
                                {name}
                              </SelectItem>
                            ))}
                        </SelectContent>
                      </Select>
                      <p className="caption text-body-muted">
                        Language to translate to
                      </p>
                    </div>
                  </div>

                  <div className="space-y-1.5">
                    <Label className="eyebrow-uppercase text-ink-mid">
                      MyMemory API key (optional)
                    </Label>
                    <input
                      type="password"
                      className="paper-input border border-hairline h-9 w-full px-3 py-1.5 text-sm"
                      style={{ background: '#ffffff', borderRadius: '8px' }}
                      placeholder="Leave empty for free tier (5,000 chars/day)"
                      value={settings.translationApiKey}
                      onChange={(e) =>
                        updateSettings({ translationApiKey: e.target.value })
                      }
                    />
                    <p className="caption text-body-muted">
                      Optional: add your own key for higher limits.
                    </p>
                  </div>
                </>
              )}
            </div>
          </section>

          {/* ─── RECORDING — White surface ─── */}
          <section className="paper-card">
            <div className="flex items-center gap-2.5 mb-3">
              <div className="min-w-0">
                <p className="eyebrow-uppercase text-ink-mid">Recording</p>
                <h3
                  className="title-md text-ink mt-0.5"
                >
                  Recording behavior
                </h3>
              </div>
            </div>

            <div className="divide-y divide-hairline-soft">
              <SettingRow
                icon={<Waves className="h-3.5 w-3.5" />}
                iconClass={settings.showRecordingOverlay ? "bg-primary/10 text-primary" : ""}
                title="Recording overlay"
                description="Show animated pill while recording"
                checked={settings.showRecordingOverlay}
                onChange={async (checked) => {
                  if (!checked) {
                    hideRecordingOverlay().catch((err) => {
                      console.warn("Failed to hide recording overlay:", err);
                    });
                  }
                  updateSettings({ showRecordingOverlay: checked });
                }}
              />
              {settings.showRecordingOverlay && (
                <div className="grid grid-cols-[auto_1fr_auto] gap-2.5 sm:gap-3 items-center py-2.5">
                  <div className="icon-plate">
                    <Maximize2 className="h-3.5 w-3.5" />
                  </div>
                  <div className="min-w-0">
                    <Label className="body-sm-strong text-ink cursor-pointer">Overlay position</Label>
                    <p className="caption text-body-muted mt-0.5">Where the recording pill appears on screen</p>
                  </div>
                  <Select
                    value={settings.recordingOverlayPosition}
                    onValueChange={(value: "top-left" | "top-center" | "top-right" | "bottom-left" | "bottom-center" | "bottom-right") =>
                      updateSettings({ recordingOverlayPosition: value })
                    }
                  >
                    <SelectTrigger className="paper-input border border-hairline h-9 w-[140px] cursor-pointer" style={{ background: '#ffffff', borderRadius: '8px' }}>
                    <SelectValue placeholder="Top center" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="top-left">Top left</SelectItem>
                      <SelectItem value="top-center">Top center</SelectItem>
                      <SelectItem value="top-right">Top right</SelectItem>
                      <SelectItem value="bottom-left">Bottom left</SelectItem>
                      <SelectItem value="bottom-center">Bottom center</SelectItem>
                      <SelectItem value="bottom-right">Bottom right</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              )}
              <SettingRow
                icon={<Volume2 className="h-3.5 w-3.5" />}
                iconClass={settings.playAudioFeedback ? "bg-primary/10 text-primary" : ""}
                title="Audio feedback"
                description="Play sound when recording starts/stops"
                checked={settings.playAudioFeedback}
                onChange={(checked) => updateSettings({ playAudioFeedback: checked })}
              />
            </div>
          </section>

          {/* ─── SYSTEM — White surface ─── */}
          <section className="paper-card">
            <div className="flex items-center gap-2.5 mb-3">
              <div className="min-w-0">
                <p className="eyebrow-uppercase text-ink-mid">System</p>
                <h3
                  className="title-md text-ink mt-0.5"
                >
                  System behavior
                </h3>
              </div>
            </div>

            <div className="divide-y divide-hairline-soft">
              <SettingRow
                icon={<Power className="h-3.5 w-3.5" />}
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
                icon={<Monitor className="h-3.5 w-3.5" />}
                iconClass={settings.minimizeToTray ? "bg-primary/10 text-primary" : ""}
                title="Minimize to tray"
                description="Keep running in system tray when closed"
                checked={settings.minimizeToTray}
                onChange={(checked) => updateSettings({ minimizeToTray: checked })}
              />
            </div>
          </section>

          {/* ─── UPDATES — White surface ─── */}
          <section className="paper-card">
            <div className="flex items-center gap-2.5 mb-3">
              <div className="min-w-0">
                <p className="eyebrow-uppercase text-ink-mid">Updates</p>
                <h3
                  className="title-md text-ink mt-0.5"
                >
                  Software updates
                </h3>
              </div>
            </div>

            <div className="divide-y divide-hairline-soft">
              <SettingRow
                icon={<RefreshCw className="h-3.5 w-3.5" />}
                iconClass={settings.autoCheckForUpdates ? "bg-primary/10 text-primary" : ""}
                title="Check for updates automatically"
                description="Check daily and notify you before downloading or installing anything."
                checked={settings.autoCheckForUpdates}
                onChange={(checked) => updateSettings({ autoCheckForUpdates: checked })}
              />
            </div>
          </section>

          {/* ─── PROCESSING — White surface ─── */}
          <section className="paper-card">
            <div className="flex items-center gap-2.5 mb-3">
              <div className="min-w-0">
                <p className="eyebrow-uppercase text-ink-mid">Processing</p>
                <h3
                  className="title-md text-ink mt-0.5"
                >
                  Text processing
                </h3>
              </div>
            </div>

            <div className="divide-y divide-hairline-soft">
              <SettingRow
                icon={<Sparkles className="h-3.5 w-3.5" />}
                iconClass={settings.postProcessingEnabled ? "bg-primary/10 text-primary" : ""}
                title="Smart text processing"
                description='Auto-format: "camel case" → camelCase'
                checked={settings.postProcessingEnabled}
                onChange={(checked) => updateSettings({ postProcessingEnabled: checked })}
              />
              <SettingRow
                icon={<Zap className="h-3.5 w-3.5" />}
                iconClass={settings.voiceCommandsEnabled ? "bg-primary/10 text-primary" : ""}
                title="Voice commands"
                description="Allow spoken editing commands like undo, paste, delete line"
                checked={settings.voiceCommandsEnabled}
                onChange={(checked) => updateSettings({ voiceCommandsEnabled: checked })}
              />
            </div>
          </section>

          {/* ─── OUTPUT — White surface ─── */}
          <section className="paper-card">
            <div className="flex items-center gap-2.5 mb-3">
              <div className="min-w-0">
                <p className="eyebrow-uppercase text-ink-mid">Output</p>
                <h3
                  className="title-md text-ink mt-0.5"
                >
                  Output behavior
                </h3>
              </div>
            </div>

            <div className="divide-y divide-hairline-soft">
              <SettingRow
                icon={<Clipboard className="h-3.5 w-3.5" />}
                iconClass={settings.clipboardMode ? "bg-primary/10 text-primary" : ""}
                title="Clipboard mode"
                description="Copy text to clipboard instead of typing at cursor"
                checked={settings.clipboardMode}
                onChange={(checked) => updateSettings({ clipboardMode: checked })}
              />
            </div>
          </section>

          {/* ─── APP INFO FOOTER — Dark coffee-ink band ─── */}
          <section className="hero-band-dark">
            <div className="flex flex-col items-center text-center p-5 sm:p-6 gap-2">
              <Logo size="sm" />
              <p className="caption text-on-dark-soft">
                Speak at the speed of thought.
              </p>
              <div className="h-px w-10 my-0.5" style={{ background: "#36342e" }} />
              <p className="caption text-on-dark-muted">
                © {new Date().getFullYear()} WhisprTypr · All rights reserved
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
    <div className="grid grid-cols-[auto_1fr_auto] gap-2.5 sm:gap-3 items-center py-2.5 first:pt-0 last:pb-0">
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
    <div className="space-y-1.5">
      <Label className="eyebrow-uppercase text-ink-mid">{label}</Label>
      <div className="flex items-stretch gap-2">
        <div
          className={cn(
            "relative flex-1 min-w-0 rounded-md transition-all flex items-center justify-between gap-2 px-3 py-2 overflow-hidden",
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
            >
              {value}
            </code>
          )}
        </div>

        <button
          className={cn(
            "shrink-0 px-3.5 py-2 text-sm rounded-md transition-colors cursor-pointer disabled:cursor-not-allowed",
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