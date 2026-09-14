import {
  CircleCheckIcon,
  InfoIcon,
  Loader2Icon,
  OctagonXIcon,
  TriangleAlertIcon,
} from "lucide-react";
import { Toaster as Sonner, type ToasterProps } from "sonner";

const Toaster = ({ ...props }: ToasterProps) => {
  return (
    <Sonner
      theme="light"
      className="toaster group"
      icons={{
        success: <CircleCheckIcon className="size-4" />,
        info: <InfoIcon className="size-4" />,
        warning: <TriangleAlertIcon className="size-4" />,
        error: <OctagonXIcon className="size-4" />,
        loading: <Loader2Icon className="size-4 animate-spin" />,
      }}
      style={
        {
          "--normal-bg": "var(--canvas)",
          "--normal-text": "var(--ink)",
          "--normal-border": "var(--hairline)",
          "--border-radius": "12px",
        } as React.CSSProperties
      }
      toastOptions={{
        classNames: {
          toast:
            "group toast group-[.toaster]:bg-canvas group-[.toaster]:text-ink group-[.toaster]:border-hairline group-[.toaster]:shadow-[0_8px_28px_-8px_rgba(32,21,21,0.18),0_2px_6px_-2px_rgba(32,21,21,0.08)] group-[.toaster]:rounded-md group-[.toaster]:p-4 group-[.toaster]:min-w-[320px] group-[.toaster]:max-w-[420px]",
          description: "group-[.toast]:text-body-muted group-[.toast]:text-[13px] group-[.toast]:leading-snug group-[.toast]:mt-1",
          title: "group-[.toast]:font-display group-[.toast]:font-medium group-[.toast]:text-ink group-[.toast]:text-[14px] group-[.toast]:tracking-[-0.01em]",
          actionButton:
            "group-[.toast]:bg-primary group-[.toast]:text-on-dark group-[.toast]:font-medium group-[.toast]:rounded-md group-[.toast]:px-3 group-[.toast]:h-7 group-[.toast]:text-[12px]",
          cancelButton:
            "group-[.toast]:bg-canvas-soft group-[.toast]:text-ink group-[.toast]:rounded-md group-[.toast]:px-3 group-[.toast]:h-7 group-[.toast]:text-[12px]",
          icon: "group-[.toast]:text-ink-mid group-[.toast]:shrink-0",
          success:
            "group-[.toaster]:bg-canvas group-[.toaster]:border-hairline",
          error:
            "group-[.toaster]:bg-canvas group-[.toaster]:border-hairline",
          info: "group-[.toaster]:bg-canvas group-[.toaster]:border-hairline",
          warning:
            "group-[.toaster]:bg-canvas group-[.toaster]:border-hairline",
        },
      }}
      {...props}
    />
  );
};

export { Toaster };
