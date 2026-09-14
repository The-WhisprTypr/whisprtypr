import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useToast } from "@/hooks/use-toast";
import { cn } from "@/lib/utils";
import { useAppStore } from "@/store";
import { BookText, Trash2, Wand2 } from "@/components/icons";
import { useState } from "react";

interface VocabularyViewProps {
  onClose: () => void;
}

export function VocabularyView(_props: VocabularyViewProps) {
  const { settings, updateSettings } = useAppStore();
  const [spoken, setSpoken] = useState("");
  const [written, setWritten] = useState("");
  const { error: toastError, success: toastSuccess } = useToast();

  const entries = settings.customVocabulary ?? [];

  const handleAdd = () => {
    const trimmedSpoken = spoken.trim();
    const trimmedWritten = written.trim();

    if (!trimmedSpoken || !trimmedWritten) {
      toastError(
        "Both fields required",
        "Enter the phrase you say and the text it should become."
      );
      return;
    }

    if (
      entries.some(
        (e) => e.spoken.toLowerCase() === trimmedSpoken.toLowerCase()
      )
    ) {
      toastError(
        "Duplicate entry",
        `"${trimmedSpoken}" is already in your vocabulary.`
      );
      return;
    }

    updateSettings({
      customVocabulary: [...entries, { spoken: trimmedSpoken, written: trimmedWritten }],
    });
    setSpoken("");
    setWritten("");
    toastSuccess?.("Vocabulary updated", `Added "${trimmedWritten}".`);
  };

  const handleRemove = (index: number) => {
    updateSettings({
      customVocabulary: entries.filter((_, i) => i !== index),
    });
  };

  const handleClear = () => {
    if (entries.length === 0) return;
    updateSettings({ customVocabulary: [] });
    toastSuccess?.("Vocabulary cleared", "All custom terms removed.");
  };

  return (
    <div className="flex h-full flex-col overflow-hidden bg-canvas">
      {/* ─── HEADER ─── */}
      <div className="shrink-0 border-b border-hairline">
        <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-3 sm:py-4">
          <p className="eyebrow-uppercase text-ink-mid">Vocabulary</p>
          <h1
            className="display-sm text-ink mt-1"
          >
            Custom <span className="text-primary">vocabulary</span>.
          </h1>
        </div>
      </div>

      {/* ─── CONTENT ─── */}
      <div className="flex-1 overflow-y-auto">
        <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-4 xl:py-5 space-y-4 xl:space-y-5">
          <section className="paper-card">
            <div className="flex items-center gap-2.5 mb-3">
              <div className="icon-plate">
                <BookText className="h-3.5 w-3.5" />
              </div>
              <div className="flex-1 min-w-0">
                <p className="eyebrow-uppercase text-ink-mid">Your terms</p>
                <h3 className="title-md text-ink mt-0.5">
                  Custom vocabulary
                </h3>
              </div>
              {entries.length > 0 && (
                <button
                  type="button"
                  className="paper-button-outline size-sm shrink-0 cursor-pointer"
                  style={{ borderColor: "#cf202f", color: "#cf202f" }}
                  onClick={handleClear}
                >
                  <Trash2 className="h-3 w-3" />
                  Clear all
                </button>
              )}
            </div>

            <p className="body-sm text-body-muted mb-3 leading-relaxed">
              When you say{" "}
              <span className="font-mono text-ink">spoke phrase</span>,
              WhisprTypr will replace it with{" "}
              <span className="font-mono text-ink">written form</span>{" "}
              (case-insensitive, whole-word match). The written form is preserved
              exactly, so include the casing and punctuation you want.
            </p>

            <div className="flex flex-col sm:flex-row gap-2 mb-3">
              <div className="flex-1 space-y-1">
                <Label
                  htmlFor="vocab-spoken"
                  className="caption-strong text-ink-mid"
                >
                  You say
                </Label>
                <Input
                  id="vocab-spoken"
                  value={spoken}
                  onChange={(e) => setSpoken(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") {
                      e.preventDefault();
                      handleAdd();
                    }
                  }}
                  placeholder="e.g. next js"
                  className="paper-input h-9"
                  style={{ borderRadius: "8px" }}
                />
              </div>
              <div className="flex-1 space-y-1">
                <Label
                  htmlFor="vocab-written"
                  className="caption-strong text-ink-mid"
                >
                  WhisprTypr writes
                </Label>
                <Input
                  id="vocab-written"
                  value={written}
                  onChange={(e) => setWritten(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") {
                      e.preventDefault();
                      handleAdd();
                    }
                  }}
                  placeholder="e.g. Next.js"
                  className="paper-input h-9"
                  style={{ borderRadius: "8px" }}
                />
              </div>
              <div className="flex items-end">
                <Button
                  type="button"
                  onClick={handleAdd}
                  className="paper-button-primary size-md cursor-pointer"
                  style={{ height: "2.25rem" }}
                >
                  Add
                </Button>
              </div>
            </div>

            {entries.length === 0 ? (
              <div className="p-3 rounded-md border border-hairline bg-canvas-soft text-center text-body-muted flex flex-col items-center gap-2">
                <Wand2 className="h-3.5 w-3.5 text-body-mid" />
                <span className="body-sm">No custom terms yet. Add one above to get started.</span>
              </div>
            ) : (
              <div className="space-y-1 max-h-48 overflow-y-auto pr-1">
                {entries.map((entry, index) => (
                  <div
                    key={`${entry.spoken}-${index}`}
                    className={cn(
                      "flex items-center gap-2 p-2 rounded-md",
                      "bg-canvas-soft border border-hairline-soft"
                    )}
                  >
                    <div className="flex-1 min-w-0 flex items-center gap-2 text-body-sm">
                      <span className="font-mono text-body truncate">
                        {entry.spoken}
                      </span>
                      <span className="text-body-mid">→</span>
                      <span className="font-mono text-ink truncate">
                        {entry.written}
                      </span>
                    </div>
                    <button
                      type="button"
                      onClick={() => handleRemove(index)}
                      className="text-body-muted hover:text-destructive transition-colors p-1 rounded-md hover:bg-canvas shrink-0"
                      aria-label={`Remove ${entry.written}`}
                    >
                      <Trash2 className="h-3 w-3" />
                    </button>
                  </div>
                ))}
              </div>
            )}

            <p className="caption text-body-muted mt-2.5">
              Common uses: product names (Next.js, Tauri), acronyms (k8s, OAuth),
              file paths, libraries, project codenames, and any term Whisper
              consistently mishears.
            </p>
          </section>
        </div>
      </div>
    </div>
  );
}
