import { Textarea } from "@/components/ui/textarea";
import { addTranscription, reportError, transcribeFilesBatch, transcribeUrl } from "@/lib/voice-api";
import { useAppStore } from "@/store";
import { open } from "@tauri-apps/plugin-dialog";
import {
  AlertCircle,
  Circle,
  FileAudio,
  Globe,
  Headphones,
  Loader2,
  Sparkles,
  Upload,
} from "@/components/icons";
import { useState, useCallback } from "react";

interface TranscribeViewProps {
  onClose: () => void;
}

type InputMode = "file" | "url";

export function TranscribeView(_props: TranscribeViewProps) {
  const { settings } = useAppStore();
  const [inputMode, setInputMode] = useState<InputMode>("file");
  const [selectedFiles, setSelectedFiles] = useState<string[]>([]);
  const [fileNames, setFileNames] = useState<string[]>([]);
  const [isSelectingFile, setIsSelectingFile] = useState(false);
  const [isTranscribing, setIsTranscribing] = useState(false);
  const [transcription, setTranscription] = useState<string>("");
  const [error, setError] = useState<string | null>(null);
  const [warning, setWarning] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [urlInput, setUrlInput] = useState("");
  const [enableSpeakerDetection, setEnableSpeakerDetection] = useState(false);
  const [isDragOver, setIsDragOver] = useState(false);

  const getErrorMessage = (err: unknown) =>
    err instanceof Error ? err.message : String(err || "Something went wrong");

  const handleSelectFiles = async () => {
    if (isSelectingFile || isTranscribing) return;

    try {
      setIsSelectingFile(true);
      setError(null);
      setWarning(null);
      const selected = await open({
        multiple: true,
        filters: [
          {
            name: "Audio",
            extensions: [
              "wav",
              "mp3",
              "m4a",
              "ogg",
              "flac",
              "webm",
              "mp4",
              "mov",
            ],
          },
        ],
      });

      if (selected && Array.isArray(selected)) {
        setSelectedFiles(selected);
        setFileNames(selected.map((f) => f.split(/[/\\]/).pop() || f));
      }
    } catch (err) {
      const message = getErrorMessage(err);
      console.error("File selection failed:", err);
      setError(message);
      await reportError("filesystem", message, "error", {
        userAction: "Select audio files",
      }).catch(console.error);
    } finally {
      setIsSelectingFile(false);
    }
  };

  const handleTranscribeFiles = async () => {
    if (selectedFiles.length === 0 || isTranscribing) return;

    setIsTranscribing(true);
    setError(null);
    setWarning(null);
    setTranscription("");

    const startTime = Date.now();
    try {
      const texts = await transcribeFilesBatch(selectedFiles);
      const combined = texts.filter((t) => t.trim().length > 0).join("\n\n");
      setTranscription(combined);

      if (combined) {
        const durationMs = Date.now() - startTime;
        try {
          await addTranscription(
            combined,
            settings.selectedModelId || "base",
            settings.language,
            durationMs
          );
        } catch (historyErr) {
          const message = getErrorMessage(historyErr);
          console.error("Failed to save to history:", historyErr);
          setWarning("Transcription completed, but history could not be saved.");
          await reportError("database", message, "warning", {
            userAction: "Save batch transcription to history",
          }).catch(console.error);
        }
      }
    } catch (err) {
      const message = getErrorMessage(err);
      console.error("Batch transcription failed:", err);
      setError(message);
      await reportError("transcription", message, "error", {
        userAction: "Transcribe files batch",
      }).catch(console.error);
    } finally {
      setIsTranscribing(false);
    }
  };

  const handleTranscribeUrl = async () => {
    if (!urlInput.trim() || isTranscribing) return;

    setIsTranscribing(true);
    setError(null);
    setWarning(null);
    setTranscription("");

    const startTime = Date.now();
    try {
      const text = await transcribeUrl(urlInput.trim(), enableSpeakerDetection);
      setTranscription(text);

      if (text) {
        const durationMs = Date.now() - startTime;
        try {
          await addTranscription(
            text,
            settings.selectedModelId || "base",
            settings.language,
            durationMs
          );
        } catch (historyErr) {
          const message = getErrorMessage(historyErr);
          console.error("Failed to save to history:", historyErr);
          setWarning("Transcription completed, but history could not be saved.");
          await reportError("database", message, "warning", {
            userAction: "Save URL transcription to history",
          }).catch(console.error);
        }
      }
    } catch (err) {
      const message = getErrorMessage(err);
      console.error("URL transcription failed:", err);
      setError(message);
      await reportError("transcription", message, "error", {
        userAction: "Transcribe URL",
        context: { url: urlInput.trim() },
      }).catch(console.error);
    } finally {
      setIsTranscribing(false);
    }
  };

  const handleCopy = async () => {
    if (!transcription) return;
    try {
      await navigator.clipboard.writeText(transcription);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      const message = getErrorMessage(err);
      console.error("Copy failed:", err);
      setError(message);
    }
  };

  const handleClear = () => {
    setSelectedFiles([]);
    setFileNames([]);
    setTranscription("");
    setError(null);
    setWarning(null);
    setUrlInput("");
  };

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(false);
  }, []);

  const handleDrop = useCallback(async (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(false);

    if (isTranscribing) return;

    const items = Array.from(e.dataTransfer.files);
    if (items.length === 0) return;

    const paths = items.map((f) => (f as any).path);
    const names = items.map((f) => f.name);

    setSelectedFiles(paths);
    setFileNames(names);
    setInputMode("file");
    setError(null);
    setWarning(null);
  }, [isTranscribing]);

  return (
    <div className="flex h-full flex-col overflow-hidden bg-canvas">
      {/* ─── HEADER ─── */}
      <div className="shrink-0 border-b border-hairline">
        <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-3 sm:py-4">
          <p className="eyebrow-uppercase text-ink-mid">Transcribe</p>
          <h1
            className="display-sm text-ink mt-1"
          >
            From audio file or URL to <span className="text-primary">text</span>.
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

          {warning && (
            <div
              className="p-3 rounded-md border flex items-center gap-2.5"
              style={{
                borderColor: 'rgba(255,79,0,0.3)',
                background: 'rgba(255,79,0,0.06)',
                color: '#ff4f00',
              }}
            >
              <AlertCircle className="h-4 w-4 shrink-0" />
              <span className="body-sm">{warning}</span>
            </div>
          )}

          {/* ─── HERO STATUS BAND ─── */}
          <section className="hero-band-dark">
            <div className="grid grid-cols-1 @xl:grid-cols-[1.4fr_1fr] gap-4 @xl:gap-6 p-4 sm:p-5 @xl:p-6 items-start @xl:items-center">
              <div className="min-w-0">
                <p className="eyebrow-uppercase text-primary mb-2">
                  <span className="inline-flex items-center gap-2">
                    <Circle className="h-1.5 w-1.5 fill-primary text-primary" />
                    Audio import & transcription
                  </span>
                </p>
                <h2
                  className="display-md text-on-dark"
                >
                  Drop in files, paste a URL. Get <span className="text-primary">words</span>.
                </h2>
                <p className="body-sm text-on-dark-soft mt-2 max-w-xl">
                  Drag and drop audio files, batch-upload recordings, or paste a YouTube or direct audio link. WhisprTypr turns them into clean, copyable text.
                </p>
              </div>

              <div className="product-ui-card-dark w-full">
                <div className="flex items-center gap-2.5 mb-3">
                  <div className="icon-plate-dark">
                    <Headphones className="h-3.5 w-3.5 text-on-dark" />
                  </div>
                  <div className="min-w-0">
                    <p className="caption-strong text-on-dark">Supported formats</p>
                    <p className="caption text-on-dark-soft mt-0.5">8 common audio & video types + URLs</p>
                  </div>
                </div>
                <div className="flex flex-wrap gap-1">
                  {["WAV", "MP3", "M4A", "OGG", "FLAC", "WEBM", "MP4", "MOV"].map((format) => (
                    <span
                      key={format}
                      className="caption-strong px-2 py-0.5 rounded-md"
                      style={{ background: '#14100e', color: '#c5c0b1', border: '1px solid #36342e' }}
                    >
                      {format}
                    </span>
                  ))}
                </div>
              </div>
            </div>
          </section>

          {/* ─── INPUT MODE TOGGLE ─── */}
          <div className="flex items-center gap-2">
            <button
              onClick={() => { setInputMode("file"); setError(null); setWarning(null); }}
              className={`paper-button-outline size-sm cursor-pointer ${inputMode === "file" ? "border-ink text-ink" : "border-hairline text-body-muted"}`}
            >
              <Upload className="h-3.5 w-3.5" />
              Files
            </button>
            <button
              onClick={() => { setInputMode("url"); setError(null); setWarning(null); }}
              className={`paper-button-outline size-sm cursor-pointer ${inputMode === "url" ? "border-ink text-ink" : "border-hairline text-body-muted"}`}
            >
              <Globe className="h-3.5 w-3.5" />
              URL / YouTube
            </button>
          </div>

          {/* ─── FILE INPUT ─── */}
          {inputMode === "file" && !selectedFiles.length ? (
            <section
              className={`card-feature-cream cursor-pointer group transition-all hover:border-ink ${isDragOver ? "border-primary bg-primary/5" : ""}`}
              onClick={handleSelectFiles}
              onDragOver={handleDragOver}
              onDragLeave={handleDragLeave}
              onDrop={handleDrop}
            >
              <div className="flex flex-col items-center justify-center text-center py-7 sm:py-9 px-5">
                <div className="icon-plate-orange mb-4 group-hover:scale-105 transition-transform">
                  <Upload className="h-4 w-4" />
                </div>
                <p className="eyebrow-uppercase text-ink-mid mb-2">Step 1</p>
                <h3
                  className="display-md text-ink"
                >
                  Drop files or browse
                </h3>
                <p className="body-sm text-body-muted mt-2 max-w-md">
                  Drag and drop audio files here, or click to browse. You can select multiple files for batch transcription.
                </p>
                <button
                  className="paper-button-primary mt-4 cursor-pointer disabled:cursor-not-allowed disabled:opacity-50"
                  disabled={isSelectingFile}
                  onClick={(e) => { e.stopPropagation(); handleSelectFiles(); }}
                >
                  {isSelectingFile ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : null}
                  {isSelectingFile ? "Opening..." : "Browse files"}
                </button>
              </div>
            </section>
          ) : inputMode === "file" && selectedFiles.length > 0 ? (
            <section className="card-feature-cream">
              <div className="flex items-center gap-2.5 mb-3">
                <div className="icon-plate">
                  <FileAudio className="h-3.5 w-3.5" />
                </div>
                <div className="min-w-0">
                  <p className="eyebrow-uppercase text-ink-mid">Step 2</p>
                  <h3
                    className="title-md text-ink mt-0.5"
                  >
                    Ready to transcribe {selectedFiles.length > 1 ? `${selectedFiles.length} files` : "file"}
                  </h3>
                </div>
              </div>

              <div className="space-y-2 mb-3">
                {fileNames.map((name, idx) => (
                  <div
                    key={idx}
                    className="flex items-center gap-2.5 sm:gap-3 p-3 rounded-md border border-hairline"
                    style={{ background: '#fffefb' }}
                  >
                    <div className="icon-plate shrink-0">
                      <FileAudio className="h-3.5 w-3.5" />
                    </div>
                    <div className="min-w-0 flex-1">
                      <p
                        className="body-sm-strong text-ink truncate"
                        title={name}
                      >
                        {name}
                      </p>
                      <p className="caption text-body-muted mt-0.5">
                        Loaded · awaiting transcription
                      </p>
                    </div>
                  </div>
                ))}
              </div>

              <div className="flex items-center gap-2 flex-wrap">
                <button
                  onClick={handleTranscribeFiles}
                  disabled={isTranscribing}
                  className="paper-button-primary w-full sm:w-auto"
                >
                  {isTranscribing ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : null}
                  {isTranscribing ? "Transcribing..." : `Transcribe ${selectedFiles.length > 1 ? `${selectedFiles.length} files` : "file"}`}
                </button>
                <button
                  onClick={handleClear}
                  disabled={isTranscribing}
                  className="paper-button-outline size-md cursor-pointer disabled:cursor-not-allowed disabled:opacity-50"
                >
                  Clear
                </button>
              </div>
            </section>
          ) : null}

          {/* ─── URL INPUT ─── */}
          {inputMode === "url" && (
            <section className="card-feature-cream">
              <div className="flex items-center gap-2.5 mb-3">
                <div className="icon-plate">
                  <Globe className="h-3.5 w-3.5" />
                </div>
                <div className="min-w-0">
                  <p className="eyebrow-uppercase text-ink-mid">Step 1</p>
                  <h3
                    className="title-md text-ink mt-0.5"
                  >
                    Paste an audio or YouTube URL
                  </h3>
                </div>
              </div>

              <div className="flex flex-col sm:flex-row gap-2 mb-3">
                <input
                  type="text"
                  value={urlInput}
                  onChange={(e) => setUrlInput(e.target.value)}
                  onKeyDown={(e) => { if (e.key === "Enter") handleTranscribeUrl(); }}
                  placeholder="https://www.youtube.com/watch?v=... or direct audio link"
                  className="paper-input flex-1"
                  disabled={isTranscribing}
                />
                <button
                  onClick={handleTranscribeUrl}
                  disabled={isTranscribing || !urlInput.trim()}
                  className="paper-button-primary w-full sm:w-auto whitespace-nowrap"
                >
                  {isTranscribing ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : null}
                  {isTranscribing ? "Transcribing..." : "Transcribe URL"}
                </button>
              </div>

              <p className="caption text-body-muted">
                {settings.selectedModelId.startsWith("cloud:deepgram")
                  ? "YouTube links and direct audio URLs are supported. Deepgram will handle extraction and transcription."
                  : "YouTube links require yt-dlp for local models. For smoother YouTube transcription, switch to Deepgram in Models settings."}
              </p>
            </section>
          )}

          {/* ─── OPTIONS ─── */}
          <section className="paper-card">
            <div className="flex items-center gap-3">
              <label className="flex items-center gap-2 cursor-pointer select-none">
                <input
                  type="checkbox"
                  checked={enableSpeakerDetection}
                  onChange={(e) => setEnableSpeakerDetection(e.target.checked)}
                  disabled={isTranscribing}
                  className="h-4 w-4 rounded border-hairline accent-primary"
                />
                <Sparkles className="h-4 w-4 text-ink-mid" />
                <span className="body-sm text-ink">Enable speaker detection</span>
              </label>
              <span className="caption text-body-muted">
                {enableSpeakerDetection ? "Diarization enabled for supported cloud providers" : "Available with Deepgram and select cloud models"}
              </span>
            </div>
          </section>

          {/* ─── TRANSCRIPTION RESULT ─── */}
          {transcription && (
            <section className="paper-card">
              <div className="flex items-center justify-between gap-2 mb-3 flex-wrap">
                <div className="min-w-0 flex-1">
                  <p className="eyebrow-uppercase text-ink-mid mb-1">Result</p>
                  <h3
                    className="title-md text-ink"
                  >
                    Transcription
                  </h3>
                </div>
                <button
                  onClick={handleCopy}
                  className="paper-button-outline size-md shrink-0 cursor-pointer"
                  style={{ borderColor: copied ? '#ff4f00' : '#201515', color: copied ? '#ff4f00' : '#201515' }}
                >
                  {copied ? "Copied" : "Copy text"}
                </button>
              </div>

              <Textarea
                readOnly
                value={transcription}
                className="paper-input min-h-[140px] sm:min-h-[180px] resize-none leading-relaxed"
                style={{
                  fontSize: 'var(--type-body-sm)',
                  letterSpacing: '-0.005em',
                  borderRadius: '8px',
                }}
              />

              <div className="flex items-center gap-1.5 mt-3 flex-wrap">
                <span className="inline-flex items-center gap-1 caption px-2 py-0.5 rounded-md bg-canvas-soft text-body">
                  {transcription.trim().split(/\s+/).length} words
                </span>
                <span className="inline-flex items-center gap-1 caption px-2 py-0.5 rounded-md bg-canvas-soft text-body">
                  {transcription.length} characters
                </span>
              </div>
            </section>
          )}
        </div>
      </div>
    </div>
  );
}
