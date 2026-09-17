import { Check, Cpu, Keyboard, Mic, Sparkles } from "@/components/icons";
import { Logo } from "@/components/logo";
import { useAppStore } from "@/store";

interface CompleteStepProps {
  onFinish: () => void;
}

export function CompleteStep({ onFinish }: CompleteStepProps) {
  const { settings, selectedModel } = useAppStore();

  const currentHotkey =
    settings.hotkeyMode === "push-to-talk"
      ? settings.pushToTalkKey
      : settings.toggleKey;

  return (
    <div className="flex h-full flex-col overflow-hidden bg-canvas">
      <div className="flex-1 overflow-y-auto">
        <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-6 xl:py-10 space-y-6">
          {/* HERO — Dark band with success accent */}
          <section
            className="hero-band-dark relative"
            style={{
              background:
                "radial-gradient(circle at 50% 0%, rgba(255,79,0,0.18) 0%, transparent 60%), linear-gradient(180deg, #22171a 0%, #1a1212 100%)",
            }}
          >
            <div className="flex flex-col items-center text-center gap-4 p-8 sm:p-12">
              <div
                className="h-14 w-14 rounded-full flex items-center justify-center bg-primary/15 text-primary"
              >
                <Check className="h-7 w-7" strokeWidth={2.5} />
              </div>
              <p className="text-[11px] font-medium tracking-[0.1em] uppercase leading-none text-primary">
                <span className="inline-flex items-center gap-2">
                  <span className="h-1 w-1 rounded-full bg-primary" />
                  All set
                </span>
              </p>
              <h2 className="text-[28px] font-medium leading-tight tracking-tight text-on-dark">
                You're <span className="text-primary">ready</span> to dictate.
              </h2>
              <p className="text-sm text-on-dark-soft max-w-md leading-relaxed">
                Whisprtypr is installed, configured, and waiting for your voice.
              </p>
            </div>
          </section>

          {/* CONFIGURATION SUMMARY */}
          <section className="card-feature-cream">
            <div className="flex items-center gap-3 mb-5">
              <div className="flex items-center justify-center size-8 rounded-full bg-canvas-soft text-ink">
                <Sparkles className="h-4 w-4" />
              </div>
              <div className="min-w-0 flex-1">
                <p className="text-[11px] font-medium tracking-[0.1em] uppercase leading-none text-ink-mid">Setup</p>
                <h3 className="text-[15px] font-semibold tracking-tight text-ink mt-1">
                  Your configuration
                </h3>
              </div>
            </div>

            <div className="space-y-2.5">
              <div className="flex items-center justify-between p-3.5 rounded-md border border-hairline bg-canvas">
                <div className="flex items-center gap-3 min-w-0">
                  <div className="flex items-center justify-center size-8 rounded-full bg-canvas-soft text-ink">
                    <Mic className="h-4 w-4" />
                  </div>
                  <span className="text-xs font-semibold text-ink">
                    Microphone
                  </span>
                </div>
                <span className="text-[11px] font-medium uppercase tracking-[0.08em] inline-flex items-center gap-1.5 text-primary">
                  <Check className="h-3.5 w-3.5" />
                  Ready
                </span>
              </div>

              <div className="flex items-center justify-between p-3.5 rounded-md border border-hairline bg-canvas">
                <div className="flex items-center gap-3 min-w-0">
                  <div className="flex items-center justify-center size-8 rounded-full bg-canvas-soft text-ink">
                    <Cpu className="h-4 w-4" />
                  </div>
                  <span className="text-xs font-semibold text-ink">
                    Model
                  </span>
                </div>
                <span className="text-xs font-semibold text-ink truncate ml-3">
                  {selectedModel?.name || "Base"}
                </span>
              </div>

              <div className="flex items-center justify-between p-3.5 rounded-md border border-hairline bg-canvas">
                <div className="flex items-center gap-3 min-w-0">
                  <div className="flex items-center justify-center size-8 rounded-full bg-canvas-soft text-ink">
                    <Keyboard className="h-4 w-4" />
                  </div>
                  <span className="text-xs font-semibold text-ink">
                    Hotkey
                  </span>
                </div>
                <code className="text-[11px] font-medium uppercase tracking-[0.08em] text-ink font-mono px-2.5 py-1 rounded-md bg-canvas-soft">
                  {currentHotkey}
                </code>
              </div>
            </div>
          </section>

          {/* HOW TO USE */}
          <section className="card-feature-cream">
            <div className="flex items-center gap-3 mb-5">
              <div className="flex items-center justify-center size-8 rounded-full bg-primary/12 text-primary">
                <Sparkles className="h-4 w-4" />
              </div>
              <div className="min-w-0 flex-1">
                <p className="text-[11px] font-medium tracking-[0.1em] uppercase leading-none text-ink-mid">Get started</p>
                <h3 className="text-[15px] font-semibold tracking-tight text-ink mt-1">
                  How to use
                </h3>
              </div>
            </div>

            <ol className="space-y-3">
              <li className="flex items-start gap-3">
                <span
                  className="flex h-6 w-6 items-center justify-center rounded-full shrink-0 bg-primary/10 text-primary"
                >
                  <span className="text-[11px] font-medium uppercase tracking-[0.08em] tabular-nums">1</span>
                </span>
                <span className="text-sm text-ink pt-0.5">Click where you want to type.</span>
              </li>
              <li className="flex items-start gap-3">
                <span
                  className="flex h-6 w-6 items-center justify-center rounded-full shrink-0 bg-primary/10 text-primary"
                >
                  <span className="text-[11px] font-medium uppercase tracking-[0.08em] tabular-nums">2</span>
                </span>
                <span className="text-sm text-ink pt-0.5">
                  {settings.hotkeyMode === "push-to-talk"
                    ? `Hold ${currentHotkey} and speak.`
                    : `Press ${currentHotkey} to start.`}
                </span>
              </li>
              <li className="flex items-start gap-3">
                <span
                  className="flex h-6 w-6 items-center justify-center rounded-full shrink-0 bg-primary/10 text-primary"
                >
                  <span className="text-[11px] font-medium uppercase tracking-[0.08em] tabular-nums">3</span>
                </span>
                <span className="text-sm text-ink pt-0.5">
                  {settings.hotkeyMode === "push-to-talk"
                    ? "Release to insert the text."
                    : `Press ${currentHotkey} again to stop.`}
                </span>
              </li>
            </ol>
          </section>

          {/* FOOTER LOGO CARD */}
          <div className="flex items-center justify-center gap-2.5 pt-2 pb-1">
            <Logo size="sm" />
            <span className="text-xs font-semibold text-ink">
              Whisprtypr
            </span>
          </div>
        </div>
      </div>

      {/* STICKY FOOTER */}
      <div className="shrink-0 border-t border-hairline bg-canvas-soft">
        <div className="max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-4 flex items-center justify-center">
          <button
            onClick={onFinish}
            className="paper-button-primary cursor-pointer w-full sm:w-auto sm:min-w-[240px]"
          >
            Start using Whisprtypr
          </button>
        </div>
      </div>
    </div>
  );
}
