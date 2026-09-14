import { cn } from "@/lib/utils";
import { useAppStore } from "@/store";
import { AlertTriangle, Clipboard, Keyboard, Type } from "lucide-react";
import { useState } from "react";

interface HotkeyStepProps {
  onNext: () => void;
  onBack: () => void;
}

const STEPS_TOTAL = 4;
const STEP_INDEX = 4;

type HotkeyMode = "push-to-talk" | "toggle";
type OutputMode = "inject" | "clipboard";

const hotkeyOptions: {
  mode: HotkeyMode;
  title: string;
  description: string;
  defaultKey: string;
}[] = [
  {
    mode: "push-to-talk",
    title: "Push to Talk",
    description: "Hold key to record, release to transcribe",
    defaultKey: "Alt+Shift+S",
  },
  {
    mode: "toggle",
    title: "Toggle Mode",
    description: "Press to start/stop recording",
    defaultKey: "Alt+Shift+D",
  },
];

const outputOptions: {
  mode: OutputMode;
  title: string;
  description: string;
  icon: typeof Type;
}[] = [
  {
    mode: "inject",
    title: "Type Text",
    description: "Automatically type text where your cursor is",
    icon: Type,
  },
  {
    mode: "clipboard",
    title: "Copy to Clipboard",
    description: "Copy text to clipboard for manual pasting",
    icon: Clipboard,
  },
];

