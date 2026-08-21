import { ArrowRight, FolderKanban, Mail, Server, X } from "lucide-react";
import * as React from "react";
import { cn } from "@/lib/utils";
import { StatusDot } from "@/components/status/status-dot";
import { WorkingSessionToggle } from "@/components/shell/working-session-toggle";
import { useDaemonStream } from "@/hooks/use-daemon-stream";
import { useProjectActions } from "@/hooks/use-project-actions";
import { mail, operations, projects, requestDetails, services } from "@/mocks/data";
import type { OperationState, Project, ProjectStatus } from "@/lib/types";
import type { WorkingSession } from "@/components/shell/working-session-toggle";

const operationTone: Record<OperationState, "success" | "info" | "warning" | "danger" | "muted"> = {
  running: "info",
  done: "success",
  failed: "danger",
  cancelled: "muted",
};

const projectTone: Record<
  ProjectStatus,
  "success" | "warning" | "danger" | "muted"
> = {
  running: "success",
  starting: "warning",
  degraded: "warning",
  stopped: "muted",
  failed: "danger",
  crashed: "danger",
};

function statusClass(status: number): string {
  if (status >= 500) return "text-danger";
  if (status >= 400) return "text-warning";
  if (status === 304) return "text-info";
  return "text-success";
}

function matchesSession(p: Project, session: WorkingSession): boolean {
  if (session === "all") return true;
  if (session === "active") return p.pinned;
  return !p.pinned;
}

const daemonStateMeta: Record<
  "running" | "absent" | "protocol-mismatch" | "unhealthy",
  { label: string; tone: "success" | "warning" | "danger" }
> = {
  running: { label: "running", tone: "success" },
  absent: { label: "absent", tone: "danger" },
  "protocol-mismatch": { label: "protocol mismatch", tone: "warning" },
  unhealthy: { label: "unhealthy", tone: "warning" },
};

function DaemonStateBadge({
  state,
}: {
  state: "running" | "absent" | "protocol-mismatch" | "unhealthy";
}) {
  const meta = daemonStateMeta[state];
  const toneClass =
    meta.tone === "success"
      ? "border-success/40 text-success"
      : meta.tone === "warning"
        ? "border-warning/40 text-warning"
        : "border-danger/40 text-danger";
  return (
    <span
      role="status"
      aria-atomic="true"
      data-mono
      className={cn(
        "rounded border bg-background/40 px-1.5 py-0.5 text-[10px]",
        toneClass,
      )}
    >
      daemon:{meta.label}
    </span>
  );
}

function Stat({
  label,
  value,
  className,
}: {
  label: string;
  value: string;
  className?: string;
}) {
  return (
    <div className="flex flex-col gap-0.5">
      <span className="text-[10px] font-semibold tracking-[0.16em] text-muted-foreground/70 uppercase">
        {label}
      </span>
      <span
        data-mono
        className={cn("text-lg font-medium text-foreground", className)}
      >
        {value}
      </span>
    </div>
  );
}

interface OverviewScreenProps {
  workingSession: WorkingSession;
  onWorkingSessionChange: (next: WorkingSession) => void;
  onOpenProject: (id: string) => void;
  onViewAllProjects: () => void;
}

