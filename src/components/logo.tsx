import whisprtyprLogo from "@/assets/logo.png";
import { cn } from "@/lib/utils";

interface LogoProps {
  size?: "sm" | "md" | "lg";
  showText?: boolean;
  className?: string;
}

export function Logo({ size = "md", showText = true, className }: LogoProps) {
  const sizes = {
    sm: { icon: "h-7 w-7", text: "text-lg" },
    md: { icon: "h-9 w-9", text: "text-xl" },
    lg: { icon: "h-11 w-11", text: "text-2xl" },
  };

  const s = sizes[size];

  return (
    <div className={cn("flex items-center gap-2.5", className)}>
      <img
        src={whisprtyprLogo}
        alt="WhisprTypr Logo"
        className={cn(s.icon, "object-contain")}
      />
      {showText && (
        <span className={cn(s.text, "font-semibold tracking-tight")}>
          WhisprTypr
        </span>
      )}
    </div>
  );
}