export function HotkeyStep({ onNext, onBack }: HotkeyStepProps) {
  const { settings, updateSettings } = useAppStore();
  const [selectedMode, setSelectedMode] = useState<HotkeyMode>(settings.hotkeyMode);
  const [selectedOutputMode, setSelectedOutputMode] = useState<OutputMode>(
    settings.clipboardMode ? "clipboard" : "inject"
  );
  const [isRecordingHotkey, setIsRecordingHotkey] = useState(false);
  const [recordedKeys, setRecordedKeys] = useState<string[]>([]);
  const [customHotkey, setCustomHotkey] = useState<string | null>(null);
  const [conflict, setConflict] = useState<string | null>(null);

  const currentHotkey =
    customHotkey ||
    (selectedMode === "push-to-talk" ? settings.pushToTalkKey : settings.toggleKey);

  const startRecordingHotkey = () => {
    setIsRecordingHotkey(true);
    setRecordedKeys([]);
    setConflict(null);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (!isRecordingHotkey) return;
    e.preventDefault();
    e.stopPropagation();

    const key = e.key;
    const modifiers: string[] = [];
    if (e.ctrlKey) modifiers.push("Ctrl");
    if (e.altKey) modifiers.push("Alt");
    if (e.shiftKey) modifiers.push("Shift");
    if (e.metaKey) modifiers.push("Meta");

    if (["Control", "Alt", "Shift", "Meta"].includes(key)) {
      setRecordedKeys(modifiers);
      return;
    }

    const displayKey = key.length === 1 ? key.toUpperCase() : key;
    const fullHotkey = [...modifiers, displayKey].join("+");
    setRecordedKeys([...modifiers, displayKey]);

    const conflictingShortcuts: Record<string, string> = {
      "Ctrl+C": "Copy",
      "Ctrl+V": "Paste",
      "Ctrl+X": "Cut",
      "Ctrl+Z": "Undo",
      "Ctrl+S": "Save",
      "Alt+Tab": "Switch Window",
      "Alt+F4": "Close Window",
    };

    if (conflictingShortcuts[fullHotkey]) {
      setConflict(`Conflicts with "${conflictingShortcuts[fullHotkey]}"`);
    } else {
      setConflict(null);
      setCustomHotkey(fullHotkey);
    }
    setIsRecordingHotkey(false);
  };

  const handleContinue = () => {
    const newSettings: Partial<typeof settings> = {
      hotkeyMode: selectedMode,
      clipboardMode: selectedOutputMode === "clipboard",
    };
    if (customHotkey) {
      if (selectedMode === "push-to-talk") {
        newSettings.pushToTalkKey = customHotkey;
      } else {
        newSettings.toggleKey = customHotkey;
      }
    }
    updateSettings(newSettings);
    onNext();
  };

  return (
    <div className="flex h-full flex-col overflow-hidden bg-canvas">
      <div className="flex-1 overflow-y-auto">
        <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-6 xl:py-10 space-y-6">
          {/* HERO — Dark band */}
          <section className="hero-band-dark">
            <div className="flex flex-col items-center text-center gap-4 p-8 sm:p-10">
              <p className="eyebrow-uppercase text-primary">
                <span className="inline-flex items-center gap-2">
                  <span className="h-1 w-1 rounded-full bg-primary" />
                  Step {STEP_INDEX} of {STEPS_TOTAL}
                </span>
              </p>
              <h2
                className="font-display font-medium text-on-dark text-[clamp(1.625rem,3vw,2.375rem)]"
                style={{ lineHeight: 1.05, letterSpacing: "-0.02em" }}
              >
                Set your <span className="text-primary">shortcut</span>.
              </h2>
              <p className="body-md text-on-dark-soft max-w-md">
                Pick how WhisprTypr activates, where the text goes, and your key combo.
              </p>
            </div>
          </section>

          {/* ACTIVATION MODE */}
          <section className="card-feature-cream">
            <div className="flex items-center gap-3 mb-5">
              <div className="icon-plate">
                <Keyboard className="h-4 w-4" />
              </div>
              <div className="min-w-0 flex-1">
                <p className="eyebrow-uppercase text-ink-mid">Activation</p>
                <h3
                  className="font-display font-medium text-ink text-[clamp(1.0625rem,1.5vw,1.25rem)] mt-1"
                  style={{ letterSpacing: "-0.015em" }}
                >
                  Choose a mode
                </h3>
              </div>
            </div>

            <div className="space-y-3">
              {hotkeyOptions.map((option) => {
                const isSelected = selectedMode === option.mode;
                return (
                  <button
                    key={option.mode}
                    type="button"
                    onClick={() => {
                      setSelectedMode(option.mode);
                      setCustomHotkey(null);
                      setConflict(null);
                    }}
                    className={cn(
                      "w-full text-left rounded-md border p-4 transition-all cursor-pointer",
                      isSelected
                        ? "border-primary bg-canvas"
                        : "border-hairline bg-canvas hover:border-ink",
                    )}
                    style={isSelected ? { boxShadow: "0 0 0 1px #ff4f00" } : undefined}
                  >
                    <div className="flex items-start gap-3">
                      <div
                        className={cn(
                          "flex h-5 w-5 items-center justify-center rounded-full border-2 shrink-0 mt-0.5",
                          isSelected ? "border-primary" : "border-mute",
                        )}
                      >
                        {isSelected && (
                          <div className="h-2.5 w-2.5 rounded-full bg-primary" />
                        )}
                      </div>
                      <div className="min-w-0 flex-1">
                        <span
                          className="font-display font-medium text-ink text-[clamp(0.9375rem,1.2vw,1.0625rem)]"
                          style={{ letterSpacing: "-0.015em" }}
                        >
                          {option.title}
                        </span>
                        <p className="body-sm text-body-muted mt-1">
                          {option.description}
                        </p>
                        <p className="caption text-body mt-2 font-mono">
                          Default: {option.defaultKey}
                        </p>
                      </div>
                    </div>
                  </button>
                );
              })}
            </div>
          </section>

          {/* HOTKEY RECORDER */}
          <section className="card-feature-cream">
            <div className="flex items-center gap-3 mb-5">
              <div className="icon-plate">
                <Type className="h-4 w-4" />
              </div>
              <div className="min-w-0 flex-1">
                <p className="eyebrow-uppercase text-ink-mid">Hotkey</p>
                <h3
                  className="font-display font-medium text-ink text-[clamp(1.0625rem,1.5vw,1.25rem)] mt-1"
                  style={{ letterSpacing: "-0.015em" }}
                >
                  Capture a key combination
                </h3>
              </div>
            </div>

            <button
              type="button"
              onClick={startRecordingHotkey}
              onKeyDown={handleKeyDown}
              onBlur={() => setIsRecordingHotkey(false)}
              className={cn(
                "w-full p-5 rounded-md border-2 border-dashed text-center transition-all focus:outline-none cursor-pointer",
                isRecordingHotkey
                  ? "border-primary bg-primary/5"
                  : "border-hairline bg-canvas hover:border-ink",
              )}
            >
              {isRecordingHotkey ? (
                <div className="space-y-2">
                  <p className="body-sm-strong text-primary">
                    {recordedKeys.length > 0
                      ? `${recordedKeys.join(" + ")} + ...`
                      : "Press a key combination..."}
                  </p>
                  <p className="caption text-body-muted">Press a key to finish</p>
                </div>
              ) : (
                <div className="space-y-1">
                  <p className="caption eyebrow-uppercase text-ink-mid">Current</p>
                  <p className="font-mono font-display font-medium text-ink text-[clamp(1.0625rem,1.5vw,1.25rem)] mt-1">
                    {currentHotkey}
                  </p>
                </div>
              )}
            </button>

            {conflict && (
              <div
                className="mt-4 p-3.5 rounded-md border flex items-start gap-2.5"
                style={{ borderColor: "rgba(245,158,11,0.3)", background: "rgba(245,158,11,0.05)" }}
              >
                <AlertTriangle className="h-4 w-4 shrink-0 mt-0.5" style={{ color: "#d97706" }} />
                <p className="body-sm" style={{ color: "#d97706" }}>{conflict}</p>
              </div>
            )}

            <div className="mt-4 p-3.5 rounded-md border border-hairline bg-canvas-soft flex items-start gap-2.5">
              <Keyboard className="h-4 w-4 text-body-muted shrink-0 mt-0.5" />
              <div className="space-y-0.5">
                <p className="body-sm text-ink">Combine modifiers + a letter</p>
                <p className="caption text-body-muted">Avoid Ctrl+C, Ctrl+V, etc.</p>
              </div>
            </div>
          </section>

          {/* OUTPUT MODE */}
          <section className="card-feature-cream">
            <div className="flex items-center gap-3 mb-5">
              <div className="icon-plate">
                <Clipboard className="h-4 w-4" />
              </div>
              <div className="min-w-0 flex-1">
                <p className="eyebrow-uppercase text-ink-mid">Output</p>
                <h3
                  className="font-display font-medium text-ink text-[clamp(1.0625rem,1.5vw,1.25rem)] mt-1"
                  style={{ letterSpacing: "-0.015em" }}
                >
                  Where does the text go?
                </h3>
              </div>
            </div>

            <div className="space-y-3">
              {outputOptions.map((option) => {
                const isSelected = selectedOutputMode === option.mode;
                const Icon = option.icon;
                return (
                  <button
                    key={option.mode}
                    type="button"
                    onClick={() => setSelectedOutputMode(option.mode)}
                    className={cn(
                      "w-full text-left rounded-md border p-4 transition-all cursor-pointer",
                      isSelected
                        ? "border-primary bg-canvas"
                        : "border-hairline bg-canvas hover:border-ink",
                    )}
                    style={isSelected ? { boxShadow: "0 0 0 1px #ff4f00" } : undefined}
                  >
                    <div className="flex items-start gap-3">
                      <div
                        className={cn(
                          "flex h-5 w-5 items-center justify-center rounded-full border-2 shrink-0 mt-0.5",
                          isSelected ? "border-primary" : "border-mute",
                        )}
                      >
                        {isSelected && (
                          <div className="h-2.5 w-2.5 rounded-full bg-primary" />
                        )}
                      </div>
                      <div
                        className={cn(
                          "flex h-9 w-9 items-center justify-center rounded-md shrink-0",
                          isSelected ? "bg-primary/10 text-primary" : "bg-canvas-soft text-ink",
                        )}
                      >
                        <Icon className="h-4 w-4" />
                      </div>
                      <div className="min-w-0 flex-1">
                        <span
                          className="font-display font-medium text-ink text-[clamp(0.9375rem,1.2vw,1.0625rem)]"
                          style={{ letterSpacing: "-0.015em" }}
                        >
                          {option.title}
                        </span>
                        <p className="body-sm text-body-muted mt-1">
                          {option.description}
                        </p>
                      </div>
                    </div>
                  </button>
                );
              })}
            </div>
          </section>
        </div>
      </div>

      {/* STICKY FOOTER */}
      <div className="shrink-0 border-t border-hairline bg-canvas-soft">
        <div className="max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-4 flex items-center justify-between gap-3 flex-wrap">
          <button onClick={onBack} className="paper-button-outline cursor-pointer">
            Back
          </button>
          <button
            onClick={handleContinue}
            disabled={conflict !== null}
            className="paper-button-primary cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Complete setup
          </button>
        </div>
      </div>
    </div>
  );
}
