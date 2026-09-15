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
} from "@/components/icons";
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
    // Prefer the unmasked display_key (the real license key). The
    // license_key field is already masked by the backend, so copying it
    // would paste "1234****9012" instead of the real key.
    const keyToCopy = license?.display_key || license?.license_key;
    if (keyToCopy) {
      try {
        await navigator.clipboard.writeText(keyToCopy);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
      } catch (err) {
        console.error("Copy failed:", err);
      }
    }
  };

  const hasAccess = license ? isLicenseActive(license.status) : false;
  const isActiveLicense = license?.status === "active";
  const isTrial = license?.status === "trial";
  const statusMessage = license ? getLicenseStatusMessage(license.status) : null;

  return (
    <div className="flex h-full flex-col overflow-hidden bg-canvas">
      {/* ─── HEADER ─── */}
      <div className="shrink-0 border-b border-hairline">
        <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-3 sm:py-4">
          <p className="eyebrow-uppercase text-ink-mid">License</p>
          <h1
            className="display-sm text-ink mt-1"
          >
            {isActiveLicense ? (
              <>Your license is <span className="text-primary">active</span>.</>
            ) : isTrial ? (
              <>Your trial is <span className="text-primary">active</span>.</>
            ) : (
              <>Unlock everything WhisperTypr can do.</>
            )}
          </h1>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto">
        <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-4 xl:py-5 space-y-4 xl:space-y-5">
          {error && (
            <div className="p-3 rounded-md border border-destructive/30 bg-destructive/5 flex items-center gap-2.5 text-destructive">
              <AlertCircle className="h-4 w-4 shrink-0" />
              <span className="body-sm">{error}</span>
            </div>
          )}

          {isLoading ? (
            <div className="flex items-center justify-center py-12">
              <div className="flex flex-col items-center gap-3">
                <Loader2 className="h-8 w-8 animate-spin text-body-muted" />
                <p className="body-sm text-body-muted">Loading license info...</p>
              </div>
            </div>
          ) : (
            <>
              {/* ─── HERO STATUS BAND — Dark coffee-ink ─── */}
              <section className="hero-band-dark">
                <div className="grid grid-cols-1 @xl:grid-cols-[1.4fr_1fr] gap-4 @xl:gap-6 p-4 sm:p-5 @xl:p-6 items-start @xl:items-center">
                  <div className="min-w-0">
                    <p className="eyebrow-uppercase text-primary mb-2">
                      <span className="inline-flex items-center gap-2">
                         <Circle className={cn(
                           "h-1.5 w-1.5",
                           hasAccess ? "fill-primary text-primary" : "fill-on-dark-muted text-on-dark-muted"
                         )} />
                         {isActiveLicense ? "Active" : isTrial ? "Trial" : "Not activated"}
                      </span>
                    </p>
                    <h2 className="display-md text-on-dark">
                        {isActiveLicense ? (
                          <>You're all set.</>
                        ) : isTrial ? (
                          <>Trial in progress.</>
                        ) : (
                          <>
                            WhisperTypr is <span className="text-primary">unlocked</span> with a license.
                          </>
                        )}
                      </h2>
                      <p className="body-sm text-on-dark-soft mt-2 max-w-xl">
                        {isActiveLicense
                          ? statusMessage || "Your license is valid and active on this device."
                          : isTrial
                            ? statusMessage || "Your trial is active on this device."
                            : "Activate a license to unlock unlimited transcriptions, all models, and premium features."}
                      </p>
                  </div>

                  <div className="product-ui-card-dark w-full">
                    <div className="flex items-center gap-2.5 mb-3">
                       <div className={cn(
                         "shrink-0",
                         hasAccess ? "icon-plate-dark border-primary" : "icon-plate-dark"
                       )} style={hasAccess ? { background: 'rgba(255,79,0,0.15)', borderColor: 'rgba(255,79,0,0.4)' } : {}}>
                         {hasAccess ? (
                           <ShieldCheck className="h-3.5 w-3.5 text-primary" />
                         ) : (
                           <ShieldX className="h-3.5 w-3.5 text-on-dark-soft" />
                         )}
                       </div>
                       <div className="min-w-0">
                         <p className="caption-strong text-on-dark">Status</p>
                         <p className="caption text-on-dark-soft mt-0.5">
                           {isActiveLicense ? "License verified" : isTrial ? "Trial active" : "No license on this device"}
                         </p>
                       </div>
                    </div>

                    {license?.license_key && (
                      <>
                        <p className="caption-strong text-on-dark-muted mb-1.5">License key</p>
                        <div className="flex items-center gap-2">
                          <code
                            className="flex-1 px-2.5 py-2 rounded-md text-xs font-mono text-on-dark truncate"
                            style={{ background: '#14100e', border: '1px solid #36342e' }}
                          >
                            {license.display_key
                              ? maskLicenseKey(license.display_key)
                              : license.license_key}
                          </code>
                          <button
                            onClick={handleCopyLicense}
                            className="shrink-0 flex h-8 w-8 items-center justify-center rounded-md border border-hairline-soft text-on-dark-soft hover:text-on-dark hover:border-on-dark-soft transition-colors"
                            style={{ borderColor: '#36342e' }}
                            aria-label="Copy license key"
                          >
                            {copied ? <Check className="h-3.5 w-3.5 text-primary" /> : <Copy className="h-3.5 w-3.5" />}
                          </button>
                        </div>
                      </>
                    )}
                  </div>
                </div>
              </section>

              {/* ─── ACTIVATION FORM — Cream surface ─── */}
              {!hasAccess && (
                <section className="card-feature-cream">
                  <div className="flex items-center gap-2.5 mb-4">
                    <div className="icon-plate">
                      <Key className="h-3.5 w-3.5" />
                    </div>
                    <div className="min-w-0">
                      <p className="eyebrow-uppercase text-ink-mid">Activate</p>
                      <h3
                        className="title-md text-ink mt-0.5"
                      >
                        Enter your license key
                      </h3>
                    </div>
                  </div>

                  <div className="space-y-3">
                    <div>
                      <label className="caption-strong text-ink-mid mb-1.5 block">License key</label>
                      <Input
                        value={licenseKey}
                        onChange={(e) => setLicenseKey(e.target.value)}
                        placeholder="XXXXX-XXXXX-XXXXX-XXXXX"
                        className="paper-input h-9 text-sm font-m"
                        style={{ borderRadius: '8px' }}
                        onKeyDown={(e) => {
                          if (e.key === "Enter") handleActivate();
                        }}
                      />
                    </div>

                    <div className="flex flex-wrap items-center gap-2.5">
                      <button
                        onClick={handleActivate}
                        disabled={isActivating || !licenseKey.trim()}
                        className="paper-button-primary cursor-pointer disabled:cursor-not-allowed disabled:opacity-50"
                      >
                        {isActivating ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : null}
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
              {isActiveLicense && (
                <section className="paper-card" style={{ borderColor: 'rgba(207,32,47,0.25)' }}>
                  <div className="flex items-center gap-2.5 mb-3">
                    <div
                      className="flex h-9 w-9 items-center justify-center rounded-full"
                      style={{ background: 'rgba(207,32,47,0.1)', color: '#cf202f' }}
                    >
                      <Trash2 className="h-3.5 w-3.5" />
                    </div>
                    <div className="min-w-0">
                      <h3 className="title-md text-ink">
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
                        {isDeactivating ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : null}
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
                          className="bg-destructive text-white hover:bg-destructive/90"
                        >
                          Deactivate
                        </AlertDialogAction>
                      </AlertDialogFooter>
                    </AlertDialogContent>
                  </AlertDialog>
                </section>
              )}

              {/* ─── PURCHASE CTA — Orange-accented cream band ─── */}
              {!hasAccess && (
                <section className="card-feature-cream relative overflow-hidden">
                  {/* Decorative orange glow */}
                  <div
                    aria-hidden
                    className="absolute -top-12 -right-12 h-48 w-48 pointer-events-none"
                    style={{
                      background: 'radial-gradient(circle, rgba(255,79,0,0.18), transparent 70%)',
                    }}
                  />
                  <div className="relative grid grid-cols-1 @xl:grid-cols-[1.4fr_1fr] gap-4 @xl:gap-5 items-center">
                    <div className="min-w-0">
                      <p className="eyebrow-uppercase text-primary mb-2">
                        <span className="inline-flex items-center gap-2">
                          <Sparkles className="h-3 w-3" />
                          Get a license
                        </span>
                      </p>
                      <h3
                        className="display-xs text-ink"
                      >
                        Unlock unlimited transcriptions and premium models.
                      </h3>
                      <p className="body-sm text-body-muted mt-2 max-w-md">
                        Every license supports ongoing development and keeps your voice data local.
                      </p>
                    </div>
                    <div className="flex flex-wrap gap-2 @xl:justify-end">
                      <button
                        onClick={() => openUrl("https://github.com/The-WhisprTypr/whisprtypr")}
                        className="paper-button-primary cursor-pointer"
                      >
                        Learn more
                      </button>
                      <button
                        onClick={() => openUrl("https://github.com/The-WhisprTypr/whisprtypr/releases/latest")}
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