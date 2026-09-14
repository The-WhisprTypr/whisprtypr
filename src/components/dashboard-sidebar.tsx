import {
  BookText,
  Cpu,
  FileAudio,
  HelpCircle,
  History,
  Key,
  LayoutDashboard,
  Mic,
  RefreshCw,
  Settings,
  Sliders,
  Sparkles,
} from "@/components/icons";
import { useToast } from "@/hooks/use-toast";
import { checkForUpdates, getCurrentVersion } from "@/lib/updater-api";
import { cn } from "@/lib/utils";
import { useEffect, useState } from "react";

export type Page = "overview" | "history" | "models" | "transcribe" | "license" | "settings" | "advanced" | "vocabulary" | "help" | "ai-formatting";

type NavItem =
  | { type: "item"; id: Page; label: string; icon: React.ElementType }
  | { type: "header"; label: string };

const navItems: NavItem[] = [
  { type: "header", label: "General" },
  { type: "item", id: "overview", label: "Overview", icon: LayoutDashboard },
  { type: "item", id: "history", label: "History", icon: History },
  { type: "header", label: "Transcription" },
  { type: "item", id: "models", label: "Models", icon: Cpu },
  { type: "item", id: "transcribe", label: "Transcribe", icon: FileAudio },
  { type: "header", label: "Account" },
  { type: "item", id: "license", label: "License", icon: Key },
  { type: "header", label: "Configuration" },
  { type: "item", id: "settings", label: "Settings", icon: Settings },
  { type: "item", id: "advanced", label: "Advanced", icon: Sliders },
  { type: "item", id: "vocabulary", label: "Vocabulary", icon: BookText },
  { type: "item", id: "ai-formatting", label: "AI Formatting", icon: Sparkles },
  { type: "header", label: "Support" },
  { type: "item", id: "help", label: "Help", icon: HelpCircle },
];

interface SidebarProps {
  currentPage: Page;
  onNavigate: (page: Page) => void;
  appName?: string;
}

export function DashboardSidebar({ currentPage, onNavigate }: SidebarProps) {
  const [version, setVersion] = useState("");
  const [isCheckingUpdate, setIsCheckingUpdate] = useState(false);
  const { success: toastSuccess, error: toastError } = useToast();

  useEffect(() => {
    getCurrentVersion()
      .then(setVersion)
      .catch(() => {
        setVersion("?");
      });
  }, []);

  const handleCheckUpdate = async () => {
    setIsCheckingUpdate(true);
    try {
      const result = await checkForUpdates();
      if (result.status === "available") {
        toastSuccess(`Update ${result.info.version} available!`);
      } else if (result.status === "not-available") {
        toastSuccess("You're on the latest version");
      } else if (result.status === "error") {
        toastError(result.message);
      }
    } catch {
      toastError("Failed to check for updates");
    } finally {
      setIsCheckingUpdate(false);
    }
  };

  return (
    <aside className="flex h-full w-52 flex-col bg-canvas-soft border-r border-hairline">

      {/* Navigation */}
      <nav className="flex-1 overflow-y-auto px-2.5 py-1.5 space-y-0.5">
        {navItems.map((item, idx) => {
          if (item.type === "header") {
            return (
              <p
                key={`header-${idx}`}
                className="px-2.5 pt-3 pb-1 font-medium tracking-[0.08em] uppercase text-[9px] text-body-muted"
              >
                {item.label}
              </p>
            );
          }

          const Icon = item.icon;
          const isActive = currentPage === item.id;
          return (
            <button
              key={item.id}
              onClick={() => onNavigate(item.id)}
              className={cn(
                "flex items-center gap-2.5 w-full px-2.5 py-1.5 rounded-lg text-[13px] font-medium text-body bg-transparent border-none cursor-pointer transition-colors text-left tracking-[-0.005em] hover:text-ink",
                isActive && "text-ink bg-canvas shadow-xs"
              )}
            >
              <span
                className={cn(
                  "w-[3px] h-4 rounded-full bg-transparent -ml-2 transition-colors",
                  isActive && "bg-primary"
                )}
              />
              <Icon className="h-4 w-4 shrink-0" strokeWidth={isActive ? 2.25 : 1.75} />
              <span className="truncate text-base">{item.label}</span>
            </button>
          );
        })}
      </nav>

      {/* Footer — minimal one-liner */}
      <div className="shrink-0 px-4 py-3 border-t border-hairline-soft">
        <div className="flex items-center gap-2 mt-2 justify-between">
          <span className="font-medium text-body-muted text-sm">
            {version ? `v${version}` : "v..."}
          </span>
          <button
            onClick={handleCheckUpdate}
            disabled={isCheckingUpdate}
            className="text-body-muted hover:text-ink transition-colors disabled:opacity-50 cursor-pointer"
            aria-label="Check for updates"
          >
            <RefreshCw className={`h-4 w-4 ${isCheckingUpdate ? "animate-spin" : ""}`} />
          </button>
        </div>
      </div>
    </aside>
  );
}