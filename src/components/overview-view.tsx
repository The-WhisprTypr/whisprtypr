import {
    Activity,
    ArrowRight,
    Circle,
    Cpu,
    Flame,
    Globe,
    Headphones,
    History,
    Keyboard,
    Loader2,
    Mic,
    RefreshCw,
    Share2,
    Sparkles,
    Timer,
    Type,
    Waves,
    Zap
} from "@/components/icons";
import { useToast } from "@/hooks/use-toast";
import { getLicense, type LicenseData } from "@/lib/license-api";
import { cn } from "@/lib/utils";
import { getTranscriptionHistory, getTranscriptionHistoryCount, type TranscriptionHistoryItem } from "@/lib/voice-api";
import { useAppStore } from "@/store";
import { useCallback, useEffect, useMemo, useState } from "react";

interface OverviewViewProps {
  onNavigate: (page: string) => void;
  trialDaysRemaining?: number;
}

// Format model name for display - converts cloud:provider:model to "Provider: Model"
function formatModelName(modelId: string): string {
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

export function OverviewView({ onNavigate, trialDaysRemaining }: OverviewViewProps) {
  const {
    settings,
    selectedModel,
    modelReady,
  } = useAppStore();
  const { success: toastSuccess } = useToast();

  const [history, setHistory] = useState<TranscriptionHistoryItem[]>([]);
  const [totalCount, setTotalCount] = useState(0);
  const [_license, setLicense] = useState<LicenseData | null>(null);
  const [isRefreshing, setIsRefreshing] = useState(false);

  const refreshData = useCallback(async () => {
    try {
      setIsRefreshing(true);
      const [items, count, licenseData] = await Promise.all([
        getTranscriptionHistory(1000, 0),
        getTranscriptionHistoryCount(),
        getLicense()
      ]);
      setHistory(items || []);
      setTotalCount(count || 0);
      setLicense(licenseData);
    } catch (err) {
      console.error("Failed to load overview data:", err);
    } finally {
      setIsRefreshing(false);
    }
  }, []);

  useEffect(() => {
    refreshData();
  }, [refreshData]);

  const stats = useMemo(() => {
    const now = new Date();
    const todayStr = now.toDateString();

    const transcriptionsToday = history.filter(item =>
      new Date(item.created_at).toDateString() === todayStr
    ).length;

    const oneWeekAgo = new Date();
    oneWeekAgo.setDate(now.getDate() - 7);
    const transcriptionsThisWeek = history.filter(item =>
      new Date(item.created_at) > oneWeekAgo
    ).length;

    const totalWords = history.reduce((acc, item) => {
      const text = item.text || "";
      return acc + (text.trim() ? text.trim().split(/\s+/).length : 0);
    }, 0);

    const avgWords = history.length > 0 ? Math.round(totalWords / history.length) : 0;

    const typingMinutes = totalWords / 40;
    const speakingMinutes = totalWords / 150;
    const minutesSaved = Math.max(0, Math.round(typingMinutes - speakingMinutes));

    const dayMap = new Set(history.map(item => new Date(item.created_at).toDateString()));
    const days = Array.from(dayMap).sort((a, b) => new Date(b).getTime() - new Date(a).getTime());

    let currentStreak = 0;
    let bestStreak = 0;
    let tempStreak = 0;

    let checkDate = new Date();
    if (!dayMap.has(checkDate.toDateString())) {
      checkDate.setDate(checkDate.getDate() - 1);
    }

    while (dayMap.has(checkDate.toDateString())) {
      currentStreak++;
      checkDate.setDate(checkDate.getDate() - 1);
    }

    if (days.length > 0) {
      let streakCheck = new Date(days[0]);
      tempStreak = 1;
      bestStreak = 1;

      for (let i = 1; i < days.length; i++) {
        const prevDay = new Date(streakCheck);
        prevDay.setDate(prevDay.getDate() - 1);
        const currentDay = new Date(days[i]);

        if (currentDay.toDateString() === prevDay.toDateString()) {
          tempStreak++;
        } else {
          tempStreak = 1;
        }
        bestStreak = Math.max(bestStreak, tempStreak);
        streakCheck = currentDay;
      }
    }

    const weeklyBars: { label: string; shortLabel: string; value: number; isToday: boolean; date: Date }[] = [];
    const shortDayLabels = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const minDayLabels = ["S", "M", "T", "W", "T", "F", "S"];
    for (let i = 6; i >= 0; i--) {
      const d = new Date();
      d.setDate(now.getDate() - i);
      const dateStr = d.toDateString();
      const value = history.filter(item => new Date(item.created_at).toDateString() === dateStr).length;
      weeklyBars.push({
        label: shortDayLabels[d.getDay()],
        shortLabel: minDayLabels[d.getDay()],
        value,
        isToday: i === 0,
        date: d,
      });
    }

    const recentActivity = history.slice(0, 5);

    return {
      transcriptionsToday,
      transcriptionsThisWeek,
      totalWords,
      avgWords,
      minutesSaved,
      streak: currentStreak,
      bestStreak: Math.max(bestStreak, currentStreak),
      recentActivity,
      weeklyBars,
    };
  }, [history]);

  const currentHotkey =
    settings.hotkeyMode === "push-to-talk"
      ? settings.pushToTalkKey
      : settings.toggleKey;

  const formatTime = (dateStr: string) => {
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
    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
  };

  const hasData = totalCount > 0;
  const timeSavedLabel = stats.minutesSaved > 60
    ? `${Math.floor(stats.minutesSaved / 60)}h ${stats.minutesSaved % 60}m`
    : `${stats.minutesSaved}m`;

  const weeklyMax = Math.max(1, ...stats.weeklyBars.map(b => b.value));

  const handleShareStats = useCallback(async () => {
    const snapshot = [
      "My Whisprtypr Weekly Stats",
      "",
      `${stats.transcriptionsThisWeek} transcriptions this week`,
      `${stats.totalWords.toLocaleString()} words spoken total`,
      `${timeSavedLabel} saved vs typing`,
      `${stats.streak} ${stats.streak === 1 ? "day" : "days"} streak`,
      "",
      "Speak. Type. Keep flowing.",
      "https://whisprtypr.app",
    ].join("\n");

    if (navigator.share) {
      try {
        await navigator.share({ title: "Whisprtypr Weekly Stats", text: snapshot });
      } catch (err) {
        console.warn("Share failed:", err);
      }
    } else {
      await navigator.clipboard.writeText(snapshot);
      toastSuccess("Stats copied to clipboard", "Share your weekly progress!");
    }
  }, [stats, timeSavedLabel, toastSuccess]);

  return (
    <div className="flex h-full flex-col overflow-y-auto bg-canvas">
      {/* Trial banner */}
      {trialDaysRemaining !== undefined && trialDaysRemaining > 0 && (
        <div className="px-4 pt-3 sm:px-6 sm:pt-4 xl:px-10 xl:pt-5">
          <div className="flex items-center justify-between gap-3 rounded-lg border border-hairline bg-canvas-soft px-4 py-2 sm:px-5">
            <div className="flex items-center gap-2.5 min-w-0">
              <span className="flex h-2 w-2 rounded-full bg-primary shrink-0" />
              <span className="body-sm text-ink truncate">
                {trialDaysRemaining} day{trialDaysRemaining !== 1 ? "s" : ""} left in your trial
              </span>
            </div>
            <button
              className="caption-strong text-ink hover:text-primary transition-colors whitespace-nowrap"
              onClick={() => onNavigate("license")}
            >
              Upgrade →
            </button>
          </div>
        </div>
      )}

      {/* Main container — fluid, max 1280, scales padding with window */}
      <div className="@container flex flex-col gap-4 sm:gap-5 xl:gap-6 p-3 sm:p-5 xl:p-6 max-w-[1280px] mx-auto w-full">

        {/* ─── HERO BAND ─── */}
        <section className="hero-band-dark">
          <div className="grid grid-cols-1 @xl:grid-cols-[1.4fr_1fr] gap-4 @xl:gap-6 p-4 sm:p-5 @xl:p-6 relative">
            {/* Left: editorial copy */}
            <div className="flex flex-col gap-3.5 @xl:gap-4 relative z-10 min-w-0">
              <div className="eyebrow-uppercase text-primary">
                <span className="inline-flex items-center gap-2">
                  {modelReady ? (
                    <span className="h-1.5 w-1.5 rounded-full bg-primary" />
                  ) : (
                    <Loader2 className="h-3 w-3 animate-spin" />
                  )}
                  {selectedModel?.downloaded
                    ? modelReady
                      ? "Ready"
                      : "Warming up"
                    : "Setup needed"}
                </span>
              </div>

              <h1 className="display-lg text-on-dark">
                {greeting()},{" "}
                <span className="text-primary">dictate</span> at the speed of thought.
              </h1>

              <p className="body-md text-on-dark-soft max-w-xl">
                {selectedModel?.downloaded
                  ? `${selectedModel.name} is loaded. Hold ${currentHotkey}, speak naturally, and watch your words land at the cursor.`
                  : "Pick a transcription model to get started — everything runs locally on your machine."}
              </p>

              <div className="flex flex-wrap items-center gap-2 pt-1">
                <button
                  onClick={() => onNavigate("transcribe")}
                  className="paper-button-primary size-md cursor-pointer"
                >
                  Start dictating
                </button>
                <button
                  onClick={() => onNavigate("models")}
                  className="paper-button-outline-dark size-md cursor-pointer"
                >
                  Browse models
                </button>
              </div>
            </div>

            {/* Right: floating waveform card (only when there's room) */}
            <div className="relative hidden @xl:flex items-center justify-center min-w-0">
              <div className="product-ui-card-dark w-full max-w-sm">
                <div className="flex items-center justify-between mb-3 gap-2">
                  <div className="flex items-center gap-2.5 min-w-0">
                    <div className="icon-plate-dark shrink-0">
                      <Headphones className="h-3.5 w-3.5 text-on-dark" />
                    </div>
                    <div className="min-w-0">
                      <p className="caption-strong text-on-dark">Live dictation</p>
                      <p className="caption text-on-dark-soft mt-0.5 truncate">{selectedModel?.name ?? "No model"}</p>
                    </div>
                  </div>
                  <span
                    className="badge-pill text-on-dark-soft border border-hairline-soft shrink-0"
                    style={{ background: '#2f2a26' }}
                  >
                    <span className="h-1.5 w-1.5 rounded-full animate-pulse" style={{ background: '#05b169' }} />
                    Idle
                  </span>
                </div>

                <div className="flex items-end justify-between h-14 gap-1 mb-3">
                  {[0.35, 0.7, 0.5, 0.85, 0.45, 0.6, 0.95, 0.55, 0.4, 0.75, 0.5, 0.65, 0.9, 0.45, 0.6, 0.8].map((h, i) => (
                    <div
                      key={i}
                      className="flex-1 rounded-full min-w-0"
                      style={{
                        height: `${h * 100}%`,
                        background: i % 4 === 0 ? '#ff4f00' : '#c5c0b1',
                        opacity: 0.4 + h * 0.6,
                      }}
                    />
                  ))}
                </div>

                <div className="flex items-center justify-between pt-3 border-t gap-2" style={{ borderColor: '#36342e' }}>
                  <div className="flex items-center gap-2 min-w-0">
                    <Keyboard className="h-3 w-3 text-on-dark-soft shrink-0" />
                    <span className="caption text-on-dark-soft truncate">{currentHotkey}</span>
                  </div>
                  <span className="caption text-on-dark-muted whitespace-nowrap">Press & hold</span>
                </div>
              </div>

              <div className="absolute -top-3 -right-3 bg-primary text-on-dark caption-strong rounded-md px-2.5 py-1 rotate-3 hidden @3xl:block">
                Beta
              </div>
            </div>
          </div>
        </section>

        {/* ─── ACTIVITY BAND — Cream surface with weekly chart ─── */}
        {hasData && (
          <section className="card-feature-cream relative overflow-hidden">
            {/* Decorative orange glow */}
            <div
              aria-hidden
              className="absolute -top-16 -right-16 h-56 w-56 pointer-events-none"
              style={{
                background: 'radial-gradient(circle, rgba(255,79,0,0.10), transparent 70%)',
              }}
            />

            <div className="relative grid grid-cols-1 @3xl:grid-cols-[minmax(0,1fr)_minmax(0,2fr)] gap-4 @3xl:gap-6">
              {/* LEFT — narrative */}
              <div className="flex flex-col gap-3 min-w-0">
                <div className="flex items-center gap-2 flex-wrap">
                  <span className="eyebrow-uppercase text-ink-mid">This week</span>
                  {stats.streak > 0 && (
                    <span
                      className="inline-flex items-center gap-1.5 caption-strong px-2 py-0.5 rounded-full"
                      style={{ background: 'rgba(255,79,0,0.1)', color: '#ff4f00' }}
                    >
                      <Flame className="h-3 w-3" />
                      {stats.streak}-day streak
                    </span>
                  )}
                </div>

                <h2 className="display-md text-ink">
                  <span className="text-primary">{stats.transcriptionsThisWeek}</span>{" "}
                  <span className="text-ink">transcriptions</span>
                </h2>

                <p className="body-sm text-body-muted max-w-sm">
                  {stats.transcriptionsToday > 0
                    ? `${stats.transcriptionsToday} done today — keep the momentum going.`
                    : "Start one today to keep your streak alive."}
                </p>
              </div>

              {/* RIGHT — Chart */}
              <div className="flex flex-col gap-2 min-w-0">
                {/* Chart header */}
                <div className="flex items-center justify-between gap-2 flex-wrap">
                  <p className="caption-strong text-ink-mid">Last 7 days</p>
                  <div className="flex items-center gap-2 caption text-body-muted">
                    <LegendDot color="bg-primary" label="Today" />
                    <LegendDot color="bg-ink" label="Past" />
                  </div>
                </div>

                {/* Chart canvas with grid lines */}
                <div className="relative pt-5">
                  {/* Horizontal grid lines */}
                  <div className="absolute inset-x-0 top-5 bottom-5 flex flex-col justify-between pointer-events-none" aria-hidden>
                    {[0, 0.25, 0.5, 0.75, 1].map((p) => (
                      <div key={p} className="flex items-center gap-2">
                        <span className="caption text-body-mid-0-50 w-6 text-right tabular-nums" style={{ fontSize: '0.5625rem', color: '#c5c0b1' }}>
                          {Math.round(weeklyMax * (1 - p))}
                        </span>
                        <span className="flex-1 h-px" style={{ background: p === 1 ? 'transparent' : '#ece5d6' }} />
                      </div>
                    ))}
                  </div>

                  {/* Bars */}
                  <div className="flex items-end justify-between gap-1.5 sm:gap-2 h-28 sm:h-32 relative z-10 px-1">
                    {stats.weeklyBars.map((bar, i) => {
                      const heightPct = Math.max(4, (bar.value / weeklyMax) * 100);
                      const hasValue = bar.value > 0;
                      return (
                        <div key={i} className="group flex flex-col items-center justify-end gap-1.5 flex-1 min-w-0 relative h-full">
                          {/* Value tooltip on top */}
                          <div
                            className={cn(
                              "absolute -top-1 left-1/2 -translate-x-1/2 -translate-y-full",
                              "px-1.5 py-0.5 rounded-md caption-strong whitespace-nowrap",
                              "transition-opacity pointer-events-none",
                              bar.isToday
                                ? "opacity-100 text-on-dark"
                                : "opacity-0 group-hover:opacity-100 text-ink",
                            )}
                            style={bar.isToday ? { background: '#ff4f00' } : { background: '#201515', color: '#fffefb' }}
                          >
                            {bar.value}
                          </div>

                          {/* Bar */}
                          <div
                            className={cn(
                              "w-full rounded-md transition-all min-w-[10px] cursor-pointer relative overflow-hidden",
                              bar.isToday ? "bg-primary" : "bg-ink",
                            )}
                            style={{
                              height: `${heightPct}%`,
                              opacity: !hasValue ? 0.15 : bar.isToday ? 1 : 0.75 + (bar.value / weeklyMax) * 0.25,
                            }}
                          >
                            {/* subtle shine on active bar */}
                            {bar.isToday && (
                              <div
                                className="absolute inset-x-0 top-0 h-1/3"
                                style={{
                                  background: 'linear-gradient(180deg, rgba(255,255,255,0.25), transparent)',
                                }}
                              />
                            )}
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </div>

                {/* Day labels */}
                <div className="flex items-center justify-between gap-1.5 sm:gap-2 px-1 mt-0.5">
                  {stats.weeklyBars.map((bar, i) => (
                    <div key={i} className="flex flex-col items-center gap-0.5 flex-1 min-w-0">
                      <span className={cn(
                        "caption-strong whitespace-nowrap",
                        bar.isToday ? "text-primary" : "text-body-muted",
                      )}>
                        <span className="hidden sm:inline">{bar.label}</span>
                        <span className="inline sm:hidden">{bar.shortLabel}</span>
                      </span>
                      <span className="caption text-body-mid-0-50 tabular-nums" style={{ fontSize: '0.5625rem', color: '#c5c0b1' }}>
                        {bar.date.getDate()}
                      </span>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </section>
        )}

        {/* ─── STATS GRID ─── */}
        {hasData && (
          <section className="flex flex-col gap-3 sm:gap-4">
            <div className="flex items-end justify-between gap-2 flex-wrap">
              <div className="min-w-0">
                <p className="eyebrow-uppercase text-ink-mid">By the numbers</p>
                <h2
                  className="display-sm text-ink mt-1"
                >
                  Your activity, decoded.
                </h2>
              </div>
              <div className="flex items-end gap-2">
                <button
                  onClick={refreshData}
                  disabled={isRefreshing}
                  aria-label="Refresh data"
                  className="flex h-8 w-8 items-center justify-center rounded-md border border-ink bg-canvas text-ink hover:bg-canvas-soft transition-colors disabled:opacity-50 shrink-0"
                >
                  <RefreshCw className={cn("h-3.5 w-3.5", isRefreshing && "animate-spin")} />
                </button>
                <button
                  onClick={handleShareStats}
                  aria-label="Share weekly stats"
                  className="flex h-8 w-8 items-center justify-center rounded-md border border-ink bg-canvas text-ink hover:bg-canvas-soft transition-colors shrink-0"
                >
                  <Share2 className="h-3.5 w-3.5" />
                </button>
              </div>
            </div>

            <div className="grid grid-cols-1 min-[520px]:grid-cols-2 @3xl:grid-cols-4 gap-3 sm:gap-4">
              <StatCard
                icon={<History className="h-3.5 w-3.5" />}
                label="Transcriptions"
                value={totalCount.toLocaleString()}
                subtext={`${stats.transcriptionsToday} today`}
                accent
              />
              <StatCard
                icon={<Type className="h-3.5 w-3.5" />}
                label="Words spoken"
                value={stats.totalWords.toLocaleString()}
                subtext={`${stats.avgWords} avg per session`}
                accent
              />
              <StatCard
                icon={<Timer className="h-3.5 w-3.5" />}
                label="Time saved"
                value={timeSavedLabel}
                subtext="vs typing by hand"
                accent
              />
              <StatCard
                icon={<Flame className="h-3.5 w-3.5" />}
                label="Current streak"
                value={`${stats.streak} ${stats.streak === 1 ? "day" : "days"}`}
                subtext={stats.bestStreak > 0 ? `Personal best: ${stats.bestStreak}` : "Build momentum"}
                accent
              />
            </div>
          </section>
        )}

        {/* ─── MAIN CONTENT — Recent activity + side rail ─── */}
        <section className="grid grid-cols-1 @3xl:grid-cols-3 gap-3 sm:gap-4">
          {/* Recent Activity */}
          <div className="@3xl:col-span-2 paper-card">
            <div className="flex items-end justify-between mb-3 sm:mb-4 gap-2 flex-wrap">
              <div className="min-w-0">
                <p className="eyebrow-uppercase text-ink-mid mb-1">Recent</p>
                <h2
                  className="display-sm text-ink"
                >
                  Latest transcriptions
                </h2>
              </div>
              {hasData && (
                <button
                  onClick={() => onNavigate("history")}
                  className="caption-strong text-ink hover:text-primary transition-colors flex items-center gap-1 whitespace-nowrap"
                >
                  View all
                  <ArrowRight className="h-3 w-3" />
                </button>
              )}
            </div>

            {hasData ? (
              <div className="divide-y divide-hairline-soft">
                {stats.recentActivity.map((item, idx) => (
                  <div
                    key={item.id}
                    className="py-2.5 sm:py-3 first:pt-0 last:pb-0 grid grid-cols-[auto_1fr] min-[640px]:grid-cols-[auto_1fr_auto] gap-2 sm:gap-3 items-start"
                  >
                    <span className="caption-strong text-body-muted pt-1">
                      {String(idx + 1).padStart(2, "0")}
                    </span>
                    <div className="min-w-0">
                      <p
                        className="text-ink leading-relaxed line-clamp-2"
                        style={{ fontSize: 'var(--type-body-sm)' }}
                      >
                        {item.text || "Empty transcription"}
                      </p>
                      <div className="flex items-center gap-2 sm:gap-2.5 mt-1.5 caption text-body-muted flex-wrap">
                        <span className="flex items-center gap-1">
                          <Cpu className="h-2.5 w-2.5" />
                          {formatModelName(item.model_id)}
                        </span>
                        <span className="h-1 w-1 rounded-full bg-body-mid" />
                        <span>
                          {item.text ? item.text.trim().split(/\s+/).length : 0} words
                        </span>
                      </div>
                    </div>
                    <span className="caption text-body-mid pt-1 whitespace-nowrap hidden min-[640px]:block">
                      {formatTime(item.created_at)}
                    </span>
                  </div>
                ))}
              </div>
            ) : (
              <div className="flex flex-col items-center justify-center py-10 sm:py-12 text-center">
                <div className="icon-plate-orange mb-3">
                  <Mic className="h-4 w-4" />
                </div>
                <p
                  className="display-xs text-ink"
                >
                  Nothing dictated yet
                </p>
                <p className="body-sm text-body-muted mt-1.5 max-w-xs">
                  Hold {currentHotkey} and start speaking — your first transcription will appear here.
                </p>
                <button
                  onClick={() => onNavigate("transcribe")}
                  className="paper-button-primary size-md mt-4 sm:mt-5 cursor-pointer"
                >
                  Try it now
                </button>
              </div>
            )}
          </div>

          {/* Side rail — hidden on minimized windows, shown only at full screen */}
          <div className="hidden @3xl:flex flex-col gap-3 sm:gap-4 min-w-0">
            <button
              onClick={() => onNavigate("transcribe")}
              className="group paper-card text-left transition-all hover:border-ink"
            >
              <div className="flex items-center justify-between mb-3 sm:mb-4">
                <div className="icon-plate-orange">
                  <Zap className="h-4 w-4" />
                </div>
                <ArrowRight className="h-3.5 w-3.5 text-body-muted group-hover:text-primary group-hover:translate-x-1 transition-all" />
              </div>
              <p className="eyebrow-uppercase text-ink-mid mb-2">Quick action</p>
              <p
                className="display-md text-ink"
              >
                Start<br />dictating
              </p>
              <p className="body-sm text-body-muted mt-2 sm:mt-2.5">
                Open the live transcription surface and test your loaded model.
              </p>
            </button>

            <div className="card-feature-cream">
              <div className="flex items-center gap-2.5 mb-3 sm:mb-4">
                <div className="icon-plate">
                  <Sparkles className="h-3.5 w-3.5" />
                </div>
                <h3 className="title-md text-ink">Setup</h3>
              </div>
              <ul className="space-y-1">
                <SetupRow
                  icon={<Keyboard className="h-3 w-3" />}
                  label="Hotkey"
                  value={currentHotkey}
                  hint={settings.hotkeyMode === "push-to-talk" ? "Push to talk" : "Toggle"}
                />
                {settings.translationEnabled && (
                  <SetupRow
                    icon={<Globe className="h-3 w-3" />}
                    label="Translation"
                    value={settings.translationHotkey}
                    hint={`${settings.translationSourceLanguage.toUpperCase()} → ${settings.translationTargetLanguage.toUpperCase()}`}
                  />
                )}
                <SetupRow
                  icon={selectedModel?.isCloud ? <Zap className="h-3 w-3 text-primary" /> : <Cpu className="h-3 w-3" />}
                  label="Model"
                  value={selectedModel?.name ?? "Not configured"}
                  hint={selectedModel?.isCloud ? "Cloud (BYOK)" : selectedModel?.downloaded ? "Ready" : "Needs download"}
                  onClick={() => onNavigate("models")}
                />
                <SetupRow
                  icon={<Activity className="h-3 w-3" />}
                  label="Latency / Gain"
                  value={selectedModel?.latencyEstimate ? selectedModel.latencyEstimate : "~3x"}
                  hint={selectedModel?.isCloud ? "Cloud latency" : "vs typing"}
                />
                <SetupRow
                  icon={<Waves className="h-3 w-3" />}
                  label="Processing"
                  value={selectedModel?.isCloud ? "Cloud BYOK" : "Local"}
                  hint={selectedModel?.isCloud ? "Direct encrypted" : "On-device only"}
                />
              </ul>
            </div>
          </div>
        </section>

        {/* ─── FOOTER NOTE ─── */}
        <section className="hero-band-dark">
          <div className="grid grid-cols-1 @xl:grid-cols-[1.5fr_1fr] gap-4 sm:gap-5 items-center p-4 sm:p-5 @xl:p-6">
            <div className="min-w-0">
              <p className="eyebrow-uppercase text-primary mb-2">
                <span className="inline-flex items-center gap-2">
                  <Circle className="h-1.5 w-1.5 fill-primary text-primary" />
                  Privacy-first
                </span>
              </p>
              <h3
                className="display-sm text-on-dark"
              >
                Your voice never leaves your machine.
              </h3>
              <p className="body-sm text-on-dark-soft mt-2 max-w-xl">
                Every transcription runs through your selected local model. No audio uploads, no cloud dependency, no telemetry required for the core workflow.
              </p>
            </div>
            <div className="flex flex-wrap gap-2 @xl:justify-end">
              <button
                onClick={() => onNavigate("settings")}
                className="paper-button-outline-dark size-md cursor-pointer"
              >
                Open settings
              </button>
              <button
                onClick={() => onNavigate("models")}
                className="paper-button-primary size-md cursor-pointer"
              >
                Manage models
              </button>
            </div>
          </div>
        </section>
      </div>
    </div>
  );
}

function greeting() {
  const h = new Date().getHours();
  if (h < 5) return "Up late";
  if (h < 12) return "Good morning";
  if (h < 18) return "Good afternoon";
  return "Good evening";
}

function LegendDot({ color, label }: { color: string; label: string }) {
  return (
    <span className="inline-flex items-center gap-1.5">
      <span className={cn("h-2 w-2 rounded-full", color)} />
      <span>{label}</span>
    </span>
  );
}

function StatCard({
  icon,
  label,
  value,
  subtext,
  accent,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
  subtext?: string;
  accent?: boolean;
}) {
  return (
    <div className="stat-card">
      <div className="flex items-center justify-between mb-3 sm:mb-4 gap-2">
        <p className="eyebrow-uppercase text-ink-mid truncate">{label}</p>
        <div className={cn("icon-plate shrink-0", accent && "!bg-primary/10 !text-primary")}>
          {icon}
        </div>
      </div>
      <p
        className={cn(
          "display-md tabular-nums tracking-[-0.02em]",
          accent ? "text-primary" : "text-ink",
        )}
      >
        {value}
      </p>
      {subtext && <p className="caption text-body-muted mt-2">{subtext}</p>}
    </div>
  );
}

function SetupRow({
  icon,
  label,
  value,
  hint,
  onClick,
}: {
  icon?: React.ReactNode;
  label: string;
  value: string;
  hint?: string;
  onClick?: () => void;
}) {
  const content = (
    <>
      <div className="flex items-center gap-2 min-w-0">
        {icon && <span className="text-body-muted shrink-0">{icon}</span>}
        <span className="body-sm text-body-muted truncate">{label}</span>
      </div>
      <div className="text-right min-w-0 shrink-0">
        <p className="body-sm-strong text-ink leading-tight truncate">{value}</p>
        {hint && <p className="caption text-body-mid mt-0.5">{hint}</p>}
      </div>
    </>
  );

  if (onClick) {
    return (
      <li>
        <button
          onClick={onClick}
          className="flex w-full items-center justify-between gap-2 rounded-md px-2.5 py-2 -mx-1 hover:bg-canvas transition-colors text-left"
        >
          {content}
        </button>
      </li>
    );
  }

  return <li className="flex items-center justify-between gap-2 px-2.5 py-2 -mx-1">{content}</li>;
}