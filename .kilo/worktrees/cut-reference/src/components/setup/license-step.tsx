import { useToast } from "@/hooks/use-toast";
import {
  activateLicense,
  getLicense,
  isLicenseActive,
  startTrial,
  type LicenseData,
} from "@/lib/license-api";
import { cn, openUrl } from "@/lib/utils";
import {
  AlertCircle,
  Check,
  Clock,
  Key,
  Loader2,
  ShieldCheck,
  Sparkles,
} from "lucide-react";
import { useEffect, useState } from "react";

interface LicenseStepProps {
  onNext: () => void;
  onBack: () => void;
}

const STEPS_TOTAL = 4;
const STEP_INDEX = 1;

export function LicenseStep({ onNext, onBack }: LicenseStepProps) {
  const [license, setLicense] = useState<LicenseData | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isActivating, setIsActivating] = useState(false);
  const [isStartingTrial, setIsStartingTrial] = useState(false);
  const [licenseKey, setLicenseKey] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [showActivationForm, setShowActivationForm] = useState(false);

  useEffect(() => {
    loadLicense();
  }, []);

  const loadLicense = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const data = await getLicense();
      setLicense(data);
    } catch (err) {
      const msg = err instanceof Error ? err.message : "Failed to load license";
      const { error: toastError } = useToast();
      toastError("Failed to load license", msg);
    } finally {
      setIsLoading(false);
    }
  };

  const handleActivate = async () => {
    if (!licenseKey.trim()) {
      setError("Please enter a license key");
      return;
    }

    setIsActivating(true);
    setError(null);
    setSuccess(null);

    const { success: toastSuccess, error: toastError } = useToast();
    try {
      const data = await activateLicense(licenseKey.trim());
      setLicense(data);
      setLicenseKey("");
      setSuccess("License activated successfully!");
      toastSuccess("License activated", "License activated successfully");
      setTimeout(() => onNext(), 1500);
    } catch (err) {
      const msg =
        err instanceof Error ? err.message : "Failed to activate license";
      toastError("Activation failed", msg);
      setError(msg);
    } finally {
      setIsActivating(false);
    }
  };

  const handleStartTrial = async () => {
    const { success: toastSuccess, error: toastError } = useToast();
    setIsStartingTrial(true);
    setError(null);
    setSuccess(null);

    try {
      const data = await startTrial();
      setLicense(data);
      setSuccess("7-day trial started!");
      toastSuccess("Trial started", "Your 7-day trial has started");
      setTimeout(() => onNext(), 1500);
    } catch (err) {
      const msg = err instanceof Error ? err.message : "Failed to start trial";
      toastError("Trial failed", msg);
      setError(msg);
    } finally {
      setIsStartingTrial(false);
    }
  };

  const isActive = license ? isLicenseActive(license.status) : false;
  const isTrial = license?.status === "trial";
  const canProceed = isActive || isTrial;

  if (isLoading) {
    return (
      <div className="flex h-full flex-col items-center justify-center bg-canvas">
        <Loader2 className="h-7 w-7 animate-spin text-body-muted" />
        <p className="body-sm text-body-muted mt-3">Checking license...</p>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col overflow-hidden bg-canvas">
      <div className="flex-1 overflow-y-auto">
        <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-6 xl:py-10 space-y-6">
          {/* HERO — Dark band */}
          <section className="hero-band-dark">
            <div className="flex flex-col items-center text-center gap-4 p-8 sm:p-10">
              <div className="icon-plate-dark">
                <Key className="h-4 w-4 text-primary" />
              </div>
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
                {showActivationForm ? "Activate " : "Choose how to "}
                <span className="text-primary">get started</span>.
              </h2>
              <p className="body-md text-on-dark-soft max-w-md">
                {showActivationForm
                  ? "Enter your license key to activate."
                  : "Try free for 7 days or activate an existing license."}
              </p>
            </div>
          </section>

          {/* MESSAGES */}
          {error && (
            <div className="p-3.5 rounded-md border border-destructive/30 bg-destructive/5 flex items-start gap-2.5 text-destructive">
              <AlertCircle className="h-4 w-4 mt-0.5 shrink-0" />
              <span className="body-sm">{error}</span>
            </div>
          )}

          {success && (
            <div className="p-3.5 rounded-md border flex items-start gap-2.5" style={{ borderColor: "rgba(255,79,0,0.3)", background: "rgba(255,79,0,0.06)", color: "#ff4f00" }}>
              <Check className="h-4 w-4 mt-0.5 shrink-0" />
              <span className="body-sm">{success}</span>
            </div>
          )}

          {canProceed && !success && (
            <div
              className="p-5 rounded-md border flex items-center gap-4"
              style={{ borderColor: "rgba(255,79,0,0.3)", background: "rgba(255,79,0,0.06)" }}
            >
              <div
                className="flex h-10 w-10 items-center justify-center rounded-full shrink-0"
                style={{ background: "rgba(255,79,0,0.15)" }}
              >
                <ShieldCheck className="h-5 w-5 text-primary" />
              </div>
              <div className="min-w-0">
                <p className="font-display font-medium text-primary text-[clamp(1rem,1.2vw,1.125rem)]" style={{ letterSpacing: "-0.015em" }}>
                  {isTrial ? "Trial active" : "License active"}
                </p>
                <p className="caption text-body mt-0.5">
                  {isTrial
                    ? `${license?.trial_days_remaining ?? 7} days remaining`
                    : "Your license is activated"}
                </p>
              </div>
            </div>
          )}

          {/* OPTIONS */}
          {isActive ? null : !showActivationForm ? (
            <section className="card-feature-cream">
              <div className="flex items-center gap-3 mb-5">
                <div className="icon-plate">
                  <Sparkles className="h-4 w-4" />
                </div>
                <div className="min-w-0 flex-1">
                  <p className="eyebrow-uppercase text-ink-mid">Options</p>
                  <h3
                    className="font-display font-medium text-ink text-[clamp(1.0625rem,1.5vw,1.25rem)] mt-1"
                    style={{ letterSpacing: "-0.015em" }}
                  >
                    Pick a starting point
                  </h3>
                </div>
              </div>

              <div className="space-y-3">
                <button
                  onClick={handleStartTrial}
                  disabled={isStartingTrial}
                  className={cn(
                    "w-full text-left rounded-md border p-4 transition-colors cursor-pointer disabled:opacity-50",
                    "bg-canvas border-hairline hover:border-ink",
                  )}
                >
                  <div className="flex items-start gap-4">
                    <div className="icon-plate shrink-0">
                      {isStartingTrial ? (
                        <Loader2 className="h-4 w-4 animate-spin" />
                      ) : (
                        <Clock className="h-4 w-4" />
                      )}
                    </div>
                    <div className="min-w-0">
                      <h4
                        className="font-display font-medium text-ink text-[clamp(1rem,1.3vw,1.125rem)]"
                        style={{ letterSpacing: "-0.015em" }}
                      >
                        Start 7-day free trial
                      </h4>
                      <p className="body-sm text-body-muted mt-1.5">
                        Try all features free for 7 days. No credit card required.
                      </p>
                    </div>
                  </div>
                </button>

                <button
                  onClick={() => setShowActivationForm(true)}
                  className="w-full text-left rounded-md border border-hairline bg-canvas p-4 transition-colors hover:border-ink cursor-pointer"
                >
                  <div className="flex items-start gap-4">
                    <div className="icon-plate shrink-0">
                      <Key className="h-4 w-4" />
                    </div>
                    <div className="min-w-0">
                      <h4
                        className="font-display font-medium text-ink text-[clamp(1rem,1.3vw,1.125rem)]"
                        style={{ letterSpacing: "-0.015em" }}
                      >
                        I have a license key
                      </h4>
                      <p className="body-sm text-body-muted mt-1.5">
                        Already purchased? Enter your key to activate.
                      </p>
                    </div>
                  </div>
                </button>
              </div>

              <div className="pt-4 mt-4 border-t border-hairline-soft text-center">
                <button
                  onClick={() => openUrl("https://trywhisprtypr.johuniq.tech")}
                  className="paper-button cursor-pointer"
                >
                  Purchase a license
                </button>
              </div>
            </section>
          ) : (
            <section className="card-feature-cream">
              <div className="flex items-center gap-3 mb-5">
                <div className="icon-plate">
                  <Key className="h-4 w-4" />
                </div>
                <div className="min-w-0 flex-1">
                  <p className="eyebrow-uppercase text-ink-mid">Activate</p>
                  <h3
                    className="font-display font-medium text-ink text-[clamp(1.0625rem,1.5vw,1.25rem)] mt-1"
                    style={{ letterSpacing: "-0.015em" }}
                  >
                    Enter your license key
                  </h3>
                </div>
              </div>

              <p className="body-sm text-body-muted mb-4">
                Your license key was sent to your email after purchase.
              </p>

              <div className="space-y-4">
                <div className="space-y-2">
                  <label className="eyebrow-uppercase text-ink-mid block">
                    License key
                  </label>
                  <input
                    type="text"
                    placeholder="Paste your WhisprTypr license key"
                    value={licenseKey}
                    onChange={(e) => setLicenseKey(e.target.value)}
                    disabled={isActivating}
                    className="paper-input w-full h-11 px-4 font-mono text-sm disabled:opacity-50"
                    style={{ borderRadius: "8px" }}
                  />
                </div>

                <div className="flex flex-wrap items-center gap-3">
                  <button
                    onClick={handleActivate}
                    disabled={isActivating || !licenseKey.trim()}
                    className="paper-button-primary cursor-pointer disabled:cursor-not-allowed disabled:opacity-50"
                  >
                    {isActivating ? (
                      <Loader2 className="h-4 w-4 animate-spin" />
                    ) : null}
                    {isActivating ? "Activating..." : "Activate license"}
                  </button>
                  <button
                    onClick={() => setShowActivationForm(false)}
                    className="paper-button cursor-pointer"
                  >
                    Back to options
                  </button>
                </div>
              </div>
            </section>
          )}
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
          {canProceed ? (
            <button
              onClick={onNext}
              className="paper-button-primary cursor-pointer"
            >
              Continue
            </button>
          ) : (
            <p className="caption text-body-muted">Start a trial or enter a key to continue.</p>
          )}
        </div>
      </div>
    </div>
  );
}