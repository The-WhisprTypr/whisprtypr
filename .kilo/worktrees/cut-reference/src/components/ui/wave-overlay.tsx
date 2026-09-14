import { cn } from "@/lib/utils";
import { useEffect, useState } from "react";

interface WaveOverlayProps {
  isActive: boolean;
  className?: string;
}

/**
 * Fullscreen wave animation overlay for recording indication
 * Shows animated waves around the screen edges when recording
 */
export function WaveOverlay({ isActive, className }: WaveOverlayProps) {
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    if (isActive) {
      setMounted(true);
    } else {
      // Delay unmount for exit animation
      const timer = setTimeout(() => setMounted(false), 300);
      return () => clearTimeout(timer);
    }
  }, [isActive]);

  if (!mounted) return null;

  return (
    <div
      className={cn(
        "fixed inset-0 pointer-events-none z-[9999] overflow-hidden transition-opacity duration-300",
        isActive ? "opacity-100" : "opacity-0",
        className
      )}
    >
      {/* Top edge wave */}
      <div className="absolute top-0 left-0 right-0 h-1">
        <div className="wave-line wave-line-horizontal" />
      </div>

      {/* Bottom edge wave */}
      <div className="absolute bottom-0 left-0 right-0 h-1">
        <div className="wave-line wave-line-horizontal wave-line-reverse" />
      </div>

      {/* Left edge wave */}
      <div className="absolute top-0 left-0 bottom-0 w-1">
        <div className="wave-line wave-line-vertical" />
      </div>

      {/* Right edge wave */}
      <div className="absolute top-0 right-0 bottom-0 w-1">
        <div className="wave-line wave-line-vertical wave-line-reverse" />
      </div>

      {/* Corner glows */}
      <div className="absolute top-0 left-0 w-24 h-24 corner-glow corner-glow-tl" />
      <div className="absolute top-0 right-0 w-24 h-24 corner-glow corner-glow-tr" />
      <div className="absolute bottom-0 left-0 w-24 h-24 corner-glow corner-glow-bl" />
      <div className="absolute bottom-0 right-0 w-24 h-24 corner-glow corner-glow-br" />

      {/* Pulsing border effect */}
      <div className="absolute inset-0 border-2 border-red-500/50 rounded-lg animate-pulse-border" />

      {/* Audio wave visualization in corners */}
      <div className="absolute top-2 left-2">
        <AudioWaveIcon />
      </div>
      <div className="absolute top-2 right-2">
        <AudioWaveIcon />
      </div>
      <div className="absolute bottom-2 left-2">
        <AudioWaveIcon />
      </div>
      <div className="absolute bottom-2 right-2">
        <AudioWaveIcon />
      </div>

      <style>{`
        .wave-line {
          position: absolute;
          background: linear-gradient(
            90deg,
            transparent 0%,
            #ef4444 20%,
            #f87171 50%,
            #ef4444 80%,
            transparent 100%
          );
        }

        .wave-line-horizontal {
          width: 200%;
          height: 100%;
          animation: wave-horizontal 2s linear infinite;
        }

        .wave-line-vertical {
          width: 100%;
          height: 200%;
          background: linear-gradient(
            180deg,
            transparent 0%,
            #ef4444 20%,
            #f87171 50%,
            #ef4444 80%,
            transparent 100%
          );
          animation: wave-vertical 2s linear infinite;
        }

        .wave-line-reverse {
          animation-direction: reverse;
        }

        @keyframes wave-horizontal {
          0% {
            transform: translateX(-50%);
          }
          100% {
            transform: translateX(0%);
          }
        }

        @keyframes wave-vertical {
          0% {
            transform: translateY(-50%);
          }
          100% {
            transform: translateY(0%);
          }
        }

        .corner-glow {
          background: radial-gradient(
            circle at center,
            rgba(239, 68, 68, 0.4) 0%,
            rgba(239, 68, 68, 0.1) 50%,
            transparent 70%
          );
          animation: pulse-glow 1.5s ease-in-out infinite;
        }

        .corner-glow-tl {
          transform-origin: top left;
        }
        .corner-glow-tr {
          transform-origin: top right;
        }
        .corner-glow-bl {
          transform-origin: bottom left;
        }
        .corner-glow-br {
          transform-origin: bottom right;
        }

        @keyframes pulse-glow {
          0%, 100% {
            opacity: 0.5;
            transform: scale(1);
          }
          50% {
            opacity: 1;
            transform: scale(1.2);
          }
        }

        .animate-pulse-border {
          animation: pulse-border 1.5s ease-in-out infinite;
        }

        @keyframes pulse-border {
          0%, 100% {
            border-color: rgba(239, 68, 68, 0.3);
            box-shadow: inset 0 0 20px rgba(239, 68, 68, 0.1);
          }
          50% {
            border-color: rgba(239, 68, 68, 0.6);
            box-shadow: inset 0 0 40px rgba(239, 68, 68, 0.2);
          }
        }
      `}</style>
    </div>
  );
}

/**
 * Small audio wave icon animation
 */
function AudioWaveIcon() {
  return (
    <div className="flex items-end gap-[2px] h-4">
      {[1, 2, 3, 4, 5].map((i) => (
        <div
          key={i}
          className="w-[3px] bg-red-500 rounded-full"
          style={{
            animation: `wave-bar 0.8s ease-in-out infinite`,
            animationDelay: `${i * 0.1}s`,
            height: "4px",
          }}
        />
      ))}
      <style>{`
        @keyframes wave-bar {
          0%, 100% {
            height: 4px;
          }
          50% {
            height: 16px;
          }
        }
      `}</style>
    </div>
  );
}

export default WaveOverlay;
