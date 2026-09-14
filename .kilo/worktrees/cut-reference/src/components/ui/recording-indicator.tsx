/**
 * Animated recording indicator
 * Shows visual feedback when recording is active
 */

import { cn } from "@/lib/utils";

interface RecordingIndicatorProps {
  isRecording: boolean;
  isProcessing?: boolean;
  className?: string;
}

export function RecordingIndicator({
  isRecording,
  isProcessing,
  className,
}: RecordingIndicatorProps) {
  if (!isRecording && !isProcessing) return null;

  return (
    <div
      className={cn(
        "fixed top-4 left-1/2 -translate-x-1/2 z-50",
        "flex items-center gap-2 px-4 py-2 rounded-full",
        "bg-background/95 backdrop-blur border shadow-lg",
        "animate-in fade-in slide-in-from-top-2 duration-200",
        className
      )}
    >
      {isRecording && (
        <>
          <span className="relative flex h-3 w-3">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-red-400 opacity-75" />
            <span className="relative inline-flex rounded-full h-3 w-3 bg-red-500" />
          </span>
          <span className="text-sm font-medium text-red-600 dark:text-red-400">
            Recording...
          </span>
        </>
      )}
      {isProcessing && (
        <>
          <span className="relative flex h-3 w-3">
            <span className="animate-spin h-3 w-3 rounded-full border-2 border-primary border-t-transparent" />
          </span>
          <span className="text-sm font-medium text-primary">
            Processing...
          </span>
        </>
      )}
    </div>
  );
}
