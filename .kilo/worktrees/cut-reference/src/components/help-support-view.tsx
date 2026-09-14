import { FeedbackDialog } from "@/components/feedback-dialog";
import {
  ArrowRight,
  BookOpen,
  Bug,
  Circle,
  HelpCircle,
  MessageSquareHeart,
  RefreshCcw,
  Wrench
} from "lucide-react";
import { useState } from "react";

interface HelpItem {
  icon: React.ComponentType<React.SVGProps<SVGSVGElement>>;
  title: string;
  text: string;
}

interface SupportLink {
  icon: React.ComponentType<React.SVGProps<SVGSVGElement>>;
  title: string;
  text: string;
  url: string;
}

const quickFixes: HelpItem[] = [
  {
    icon: RefreshCcw,
    title: "Restart the application",
    text: "Restart WhisprTypr to resolve most temporary glitches and stuck states.",
  },
  {
    icon: Wrench,
    title: "Reset settings to defaults",
    text: "Restore all settings to their original state if something feels broken.",
  },
  {
    icon: Bug,
    title: "Check microphone permissions",
    text: "Ensure WhisprTypr has access to your microphone in system settings.",
  },
];

const supportLinks: SupportLink[] = [
  {
    icon: BookOpen,
    title: "Documentation",
    text: "Read the WhisprTypr documentation and guides.",
    url: "https://github.com/johuniq/whisprtypr#readme",
  },
  {
    icon: Bug,
    title: "Report a bug",
    text: "Submit a bug report to help us improve.",
    url: "https://github.com/johuniq/whisprtypr/issues",
  },
];

interface HelpSupportViewProps {
  onClose: () => void;
}

