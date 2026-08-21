import {
  AlertTriangle,
  Download,
  Play,
  RefreshCw,
  Square,
  Upload,
} from "lucide-react";
import * as React from "react";
import { cn } from "@/lib/utils";
import { StatusDot } from "@/components/status/status-dot";
import { Switch } from "@/components/ui/switch";
import { projects } from "@/mocks/data";
import { useServiceActions } from "@/hooks/use-service-actions";
import type { Service } from "@/lib/types";

export type ServiceStatus = "running" | "stopped" | "degraded";

interface ServiceDetailProps {
  service: Service;
}

function ServiceDetail({ service }: ServiceDetailProps) {
  const actions = useServiceActions();
  const users = projects.filter((p) => service.projectIds.includes(p.id));
  const manageable = service.class === "managed" && !service.blockerId;
  const [keepRunning, setKeepRunning] = React.useState(false);
  const [feedback, setFeedback] = React.useState<string | null>(null);

  const runAction = (label: string): void => {
    setFeedback(`${label} complete`);
    window.setTimeout(() => setFeedback(null), 1600);
  };

  const liveStatus = actions.statusFor(service);

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3 px-4 py-3">
      <div className="flex items-center gap-2">
        <StatusDot
          tone={
            liveStatus === "running"
              ? "success"
              : liveStatus === "degraded"
                ? "warning"
                : "muted"
          }
          pulse={liveStatus === "starting"}
        />
        <span className="text-lg font-semibold text-foreground">
          {service.name}
        </span>
      </div>

      <div className="flex flex-col gap-1">
        <Row label="Engine" value={service.engine} />
        <Row label="Version" value={service.version} />
        <Row label="Port" value={`:${service.port}`} />
        <Row label="Status" value={liveStatus} />
        <Row
          label="Class"
          value={service.class === "managed" ? "managed" : "external"}
        />
      </div>

      {service.blockerId ? (
        <div className="flex items-start gap-2 rounded-md border border-warning/30 bg-warning-soft/60 px-2.5 py-2">
          <AlertTriangle className="mt-0.5 size-3.5 shrink-0 text-warning" />
          <p className="text-[11px] leading-relaxed text-foreground/90">
            <span data-mono className="font-medium text-warning">
              {service.blockerId}
            </span>
            <br />
            {service.blockerLabel}
          </p>
        </div>
      ) : null}

      {manageable ? (
        <div className="flex flex-col gap-2 rounded-md border border-border/50 bg-background/40 px-3 py-2.5">
          <div className="flex items-center justify-between gap-3">
            <span className="text-[11px] text-muted-foreground">
              Keep running after project stops
            </span>
            <Switch
              checked={keepRunning}
              onCheckedChange={setKeepRunning}
              label="keepRunning"
            />
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <button
              type="button"
              aria-label={liveStatus === "running" ? "Stop" : "Start"}
              onClick={() =>
                liveStatus === "running" || liveStatus === "starting"
                  ? actions.stop(service.id)
                  : actions.start(service.id)
              }
              className={cn(
                "inline-flex items-center gap-1.5 rounded-md px-2.5 py-1 text-[11px] font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                liveStatus === "running" || liveStatus === "starting"
                  ? "border border-border bg-background text-foreground hover:bg-accent"
                  : "bg-primary text-primary-foreground hover:opacity-90",
              )}
            >
              {liveStatus === "running" || liveStatus === "starting" ? (
                <>
                  <Square className="size-3" />
                  Stop
                </>
              ) : (
                <>
                  <Play className="size-3" />
                  Start
                </>
              )}
            </button>
            <button
              type="button"
              onClick={() => actions.restart(service.id)}
              className="inline-flex items-center gap-1.5 rounded-md border border-border bg-background px-2.5 py-1 text-[11px] font-medium text-foreground transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            >
              <RefreshCw className="size-3" />
              Restart
            </button>
            <button
              type="button"
              onClick={() => runAction("Backup")}
              className="inline-flex items-center gap-1.5 rounded-md border border-border bg-background px-2.5 py-1 text-[11px] font-medium text-foreground transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            >
              <Download className="size-3" />
              Backup
            </button>
            <button
              type="button"
              onClick={() => runAction("Restore")}
              className="inline-flex items-center gap-1.5 rounded-md border border-border bg-background px-2.5 py-1 text-[11px] font-medium text-foreground transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            >
              <Upload className="size-3" />
              Restore
            </button>
            {feedback ? (
              <span role="status" className="text-[10px] text-muted-foreground/70">
                {feedback}
              </span>
            ) : null}
          </div>
        </div>
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
                <span
                  data-mono
                  className="truncate text-[11px] text-foreground/90"
                >
                  {p.domain}
                </span>
                <span
                  data-mono
                  className="text-[10px] text-muted-foreground/60"
                >
                  {p.status}
                </span>
              </div>
            ))}
          </div>
        ) : (
          <p className="text-[11px] text-muted-foreground/60">
            Not bound to any project yet.
          </p>
        )}
      </div>

      <p
        data-mono
        className={cn(
          "mt-auto rounded-md border border-border/50 bg-background/40 px-2 py-1.5 text-[10px] text-muted-foreground/70",
        )}
      >
        {manageable
          ? "nerd-managed: process, dynamic port, credentials, and data stay project-scoped"
          : "lifecycle: read-only until OD decisions resolve"}
      </p>
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

export { ServiceDetail };