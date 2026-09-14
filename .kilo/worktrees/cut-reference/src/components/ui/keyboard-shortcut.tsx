/**
 * Keyboard shortcut display component
 * Renders keyboard shortcuts in a visually appealing way
 */

import { cn } from "@/lib/utils";

interface KeyboardShortcutProps {
  shortcut: string;
  className?: string;
  size?: "sm" | "md" | "lg";
}

export function KeyboardShortcut({
  shortcut,
  className,
  size = "md",
}: KeyboardShortcutProps) {
  // Parse the shortcut string (e.g., "Alt+Shift+S" -> ["Alt", "Shift", "S"])
  const keys = shortcut.split("+").map((key) => key.trim());

  const sizeClasses = {
    sm: "px-1.5 py-0.5 text-xs min-w-[1.25rem]",
    md: "px-2 py-1 text-xs min-w-[1.5rem]",
    lg: "px-2.5 py-1.5 text-sm min-w-[2rem]",
  };

  return (
    <div className={cn("flex items-center gap-1", className)}>
      {keys.map((key, index) => (
        <span key={index} className="flex items-center gap-1">
          <kbd
            className={cn(
              "inline-flex items-center justify-center rounded font-mono font-medium",
              "bg-muted border border-border shadow-sm",
              "text-muted-foreground",
              sizeClasses[size]
            )}
          >
            {formatKey(key)}
          </kbd>
          {index < keys.length - 1 && (
            <span className="text-muted-foreground text-xs">+</span>
          )}
        </span>
      ))}
    </div>
  );
}

/**
 * Format key names for display
 */
function formatKey(key: string): string {
  const keyMap: Record<string, string> = {
    Ctrl: "⌃",
    Control: "⌃",
    Alt: "⌥",
    Option: "⌥",
    Shift: "⇧",
    Meta: "⌘",
    Command: "⌘",
    Cmd: "⌘",
    Super: "❖",
    Space: "␣",
    Enter: "↵",
    Return: "↵",
    Backspace: "⌫",
    Delete: "⌦",
    Escape: "Esc",
    Tab: "⇥",
    ArrowUp: "↑",
    ArrowDown: "↓",
    ArrowLeft: "←",
    ArrowRight: "→",
  };

  // Check if we're on macOS (simplified check, works in most cases)
  const isMac = navigator.platform?.toLowerCase().includes("mac");

  // Use symbols on Mac, text elsewhere
  if (isMac && keyMap[key]) {
    return keyMap[key];
  }

  // Return the key as-is for letter keys, capitalize it
  return key.length === 1 ? key.toUpperCase() : key;
}

/**
 * Parse a hotkey string into an array of key names
 */
export function parseHotkey(hotkey: string): string[] {
  return hotkey.split("+").map((key) => key.trim());
}

/**
 * Format a hotkey for display (full text version)
 */
export function formatHotkeyText(hotkey: string): string {
  return hotkey
    .split("+")
    .map((key) => key.trim())
    .join(" + ");
}
