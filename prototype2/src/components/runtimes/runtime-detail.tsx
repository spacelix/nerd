import { Star } from "lucide-react";
import { cn } from "@/lib/utils";
import { projects } from "@/mocks/data";
import type { Runtime, RuntimeClass } from "@/lib/types";

const classBadge: Record<RuntimeClass, string> = {
  managed: "border-success/40 text-success",
  external: "border-border/60 text-muted-foreground",
  degraded: "border-warning/40 text-warning",
};

interface RuntimeDetailProps {
  runtime: Runtime;
}

function RuntimeDetail({ runtime }: RuntimeDetailProps) {
  const users = projects.filter(
    (p) => p.runtime === `node-${runtime.version}`,
  );
  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3 px-4 py-3">
      <div className="flex items-center gap-2">
        <span
          data-mono
          className="text-lg font-semibold text-foreground"
        >
          v{runtime.version}
        </span>
        <span
          data-mono
          className={cn(
            "rounded border px-1.5 py-0.5 text-[10px]",
            classBadge[runtime.class],
          )}
        >
          {runtime.class}
        </span>
      </div>

      <div className="flex flex-col gap-1">
        <Row label="Default" value={runtime.isDefault ? "yes" : "no"} />
        <Row label="Usage" value={`${runtime.usageCount} projects`} />
      </div>

      {runtime.isDefault ? (
        <p className="flex items-center gap-1.5 rounded-md border border-warning/30 bg-warning-soft/60 px-2 py-1.5 text-[11px] text-foreground/90">
          <Star className="size-3 shrink-0 fill-warning text-warning" />
          Default runtime — new projects use this version.
        </p>
      ) : null}

      <div className="flex flex-col gap-1.5">
        <span className="text-[10px] font-semibold tracking-[0.16em] text-muted-foreground/70 uppercase">
          Used by
        </span>
        {users.length > 0 ? (
          <div className="flex flex-col gap-1">
            {users.map((p) => (
              <div
                key={p.id}
                className="flex items-center justify-between rounded-md border border-border/50 bg-background/40 px-2 py-1.5"
              >
                <span data-mono className="truncate text-[11px] text-foreground/90">
                  {p.domain}
                </span>
                <span
                  data-mono
                  className="text-[10px] text-muted-foreground/60"
                >
                  :{p.port}
                </span>
              </div>
            ))}
          </div>
        ) : (
          <p className="text-[11px] text-muted-foreground/60">
            No projects pinned to this version.
          </p>
        )}
      </div>
    </div>
  );
}

function Row({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <div className="grid grid-cols-[4.5rem_minmax(0,1fr)] items-baseline gap-2 py-0.5">
      <span className="text-[11px] text-muted-foreground/70">{label}</span>
      <span className="text-xs text-foreground/90">{value}</span>
    </div>
  );
}

export { RuntimeDetail };