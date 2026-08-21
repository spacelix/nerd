import { Minus, Square, X } from "lucide-react";
import { cn } from "@/lib/utils";

interface WindowControlsProps {
  className?: string;
}

function WindowControls({ className }: WindowControlsProps) {
  const baseBtn =
    "inline-flex size-11 items-center justify-center text-muted-foreground transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring";

  return (
    <div
      role="group"
      aria-label="Window controls"
      className={cn("flex shrink-0 items-center", className)}
    >
      <button
        type="button"
        aria-label="Minimize"
        title="Minimize"
        className={cn(baseBtn, "hover:bg-foreground/10")}
      >
        <Minus className="size-3.5" />
      </button>
      <button
        type="button"
        aria-label="Maximize"
        title="Maximize"
        className={cn(baseBtn, "hover:bg-foreground/10")}
      >
        <Square className="size-3" />
      </button>
      <button
        type="button"
        aria-label="Close"
        title="Close"
        className={cn(
          baseBtn,
          "hover:bg-danger hover:text-danger-foreground",
        )}
      >
        <X className="size-4" />
      </button>
    </div>
  );
}

export { WindowControls };
