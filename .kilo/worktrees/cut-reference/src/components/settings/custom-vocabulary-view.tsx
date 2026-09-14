import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useToast } from "@/hooks/use-toast";
import { cn } from "@/lib/utils";
import type { VocabularyEntry } from "@/types";
import { BookText, Trash2, Wand2 } from "lucide-react";
import { useState } from "react";

interface CustomVocabularyViewProps {
  entries: VocabularyEntry[];
  onChange: (entries: VocabularyEntry[]) => void;
}

export function CustomVocabularyView({ entries, onChange }: CustomVocabularyViewProps) {
  const [spoken, setSpoken] = useState("");
  const [written, setWritten] = useState("");
  const { error: toastError, success: toastSuccess } = useToast();

  const update = (next: VocabularyEntry[]) => {
    onChange(next);
  };

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

    update([...entries, { spoken: trimmedSpoken, written: trimmedWritten }]);
    setSpoken("");
    setWritten("");
    toastSuccess?.("Vocabulary updated", `Added "${trimmedWritten}".`);
  };

  const handleRemove = (index: number) => {
    update(entries.filter((_, i) => i !== index));
  };

  const handleClear = () => {
    if (entries.length === 0) return;
    update([]);
    toastSuccess?.("Vocabulary cleared", "All custom terms removed.");
  };

  return (
    <div className="glass-card p-4 rounded-2xl">
      <div className="flex items-center gap-3 mb-4">
        <div className="p-2 rounded-xl bg-white/30 dark:bg-white/10">
          <BookText className="h-4 w-4 text-foreground/60" />
        </div>
        <div className="flex-1 min-w-0">
          <h2 className="font-semibold text-sm text-foreground">
            Custom Vocabulary
          </h2>
          <p className="text-xs text-foreground/60">
            Domain terms Whisper keeps mangling — fixed automatically.
          </p>
        </div>
        {entries.length > 0 && (
          <button
            type="button"
            className="glass-button px-2.5 py-1.5 rounded-xl text-xs font-medium text-red-500 hover:text-red-600 flex items-center gap-1.5"
            onClick={handleClear}
          >
            <Trash2 className="h-3.5 w-3.5" />
            Clear all
          </button>
        )}
      </div>

      <div className="space-y-2 mb-3">
        <p className="text-xs text-foreground/60">
          When you say{" "}
          <span className="font-mono text-foreground/80">spoke phrase</span>,
          WhisprTypr will replace it with{" "}
          <span className="font-mono text-foreground/80">written form</span>{" "}
          (case-insensitive, whole-word match). The written form is preserved
          exactly, so include the casing and punctuation you want.
        </p>
      </div>

      <div className="flex flex-col sm:flex-row gap-2 mb-3">
        <div className="flex-1 space-y-1">
          <Label
            htmlFor="vocab-spoken"
            className="text-[10px] font-medium text-foreground/60 uppercase tracking-wider"
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
            className="glass-button border-0 h-9"
          />
        </div>
        <div className="flex-1 space-y-1">
          <Label
            htmlFor="vocab-written"
            className="text-[10px] font-medium text-foreground/60 uppercase tracking-wider"
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
            className="glass-button border-0 h-9"
          />
        </div>
        <div className="flex items-end">
          <Button
            type="button"
            onClick={handleAdd}
            className="glass-button h-9 px-3"
          >
            Add
          </Button>
        </div>
      </div>

      {entries.length === 0 ? (
        <div className="p-4 rounded-xl bg-white/30 dark:bg-white/5 border border-white/30 dark:border-white/10 text-center text-xs text-foreground/60 flex flex-col items-center gap-2">
          <Wand2 className="h-4 w-4 text-foreground/40" />
          <span>No custom terms yet. Add one above to get started.</span>
        </div>
      ) : (
        <div className="space-y-1.5 max-h-64 overflow-y-auto pr-1">
          {entries.map((entry, index) => (
            <div
              key={`${entry.spoken}-${index}`}
              className={cn(
                "flex items-center gap-2 p-2.5 rounded-xl",
                "bg-white/30 dark:bg-white/5 border border-white/30 dark:border-white/10"
              )}
            >
              <div className="flex-1 min-w-0 flex items-center gap-2 text-sm">
                <span className="font-mono text-foreground/70 truncate">
                  {entry.spoken}
                </span>
                <span className="text-foreground/40">→</span>
                <span className="font-mono text-foreground truncate">
                  {entry.written}
                </span>
              </div>
              <button
                type="button"
                onClick={() => handleRemove(index)}
                className="text-foreground/50 hover:text-red-500 transition-colors p-1 rounded-md hover:bg-white/40 dark:hover:bg-white/5"
                aria-label={`Remove ${entry.written}`}
              >
                <Trash2 className="h-3.5 w-3.5" />
              </button>
            </div>
          ))}
        </div>
      )}

      <p className="text-[11px] text-foreground/50 mt-3 leading-relaxed">
        Common uses: product names (Next.js, Tauri), acronyms (k8s, OAuth),
        file paths, libraries, project codenames, and any term Whisper
        consistently mishears.
      </p>
    </div>
  );
}