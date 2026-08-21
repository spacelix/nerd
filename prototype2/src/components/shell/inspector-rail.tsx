import { cn } from "@/lib/utils";

interface InspectorRailProps {
  className?: string;
  children?: React.ReactNode;
}

function InspectorRail({ className, children }: InspectorRailProps) {
  return (
    <aside
      data-slot="inspector-rail"
      aria-label="Contextual inspector"
      className={cn(
        "flex h-full w-full flex-col border-l border-border/60 bg-sidebar/30",
        className,
      )}
    >
      <div className="flex h-10 shrink-0 items-center justify-between gap-2 border-b border-border/60 px-3">
        <span className="text-[10px] font-semibold tracking-[0.16em] text-muted-foreground uppercase">
          Inspector
        </span>
        {children ? null : (
          <span data-mono className="text-[10px] text-muted-foreground/60">
            N3+
          </span>
        )}
      </div>
      {children ?? (
        <div className="flex flex-1 items-center justify-center p-6">
          <div className="max-w-[220px] text-center">
            <p
              data-mono
              className="text-[11px] tracking-wider text-muted-foreground/60 uppercase"
            >
              Contextual
            </p>
            <p className="mt-2 text-xs text-muted-foreground">
              Select a request, runtime, service, or diagnostic probe to inspect
              here. Project details live on their own page.
            </p>
          </div>
        </div>
      )}
    </aside>
  );
}

export { InspectorRail };