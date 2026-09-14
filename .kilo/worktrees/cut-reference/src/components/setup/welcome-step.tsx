import { Logo } from "@/components/logo";
import { Globe, Keyboard, Mic, Zap } from "lucide-react";

interface WelcomeStepProps {
  onNext: () => void;
}

const features = [
  {
    icon: Mic,
    title: "Voice to cursor",
    description: "Speak and watch your words appear at the cursor.",
  },
  {
    icon: Keyboard,
    title: "Universal",
    description: "Works in any app — docs, messages, code editors.",
  },
  {
    icon: Zap,
    title: "Fast & private",
    description: "On-device AI. Your audio never leaves your machine.",
  },
  {
    icon: Globe,
    title: "Multi-language",
    description: "99+ languages, including technical vocabulary.",
  },
];

export function WelcomeStep({ onNext }: WelcomeStepProps) {
  return (
    <div className="flex h-full flex-col overflow-hidden bg-canvas">
      <div className="flex-1 overflow-y-auto">
        <div className="@container max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-6 xl:py-10">
          {/* HERO — Dark coffee-ink band with editorial copy */}
          <section className="hero-band-dark mb-6">
            <div className="flex flex-col items-center text-center gap-5 p-8 sm:p-10">
              <Logo size="md" />
              <p
                className="font-display font-medium text-on-dark text-[clamp(1.625rem,3vw,2.5rem)]"
                style={{ lineHeight: 1.05, letterSpacing: "-0.02em" }}
              >
                Welcome to <span className="text-primary">WhisprTypr</span>.
              </p>
              <p
                className="body-lg text-on-dark-soft max-w-md"
                style={{ lineHeight: 1.5 }}
              >
                Your voice, at your cursor. Set up takes about 2 minutes.
              </p>
            </div>
          </section>

          {/* FEATURE GRID — Cream surface 2x2 */}
          <section className="card-feature-cream">
            <div className="flex items-center gap-3 mb-5">
              <div className="icon-plate-orange">
                <Zap className="h-4 w-4" />
              </div>
              <div className="min-w-0 flex-1">
                <p className="eyebrow-uppercase text-ink-mid">Why WhisprTypr</p>
                <h3
                  className="font-display font-medium text-ink text-[clamp(1.0625rem,1.5vw,1.25rem)] mt-1"
                  style={{ letterSpacing: "-0.015em" }}
                >
                  Built for real workflows
                </h3>
              </div>
            </div>

            <div className="grid grid-cols-1 min-[520px]:grid-cols-2 gap-4">
              {features.map((feature) => {
                const Icon = feature.icon;
                return (
                  <div
                    key={feature.title}
                    className="rounded-md border border-hairline p-4 transition-colors hover:border-ink"
                    style={{ background: "#fffefb" }}
                  >
                    <div className="flex items-center gap-3">
                      <div className="icon-plate shrink-0">
                        <Icon className="h-4 w-4" />
                      </div>
                      <h4
                        className="font-display font-medium text-ink text-[clamp(0.9375rem,1.2vw,1.0625rem)]"
                        style={{ letterSpacing: "-0.015em" }}
                      >
                        {feature.title}
                      </h4>
                    </div>
                    <p className="body-sm text-body-muted mt-3 leading-relaxed">
                      {feature.description}
                    </p>
                  </div>
                );
              })}
            </div>
          </section>
        </div>
      </div>

      {/* STICKY ACTION FOOTER */}
      <div className="shrink-0 border-t border-hairline bg-canvas-soft">
        <div className="max-w-[1280px] mx-auto w-full px-4 sm:px-6 xl:px-10 py-4 flex items-center justify-between gap-4 flex-wrap">
          <p className="caption text-body-muted">
            Takes about 2 minutes to set up.
          </p>
          <button
            onClick={onNext}
            className="paper-button-primary cursor-pointer"
          >
            Get started
          </button>
        </div>
      </div>
    </div>
  );
}