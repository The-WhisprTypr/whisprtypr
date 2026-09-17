import {
    AlertCircle,
    Check,
    Clock,
    ExternalLink,
    Key,
    Loader2,
    ShieldAlert,
    ShieldCheck,
    Sparkles,
} from "@/components/icons";
import { Button } from "@/components/ui/button";
import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useToast } from "@/hooks/use-toast";
import { activateLicense } from "@/lib/license-api";
import { openUrl } from "@/lib/utils";
import { useState } from "react";

interface TrialExpiredViewProps {
  onLicenseActivated: () => void;
  reason?: "trial_expired" | "no_license";
}

export function TrialExpiredView({
  onLicenseActivated,
  reason = "trial_expired",
}: TrialExpiredViewProps) {
  const [isActivating, setIsActivating] = useState(false);
  const [licenseKey, setLicenseKey] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [showActivationForm, setShowActivationForm] = useState(false);
  const isTrialExpired = reason === "trial_expired";

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
      if (data.is_activated && data.status === "active") {
        setSuccess("License activated successfully!");
        toastSuccess("License activated", "License activated successfully");
        onLicenseActivated();
      } else {
        const msg = "License activation failed. Please check your key.";
        toastError("Activation failed", msg);
        setError(msg);
      }
    } catch (err) {
      const msg =
        err instanceof Error ? err.message : "Failed to activate license";
      toastError("Activation failed", msg);
      setError(msg);
    } finally {
      setIsActivating(false);
    }
  };

  return (
    <div className="flex flex-col items-center justify-center h-full px-6 py-8">
      <div className="flex flex-col items-center max-w-sm w-full">
        {/* Warning Icon */}
        <div className="w-16 h-16 mb-4 rounded-full bg-amber-500/10 flex items-center justify-center">
          <ShieldAlert className="h-8 w-8 text-amber-500" />
        </div>

        {/* Title */}
        <h2 className="text-xl font-semibold text-center">
          {isTrialExpired ? "Trial Period Ended" : "License Required"}
        </h2>
        <p className="text-sm text-muted-foreground text-center mt-2">
          {isTrialExpired
            ? "Your 7-day free trial has expired. Purchase a license to continue using Whisprtypr."
            : "Your license is no longer active on this device. Activate a license to continue using Whisprtypr."}
        </p>

        {/* Messages */}
        {error && (
          <div className="mt-4 w-full p-3 rounded-lg bg-destructive/10 border border-destructive/20 flex items-start gap-2">
            <AlertCircle className="h-4 w-4 text-destructive mt-0.5 shrink-0" />
            <p className="text-sm text-destructive">{error}</p>
          </div>
        )}

        {success && (
          <div className="mt-4 w-full p-3 rounded-lg bg-green-500/10 border border-green-500/20 flex items-start gap-2">
            <Check className="h-4 w-4 text-green-600 mt-0.5 shrink-0" />
            <p className="text-sm text-green-600">{success}</p>
          </div>
        )}

        {/* Main content */}
        <div className="mt-6 w-full space-y-3">
          {!showActivationForm ? (
            <>
              {/* Purchase CTA */}
              <Card className="border-primary/30 bg-primary/5">
                <CardContent className="p-4">
                  <div className="flex items-start gap-3">
                    <div className="w-10 h-10 rounded-full bg-primary/20 flex items-center justify-center shrink-0">
                      <Sparkles className="h-5 w-5 text-primary" />
                    </div>
                    <div className="flex-1">
                      <h3 className="font-medium">Get Whisprtypr Pro</h3>
                      <p className="text-xs text-muted-foreground mt-0.5">
                        Unlimited voice-to-cursor, lifetime updates
                      </p>
                      <Button
                        className="mt-3 w-full"
                        onClick={() => openUrl("https://whisprtypr.app")}
                      >
                        <ExternalLink className="mr-2 h-4 w-4" />
                        Purchase License
                      </Button>
                    </div>
                  </div>
                </CardContent>
              </Card>

              {/* Already have a license */}
              <Button
                variant="outline"
                className="w-full"
                onClick={() => setShowActivationForm(true)}
              >
                <Key className="mr-2 h-4 w-4" />I have a license key
              </Button>
            </>
          ) : (
            <>
              {/* License activation form */}
              <Card>
                <CardHeader className="pb-3">
                  <CardTitle className="text-base">Enter License Key</CardTitle>
                  <CardDescription className="text-xs">
                    Your license key was sent to your email after purchase
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="space-y-2">
                    <Label htmlFor="license-key" className="text-sm">
                      License Key
                    </Label>
                    <Input
                      id="license-key"
                      placeholder="Paste your Whisprtypr license key"
                      value={licenseKey}
                      onChange={(e) => setLicenseKey(e.target.value)}
                      className="font-mono text-sm"
                      disabled={isActivating}
                    />
                  </div>

                  <Button
                    onClick={handleActivate}
                    disabled={isActivating || !licenseKey.trim()}
                    className="w-full"
                  >
                    {isActivating ? (
                      <>
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        Activating...
                      </>
                    ) : (
                      <>
                        <ShieldCheck className="mr-2 h-4 w-4" />
                        Activate License
                      </>
                    )}
                  </Button>
                </CardContent>
              </Card>

              <Button
                variant="ghost"
                onClick={() => setShowActivationForm(false)}
                className="w-full"
              >
                Back to options
              </Button>
            </>
          )}
        </div>

        {/* Trial info */}
        {isTrialExpired && (
          <div className="mt-6 pt-4 border-t w-full">
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
              <Clock className="h-3.5 w-3.5" />
              <span>Your trial started 7+ days ago</span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
