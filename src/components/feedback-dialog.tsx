import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { useToast } from "@/hooks/use-toast";
import { cn, openUrl } from "@/lib/utils";
import { ExternalLink, Loader2, MessageSquareHeart, Star } from "@/components/icons";
import { useEffect, useState } from "react";

interface FeedbackDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

const feedbackCategories = [
  { value: "general", label: "General feedback" },
  { value: "ideas", label: "Feature idea" },
  { value: "q-a", label: "Question / Help" },
];

const ratingLabels: Record<number, string> = {
  1: "Not great",
  2: "Could be better",
  3: "It's okay",
  4: "Pretty good",
  5: "Loving it",
};

const DISCUSSIONS_URL = "https://github.com/The-WhisprTypr/whisprtypr/discussions/new";

export function FeedbackDialog({ open, onOpenChange }: FeedbackDialogProps) {
  const { success: toastSuccess, error: toastError } = useToast();

  const [rating, setRating] = useState<number>(0);
  const [hoverRating, setHoverRating] = useState<number>(0);
  const [category, setCategory] = useState<string>("general");
  const [message, setMessage] = useState("");
  const [email, setEmail] = useState("");
  const [appVersion, setAppVersion] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);

  useEffect(() => {
    if (!open) {
      setRating(0);
      setHoverRating(0);
      setCategory("general");
      setMessage("");
      setEmail("");
      setAppVersion("");
      setIsSubmitting(false);
    }
  }, [open]);

  const buildDiscussionBody = () => {
    const selectedRating = rating > 0 ? `${"★".repeat(rating)}${"☆".repeat(5 - rating)} (${rating}/5)` : "Not provided";
    const categoryLabel = feedbackCategories.find((c) => c.value === category)?.label ?? category;
    const versionLine = appVersion ? `**WhisprTypr version:** ${appVersion}` : "";
    const emailLine = email ? `**Reply email (optional):** ${email}` : "";
    const messageBody = message.trim() || "_No additional message provided._";

    return [
      `**Category:** ${categoryLabel}`,
      `**Rating:** ${selectedRating}`,
      versionLine,
      emailLine,
      "",
      "### What I wanted to share",
      "",
      messageBody,
    ]
      .filter((line) => line !== null && line !== undefined)
      .join("\n");
  };

  const buildDiscussionTitle = () => {
    const trimmed = message.trim().split("\n")[0]?.slice(0, 80) ?? "";
    if (trimmed) return trimmed;
    return category === "ideas"
      ? "Feature idea from WhisprTypr user"
      : "Feedback from a WhisprTypr user";
  };

  const handleSubmit = async () => {
    if (!message.trim() && rating === 0) {
      toastError("Add a message or rating", "Tell us a bit before sending.");
      return;
    }

    try {
      setIsSubmitting(true);
      const params = new URLSearchParams({
        category,
        title: buildDiscussionTitle(),
        body: buildDiscussionBody(),
      });
      await openUrl(`${DISCUSSIONS_URL}?${params.toString()}`);
      toastSuccess("Opening GitHub Discussions", "Review and submit your post there.");
      onOpenChange(false);
    } catch (err) {
      console.error("Failed to open feedback URL:", err);
      toastError("Could not open GitHub", "Please try again or report a bug instead.");
    } finally {
      setIsSubmitting(false);
    }
  };

  const activeRating = hoverRating || rating;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="glass-card border-0 max-w-md">
        <DialogHeader>
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-xl bg-white/30 dark:bg-white/10">
              <MessageSquareHeart className="h-4 w-4 text-foreground/70" />
            </div>
            <div>
              <DialogTitle className="text-base">Send feedback</DialogTitle>
              <DialogDescription className="text-xs">
                Share what you think. Your message opens in a GitHub Discussion so
                you can review and submit it.
              </DialogDescription>
            </div>
          </div>
        </DialogHeader>

        <div className="space-y-4">
          <div className="space-y-2">
            <Label className="text-xs font-medium text-foreground/60 uppercase tracking-wider">
              How is WhisprTypr working for you?
            </Label>
            <div className="flex items-center gap-2">
              <div
                className="flex items-center gap-1"
                onMouseLeave={() => setHoverRating(0)}
              >
                {[1, 2, 3, 4, 5].map((value) => {
                  const filled = value <= activeRating;
                  return (
                    <button
                      key={value}
                      type="button"
                      onMouseEnter={() => setHoverRating(value)}
                      onClick={() => setRating(value)}
                      className="p-1 rounded-lg transition-colors hover:bg-white/30 dark:hover:bg-white/10"
                      aria-label={`Rate ${value} star${value === 1 ? "" : "s"}`}
                    >
                      <Star
                        className={cn(
                          "h-5 w-5 transition-colors",
                          filled
                            ? "fill-amber-400 text-amber-400"
                            : "text-foreground/40",
                        )}
                      />
                    </button>
                  );
                })}
              </div>
              <span className="text-xs text-foreground/60 min-w-[100px]">
                {activeRating > 0 ? ratingLabels[activeRating] : "Tap to rate"}
              </span>
            </div>
          </div>

          <div className="space-y-2">
            <Label className="text-xs font-medium text-foreground/60 uppercase tracking-wider">
              Category
            </Label>
            <Select value={category} onValueChange={setCategory}>
              <SelectTrigger className="glass-button border-0 h-10">
                <SelectValue />
              </SelectTrigger>
              <SelectContent className="glass-card border-0">
                {feedbackCategories.map((item) => (
                  <SelectItem key={item.value} value={item.value}>
                    {item.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-2">
            <Label
              htmlFor="feedback-message"
              className="text-xs font-medium text-foreground/60 uppercase tracking-wider"
            >
              Your feedback
            </Label>
            <Textarea
              id="feedback-message"
              value={message}
              onChange={(event) => setMessage(event.target.value)}
              placeholder="What worked, what didn't, what you'd love to see next..."
              className="glass-button border-0 min-h-28 resize-none"
              maxLength={2000}
            />
            <p className="text-[10px] text-foreground/50 text-right">
              {message.length}/2000
            </p>
          </div>

          <div className="space-y-2">
            <Label
              htmlFor="feedback-email"
              className="text-xs font-medium text-foreground/60 uppercase tracking-wider"
            >
              Reply email (optional)
            </Label>
            <input
              id="feedback-email"
              type="email"
              value={email}
              onChange={(event) => setEmail(event.target.value)}
              placeholder="you@example.com"
              className="glass-button border-0 h-10 w-full rounded-md px-3 text-sm outline-none placeholder:text-foreground/40"
            />
            <p className="text-[10px] text-foreground/50">
              Only included if you want a follow-up.
            </p>
          </div>
        </div>

        <DialogFooter className="gap-2 sm:gap-2">
          <Button
            type="button"
            variant="outline"
            onClick={() => onOpenChange(false)}
            className="glass-button border-0"
            disabled={isSubmitting}
          >
            Cancel
          </Button>
          <Button
            type="button"
            onClick={handleSubmit}
            disabled={isSubmitting}
            className="gap-2"
          >
            {isSubmitting ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <ExternalLink className="h-4 w-4" />
            )}
            Continue on GitHub
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}