import {
  CheckCircle2,
  CircleDashed,
  Pause,
  ServerCrash,
  TriangleAlert,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useRoute } from "@/app/RouteContext";
import { cn } from "@/lib/utils";
import { PROJECTS, findRuntime } from "@/mocks/projects";
import { countProjectsByStatus } from "@/lib/projectCounts";
import type { ProjectStatus } from "@/lib/types";

type Tone = "success" | "info" | "muted" | "warning" | "danger";

const STATUS_META: Record<
  ProjectStatus,
  { label: string; tone: Tone; Icon: LucideIcon }
> = {
  running: { label: "Running", tone: "success", Icon: CheckCircle2 },
  starting: { label: "Starting", tone: "info", Icon: CircleDashed },
  installing: { label: "Installing", tone: "info", Icon: CircleDashed },
  waiting: { label: "Waiting", tone: "info", Icon: CircleDashed },
  stopped: { label: "Stopped", tone: "muted", Icon: Pause },
  degraded: { label: "Degraded", tone: "warning", Icon: TriangleAlert },
  failed: { label: "Failed", tone: "danger", Icon: ServerCrash },
};

const DOT_BG: Record<Tone, string> = {
  success: "bg-success",
  info: "bg-info",
  muted: "bg-muted-foreground",
  warning: "bg-warning",
  danger: "bg-destructive",
};

const TEXT_BG: Record<Tone, string> = {
  success: "text-success",
  info: "text-info",
  muted: "text-muted-foreground",
  warning: "text-warning",
  danger: "text-destructive",
};

function StatusDot({ tone }: { tone: Tone }) {
  return (
    <span
      aria-hidden="true"
      className={cn("inline-block h-1.5 w-1.5 shrink-0 rounded-full", DOT_BG[tone])}
    />
  );
}

