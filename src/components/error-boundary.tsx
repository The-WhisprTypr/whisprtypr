import { invoke } from "@tauri-apps/api/core";
import { AlertCircle, MessageSquare, RefreshCw } from "@/components/icons";
import { Component, ErrorInfo, ReactNode } from "react";
import { captureSentryException } from "@/lib/sentry";

interface Props {
  children: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  public state: State = {
    hasError: false,
    error: null,
  };

  public static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error("Uncaught error:", error, errorInfo);

    captureSentryException(error, {
      ...(errorInfo.componentStack ? { componentStack: errorInfo.componentStack } : {}),
    });

    try {
      invoke("report_error", {
        severity: "fatal",
        category: "ui",
        message: error.message || "Uncaught UI Error",
        details: JSON.stringify({
          stack: error.stack,
          componentStack: errorInfo.componentStack,
        }),
      }).catch(console.error);
    } catch (err) {
      console.error("Failed to report error to backend:", err);
    }
  }

  private handleRestart = () => {
    window.location.reload();
  };

  private handleReportIssue = async () => {
    try {
      const { openUrl } = await import("@tauri-apps/plugin-opener");
      await openUrl("https://github.com/The-WhisprTypr/whisprtypr/issues");
    } catch (err) {
      window.open("https://github.com/The-WhisprTypr/whisprtypr/issues", "_blank");
    }
  };

  public render() {
    if (this.state.hasError) {
      return (
        <div className="h-screen w-screen flex items-center justify-center p-6 bg-background text-foreground">
          <div className="glass-card max-w-md w-full p-8 rounded-3xl flex flex-col items-center text-center space-y-6 border border-white/10 shadow-2xl">
            <div className="w-16 h-16 rounded-2xl bg-red-500/10 flex items-center justify-center">
              <AlertCircle className="h-8 w-8 text-red-500" />
            </div>
            
            <div className="space-y-2">
              <h1 className="text-2xl font-bold tracking-tight">Something went wrong</h1>
              <p className="text-muted-foreground text-sm">
                The application encountered an unexpected error and needs to restart.
              </p>
            </div>

            {this.state.error && (
              <div className="w-full p-4 rounded-xl bg-black/5 dark:bg-white/5 border border-white/10 text-left overflow-hidden">
                <p className="text-xs font-mono text-muted-foreground break-all line-clamp-3">
                  {this.state.error.toString()}
                </p>
              </div>
            )}

            <div className="grid grid-cols-2 gap-3 w-full">
              <button
                onClick={this.handleRestart}
                className="flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl bg-foreground text-background font-medium hover:opacity-90 transition-all"
              >
                <RefreshCw className="h-4 w-4" />
                Restart
              </button>
              <button
                onClick={this.handleReportIssue}
                className="flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl bg-white/10 border border-white/10 font-medium hover:bg-white/20 transition-all"
              >
                <MessageSquare className="h-4 w-4" />
                Report
              </button>
            </div>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
