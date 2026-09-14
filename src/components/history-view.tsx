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
import {
  clearTranscriptionHistory,
  deleteTranscriptionItem,
  getTranscriptionHistory,
  getTranscriptionHistoryCount,
  reportError,
  type TranscriptionHistoryItem,
} from "@/lib/voice-api";
import { downloadFile, exportAppData } from "@/lib/data-management";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useToast } from "@/hooks/use-toast";
import {
  AlertCircle,
  Check,
  ChevronLeftIcon,
  ChevronRightIcon,
  Clock,
  Copy,
  Cpu,
  Download,
  History as HistoryIcon,
  Loader2,
  Search,
  Trash2,
  X,
} from "@/components/icons";
import { useCallback, useEffect, useMemo, useState } from "react";
import { cn } from "@/lib/utils";

const DEFAULT_PAGE_SIZE = 20;
const PAGE_SIZE_OPTIONS = [20, 50, 100] as const;

type PageSize = (typeof PAGE_SIZE_OPTIONS)[number];

// Format model name for display - converts cloud:provider:model to "Provider: Model"
function formatModelName(modelId: string): string {
  if (modelId.startsWith("translation:")) {
    const parts = modelId.split(":");
    if (parts.length >= 2) {
      const langs = parts.slice(1).join(":").replace(/->/g, " → ");
      return `Translation (${langs})`;
    }
    return "Translation";
  }
  if (modelId.startsWith("cloud:")) {
    const parts = modelId.split(":");
    if (parts.length >= 3) {
      const provider = parts[1].charAt(0).toUpperCase() + parts[1].slice(1);
      const model = parts.slice(2).join(":");
      return `${provider}: ${model}`;
    }
  }
  return modelId;
}

