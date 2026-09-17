import { Globe, Keyboard, Mic, Zap } from "@/components/icons";
import { Logo } from "@/components/logo";

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
              <p className="text-[28px] font-medium leading-tight tracking-tight text-on-dark">
                Welcome to <span className="text-primary">Whisprtypr</span>.
              </p>
              <p className="text-[15px] leading-relaxed text-on-dark-soft max-w-md">
                Your voice, at your cursor. Set up takes about 2 minutes.
              </p>
            </div>
          </section>

          {/* FEATURE GRID — Cream surface 2x2 */}
          <section className="card-feature-cream">
            <div className="flex items-center gap-3 mb-5">
              <div className="flex items-center justify-center size-8 rounded-full bg-primary/12 text-primary">
                <Zap className="h-4 w-4" />
              </div>
              <div className="min-w-0 flex-1">
                <p className="text-[11px] font-medium tracking-[0.1em] uppercase leading-none text-ink-mid">Why Whisprtypr</p>
                <h3 className="text-[15px] font-semibold tracking-tight text-ink mt-1">
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
                    className="rounded-md border border-hairline bg-canvas p-4 transition-colors hover:border-ink"
                  >
                    <div className="flex items-center gap-3">
                      <div className="flex items-center justify-center size-8 rounded-full bg-canvas-soft text-ink shrink-0">
                        <Icon className="h-4 w-4" />
                      </div>
                      <h4 className="text-xs font-semibold text-ink">
                        {feature.title}
                      </h4>
                    </div>
                    <p className="text-xs text-body-muted mt-3 leading-relaxed">
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
          <p className="text-[11px] text-body-muted">
            Takes about 2 minutes to set up.
          </p>
          <button
            onClick={onNext}
            className="inline-flex items-center justify-center gap-2 bg-primary text-on-dark font-semibold text-sm px-[1.125rem] py-2 rounded-[10px] cursor-pointer transition-all hover:bg-[#e64500] active:scale-[0.98] whitespace-nowrap min-h-9"
          >
            Get started
          </button>
        </div>
      </div>
    </div>
  );
}