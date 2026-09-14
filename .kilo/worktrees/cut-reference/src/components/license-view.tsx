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
import { Input } from "@/components/ui/input";
import { useToast } from "@/hooks/use-toast";
import { getErrorMessage } from "@/lib/errors";
import {
  activateLicense,
  deactivateLicense,
  getLicense,
  getLicenseStatusMessage,
  isLicenseActive,
  maskLicenseKey,
  type LicenseData,
} from "@/lib/license-api";
import { cn, openUrl } from "@/lib/utils";
import { reportError } from "@/lib/voice-api";
import {
  AlertCircle,
  Check,
  Circle,
  Copy,
  Key,
  Loader2,
  ShieldCheck,
  ShieldX,
  Sparkles,
  Trash2,
} from "lucide-react";
import { useEffect, useState } from "react";

interface LicenseViewProps {
  onClose: () => void;
  onLicenseChange?: (isValid: boolean) => void;
}

export function LicenseView({ onClose: _onClose, onLicenseChange }: LicenseViewProps) {
  const { success: toastSuccess } = useToast();
  const [license, setLicense] = useState<LicenseData | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [licenseKey, setLicenseKey] = useState("");
  const [isActivating, setIsActivating] = useState(false);
  const [isDeactivating, setIsDeactivating] = useState(false);
  const [copied, setCopied] = useState(false);

  const loadLicense = async () => {
    try {
      setIsLoading(true);
      setError(null);
      const data = await getLicense();
      setLicense(data);
    } catch (err) {
      const message = getErrorMessage(err);
      console.error("Failed to load license:", err);
      setError(message);
      await reportError("license", message, "error", {
        userAction: "Load license info",
      }).catch(console.error);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    loadLicense();
  }, []);

  const handleActivate = async () => {
    if (!licenseKey.trim()) {
      setError("Please enter a license key");
      return;
    }

    try {
      setIsActivating(true);
      setError(null);
      const data = await activateLicense(licenseKey.trim());
      setLicense(data);
      setLicenseKey("");
      toastSuccess("License activated", "Your license has been activated successfully");
      onLicenseChange?.(data.is_activated && data.status === "active");
    } catch (err) {
      const message = getErrorMessage(err);
      console.error("Activation failed:", err);
      setError(message);
      await reportError("license", message, "error", {
        userAction: "Activate license",
      }).catch(console.error);
    } finally {
      setIsActivating(false);
    }
  };

  const handleDeactivate = async () => {
    try {
      setIsDeactivating(true);
      setError(null);
      await deactivateLicense();
      toastSuccess("License deactivated", "Your license has been deactivated");
      await loadLicense();
      onLicenseChange?.(false);
    } catch (err) {
      const message = getErrorMessage(err);
      console.error("Deactivation failed:", err);
      setError(message);
      await reportError("license", message, "error", {
        userAction: "Deactivate license",
      }).catch(console.error);
    } finally {
      setIsDeactivating(false);
    }
  };

  const handleCopyLicense = async () => {
    if (license?.license_key) {
      try {
        await navigator.clipboard.writeText(license.license_key);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
      } catch (err) {
        console.error("Copy failed:", err);
      }
    }
  };

  const active = license ? isLicenseActive(license.status) : false;
  const statusMessage = license ? getLicenseStatusMessage(license.status) : null;

  return (
    <div className="flex h-full flex-col overflow-hidden bg-canvas">
      {/* ─── HEADER ─── */}
      <div className="shrink-0 border-b border-hairline">
        <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-4 sm:py-5">
          <p className="eyebrow-uppercase text-ink-mid">License</p>
          <h1
            className="font-display font-medium text-ink mt-2 text-[clamp(1.25rem,2.2vw,1.75rem)]"
            style={{ lineHeight: 1.1, letterSpacing: '-0.02em' }}
          >
            {active ? (
              <>Your license is <span className="text-primary">active</span>.</>
            ) : (
              <>Unlock everything WhisprTypr can do.</>
            )}
          </h1>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto">
        <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-6 xl:py-10 space-y-6 xl:space-y-8">
          {error && (
            <div className="p-3.5 rounded-md border border-destructive/30 bg-destructive/5 flex items-center gap-2.5 text-destructive">
              <AlertCircle className="h-4 w-4 shrink-0" />
              <span className="body-sm">{error}</span>
            </div>
          )}

          {isLoading ? (
            <div className="flex items-center justify-center py-16">
              <div className="flex flex-col items-center gap-3">
                <Loader2 className="h-8 w-8 animate-spin text-body-muted" />
                <p className="body-sm text-body-muted">Loading license info...</p>
              </div>
            </div>
          ) : (
            <>
              {/* ─── HERO STATUS BAND — Dark coffee-ink ─── */}
              <section className="hero-band-dark">
                <div className="grid grid-cols-1 @xl:grid-cols-[1.4fr_1fr] gap-6 @xl:gap-10 p-5 sm:p-7 @xl:p-10 items-start @xl:items-center">
                  <div className="min-w-0">
                    <p className="eyebrow-uppercase text-primary mb-3">
                      <span className="inline-flex items-center gap-2">
                        <Circle className={cn(
                          "h-1.5 w-1.5",
                          active ? "fill-primary text-primary" : "fill-on-dark-muted text-on-dark-muted"
                        )} />
                        {active ? "Active" : "Not activated"}
                      </span>
                    </p>
                    <h2
                      className="font-display font-medium text-on-dark text-[clamp(1.375rem,2.6vw,2.375rem)]"
                      style={{ lineHeight: 1.05, letterSpacing: '-0.02em' }}
                    >
                      {active ? (
                        <>You're all set.</>
                      ) : (
                        <>
                          WhisprTypr is<br />
                          <span className="text-primary">unlocked</span> with a license.
                        </>
                      )}
                    </h2>
                    <p className="body-md text-on-dark-soft mt-4 max-w-xl">
                      {active
                        ? statusMessage || "Your license is valid and active on this device."
                        : "Activate a license to unlock unlimited transcriptions, all models, and premium features."}
                    </p>
                  </div>

                  <div className="product-ui-card-dark w-full">
                    <div className="flex items-center gap-3 mb-4">
                      <div className={cn(
                        "shrink-0",
                        active ? "icon-plate-dark border-primary" : "icon-plate-dark"
                      )} style={active ? { background: 'rgba(255,79,0,0.15)', borderColor: 'rgba(255,79,0,0.4)' } : {}}>
                        {active ? (
                          <ShieldCheck className="h-4 w-4 text-primary" />
                        ) : (
                          <ShieldX className="h-4 w-4 text-on-dark-soft" />
                        )}
                      </div>
                      <div className="min-w-0">
                        <p className="caption-strong text-on-dark">Status</p>
                        <p className="caption text-on-dark-soft mt-1">
                          {active ? "License verified" : "No license on this device"}
                        </p>
                      </div>
                    </div>

                    {license?.license_key && (
                      <>
                        <p className="caption-strong text-on-dark-muted mb-2">License key</p>
                        <div className="flex items-center gap-2">
                          <code
                            className="flex-1 px-3 py-2.5 rounded-md text-xs font-mono text-on-dark truncate"
                            style={{ background: '#14100e', border: '1px solid #36342e' }}
                          >
                            {maskLicenseKey(license.license_key)}
                          </code>
                          <button
                            onClick={handleCopyLicense}
                            className="shrink-0 flex h-9 w-9 items-center justify-center rounded-md border border-hairline-soft text-on-dark-soft hover:text-on-dark hover:border-on-dark-soft transition-colors"
                            style={{ borderColor: '#36342e' }}
                            aria-label="Copy license key"
                          >
                            {copied ? <Check className="h-4 w-4 text-primary" /> : <Copy className="h-4 w-4" />}
                          </button>
                        </div>
                      </>
                    )}
                  </div>
                </div>
              </section>

              {/* ─── ACTIVATION FORM — Cream surface ─── */}
              {!active && (
                <section className="card-feature-cream">
                  <div className="flex items-center gap-3 mb-6">
                    <div className="icon-plate">
                      <Key className="h-4 w-4" />
                    </div>
                    <div className="min-w-0">
                      <p className="eyebrow-uppercase text-ink-mid">Activate</p>
                      <h3
                        className="font-display font-medium text-ink text-[clamp(1rem,1.6vw,1.25rem)] mt-1"
                        style={{ letterSpacing: '-0.015em' }}
                      >
                        Enter your license key
                      </h3>
                    </div>
                  </div>

                  <div className="space-y-4">
                    <div>
                      <label className="caption-strong text-ink-mid mb-2 block">License key</label>
                      <Input
                        value={licenseKey}
                        onChange={(e) => setLicenseKey(e.target.value)}
                        placeholder="XXXXX-XXXXX-XXXXX-XXXXX"
                        className="paper-input h-11 text-sm font-mono"
                        style={{ borderRadius: '8px' }}
                        onKeyDown={(e) => {
                          if (e.key === "Enter") handleActivate();
                        }}
                      />
                    </div>

                    <div className="flex flex-wrap items-center gap-3">
                      <button
                        onClick={handleActivate}
                        disabled={isActivating || !licenseKey.trim()}
                        className="paper-button-primary cursor-pointer disabled:cursor-not-allowed disabled:opacity-50"
                      >
                        {isActivating ? <Loader2 className="h-4 w-4 animate-spin" /> : null}
                        {isActivating ? "Activating..." : "Activate license"}
                      </button>
                      <p className="caption text-body-muted">
                        Press Enter to activate
                      </p>
                    </div>
                  </div>
                </section>
              )}

              {/* ─── DEACTIVATION — Cream surface with destructive accent ─── */}
              {active && (
                <section className="paper-card" style={{ borderColor: 'rgba(207,32,47,0.25)' }}>
                  <div className="flex items-center gap-3 mb-5">
                    <div
                      className="flex h-10 w-10 items-center justify-center rounded-full"
                      style={{ background: 'rgba(207,32,47,0.1)', color: '#cf202f' }}
                    >
                      <Trash2 className="h-4 w-4" />
                    </div>
                    <div className="min-w-0">
                      <h3 className="font-display font-medium text-ink text-[clamp(1rem,1.6vw,1.25rem)]" style={{ letterSpacing: '-0.015em' }}>
                        Deactivate on this device
                      </h3>
                      <p className="body-sm text-body-muted mt-1">
                        Removes the license from this machine. You can reactivate anytime.
                      </p>
                    </div>
                  </div>
                  <AlertDialog>
                    <AlertDialogTrigger asChild>
                      <button
                        className="paper-button-outline cursor-pointer disabled:cursor-not-allowed disabled:opacity-50"
                        style={{ borderColor: '#cf202f', color: '#cf202f' }}
                        disabled={isDeactivating}
                      >
                        {isDeactivating ? <Loader2 className="h-4 w-4 animate-spin" /> : null}
                        {isDeactivating ? "Deactivating..." : "Deactivate license"}
                      </button>
                    </AlertDialogTrigger>
                    <AlertDialogContent>
                      <AlertDialogHeader>
                        <AlertDialogTitle>Deactivate license?</AlertDialogTitle>
                        <AlertDialogDescription>
                          This will remove the license from this device. You can reactivate it later.
                        </AlertDialogDescription>
                      </AlertDialogHeader>
                      <AlertDialogFooter>
                        <AlertDialogCancel className="paper-button-secondary">Cancel</AlertDialogCancel>
                        <AlertDialogAction
                          onClick={handleDeactivate}
                          className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
                        >
                          Deactivate
                        </AlertDialogAction>
                      </AlertDialogFooter>
                    </AlertDialogContent>
                  </AlertDialog>
                </section>
              )}

              {/* ─── PURCHASE CTA — Orange-accented cream band ─── */}
              {!active && (
                <section className="card-feature-cream relative overflow-hidden">
                  {/* Decorative orange glow */}
                  <div
                    aria-hidden
                    className="absolute -top-12 -right-12 h-48 w-48 pointer-events-none"
                    style={{
                      background: 'radial-gradient(circle, rgba(255,79,0,0.18), transparent 70%)',
                    }}
                  />
                  <div className="relative grid grid-cols-1 @xl:grid-cols-[1.4fr_1fr] gap-6 @xl:gap-8 items-center">
                    <div className="min-w-0">
                      <p className="eyebrow-uppercase text-primary mb-3">
                        <span className="inline-flex items-center gap-2">
                          <Sparkles className="h-3.5 w-3.5" />
                          Get a license
                        </span>
                      </p>
                      <h3
                        className="font-display font-medium text-ink text-[clamp(1.125rem,1.8vw,1.5rem)]"
                        style={{ letterSpacing: '-0.02em' }}
                      >
                        Unlock unlimited transcriptions and premium models.
                      </h3>
                      <p className="body-sm text-body-muted mt-3 max-w-md">
                        Every license supports ongoing development and keeps your voice data local.
                      </p>
                    </div>
                    <div className="flex flex-wrap gap-3 @xl:justify-end">
                      <button
                        onClick={() => openUrl("https://github.com/johuniq/whisprtypr")}
                        className="paper-button-primary cursor-pointer"
                      >
                        Learn more
                      </button>
                      <button
                        onClick={() => openUrl("https://github.com/johuniq/whisprtypr/releases/latest")}
                        className="paper-button-outline cursor-pointer"
                      >
                        Releases
                      </button>
                    </div>
                  </div>
                </section>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
}