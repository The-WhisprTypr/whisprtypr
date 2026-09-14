import { DashboardSidebar, type Page } from "@/components/dashboard-sidebar";
import { MainView } from "@/components/main-view";
import { SetupWizard } from "@/components/setup";
import { TrialExpiredView } from "@/components/trial-expired-view";
import { Toaster } from "@/components/ui/sonner";
import { canUseApp } from "@/lib/license-api";
import { checkForUpdates } from "@/lib/updater-api";
import { initSentry } from "@/lib/sentry";
import { useAppStore, useIsInitialized } from "@/store";
import { useHotkey } from "@/hooks/use-hotkey";
import { useToast } from "@/hooks/use-toast";
import { Loader2 } from "@/components/icons";
import { useEffect, useState } from "react";
import "./App.css";

type AppAccessState = {
  canUse: boolean;
  reason: "licensed" | "trial" | "trial_expired" | "no_license";
  daysRemaining?: number;
};

function AppContent() {
  const { setupComplete, initializeFromDb } = useAppStore();
  const isInitialized = useIsInitialized();
  const [accessState, setAccessState] = useState<AppAccessState | null>(null);
  const [checkingAccess, setCheckingAccess] = useState(true);
  const [currentPage, setCurrentPage] = useState<Page>("overview");
  const { success: toastSuccess } = useToast();

  const refreshAccessState = async () => {
    try {
      const status = await canUseApp();
      setAccessState(status);
    } catch (err) {
      console.error("Failed to check app access:", err);
      setAccessState({ canUse: false, reason: "no_license" });
    }
  };

  useEffect(() => {
    initializeFromDb();
  }, [initializeFromDb]);

  useEffect(() => {
    const checkAccess = async () => {
      if (!isInitialized || !setupComplete) {
        setCheckingAccess(false);
        return;
      }

      try {
        await refreshAccessState();
      } finally {
        setCheckingAccess(false);
      }
    };

    checkAccess();
  }, [isInitialized, setupComplete]);

  useEffect(() => {
    if (!isInitialized || !setupComplete) {
      return;
    }

    initSentry();

    const { settings } = useAppStore.getState();
    if (!settings.autoCheckForUpdates) {
      return;
    }

    const runCheck = async () => {
      try {
        const result = await checkForUpdates();
        if (result.status === "available") {
          toastSuccess(`Update ${result.info.version} available!`);
        }
      } catch {
        // Silently fail background checks
      }
    };

    runCheck();

    const intervalId = setInterval(runCheck, 24 * 60 * 60 * 1000);

    return () => clearInterval(intervalId);
  }, [isInitialized, setupComplete, toastSuccess]);

  const handleLicenseActivated = async () => {
    await refreshAccessState();
  };

  const handleNavigate = (page: string) => {
    setCurrentPage(page as Page);
  };

  if (!isInitialized) {
    return (
      <div className="h-full w-full flex items-center justify-center">
        <div className="flex flex-col items-center gap-3">
          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
          <p className="text-sm text-muted-foreground">Loading...</p>
        </div>
      </div>
    );
  }

  if (!setupComplete) {
    return (
      <div className="h-full w-full">
        <SetupWizard />
      </div>
    );
  }

  if (checkingAccess) {
    return (
      <div className="h-full w-full flex items-center justify-center">
        <div className="flex flex-col items-center gap-3">
          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
          <p className="text-sm text-muted-foreground">Checking license...</p>
        </div>
      </div>
    );
  }

  if (accessState && !accessState.canUse) {
    return (
      <div className="h-full w-full">
        <TrialExpiredView
          onLicenseActivated={handleLicenseActivated}
          reason={
            accessState.reason === "trial_expired" ? "trial_expired" : "no_license"
          }
        />
      </div>
    );
  }

  return (
    <div className="flex h-full w-full">
      <DashboardSidebar currentPage={currentPage} onNavigate={handleNavigate} />
      <main className="flex-1 min-w-0 bg-background">
        <MainView
          currentPage={currentPage}
          trialDaysRemaining={accessState?.daysRemaining}
          onLicenseChange={handleLicenseActivated}
          onNavigate={handleNavigate}
        />
      </main>
    </div>
  );
}

function App() {
  useHotkey();

  useEffect(() => {
    const disableContextMenu = (e: Event) => {
      const target = e.target as HTMLElement | null;
      const tag = target?.tagName?.toLowerCase();
      if (tag === "input" || tag === "textarea" || target?.isContentEditable) {
        return;
      }
      e.preventDefault();
    };

    const disableCopyPaste = (e: ClipboardEvent) => {
      const target = e.target as HTMLElement | null;
      const tag = target?.tagName?.toLowerCase();
      if (tag === "input" || tag === "textarea" || target?.isContentEditable) {
        return;
      }
      e.preventDefault();
    };

    document.addEventListener("contextmenu", disableContextMenu);
    document.addEventListener("copy", disableCopyPaste);
    document.addEventListener("cut", disableCopyPaste);
    document.addEventListener("paste", disableCopyPaste);

    return () => {
      document.removeEventListener("contextmenu", disableContextMenu);
      document.removeEventListener("copy", disableCopyPaste);
      document.removeEventListener("cut", disableCopyPaste);
      document.removeEventListener("paste", disableCopyPaste);
    };
  }, []);

  return (
    <>
      <main className="h-screen w-screen overflow-hidden bg-background text-foreground">
        <AppContent />
      </main>
      <Toaster position="top-right" offset={16} gap={8} />
    </>
  );
}

export default App;
