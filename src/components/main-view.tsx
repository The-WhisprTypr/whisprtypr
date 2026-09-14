import { lazy, Suspense } from "react";

const AdvancedView = lazy(() =>
  import("@/components/advanced-view").then((m) => ({ default: m.AdvancedView })),
);
const HistoryView = lazy(() =>
  import("@/components/history-view").then((m) => ({ default: m.HistoryView })),
);
const HelpSupportView = lazy(() =>
  import("@/components/help-support-view").then((m) => ({
    default: m.HelpSupportView,
  })),
);
const LicenseView = lazy(() =>
  import("@/components/license-view").then((m) => ({ default: m.LicenseView })),
);
const ModelsView = lazy(() =>
  import("@/components/models-view").then((m) => ({ default: m.ModelsView })),
);
const SettingsView = lazy(() =>
  import("@/components/settings-view").then((m) => ({
    default: m.SettingsView,
  })),
);
const TranscribeView = lazy(() =>
  import("@/components/transcribe-view").then((m) => ({
    default: m.TranscribeView,
  })),
);
const VocabularyView = lazy(() =>
  import("@/components/vocabulary-view").then((m) => ({
    default: m.VocabularyView,
  })),
);
const AiFormattingView = lazy(() =>
  import("@/components/ai-formatting-view").then((m) => ({
    default: m.AiFormattingView,
  })),
);
const OverviewView = lazy(() =>
  import("@/components/overview-view").then((m) => ({
    default: m.OverviewView,
  })),
);

interface MainViewProps {
  currentPage: string;
  trialDaysRemaining?: number;
  onLicenseChange?: () => void | Promise<void>;
  onNavigate?: (page: string) => void;
}

const ViewLoadingFallback = () => (
  <div className="flex h-full w-full items-center justify-center">
    <div className="flex flex-col items-center gap-3">
      <div className="h-8 w-8 animate-spin rounded-full border-2 border-primary border-t-transparent" />
      <p className="text-sm text-muted-foreground">Loading...</p>
    </div>
  </div>
);

export function MainView({ currentPage, trialDaysRemaining, onLicenseChange, onNavigate }: MainViewProps) {
  const navigate = onNavigate || (() => {});

  if (currentPage === "history") {
    return (
      <Suspense fallback={<ViewLoadingFallback />}>
        <HistoryView onClose={() => navigate("overview")} />
      </Suspense>
    );
  }

  if (currentPage === "models") {
    return (
      <Suspense fallback={<ViewLoadingFallback />}>
        <ModelsView onClose={() => navigate("overview")} />
      </Suspense>
    );
  }

  if (currentPage === "transcribe") {
    return (
      <Suspense fallback={<ViewLoadingFallback />}>
        <TranscribeView onClose={() => navigate("overview")} />
      </Suspense>
    );
  }

  if (currentPage === "license") {
    return (
      <Suspense fallback={<ViewLoadingFallback />}>
        <LicenseView
          onClose={() => navigate("overview")}
          onLicenseChange={(_isValid) => {
            void onLicenseChange?.();
          }}
        />
      </Suspense>
    );
  }

  if (currentPage === "settings") {
    return (
      <Suspense fallback={<ViewLoadingFallback />}>
        <SettingsView onClose={() => navigate("overview")} />
      </Suspense>
    );
  }

  if (currentPage === "advanced") {
    return (
      <Suspense fallback={<ViewLoadingFallback />}>
        <AdvancedView onClose={() => navigate("overview")} />
      </Suspense>
    );
  }

  if (currentPage === "vocabulary") {
    return (
      <Suspense fallback={<ViewLoadingFallback />}>
        <VocabularyView onClose={() => navigate("overview")} />
      </Suspense>
    );
  }

  if (currentPage === "ai-formatting") {
    return (
      <Suspense fallback={<ViewLoadingFallback />}>
        <AiFormattingView onClose={() => navigate("overview")} />
      </Suspense>
    );
  }

  if (currentPage === "help") {
    return (
      <Suspense fallback={<ViewLoadingFallback />}>
        <HelpSupportView onClose={() => navigate("overview")} />
      </Suspense>
    );
  }

  return (
    <Suspense fallback={<ViewLoadingFallback />}>
      <OverviewView onNavigate={navigate} trialDaysRemaining={trialDaysRemaining} />
    </Suspense>
  );
}