export function HelpSupportView(_props: HelpSupportViewProps) {
  const [feedbackOpen, setFeedbackOpen] = useState(false);

  return (
    <div className="flex h-full flex-col overflow-hidden bg-canvas">
      {/* ─── HEADER ─── */}
      <div className="shrink-0 border-b border-hairline">
        <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-4 sm:py-5">
          <p className="eyebrow-uppercase text-ink-mid">Help & Support</p>
          <h1
            className="font-display font-medium text-ink mt-2 text-[clamp(1.25rem,2.2vw,1.75rem)]"
            style={{ lineHeight: 1.1, letterSpacing: '-0.02em' }}
          >
            We've got your <span className="text-primary">back</span>.
          </h1>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto">
        <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-6 xl:py-10 space-y-6 xl:space-y-8">

          {/* ─── HERO STATUS BAND — Dark coffee-ink ─── */}
          <section className="hero-band-dark">
            <div className="grid grid-cols-1 @xl:grid-cols-[1.4fr_1fr] gap-6 @xl:gap-10 p-5 sm:p-7 @xl:p-10 items-start @xl:items-center">
              <div className="min-w-0">
                <p className="eyebrow-uppercase text-primary mb-3">
                  <span className="inline-flex items-center gap-2">
                    <Circle className="h-1.5 w-1.5 fill-primary text-primary" />
                    Quick help
                  </span>
                </p>
                <h2
                  className="font-display font-medium text-on-dark text-[clamp(1.375rem,2.6vw,2.125rem)]"
                  style={{ lineHeight: 1.05, letterSpacing: '-0.02em' }}
                >
                  Most issues are solved in <span className="text-primary">3 minutes</span>.
                </h2>
                <p className="body-md text-on-dark-soft mt-3 max-w-xl">
                  Try the quick fixes below first — they cover 90% of what we get asked about.
                </p>
              </div>

              <div className="product-ui-card-dark w-full">
                <div className="flex items-center gap-3 mb-5">
                  <div className="icon-plate-dark">
                    <MessageSquareHeart className="h-4 w-4 text-on-dark" />
                  </div>
                  <div className="min-w-0">
                    <p className="caption-strong text-on-dark">Need a human?</p>
                    <p className="caption text-on-dark-soft mt-1">Share feedback directly with us.</p>
                  </div>
                </div>
                <button
                  onClick={() => setFeedbackOpen(true)}
                  className="paper-button-primary w-full cursor-pointer"
                >
                  Send feedback
                </button>
              </div>
            </div>
          </section>

          {/* ─── QUICK FIXES — White surface with hairline-divided rows ─── */}
          <section className="paper-card">
            <div className="flex items-center gap-3 mb-5">
              <div className="icon-plate">
                <HelpCircle className="h-4 w-4" />
              </div>
              <div className="min-w-0">
                <p className="eyebrow-uppercase text-ink-mid">Quick fixes</p>
                <h3
                  className="font-display font-medium text-ink text-[clamp(1rem,1.6vw,1.25rem)] mt-1"
                  style={{ letterSpacing: '-0.015em' }}
                >
                  Common fixes that solve most issues
                </h3>
              </div>
            </div>

            <div className="divide-y divide-hairline-soft">
              {quickFixes.map((item, idx) => {
                const Icon = item.icon;
                return (
                  <div
                    key={item.title}
                    className="grid grid-cols-[auto_1fr] gap-4 sm:gap-5 items-start py-4 first:pt-0 last:pb-0"
                  >
                    <span className="caption-strong text-body-muted pt-1 shrink-0">
                      {String(idx + 1).padStart(2, "0")}
                    </span>
                    <div className="flex items-start gap-3 sm:gap-4 min-w-0">
                      <div className="icon-plate shrink-0">
                        <Icon className="h-4 w-4" />
                      </div>
                      <div className="min-w-0">
                        <h4
                          className="font-display font-medium text-ink text-[clamp(0.8125rem,1.1vw,0.9375rem)]"
                          style={{ letterSpacing: '-0.01em' }}
                        >
                          {item.title}
                        </h4>
                        <p className="body-sm text-body-muted mt-1.5 leading-relaxed">
                          {item.text}
                        </p>
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          </section>

          {/* ─── RESOURCES — White surface with link cards ─── */}
          <section className="paper-card">
            <div className="flex items-center gap-3 mb-5">
              <div className="icon-plate">
                <BookOpen className="h-4 w-4" />
              </div>
              <div className="min-w-0">
                <p className="eyebrow-uppercase text-ink-mid">Resources</p>
                <h3
                  className="font-display font-medium text-ink text-[clamp(1rem,1.6vw,1.25rem)] mt-1"
                  style={{ letterSpacing: '-0.015em' }}
                >
                  Documentation and community
                </h3>
              </div>
            </div>

            <div className="grid grid-cols-1 min-[520px]:grid-cols-2 gap-4">
              {supportLinks.map((item, idx) => {
                const Icon = item.icon;
                return (
                  <a
                    key={item.title}
                    href={item.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="group rounded-md border border-hairline bg-canvas-soft p-4 sm:p-5 transition-all hover:border-ink hover:bg-canvas hover:shadow-[0_8px_24px_-16px_rgba(32,21,21,0.2)]"
                  >
                    <div className="flex items-start justify-between gap-3 mb-4">
                      <div className="icon-plate">
                        <Icon className="h-4 w-4" />
                      </div>
                      <span className="caption-strong text-body-muted">
                        {String(idx + 1).padStart(2, "0")}
                      </span>
                    </div>
                    <h4
                      className="font-display font-medium text-ink text-[clamp(0.875rem,1.3vw,1rem)]"
                      style={{ letterSpacing: '-0.015em' }}
                    >
                      {item.title}
                    </h4>
                    <p className="body-sm text-body-muted mt-1.5">{item.text}</p>
                    <div className="flex items-center gap-1 mt-4 caption-strong text-ink group-hover:text-primary transition-colors">
                      Visit
                      <ArrowRight className="h-3.5 w-3.5 transition-transform group-hover:translate-x-0.5" />
                    </div>
                  </a>
                );
              })}
            </div>
          </section>
        </div>
      </div>

      <FeedbackDialog open={feedbackOpen} onOpenChange={setFeedbackOpen} />
    </div>
  );
}