import {
    AlertCircle,
    Check,
    Info,
    Loader2,
    Mic,
    MicOff,
} from "@/components/icons";
import { cn } from "@/lib/utils";
import { useState } from "react";

interface MicrophoneStepProps {
  onNext: () => void;
  onBack: () => void;
}

const STEPS_TOTAL = 4;
const STEP_INDEX = 2;

type PermissionStatus = "pending" | "checking" | "granted" | "denied";

export function MicrophoneStep({ onNext, onBack }: MicrophoneStepProps) {
  const [status, setStatus] = useState<PermissionStatus>("pending");
  const [isTestingMic, setIsTestingMic] = useState(false);
  const [micLevel, setMicLevel] = useState(0);
  const [testCompleted, setTestCompleted] = useState(false);

  const requestPermission = async () => {
    setStatus("checking");
    setTestCompleted(false);

    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      const audioContext = new AudioContext();
      const analyser = audioContext.createAnalyser();
      const source = audioContext.createMediaStreamSource(stream);
      source.connect(analyser);

      analyser.fftSize = 256;
      const dataArray = new Uint8Array(analyser.frequencyBinCount);

      setStatus("granted");
      setIsTestingMic(true);

      let animationId: number;
      const updateLevel = () => {
        analyser.getByteFrequencyData(dataArray);
        const average = dataArray.reduce((a, b) => a + b) / dataArray.length;
        setMicLevel(Math.min(100, average * 2));
        animationId = requestAnimationFrame(updateLevel);
      };
      updateLevel();

      setTimeout(() => {
        cancelAnimationFrame(animationId);
        stream.getTracks().forEach((track) => track.stop());
        audioContext.close();
        setIsTestingMic(false);
        setMicLevel(0);
        setTestCompleted(true);
      }, 3000);
    } catch (error) {
      console.error("Microphone permission error:", error);
      setStatus("denied");
    }
  };

  const skipTest = () => {
    setStatus("granted");
    setTestCompleted(true);
  };

  return (
    <div className="flex h-full flex-col overflow-hidden bg-canvas">
      <div className="flex-1 overflow-y-auto">
        <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-6 xl:py-10 space-y-6">
          {/* HERO — Dark band */}
          <section className="hero-band-dark">
            <div className="flex flex-col items-center text-center gap-4 p-8 sm:p-10">
              <p className="text-[11px] font-medium tracking-[0.1em] uppercase leading-none text-primary">
                <span className="inline-flex items-center gap-2">
                  <span className="h-1 w-1 rounded-full bg-primary" />
                  Step {STEP_INDEX} of {STEPS_TOTAL}
                </span>
              </p>
              <h2 className="text-[28px] font-medium leading-tight tracking-tight text-on-dark">
                Allow <span className="text-primary">microphone</span> access.
              </h2>
              <p className="text-sm text-on-dark-soft max-w-md leading-relaxed">
                Whisprtypr needs to hear you. We capture audio natively — no cloud calls.
              </p>
            </div>
          </section>

          {/* MIC TEST PANEL */}
          <section className="card-feature-cream">
            <div className="flex items-center gap-3 mb-6">
              <div className="flex items-center justify-center size-8 rounded-full bg-canvas-soft text-ink">
                <Mic className="h-4 w-4" />
              </div>
              <div className="min-w-0 flex-1">
                <p className="text-[11px] font-medium tracking-[0.1em] uppercase leading-none text-ink-mid">Test</p>
                <h3 className="text-[15px] font-semibold tracking-tight text-ink mt-1">
                  Microphone access
                </h3>
              </div>
            </div>

            {/* Mic icon with pulse ring */}
            <div className="flex flex-col items-center justify-center py-6">
              <div className="relative">
                <div
                  className={cn(
                    "h-28 w-28 rounded-full flex items-center justify-center transition-all",
                    status === "pending" && "bg-canvas-soft text-ink",
                    status === "checking" && "bg-canvas-soft text-ink",
                    status === "granted" && "bg-primary/10 text-primary",
                    status === "denied" && "bg-destructive/10 text-destructive",
                  )}
                  style={{
                    boxShadow: isTestingMic
                      ? `0 0 0 ${micLevel / 3}px rgba(255, 79, 0, 0.25)`
                      : "none",
                  }}
                >
                  {status === "checking" ? (
                    <Loader2 className="h-12 w-12 animate-spin" />
                  ) : status === "granted" ? (
                    <Check className="h-12 w-12" />
                  ) : status === "denied" ? (
                    <MicOff className="h-12 w-12" />
                  ) : (
                    <Mic className="h-12 w-12" />
                  )}
                </div>
              </div>

              {isTestingMic && (
                <p className="text-xs text-body-muted mt-5 animate-pulse">
                  Speak to test your microphone...
                </p>
              )}

              {status === "granted" && !isTestingMic && testCompleted && (
                <p className="text-xs font-semibold text-primary mt-5 flex items-center gap-1.5">
                  <Check className="h-4 w-4" />
                  Microphone ready
                </p>
              )}
            </div>

            {/* Info / Error messages */}
            {status === "denied" && (
              <div
                className="p-4 rounded-md border flex items-start gap-3 border-[#cf202f]/30 bg-[#cf202f]/5"
              >
                <AlertCircle className="h-5 w-5 text-destructive shrink-0 mt-0.5" />
                <div className="space-y-1 min-w-0">
                  <p className="text-xs font-semibold text-ink">Permission denied</p>
                  <p className="text-xs text-body-muted">
                    The browser test failed, but don't worry — Whisprtypr uses native audio capture which may still work.
                  </p>
                </div>
              </div>
            )}

            {status === "pending" && (
              <div className="p-4 rounded-md border border-hairline bg-canvas-soft flex items-start gap-3">
                <Info className="h-5 w-5 text-body-muted shrink-0 mt-0.5" />
                <div className="space-y-1 min-w-0">
                  <p className="text-xs font-semibold text-ink">Native audio capture</p>
                  <p className="text-xs text-body-muted">
                    Whisprtypr uses native audio capture. Test your mic here, or skip if the browser blocks it.
                  </p>
                </div>
              </div>
            )}

            {/* Action buttons */}
            <div className="flex flex-wrap items-center gap-3 pt-2">
              {status === "pending" || status === "checking" ? (
                <>
                  <button
                    onClick={requestPermission}
                    disabled={status === "checking"}
                    className="paper-button-primary cursor-pointer disabled:cursor-not-allowed disabled:opacity-50"
                  >
                    {status === "checking" ? (
                      <Loader2 className="h-4 w-4 animate-spin" />
                    ) : null}
                    {status === "checking" ? "Requesting..." : "Test microphone"}
                  </button>
                  <button
                    onClick={skipTest}
                    className="paper-button-outline cursor-pointer"
                  >
                    Skip
                  </button>
                </>
              ) : null}

              {status === "denied" ? (
                <>
                  <button
                    onClick={requestPermission}
                    className="paper-button-primary cursor-pointer"
                  >
                    Try again
                  </button>
                  <button
                    onClick={skipTest}
                    className="paper-button-outline cursor-pointer"
                  >
                    Skip test
                  </button>
                </>
              ) : null}
            </div>
          </section>
        </div>
      </div>

      {/* STICKY FOOTER */}
      <div className="shrink-0 border-t border-hairline bg-canvas-soft">
        <div className="max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-4 flex items-center justify-between gap-3 flex-wrap">
          <button
            onClick={onBack}
            className="paper-button-outline cursor-pointer"
          >
            Back
          </button>
          {status === "granted" && testCompleted ? (
            <button
              onClick={onNext}
              className="paper-button-primary cursor-pointer"
            >
              Continue
            </button>
          ) : (
            <p className="text-[11px] text-body-muted">Test or skip to continue.</p>
          )}
        </div>
      </div>
    </div>
  );
}