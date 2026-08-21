import { cn } from "@/lib/utils";

export type WorkingSession = "active" | "all" | "background";

interface WorkingSessionToggleProps {
  value: WorkingSession;
  onValueChange: (next: WorkingSession) => void;
  className?: string;
}

const options: ReadonlyArray<{ value: WorkingSession; label: string }> = [
  { value: "active", label: "Active" },
  { value: "all", label: "All" },
  { value: "background", label: "Background" },
];

function WorkingSessionToggle({
  value,
  onValueChange,
  className,
}: WorkingSessionToggleProps) {
  return (
    <div
      role="group"
      aria-label="Working session filter"
      className={cn(
        "inline-flex h-7 items-center gap-0.5 rounded-md border border-border bg-muted/40 p-0.5",
        className,
      )}
    >
      {options.map((opt) => {
        const isActive = opt.value === value;
        return (
          <button
            key={opt.value}
            type="button"
            aria-pressed={isActive}
            onClick={() => onValueChange(opt.value)}
            className={cn(
              "inline-flex h-6 flex-1 items-center justify-center rounded px-2 text-[11px] font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
              isActive
                ? "bg-background text-foreground shadow-sm"
                : "text-muted-foreground hover:text-foreground",
            )}
          >
            {opt.label}
          </button>
        );
      })}
    </div>
  );
}

export { WorkingSessionToggle };