export function HistoryView(_props: { onClose: () => void }) {
  const [history, setHistory] = useState<TranscriptionHistoryItem[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [copiedId, setCopiedId] = useState<number | null>(null);
  const [totalCount, setTotalCount] = useState(0);
  const [currentPage, setCurrentPage] = useState(1);
  const [pageSize, setPageSize] = useState<PageSize>(DEFAULT_PAGE_SIZE);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [deletingId, setDeletingId] = useState<number | null>(null);
  const [isClearing, setIsClearing] = useState(false);
  const [isExporting, setIsExporting] = useState(false);
  const [searchTerm, setSearchTerm] = useState("");
  const [debouncedSearchTerm, setDebouncedSearchTerm] = useState("");
  const { success: toastSuccess, error: toastError } = useToast();

  const getErrorMessage = (error: unknown) =>
    error instanceof Error ? error.message : String(error || "Something went wrong");

  const totalPages = useMemo(
    () => Math.max(1, Math.ceil(totalCount / pageSize)),
    [totalCount, pageSize]
  );

  const safeCurrentPage = useMemo(
    () => Math.min(currentPage, totalPages),
    [currentPage, totalPages]
  );

  const loadHistory = useCallback(
    async (page: number, reset: boolean = false) => {
      const search = debouncedSearchTerm.trim() || undefined;
      if (reset) {
        setIsLoading(true);
        setHistory([]);
        setLoadError(null);
        setActionError(null);
      }

      const safePage = Math.max(1, page);
      const offset = (safePage - 1) * pageSize;

      try {
        const [items, count] = await Promise.all([
          getTranscriptionHistory(pageSize, offset, search),
          getTranscriptionHistoryCount(search),
        ]);

        setHistory(items);
        setTotalCount(count);
        setCurrentPage(safePage);
      } catch (error) {
        const message = getErrorMessage(error);
        console.error("Failed to load history:", error);
        setLoadError(message);
        await reportError("database", message, "error", {
          userAction: "Load transcription history",
        }).catch(console.error);
      } finally {
        setIsLoading(false);
      }
    },
    [debouncedSearchTerm, pageSize]
  );

  useEffect(() => {
    loadHistory(1, true);
  }, [loadHistory]);

  useEffect(() => {
    const timeout = window.setTimeout(() => {
      setDebouncedSearchTerm(searchTerm);
    }, 250);

    return () => window.clearTimeout(timeout);
  }, [searchTerm]);

  const handleCopy = async (text: string, id: number) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedId(id);
      setTimeout(() => setCopiedId(null), 2000);
    } catch (error) {
      const message = getErrorMessage(error);
      console.error("Failed to copy:", error);
      setActionError("Could not copy transcription.");
      await reportError("ui", message, "error", {
        userAction: "Copy transcription history item",
      }).catch(console.error);
    }
  };

  const handleDelete = async (id: number) => {
    try {
      setDeletingId(id);
      setActionError(null);
      await deleteTranscriptionItem(id);
      setHistory((prev) => prev.filter((item) => item.id !== id));
      setTotalCount((prev) => prev - 1);
    } catch (error) {
      const message = getErrorMessage(error);
      console.error("Failed to delete:", error);
      setActionError("Could not delete transcription.");
      await reportError("database", message, "error", {
        userAction: "Delete transcription history item",
        context: { id: String(id) },
      }).catch(console.error);
    } finally {
      setDeletingId(null);
    }
  };

  const handleClearAll = async () => {
    try {
      setIsClearing(true);
      setActionError(null);
      await clearTranscriptionHistory();
      setHistory([]);
      setTotalCount(0);
      setCurrentPage(1);
      setSearchTerm("");
      setDebouncedSearchTerm("");
    } catch (error) {
      const message = getErrorMessage(error);
      console.error("Failed to clear history:", error);
      setActionError("Could not clear history.");
      await reportError("database", message, "error", {
        userAction: "Clear transcription history",
      }).catch(console.error);
    } finally {
      setIsClearing(false);
    }
  };

  const handleExport = async () => {
    try {
      setIsExporting(true);
      setActionError(null);
      const data = await exportAppData();
      const filename = `WhisprTypr-history-${new Date()
        .toISOString()
        .slice(0, 10)}.json`;
      const saved = await downloadFile(data, filename);
      if (saved) {
        toastSuccess("Export complete", "History exported successfully");
      }
    } catch (err) {
      const message = getErrorMessage(err);
      console.error("Export failed:", err);
      setActionError("Could not export history.");
      toastError("Export failed", "Failed to export history");
      await reportError("filesystem", message, "error", {
        userAction: "Export history",
      }).catch(console.error);
    } finally {
      setIsExporting(false);
    }
  };

  const goToPage = (page: number) => {
    const safePage = Math.max(1, Math.min(page, totalPages));
    if (safePage === currentPage && history.length > 0) return;
    loadHistory(safePage);
  };

  const pageNumbers = useMemo(() => {
    const pages: (number | "ellipsis")[] = [];
    const maxVisible = 5;

    if (totalPages <= maxVisible + 2) {
      for (let i = 1; i <= totalPages; i++) pages.push(i);
      return pages;
    }

    pages.push(1);

    let start = Math.max(2, safeCurrentPage - 1);
    let end = Math.min(totalPages - 1, safeCurrentPage + 1);

    if (safeCurrentPage <= 3) {
      end = Math.min(totalPages - 1, 4);
    }
    if (safeCurrentPage >= totalPages - 2) {
      start = Math.max(2, totalPages - 3);
    }

    if (start > 2) pages.push("ellipsis");
    for (let i = start; i <= end; i++) pages.push(i);
    if (end < totalPages - 1) pages.push("ellipsis");

    pages.push(totalPages);

    return pages;
  }, [safeCurrentPage, totalPages]);

  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return "Just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
  };

  const formatDuration = (ms: number) => {
    const seconds = Math.round(ms / 1000);
    if (seconds < 60) return `${seconds}s`;
    const minutes = Math.floor(seconds / 60);
    const remainingSecs = seconds % 60;
    return `${minutes}m ${remainingSecs}s`;
  };

  const startItem = totalCount === 0 ? 0 : (safeCurrentPage - 1) * pageSize + 1;
  const endItem = Math.min(safeCurrentPage * pageSize, totalCount);

  return (
    <div className="flex h-full flex-col overflow-hidden bg-canvas">
      {/* ─── HEADER — Cream band, editorial style ─── */}
      <div className="shrink-0 border-b border-hairline">
        <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-3 sm:py-4 flex items-center justify-between gap-3 flex-wrap">
          <div className="min-w-0">
            <p className="eyebrow-uppercase text-ink-mid">History</p>
            <h1
              className="display-sm text-ink mt-1"
            >
              {totalCount > 0 ? (
                <>
                  <span className="text-primary">{totalCount}</span> transcriptions saved.
                </>
              ) : (
                "Your transcription archive."
              )}
            </h1>
          </div>
          {history.length > 0 && (
            <div className="flex items-center gap-2 shrink-0">
              <button
                className="paper-button-outline size-md cursor-pointer"
                onClick={handleExport}
                disabled={isExporting}
              >
                {isExporting ? <Loader2 className="h-4 w-4 animate-spin" /> : <Download className="h-4 w-4" />}
                {isExporting ? "Exporting..." : "Export"}
              </button>
              <AlertDialog>
                <AlertDialogTrigger asChild>
                  <button
                    className="paper-button-outline size-md shrink-0 cursor-pointer"
                    style={{ borderColor: '#cf202f', color: '#cf202f' }}
                    disabled={isClearing}
                  >
                    {isClearing ? <Loader2 className="h-4 w-4 animate-spin" /> : null}
                    {isClearing ? "Clearing..." : "Clear all"}
                  </button>
                </AlertDialogTrigger>
                <AlertDialogContent>
                  <AlertDialogHeader>
                    <AlertDialogTitle>Clear all history?</AlertDialogTitle>
                    <AlertDialogDescription>
                      This will permanently delete all your transcription history.
                      This action cannot be undone.
                    </AlertDialogDescription>
                  </AlertDialogHeader>
                  <AlertDialogFooter>
                    <AlertDialogCancel className="paper-button-secondary">Cancel</AlertDialogCancel>
                    <AlertDialogAction
                      onClick={handleClearAll}
                      className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
                    >
                      Clear all
                    </AlertDialogAction>
                  </AlertDialogFooter>
                </AlertDialogContent>
              </AlertDialog>
            </div>
          )}
        </div>
      </div>

      {/* ─── SEARCH BAR — Cream surface ─── */}
      <div className="shrink-0 border-b border-hairline">
        <div className="max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-2.5 sm:py-3">
          <div className="@container flex items-center gap-2.5 flex-wrap">
            <div className="relative flex items-center flex-1 min-w-0 max-w-2xl">
              <Search className="pointer-events-none absolute left-3 h-3.5 w-3.5 text-body-muted" />
              <Input
                value={searchTerm}
                onChange={(event) => setSearchTerm(event.target.value)}
                placeholder="Search transcriptions"
                aria-label="Search transcription history"
                className="paper-input h-9 pl-9 pr-9 text-sm"
                style={{ borderRadius: '8px' }}
              />
              {searchTerm && (
                <button
                  className="absolute right-1.5 rounded-md p-1.5 text-body-muted transition-colors hover:bg-canvas-soft hover:text-ink"
                  onClick={() => setSearchTerm("")}
                  aria-label="Clear search"
                >
                  <X className="h-3 w-3" />
                </button>
              )}
            </div>
            <div className="flex items-center gap-2">
              <span className="caption text-body-muted">Page size</span>
              <Select
                value={String(pageSize)}
                onValueChange={(value) => {
                  const newSize = Number(value) as PageSize;
                  setPageSize(newSize);
                  setCurrentPage(1);
                }}
              >
                <SelectTrigger className="paper-input h-9 w-[80px]">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {PAGE_SIZE_OPTIONS.map((size) => (
                    <SelectItem key={size} value={String(size)}>
                      {size}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            {debouncedSearchTerm && (
              <p className="caption text-body-muted">
                Showing matches for "{debouncedSearchTerm}"
              </p>
            )}
          </div>
        </div>
      </div>

      {/* ─── CONTENT ─── */}
      {isLoading ? (
        <div className="flex-1 flex items-center justify-center p-6">
          <div className="flex flex-col items-center gap-3">
            <Loader2 className="h-8 w-8 animate-spin text-body-muted" />
            <p className="body-sm text-body-muted">Loading history...</p>
          </div>
        </div>
      ) : history.length === 0 ? (
        <div className="flex-1 flex items-center justify-center p-6">
          <div className="paper-card p-6 sm:p-7 flex flex-col items-center text-center max-w-sm">
            <div className={cn(
              "mb-4 flex h-10 w-10 items-center justify-center rounded-full",
              loadError ? "bg-destructive/10 text-destructive" : "bg-primary/10 text-primary"
            )}>
              {loadError ? (
                <AlertCircle className="h-5 w-5" />
              ) : (
                <HistoryIcon className="h-5 w-5" />
              )}
            </div>
            <h3
              className="display-xs text-ink"
            >
              {loadError
                ? "History unavailable"
                : debouncedSearchTerm
                  ? "No matches found"
                  : "No transcriptions yet"}
            </h3>
            <p className="body-sm text-body-muted mt-1.5 max-w-xs">
              {loadError ||
                (debouncedSearchTerm
                  ? `Nothing matches "${debouncedSearchTerm}". Try a different search.`
                  : "Your transcription archive will appear here once you start dictating.")}
            </p>
            <div className="mt-4 flex items-center gap-2 flex-wrap justify-center">
              {debouncedSearchTerm && !loadError && (
                <button
                  className="paper-button-outline size-md cursor-pointer"
                  onClick={() => setSearchTerm("")}
                >
                  Clear search
                </button>
              )}
              {loadError && (
                <button
                  className="paper-button-primary size-md cursor-pointer"
                  onClick={() => loadHistory(1, true)}
                  disabled={isLoading}
                >
                  {isLoading ? <Loader2 className="h-4 w-4 animate-spin" /> : null}
                  Retry
                </button>
              )}
            </div>
          </div>
        </div>
      ) : (
        <div className="flex-1 flex flex-col overflow-hidden">
          <div className="flex-1 overflow-y-auto">
            <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-4 xl:py-5">
              {actionError && (
                <div className="mb-3 p-3 rounded-md border border-destructive/30 bg-destructive/5 flex items-center gap-2.5 text-destructive">
                  <AlertCircle className="h-4 w-4 shrink-0" />
                  <span className="body-sm">{actionError}</span>
                </div>
              )}

              <div className="space-y-2.5 sm:space-y-3">
                {history.map((item, idx) => (
                  <article
                    key={item.id}
                    className="paper-card group"
                    style={{
                      animationDelay: `${Math.min(idx * 30, 300)}ms`,
                    }}
                  >
                    <div className="flex items-start justify-between gap-2.5 sm:gap-3">
                      <div className="min-w-0 flex-1">
                        {/* Numbered marker */}
                        <div className="flex items-center gap-2 mb-1.5">
                          <span className="caption-strong text-body-muted">
                            {String(startItem + idx).padStart(2, "0")}
                          </span>
                          <span className="h-px flex-1 bg-hairline-soft" />
                          <span className="caption text-body-mid whitespace-nowrap">
                            {formatDate(item.created_at)}
                          </span>
                        </div>

                        <p
                          className="text-ink leading-relaxed"
                          style={{
                            fontSize: 'var(--type-body-sm)',
                            letterSpacing: '-0.005em',
                            wordBreak: 'break-word',
                          }}
                        >
                          {item.text}
                        </p>

                        <div className="flex items-center gap-1.5 sm:gap-2 mt-2.5 flex-wrap">
                          <span className="inline-flex items-center gap-1 caption px-2 py-0.5 rounded-md bg-canvas-soft text-body">
                            <Cpu className="h-2.5 w-2.5" />
                            <span className="capitalize">{formatModelName(item.model_id)}</span>
                          </span>
                          <span className="inline-flex items-center gap-1 caption px-2 py-0.5 rounded-md bg-canvas-soft text-body uppercase">
                            {item.language}
                          </span>
                          {item.duration_ms > 0 && (
                            <span className="inline-flex items-center gap-1 caption px-2 py-0.5 rounded-md bg-canvas-soft text-body">
                              <Clock className="h-2.5 w-2.5" />
                              {formatDuration(item.duration_ms)}
                            </span>
                          )}
                        </div>
                      </div>

                      <div className="flex flex-col items-center gap-1.5 shrink-0">
                        <button
                          className={cn(
                            "flex h-8 w-8 items-center justify-center rounded-md border transition-colors",
                            copiedId === item.id
                              ? "border-primary text-primary bg-primary/10"
                              : "border-hairline text-body-muted hover:border-ink hover:text-ink"
                          )}
                          onClick={() => handleCopy(item.text, item.id)}
                          aria-label="Copy transcription"
                        >
                          {copiedId === item.id ? (
                            <Check className="h-3.5 w-3.5" />
                          ) : (
                            <Copy className="h-3.5 w-3.5" />
                          )}
                        </button>
                        <button
                          className="flex h-8 w-8 items-center justify-center rounded-md border border-hairline text-body-muted hover:border-destructive hover:text-destructive transition-colors"
                          onClick={() => handleDelete(item.id)}
                          disabled={deletingId === item.id}
                          aria-label="Delete transcription"
                        >
                          {deletingId === item.id ? (
                            <Loader2 className="h-3.5 w-3.5 animate-spin" />
                          ) : (
                            <Trash2 className="h-3.5 w-3.5" />
                          )}
                        </button>
                      </div>
                    </div>
                  </article>
                ))}
              </div>
            </div>
          </div>

          {/* ─── PAGINATION FOOTER ─── */}
          {totalCount > pageSize && (
            <div className="shrink-0 border-t border-hairline">
              <div className="max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-3 sm:py-4">
                <div className="flex flex-col sm:flex-row items-center justify-between gap-3">
                  <div className="flex items-center gap-2 order-2 sm:order-1">
                    <span className="caption text-body-muted">
                      {startItem}-{endItem} of {totalCount}
                    </span>
                    {debouncedSearchTerm && (
                      <span className="caption text-body-muted">
                        · matches for "{debouncedSearchTerm}"
                      </span>
                    )}
                  </div>

                  <div className="flex items-center gap-1 order-1 sm:order-2">
                    <button
                      onClick={() => goToPage(safeCurrentPage - 1)}
                      disabled={safeCurrentPage === 1}
                      className="paper-button-outline size-sm cursor-pointer disabled:cursor-not-allowed disabled:opacity-50"
                      aria-label="Previous page"
                    >
                      <ChevronLeftIcon className="h-3.5 w-3.5" />
                    </button>

                    <div className="flex items-center gap-1">
                      {pageNumbers.map((page, idx) =>
                        page === "ellipsis" ? (
                          <span
                            key={`ellipsis-${idx}`}
                            className="px-2 caption text-body-muted"
                          >
                            …
                          </span>
                        ) : (
                          <button
                            key={page}
                            onClick={() => goToPage(page)}
                            className={cn(
                              "h-8 w-8 rounded-md caption-strong transition-colors",
                              safeCurrentPage === page
                                ? "bg-primary text-primary-foreground"
                                : "border border-hairline text-body-muted hover:border-ink hover:text-ink"
                            )}
                            aria-label={`Page ${page}`}
                            aria-current={safeCurrentPage === page ? "page" : undefined}
                          >
                            {page}
                          </button>
                        )
                      )}
                    </div>

                    <button
                      onClick={() => goToPage(safeCurrentPage + 1)}
                      disabled={safeCurrentPage === totalPages}
                      className="paper-button-outline size-sm cursor-pointer disabled:cursor-not-allowed disabled:opacity-50"
                      aria-label="Next page"
                    >
                      <ChevronRightIcon className="h-3.5 w-3.5" />
                    </button>
                  </div>

                  <div className="caption text-body-muted order-3">
                    Page {safeCurrentPage} of {totalPages}
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
