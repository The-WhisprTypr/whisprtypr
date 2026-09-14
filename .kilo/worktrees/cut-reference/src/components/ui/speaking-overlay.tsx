interface SpeakingOverlayProps {
  visible: boolean;
}

export function SpeakingOverlay({ visible }: SpeakingOverlayProps) {
  if (!visible) return null;

  return (
    <div
      aria-hidden
      className="fixed inset-0 z-[9999] pointer-events-none flex items-center justify-center"
    >
      <div className="absolute inset-0 bg-black/40 backdrop-blur-sm" />

      <div className="relative flex items-center justify-center">
        <div className="speaking-ring -mt-2" />
        <div className="speaking-ring delay-150 -mt-2" />
        <div className="speaking-ring delay-300 -mt-2" />
        <div className="h-16 w-16 rounded-full bg-destructive/90 flex items-center justify-center text-destructive-foreground shadow-2xl">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="currentColor"
            className="h-8 w-8"
          >
            <path d="M12 1a1 1 0 0 0-1 1v10a1 1 0 0 0 2 0V2a1 1 0 0 0-1-1z" />
            <path d="M7 6.5a1 1 0 0 0-1.6.8A7 7 0 0 0 9 16.9v1.6a4 4 0 0 0-2 3.5h10a4 4 0 0 0-2-3.5v-1.6a7 7 0 0 0 3.6-9.6 1 1 0 0 0-1.6-.8A5 5 0 0 1 12 12a5 5 0 0 1-5-5.5z" />
          </svg>
        </div>
      </div>
    </div>
  );
}

export default SpeakingOverlay;
