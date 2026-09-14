import { cn } from "@/lib/utils";
import {
  Cpu,
  FileAudio,
  HelpCircle,
  History,
  Key,
  LayoutDashboard,
  Settings,
} from "lucide-react";

export type Page = "overview" | "history" | "models" | "transcribe" | "license" | "settings" | "help";

interface SidebarProps {
  currentPage: Page;
  onNavigate: (page: Page) => void;
  appName?: string;
}

const navItems: { id: Page; label: string; icon: React.ElementType }[] = [
  { id: "overview", label: "Overview", icon: LayoutDashboard },
  { id: "history", label: "History", icon: History },
  { id: "models", label: "Models", icon: Cpu },
  { id: "transcribe", label: "Transcribe", icon: FileAudio },
  { id: "license", label: "License", icon: Key },
  { id: "settings", label: "Settings", icon: Settings },
  { id: "help", label: "Help", icon: HelpCircle },
];

export function DashboardSidebar({ currentPage, onNavigate }: SidebarProps) {
  return (
    <aside className="flex h-full w-60 flex-col bg-canvas-soft border-r border-hairline">

      {/* Eyebrow / section label */}
      <div className="px-5 pt-3 pb-1 shrink-0">
        <p className="caption-strong text-body-mid-0-40" style={{ color: '#c5c0b1', fontSize: '0.625rem', letterSpacing: '0.08em' }}>
          Workspace
        </p>
      </div>

      {/* Navigation */}
      <nav className="flex-1 overflow-y-auto px-3 py-2 space-y-0.5">
        {navItems.map((item) => {
          const Icon = item.icon;
          const isActive = currentPage === item.id;
          return (
            <button
              key={item.id}
              onClick={() => onNavigate(item.id)}
              className={cn("sidebar-nav-item cursor-pointer", isActive && "active")}
            >
              <span className="nav-indicator" />
              <Icon className="h-4 w-4 shrink-0" strokeWidth={isActive ? 2.25 : 1.75} />
              <span className="truncate">{item.label}</span>
            </button>
          );
        })}
      </nav>

      {/* Footer — minimal one-liner */}
      <div className="shrink-0 px-5 py-4 border-t border-hairline-soft">
        <p className="caption text-body-muted leading-relaxed">
          Hold your hotkey
          <br />
          to start dictating.
        </p>
      </div>
    </aside>
  );
}