import { AiFormattingProvidersTab } from "@/components/ai-formatting-providers-tab";
import {
  Check,
  Wand2,
  Zap
} from "@/components/icons";
import { Switch } from "@/components/ui/switch";
import { useToast } from "@/hooks/use-toast";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/store";
import {
  AI_FORMATTING_STYLES,
  AI_FORMATTING_STYLE_LABELS,
  type AiFormattingStyle,
} from "@/types";
import { useMemo } from "react";

interface AiFormattingViewProps {
  onClose: () => void;
}

export function AiFormattingView(_props: AiFormattingViewProps) {
  const { settings, updateSettings, aiFormattingProviders } = useAppStore();
  const { success: toastSuccess, error: _toastError } = useToast();

  const aiFormattingEnabled = settings.aiFormattingEnabled ?? false;
  const aiFormattingProviderId = settings.aiFormattingProviderId;
  const aiFormattingStyle = settings.aiFormattingStyle ?? "clean";

  const activeStyleInfo = useMemo(
    () => AI_FORMATTING_STYLES.find((s) => s.id === aiFormattingStyle) ?? AI_FORMATTING_STYLES[0],
    [aiFormattingStyle]
  );

  const activeProviderName =
    aiFormattingProviders.find((p) => p.id === aiFormattingProviderId)?.name ?? null;
  const configuredCount = aiFormattingProviders.filter((p) => p.configured).length;

  const handleToggleEnable = async (enabled: boolean) => {
    updateSettings({ aiFormattingEnabled: enabled });
    toastSuccess(
      enabled ? "AI Formatting Enabled" : "AI Formatting Disabled",
      enabled
        ? "Your transcribed text will be formatted before insertion"
        : "Your transcribed text will be inserted as-is"
    );
  };

  const handleStyleSelect = async (style: AiFormattingStyle) => {
    updateSettings({ aiFormattingStyle: style });
    toastSuccess(
      "Style Updated",
      `Now using ${AI_FORMATTING_STYLE_LABELS[style] ?? style} formatting`
    );
  };

  return (
    <div className="flex h-full flex-col overflow-hidden bg-canvas">
      {/* ─── HEADER ─── */}
      <div className="shrink-0 border-b border-hairline">
        <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-3 sm:py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">

              <div>
                <p className="eyebrow-uppercase text-ink-mid">AI Formatting</p>
                <h1 className="display-sm text-ink mt-1">
                  Polish your <span className="text-primary">dictated text</span>.
                </h1>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto">
        <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-4 xl:py-5 space-y-4 xl:space-y-5">
          {/* ─── HERO STATUS BAND — Dark coffee-ink ─── */}
          <section className="hero-band-dark">
            <div className="grid grid-cols-1 @xl:grid-cols-[1.4fr_1fr] gap-4 @xl:gap-5 p-4 sm:p-5 @xl:p-6 items-start @xl:items-center">
              <div className="min-w-0">
                <p className="eyebrow-uppercase text-primary mb-2">
                  <span className="inline-flex items-center gap-2">
                    <span className="h-1.5 w-1.5 rounded-full bg-primary" />
                    {aiFormattingEnabled ? "Active" : "Inactive"}
                  </span>
                </p>
                <h2 className="display-md text-on-dark">
                  {aiFormattingEnabled
                    ? activeProviderName
                      ? <>Formatting with <span className="text-primary">{activeProviderName}</span>.</>
                      : <>AI formatting is <span className="text-primary">enabled</span>.</>
                    : <>AI formatting is <span className="text-primary">off</span>.</>}
                </h2>
                <p className="body-sm text-on-dark-soft mt-2 max-w-xl leading-relaxed">
                  {aiFormattingEnabled
                    ? activeProviderName
                      ? `Using ${AI_FORMATTING_STYLE_LABELS[aiFormattingStyle] ?? aiFormattingStyle} style. Your voice will be reformatted before it reaches your cursor.`
                      : "Enable and configure a provider below to start formatting your transcriptions."
                    : "Turn on AI formatting to reformat your dictated text with a chosen style and LLM provider."}
                </p>
              </div>

              <div className="product-ui-card-dark w-full">
                <div className="flex items-center justify-between gap-2 pt-3 border-t" style={{ borderColor: "#36342e" }}>
                  <span className="caption text-on-dark-muted">Providers ready</span>
                  <span className="caption-strong text-on-dark">
                    {configuredCount} of {aiFormattingProviders.length}
                  </span>
                </div>
                <div className="flex items-center justify-between gap-2 pt-2">
                  <span className="caption text-on-dark-muted">Active style</span>
                  <span className="caption-strong text-on-dark">{activeStyleInfo.name}</span>
                </div>
              </div>
            </div>
          </section>

          {/* ─── GLOBAL TOGGLE — Cream surface ─── */}
          <section className="card-feature-cream">
            <div className="flex items-center gap-2.5">
              <div className="icon-plate">
                <Zap className="h-3.5 w-3.5 text-primary" />
              </div>
              <div className="min-w-0 flex-1">
                <p className="eyebrow-uppercase text-ink-mid">Global toggle</p>
                <h3 className="title-md text-ink mt-0.5">Enable AI Formatting</h3>
              </div>
              <Switch
                checked={aiFormattingEnabled}
                onCheckedChange={handleToggleEnable}
                className="shrink-0"
              />
            </div>
            <p className="body-sm text-body-muted mt-2 leading-relaxed">
              When enabled, transcribed text is sent to your selected LLM provider and
              reformatted using the chosen style before insertion.
            </p>
          </section>

          {/* ─── STYLE SELECTION — White surface ─── */}
          <section className="paper-card">
            <div className="flex items-center gap-2.5 mb-4">
              <div className="icon-plate">
                <Wand2 className="h-3.5 w-3.5 text-primary" />
              </div>
              <div className="min-w-0">
                <p className="eyebrow-uppercase text-ink-mid">Format Style</p>
                <h3 className="title-md text-ink mt-0.5">Choose output style</h3>
              </div>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 gap-3">
              {AI_FORMATTING_STYLES.map((style) => {
                const isActive = aiFormattingStyle === style.id;
                return (
                  <button
                    key={style.id}
                    onClick={() => handleStyleSelect(style.id)}
                    disabled={!aiFormattingEnabled}
                    className={cn(
                      "relative flex flex-col items-start gap-2 p-3.5 rounded-[10px] border text-left transition-all cursor-pointer",
                      isActive
                        ? "border-primary bg-primary/5"
                        : "border-hairline hover:border-ink/20 hover:bg-canvas-soft"
                    )}
                  >
                    <div className="flex items-start justify-between w-full gap-2">
                      <span className="title-sm text-ink">{style.name}</span>
                      {isActive && (
                        <Check className="h-3.5 w-3.5 text-primary shrink-0 mt-0.5" />
                      )}
                    </div>
                    <p className="body-sm text-body-muted">{style.description}</p>
                  </button>
                );
              })}
            </div>

            <div className="mt-5">
              <h4 className="caption-strong text-ink mb-2">
                Active: {activeStyleInfo.name}
              </h4>
              {activeStyleInfo.prompt ? (
                <pre className="whitespace-pre-wrap text-xs bg-canvas border border-hairline rounded-[10px] p-3 text-body-muted overflow-x-auto">
                  {activeStyleInfo.prompt}
                </pre>
              ) : (
                <p className="caption text-body-muted italic">
                  (Default formatting)
                </p>
              )}
            </div>
          </section>

          {/* ─── Provider BYOK Configuration ─── */}
          <AiFormattingProvidersTab selectedStyle={aiFormattingStyle} />
        </div>
      </div>
    </div>
  );
}
