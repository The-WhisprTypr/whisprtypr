/**
 * Toast notification hook using Sonner (shadcn/ui)
 * Production-grade notification system for WhisprTypr
 */

import { toast as sonnerToast } from "sonner";

export interface ToastOptions {
  description?: string;
  duration?: number;
  action?: {
    label: string;
    onClick: () => void;
  };
}

export function useToast() {
  const toast = (title: string, options?: ToastOptions) => {
    return sonnerToast(title, {
      description: options?.description,
      duration: options?.duration ?? 5000,
      action: options?.action
        ? {
            label: options.action.label,
            onClick: options.action.onClick,
          }
        : undefined,
    });
  };

  const success = (title: string, description?: string) => {
    return sonnerToast.success(title, {
      description,
      duration: 5000,
    });
  };

  const error = (title: string, description?: string) => {
    return sonnerToast.error(title, {
      description,
      duration: 7000,
    });
  };

  const warning = (title: string, description?: string) => {
    return sonnerToast.warning(title, {
      description,
      duration: 6000,
    });
  };

  const info = (title: string, description?: string) => {
    return sonnerToast.info(title, {
      description,
      duration: 5000,
    });
  };

  const loading = (title: string, description?: string) => {
    return sonnerToast.loading(title, {
      description,
    });
  };

  const dismiss = (toastId?: string | number) => {
    sonnerToast.dismiss(toastId);
  };

  const promise = <T>(
    promiseArg: Promise<T>,
    options: {
      loading: string;
      success: string | ((data: T) => string);
      error: string | ((err: unknown) => string);
    }
  ) => {
    return sonnerToast.promise(promiseArg, options);
  };

  return {
    toast,
    success,
    error,
    warning,
    info,
    loading,
    dismiss,
    promise,
  };
}

// Re-export for direct usage
export { sonnerToast as toast };
