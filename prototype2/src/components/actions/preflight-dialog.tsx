import { ShieldCheck } from "lucide-react";
import { cn } from "@/lib/utils";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import { useProjectActions } from "@/hooks/use-project-actions";
import { useProjectPreflight } from "@/hooks/use-project-preflight";
import { diagnosticProbes, projects, servicesByProject } from "@/mocks/data";
import type { Project } from "@/lib/types";

function PreflightRow({
  label,
  value,
  conflict = false,
}: {
  label: string;
  value: React.ReactNode;
  conflict?: boolean;
}) {
  return (
    <div className="flex items-start justify-between gap-3 py-1">
      <span className="shrink-0 text-[11px] text-muted-foreground">{label}</span>
      <span
        data-mono
        className={cn(
          "min-w-0 truncate text-right text-[11px]",
          conflict ? "text-warning" : "text-foreground",
        )}
      >
        {value}
      </span>
    </div>
  );
}

function PreflightDialog() {
  const preflight = useProjectPreflight();
  const actions = useProjectActions();
  const project: Project | undefined = projects.find(
    (p) => p.id === preflight.pendingId,
  );

  const portsProbe = diagnosticProbes.find((probe) => probe.id === "probe-ports");
  const services = project ? servicesByProject[project.id] ?? [] : [];
  const hasPortConflict =
    project?.id === "p-app" && portsProbe?.status === "fail";

  return (
    <Dialog open={preflight.pendingId !== null} onOpenChange={(open) => !open && preflight.cancel()}>
      <DialogContent className="max-w-md">
        <div className="flex flex-col gap-1">
          <h2 className="text-sm font-semibold tracking-tight text-foreground">
            Trust and start {project?.name ?? "project"}
          </h2>
          <p className="text-[11px] leading-relaxed text-muted-foreground">
            {project?.domain ?? ""} is not trusted yet. Review the preflight before
            this project's code runs for the first time.
          </p>
        </div>

        {project ? (
          <div className="flex flex-col rounded-md border border-border/50 bg-background/40 px-3 py-1.5">
            <PreflightRow label="Command" value={`${project.packageManager} ${project.command}`} />
            <PreflightRow
              label="Working dir"
              value={`${project.source}${project.source === "parked" ? "/app" : ""}`}
            />
            <PreflightRow
              label="Runtime"
              value={`${project.runtime} (${project.versionSource})`}
            />
            <PreflightRow label="Package manager" value={project.packageManager} />
            <PreflightRow
              label="Services"
              value={services.length > 0 ? services.map((s) => s.name).join(", ") : "none"}
            />
            <PreflightRow
              label="Port"
              value={project.port}
              conflict={hasPortConflict}
            />
            {hasPortConflict ? (
              <p
                role="alert"
                className="mt-1 rounded border border-warning/30 bg-warning-soft/60 px-2 py-1 text-[10px] leading-relaxed text-foreground/90"
              >
                Port {project.port} is occupied by PID 8124 (unmanaged process).
                Nerd never stops foreign processes.
              </p>
            ) : (
              <p className="mt-1 text-[10px] leading-relaxed text-muted-foreground/70">
                No environment conflicts detected for this project.
              </p>
            )}
          </div>
        ) : null}

        <p className="text-[10px] leading-relaxed text-muted-foreground/70">
          This project runs as the current user, never elevated. Trust binds to the
          stable project identity and canonical path; a material identity change
          requires trust again.
        </p>

        <div className="flex justify-end gap-2">
          <button
            type="button"
            onClick={preflight.cancel}
            className="rounded-md border border-border bg-background px-3 py-1.5 text-[11px] font-medium text-foreground transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={() => {
              if (!project) return;
              actions.trust(project.id);
              actions.setStatus(project.id, "running");
              preflight.cancel();
            }}
            className="inline-flex items-center gap-1.5 rounded-md bg-primary px-3 py-1.5 text-[11px] font-medium text-primary-foreground transition-opacity hover:opacity-90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            <ShieldCheck className="size-3.5" />
            Trust and start
          </button>
        </div>
      </DialogContent>
    </Dialog>
  );
}

export { PreflightDialog };