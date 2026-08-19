import { CheckCircle2, CircleAlert, FolderTree, Plus, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { RUNTIMES } from "@/mocks/projects";
import type { Runtime } from "@/lib/types";

type Tone = "managed" | "external" | "degraded";

const TONE: Record<Tone, { label: string; variant: "default" | "secondary" | "destructive" }> = {
  managed: { label: "managed", variant: "default" },
  external: { label: "external", variant: "secondary" },
  degraded: { label: "degraded", variant: "destructive" },
};

function RuntimeRow({
  runtime,
  usageByRuntimeId,
}: {
  runtime: Runtime;
  usageByRuntimeId: Record<string, number>;
}) {
  const tone = TONE[runtime.ownership];
  const used = usageByRuntimeId[runtime.id] ?? 0;
  return (
    <li className="grid h-14 grid-cols-[100px_minmax(0,1fr)_120px_120px_auto] items-center gap-4 border-b border-border/40 px-2 text-sm transition-colors hover:bg-accent/30">
      <Badge variant={tone.variant}>{tone.label}</Badge>
      <span className="flex items-baseline gap-2 truncate">
        <span className="font-mono text-[13px] tabular-nums">
          {runtime.version}
        </span>
        <span className="text-[10px] uppercase tracking-[0.1em] text-muted-foreground">
          {runtime.channel}
        </span>
        {runtime.default && (
          <span className="rounded-md bg-accent-soft px-1.5 py-px text-[10px] font-semibold uppercase tracking-[0.08em] text-accent">
            Default
          </span>
        )}
      </span>
      <span className="font-mono text-xs tabular-nums text-muted-foreground">
        {runtime.id === "node-system" ? "system" : "managed"}
      </span>
      <span className="font-mono text-xs tabular-nums text-muted-foreground">
        {used} {used === 1 ? "project" : "projects"}
      </span>
      <div className="flex items-center gap-1">
        {!runtime.default && runtime.ownership === "managed" && (
          <Button variant="ghost" size="sm">
            Set default
          </Button>
        )}
        {runtime.ownership === "external" && (
          <Button variant="ghost" size="sm">
            Update ref
          </Button>
        )}
        <Button variant="ghost" size="icon" aria-label="Remove">
          <Trash2 className="h-3.5 w-3.5" />
        </Button>
      </div>
    </li>
  );
}

export function RuntimesScreen() {
  const managedCount = RUNTIMES.filter((r) => r.ownership === "managed").length;
  const externalCount = RUNTIMES.filter((r) => r.ownership === "external").length;
  const defaultRuntime = RUNTIMES.find((r) => r.default);

  const usageByRuntimeId: Record<string, number> = {};
  for (const id of ["node-22", "node-20", "node-24", "node-system"]) {
    usageByRuntimeId[id] = 0;
  }
  usageByRuntimeId["node-22"] = 4;
  usageByRuntimeId["node-20"] = 3;
  usageByRuntimeId["node-24"] = 1;

  return (
    <div className="h-full overflow-auto">
      <div className="flex flex-col gap-8 p-8">
        <div className="flex flex-wrap items-center justify-end gap-2">
          <span className="mr-auto font-mono text-xs tabular-nums text-muted-foreground">
            {RUNTIMES.length} installed
          </span>
          <Button variant="ghost" size="sm">
            Register external
          </Button>
          <Button variant="default" size="sm">
            <Plus className="h-3.5 w-3.5" />
            Install Node
          </Button>
        </div>

        {defaultRuntime && (
          <div className="flex items-start gap-3 rounded-md border border-border/40 bg-accent-soft/40 px-4 py-3">
            <CheckCircle2 className="mt-0.5 h-4 w-4 shrink-0 text-success" />
            <div className="text-sm">
              <p className="font-medium text-foreground">
                Default runtime: Node {defaultRuntime.version}
              </p>
              <p className="mt-0.5 text-xs text-muted-foreground">
                Projects without an explicit version will use this Node
                installation. {managedCount} managed, {externalCount}{" "}
                external.
              </p>
            </div>
          </div>
        )}

        {externalCount > 0 && (
          <div className="flex items-start gap-3 rounded-md border border-border/40 bg-warning-soft/40 px-4 py-3">
            <CircleAlert className="mt-0.5 h-4 w-4 shrink-0 text-warning" />
            <div className="text-sm">
              <p className="font-medium text-foreground">
                External runtimes are read-only
              </p>
              <p className="mt-0.5 text-xs text-muted-foreground">
                Nerd never repairs, updates, or uninstalls external Node
                installations. Register or install a managed version to
                take ownership.
              </p>
            </div>
          </div>
        )}

        <section>
          <header className="mb-3 flex items-baseline justify-between">
            <h2 className="text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
              Installed
            </h2>
            <span className="font-mono text-[10px] tabular-nums text-muted-foreground">
              {RUNTIMES.length}
            </span>
          </header>
          <ul className="overflow-hidden rounded-md border border-border/40 bg-surface">
            <li className="grid h-9 grid-cols-[100px_minmax(0,1fr)_120px_120px_auto] items-center gap-4 border-b border-border/40 bg-muted/30 px-2 text-[10px] font-semibold uppercase tracking-[0.1em] text-muted-foreground">
              <span>Source</span>
                <span>Version</span>
                <span>Channel</span>
                <span>Usage</span>
                <span className="w-32 text-right">Actions</span>
              </li>
              {RUNTIMES.map((runtime) => (
                <RuntimeRow
                  key={runtime.id}
                  runtime={runtime}
                  usageByRuntimeId={usageByRuntimeId}
                />
              ))}
            </ul>
          </section>

          <section>
            <header className="mb-3 flex items-baseline justify-between">
              <h2 className="text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
                External references
              </h2>
              <Button variant="ghost" size="sm">
                <FolderTree className="h-3.5 w-3.5" />
                Register another
              </Button>
            </header>
            <div className="rounded-md border border-border/40 bg-surface p-6 text-sm text-muted-foreground">
              <p>
                External runtimes are detected by Nerd but never mutated. Use
                Register to add a system Node installation to the registry
                without taking ownership.
              </p>
            </div>
          </section>
        </div>
      </div>
    );
}