function Hero() {
  const counts = countProjectsByStatus(PROJECTS);
  const total = PROJECTS.length;
  const active = counts.running + counts.starting + counts.installing + counts.waiting;

  const breakdown: { status: ProjectStatus; count: number }[] = (
    [
      { status: "running" as const, count: counts.running },
      { status: "starting" as const, count: counts.starting },
      { status: "stopped" as const, count: counts.stopped },
      { status: "degraded" as const, count: counts.degraded },
      { status: "failed" as const, count: counts.failed },
    ] as { status: ProjectStatus; count: number }[]
  ).filter((b) => b.count > 0);

  return (
    <div className="border-b border-border/40 px-8 py-8">
      <div className="flex items-center gap-2 text-xs text-muted-foreground">
        <span
          aria-hidden="true"
          className="relative inline-flex h-1.5 w-1.5"
        >
          <span className="absolute inline-flex h-full w-full animate-[nerd-pulse_1.6s_ease-in-out_infinite] rounded-full bg-success opacity-60" />
          <span className="relative inline-flex h-1.5 w-1.5 rounded-full bg-success" />
        </span>
        <span className="font-mono uppercase tracking-[0.14em]">
          Daemon healthy
        </span>
      </div>

      <div className="mt-5 flex flex-wrap items-baseline gap-x-6 gap-y-2">
        <h1 className="font-mono text-[44px] font-semibold leading-[1.05] tracking-[-0.03em]">
          {counts.running}
          <span className="text-muted-foreground">/{total}</span>
        </h1>
        <span className="text-[20px] font-normal text-muted-foreground">
          projects running
        </span>
      </div>

      <p className="mt-3 max-w-2xl text-sm leading-relaxed text-muted-foreground">
        Nerd daemon is healthy. Managed Node versions are
        checksum-verified. Foreign listeners on ports 53, 80, and 443 are
        reported but never terminated.
      </p>

      <dl className="mt-8 grid grid-cols-2 gap-x-8 gap-y-4 border-t border-border/40 pt-6 sm:grid-cols-5">
        <div>
          <dt className="text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
            Uptime
          </dt>
          <dd className="mt-1.5 font-mono text-[15px] tabular-nums">14h 22m</dd>
        </div>
        <div>
          <dt className="text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
            Memory
          </dt>
          <dd className="mt-1.5 font-mono text-[15px] tabular-nums">
            18<span className="text-muted-foreground"> MB</span>
          </dd>
        </div>
        <div>
          <dt className="text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
            CPU
          </dt>
          <dd className="mt-1.5 font-mono text-[15px] tabular-nums">
            0.1<span className="text-muted-foreground">%</span>
          </dd>
        </div>
        <div>
          <dt className="text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
            Active
          </dt>
          <dd className="mt-1.5 font-mono text-[15px] tabular-nums">
            {active}
            <span className="text-muted-foreground">/{total}</span>
          </dd>
        </div>
        <div>
          <dt className="text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
            Idle
          </dt>
          <dd className="mt-1.5 font-mono text-[15px] tabular-nums text-muted-foreground">
            {total - active}
          </dd>
        </div>
      </dl>

      {breakdown.length > 1 && (
        <div className="mt-6 flex flex-wrap gap-x-5 gap-y-2 border-t border-border/40 pt-6 text-sm">
          {breakdown.map((b) => {
            const meta = STATUS_META[b.status];
            return (
              <div key={b.status} className="flex items-center gap-2">
                <StatusDot tone={meta.tone} />
                <span className="text-muted-foreground">{meta.label}</span>
                <span
                  className={cn(
                    "font-mono tabular-nums",
                    TEXT_BG[meta.tone],
                  )}
                >
                  {b.count}
                </span>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

function RunningProjectsList() {
  const { setRoute } = useRoute();
  const running = PROJECTS.filter((p) => p.status === "running");
  return (
    <div className="flex flex-col px-8 py-8">
      <div className="mb-5 flex items-baseline justify-between">
        <div>
          <h2 className="text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
            Active projects
          </h2>
          <p className="mt-1 text-sm text-muted-foreground">
            {running.length} running. Click to inspect logs and lifecycle.
          </p>
        </div>
        <Button variant="ghost" size="sm" onClick={() => setRoute("projects")}>
          View all
        </Button>
      </div>
      <ul className="border-y border-border/40">
        {running.map((project, index) => {
          const runtime = findRuntime(project.runtimeId);
          return (
            <li
              key={project.id}
              className={cn(
                "grid h-14 grid-cols-[12px_minmax(0,1fr)_minmax(0,1fr)_auto] items-center gap-4 px-2 text-sm transition-colors hover:bg-accent/30 md:grid-cols-[12px_180px_minmax(0,1fr)_auto_auto]",
                index > 0 && "border-t border-border/40",
              )}
            >
              <StatusDot tone="success" />
              <button
                type="button"
                onClick={() => setRoute("projects")}
                className="truncate text-left font-medium hover:text-foreground"
              >
                {project.name}
              </button>
              <span className="truncate font-mono text-xs tabular-nums text-muted-foreground">
                https://{project.domain}
              </span>
              <span className="hidden font-mono text-xs tabular-nums text-muted-foreground md:inline">
                {runtime ? `Node ${runtime.version.split(".").slice(0, 2).join(".")}` : "—"}
              </span>
              <span className="font-mono text-xs tabular-nums text-muted-foreground">
                :{project.port}
              </span>
            </li>
          );
        })}
      </ul>
    </div>
  );
}

function RecentActivity() {
  const MAX_EVENTS = 5;
  const events = PROJECTS.flatMap((p) =>
    p.logs.slice(-1).map((line) => ({ project: p, line })),
  )
    .slice(-MAX_EVENTS)
    .reverse();
  return (
    <div className="flex flex-col px-8 py-8 lg:border-l lg:border-border/40">
      <div className="mb-5 flex items-baseline justify-between">
        <div>
          <h2 className="text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
            Recent activity
          </h2>
          <p className="mt-1 text-sm text-muted-foreground">
            Last {events.length} events from project logs.
          </p>
        </div>
        <span className="font-mono text-[10px] tabular-nums text-muted-foreground">
          last {events.length}
        </span>
      </div>
      <ul className="space-y-3" aria-label={`Last ${events.length} events`}>
        {events.map(({ project, line }, index) => {
          const meta = STATUS_META[project.status];
          return (
            <li
              key={`${project.id}-${index}-${line.ts}`}
              className="grid grid-cols-[72px_minmax(0,1fr)] gap-x-4 gap-y-1 text-sm"
            >
              <span className="font-mono text-xs tabular-nums text-muted-foreground">
                {line.ts}
              </span>
              <span className="flex min-w-0 items-center gap-2 truncate font-medium text-foreground">
                <StatusDot tone={meta.tone} />
                <span className="truncate">{project.name}</span>
              </span>
              <span className="col-start-2 min-w-0 text-xs leading-relaxed text-muted-foreground">
                {line.text}
              </span>
            </li>
          );
        })}
      </ul>
    </div>
  );
}

export function OverviewScreen() {
  return (
    <div className="flex h-full flex-col animate-[nerd-fade-up_180ms_ease-out]">
      <div className="flex-1 overflow-auto">
        <div className="flex flex-col">
          <Hero />
          <RunningProjectsList />
          <RecentActivity />
        </div>
      </div>
    </div>
  );
}