function OverviewScreen({
  workingSession,
  onWorkingSessionChange,
  onOpenProject,
  onViewAllProjects,
}: OverviewScreenProps) {
  const snapshot = useDaemonStream();
  const actions = useProjectActions();
  const runningCount = projects.filter(
    (p) => !actions.isRemoved(p.id) && actions.statusFor(p) === "running",
  ).length;
  const servicesRunning = services.filter(
    (s) => s.status === "running",
  ).length;
  const mailUnread = mail.filter((m) => m.unread).length;
  const activeProjects = React.useMemo(
    () =>
      projects.filter(
        (p) => !actions.isRemoved(p.id) && matchesSession(p, workingSession),
      ),
    [workingSession, actions],
  );
  const recent = React.useMemo(
    () =>
      [...requestDetails]
        .sort((a, b) => b.startedAt.localeCompare(a.startedAt))
        .slice(0, 5),
    [],
  );
  const [ops, setOps] = React.useState(operations);
  const cancelOperation = (id: string): void => {
    setOps((prev) =>
      prev.map((op) =>
        op.id === id ? { ...op, state: "cancelled" as const } : op,
      ),
    );
  };

  return (
    <div className="flex min-h-full flex-col gap-5 px-10 py-10">
      <header className="flex flex-col gap-2">
        <span
          data-mono
          className="text-[11px] tracking-[0.18em] text-muted-foreground/60 uppercase"
        >
          Route · overview · N6
        </span>
        <h1 className="text-xl font-semibold tracking-tight text-foreground">
          Overview
        </h1>
        <p className="max-w-xl text-xs text-muted-foreground">
          Daemon health, live metrics, and your working session at a glance.
        </p>
      </header>

      <section className="flex flex-col gap-3 rounded-lg border border-border/60 bg-card/40 p-4">
        <div className="flex flex-wrap items-center gap-x-2 gap-y-1">
          <StatusDot
            tone={snapshot.daemon.connected ? "success" : "danger"}
            pulse={snapshot.daemon.connected}
          />
          <span
            role="status"
            aria-atomic="true"
            className="text-sm font-semibold text-foreground"
          >
            Daemon {snapshot.daemon.connected ? "healthy" : "unreachable"}
          </span>
          <span data-mono className="text-xs text-muted-foreground">
            {snapshot.daemon.version}
          </span>
          <span
            data-mono
            className="rounded border border-border/50 bg-background/40 px-1.5 py-0.5 text-[10px] text-muted-foreground"
          >
            ipc:{snapshot.daemon.ipc}
          </span>
          <DaemonStateBadge state={snapshot.daemon.state} />
          <span data-mono className="text-[11px] text-muted-foreground/60">
            {snapshot.project.name} · :{snapshot.project.port} ·{" "}
            {snapshot.project.runtime}
          </span>
        </div>
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
          <Stat label="Uptime" value="2d 04h" />
          <Stat label="Requests" value={String(snapshot.requests)} />
          <Stat label="Services" value={`${servicesRunning}/${services.length}`} />
          <Stat label="Mail unread" value={String(mailUnread)} />
        </div>
      </section>

      <section className="grid grid-cols-2 gap-3 md:grid-cols-4">
        <Metric
          icon={<FolderKanban className="size-4 text-primary" />}
          label="Projects running"
          value={`${runningCount}/${projects.length}`}
        />
        <Metric
          icon={<Server className="size-4 text-info" />}
          label="Services"
          value={`${servicesRunning} running`}
        />
        <Metric
          icon={<Mail className="size-4 text-warning" />}
          label="Unread mail"
          value={String(mailUnread)}
        />
        <Metric
          icon={<span className="text-lg leading-none text-success">⌁</span>}
          label="Default runtime"
          value="v22.11.0"
        />
      </section>

      <div className="grid min-h-0 flex-1 grid-cols-1 gap-3 lg:grid-cols-2">
        <section className="flex flex-col gap-2">
          <div className="flex items-center justify-between gap-2">
            <span className="text-[10px] font-semibold tracking-[0.16em] text-muted-foreground/70 uppercase">
              Active projects
            </span>
            <WorkingSessionToggle
              value={workingSession}
              onValueChange={onWorkingSessionChange}
            />
          </div>
          <div className="flex flex-col gap-1.5">
            {activeProjects.map((p) => {
              const status = actions.statusFor(p);
              return (
                <button
                  key={p.id}
                  type="button"
                  onClick={() => onOpenProject(p.id)}
                  className="group flex items-center gap-2.5 rounded-lg border border-border/60 bg-card/40 px-3 py-2 text-left transition-colors hover:border-border focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                >
                  <StatusDot tone={projectTone[status]} />
                  <span className="min-w-0 flex-1">
                    <span className="block text-xs font-medium text-foreground">
                      {p.name}
                    </span>
                    <span
                      data-mono
                      className="block truncate text-[11px] text-muted-foreground"
                    >
                      {p.domain} · :{p.port}
                    </span>
                  </span>
                  <span
                    data-mono
                    className="text-[11px] text-muted-foreground/80"
                  >
                    {status}
                  </span>
                  <ArrowRight className="size-3.5 text-muted-foreground/40 transition-transform group-hover:translate-x-0.5 group-hover:text-foreground" />
                </button>
              );
            })}
            <button
              type="button"
              onClick={onViewAllProjects}
              className="rounded-lg border border-dashed border-border/60 px-3 py-1.5 text-[11px] text-muted-foreground transition-colors hover:border-border hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            >
              View all projects
            </button>
          </div>
        </section>

        <section className="flex flex-col gap-2">
          <span className="text-[10px] font-semibold tracking-[0.16em] text-muted-foreground/70 uppercase">
            Recent requests
          </span>
          <div className="flex flex-col gap-1.5">
            {recent.map((r) => (
              <div
                key={r.id}
                className="flex items-center gap-2.5 rounded-lg border border-border/60 bg-card/40 px-3 py-2"
              >
                <span
                  data-mono
                  className={cn("text-xs font-medium", statusClass(r.status))}
                >
                  {r.status}
                </span>
                <span data-mono className="w-10 shrink-0 text-[11px] text-muted-foreground">
                  {r.method}
                </span>
                <span className="min-w-0 flex-1 truncate text-xs text-foreground/90">
                  {r.url}
                </span>
                <span
                  data-mono
                  className="text-[11px] text-muted-foreground/60"
                >
                  {r.durationMs}ms
                </span>
              </div>
            ))}
          </div>
        </section>
      </div>

      <section className="flex flex-col gap-2">
        <span className="text-[10px] font-semibold tracking-[0.16em] text-muted-foreground/70 uppercase">
          Recent operations
        </span>
        <div className="flex flex-col gap-1.5">
          {ops.map((op) => (
            <div
              key={op.id}
              role={op.state === "failed" ? "alert" : undefined}
              className="flex items-center gap-2.5 rounded-lg border border-border/60 bg-card/40 px-3 py-2"
            >
              <StatusDot tone={operationTone[op.state]} pulse={op.state === "running"} />
              <span className="min-w-0 flex-1">
                <span className="block text-xs font-medium text-foreground/90">
                  {op.label}
                </span>
                <span
                  data-mono
                  className="block truncate text-[11px] text-muted-foreground"
                >
                  {op.id} · {new Date(op.startedAt).toLocaleString([], {
                    dateStyle: "medium",
                    timeStyle: "short",
                  })}
                </span>
              </span>
              <div className="w-24">
                <div
                  role="progressbar"
                  aria-valuenow={op.state === "cancelled" ? 0 : op.progress}
                  aria-valuemin={0}
                  aria-valuemax={100}
                  aria-label={`${op.label} progress`}
                  className="h-1 overflow-hidden rounded-full bg-foreground/10"
                >
                  <div
                    className={cn(
                      "h-full rounded-full transition-all",
                      op.state === "failed"
                        ? "bg-danger"
                        : op.state === "done"
                          ? "bg-success"
                          : op.state === "cancelled"
                            ? "bg-muted-foreground/40"
                            : "bg-primary",
                    )}
                    style={{ width: `${op.state === "cancelled" ? 0 : op.progress}%` }}
                  />
                </div>
              </div>
              <span
                data-mono
                className="w-16 shrink-0 text-right text-[11px] text-muted-foreground/60"
              >
                {op.state === "cancelled" ? "cancelled" : `${op.progress}%`}
              </span>
              {op.state === "running" ? (
                <button
                  type="button"
                  aria-label={`Cancel ${op.label}`}
                  title="Cancel"
                  onClick={() => cancelOperation(op.id)}
                  className="rounded p-1 text-muted-foreground/60 transition-colors hover:bg-accent hover:text-danger focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                >
                  <X className="size-3.5" />
                </button>
              ) : null}
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}

function Metric({
  icon,
  label,
  value,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
}) {
  return (
    <div className="flex items-center gap-3 rounded-lg border border-border/60 bg-card/40 px-3 py-2.5">
      {icon}
      <span className="min-w-0">
        <span className="block text-[10px] font-semibold tracking-[0.16em] text-muted-foreground/70 uppercase">
          {label}
        </span>
        <span data-mono className="block text-sm text-foreground">
          {value}
        </span>
      </span>
    </div>
  );
}

export { OverviewScreen };