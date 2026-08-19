import { useMemo, useState } from "react";
import {
  ArrowDown,
  ArrowUp,
  CheckCircle2,
  CircleDashed,
  Pause,
  Play,
  RefreshCw,
  Square,
  TriangleAlert,
  XCircle,
} from "lucide-react";
import { useRoute } from "@/app/RouteContext";
import { PROJECTS, findRuntime } from "@/mocks/projects";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import type { Project, ProjectStatus } from "@/lib/types";
import { cn } from "@/lib/utils";

const STATUS_META: Record<
  ProjectStatus,
  { label: string; tone: "success" | "info" | "warning" | "destructive"; Icon: typeof CheckCircle2 }
> = {
  running: { label: "Running", tone: "success", Icon: CheckCircle2 },
  starting: { label: "Starting", tone: "info", Icon: CircleDashed },
  installing: { label: "Installing", tone: "info", Icon: CircleDashed },
  waiting: { label: "Waiting", tone: "info", Icon: CircleDashed },
  stopped: { label: "Stopped", tone: "warning", Icon: Pause },
  degraded: { label: "Degraded", tone: "warning", Icon: TriangleAlert },
  failed: { label: "Failed", tone: "destructive", Icon: XCircle },
};

type Filter = "all" | "running" | "stopped" | "degraded" | "failed";

const FILTERS: { id: Filter; label: string }[] = [
  { id: "all", label: "All" },
  { id: "running", label: "Running" },
  { id: "stopped", label: "Stopped" },
  { id: "degraded", label: "Degraded" },
  { id: "failed", label: "Failed" },
];

function filterProjects(projects: readonly Project[], filter: Filter): Project[] {
  if (filter === "all") return projects.slice();
  const match: Record<Exclude<Filter, "all">, ProjectStatus[]> = {
    running: ["running", "starting", "installing", "waiting"],
    stopped: ["stopped"],
    degraded: ["degraded"],
    failed: ["failed"],
  };
  return projects.filter((p) => match[filter].includes(p.status));
}

function ProjectsList() {
  const { setSelectedProjectId } = useRoute();
  const [filter, setFilter] = useState<Filter>("all");
  const filtered = useMemo(() => filterProjects(PROJECTS, filter), [filter]);

  return (
    <div className="flex h-full flex-col">
      <div className="flex flex-wrap items-center gap-2 border-b border-border/40 px-8 py-3">
        <span className="mr-auto font-mono text-xs tabular-nums text-muted-foreground">
          {filtered.length} of {PROJECTS.length}
        </span>
        <div className="flex gap-1" role="radiogroup" aria-label="Filter projects">
          {FILTERS.map((f) => (
            <Button
              key={f.id}
              variant={filter === f.id ? "default" : "ghost"}
              size="sm"
              onClick={() => setFilter(f.id)}
              aria-pressed={filter === f.id}
            >
              {f.label}
            </Button>
          ))}
        </div>
        <Button variant="ghost" size="sm">
          <RefreshCw className="h-3.5 w-3.5" />
          Refresh
        </Button>
      </div>

      <div className="flex-1 overflow-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-border/40 bg-muted/30 text-[10px] font-semibold uppercase tracking-[0.1em] text-muted-foreground">
              <th className="h-9 w-12" />
              <th className="h-9 px-3 text-left">Name</th>
              <th className="h-9 px-3 text-left">Domain</th>
              <th className="h-9 px-3 text-left">Framework</th>
              <th className="h-9 px-3 text-left">Runtime</th>
              <th className="h-9 px-3 text-right">Port</th>
              <th className="h-9 px-3 text-left">Source</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((project) => {
              const runtime = findRuntime(project.runtimeId);
              const meta = STATUS_META[project.status];
              const Icon = meta.Icon;
              return (
                <tr
                  key={project.id}
                  onClick={() => setSelectedProjectId(project.id)}
                  className="cursor-pointer border-b border-border/40 transition-colors hover:bg-accent/30"
                >
                  <td className="h-11 px-3 align-middle">
                    <span
                      aria-hidden="true"
                      className={cn(
                        "inline-flex h-1.5 w-1.5 rounded-full",
                        meta.tone === "success" && "bg-success",
                        meta.tone === "info" && "bg-info",
                        meta.tone === "warning" && "bg-warning",
                        meta.tone === "destructive" && "bg-destructive",
                      )}
                    />
                  </td>
                  <td className="px-3 align-middle">
                    <span className="flex items-center gap-2">
                      <span className="font-medium text-foreground">
                        {project.name}
                      </span>
                      <Badge variant={meta.tone} className="gap-1">
                        <Icon className="h-3 w-3" />
                        {meta.label}
                      </Badge>
                    </span>
                  </td>
                  <td className="px-3 align-middle font-mono text-xs tabular-nums text-muted-foreground">
                    {project.domain}
                  </td>
                  <td className="px-3 align-middle text-text-muted">
                    {project.frameworkLabel}
                  </td>
                  <td className="px-3 align-middle font-mono text-xs tabular-nums text-text-muted">
                    {runtime ? `Node ${runtime.version}` : "—"}
                  </td>
                  <td className="px-3 text-right align-middle font-mono text-xs tabular-nums text-text-faint">
                    :{project.port}
                  </td>
                  <td className="px-3 align-middle">
                    <Badge variant="outline" className="capitalize">
                      {project.source}
                    </Badge>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}

type LifecycleState = "done" | "active" | "pending" | "failed";

const LIFECYCLE_STEPS = [
  { id: "trust", label: "Trust and Start" },
  { id: "runtime", label: "Resolve runtime" },
  { id: "services", label: "Start services" },
  { id: "application", label: "Start application" },
  { id: "readiness", label: "Readiness probe" },
  { id: "route", label: "Route enabled" },
] as const;

function lifecycleStateFor(
  project: Project,
  stepId: (typeof LIFECYCLE_STEPS)[number]["id"],
): LifecycleState {
  if (project.status === "failed") return stepId === "trust" ? "done" : "failed";
  if (project.status === "degraded") {
    if (stepId === "trust" || stepId === "runtime") return "done";
    if (stepId === "services" || stepId === "readiness") return "failed";
    return "pending";
  }
  if (project.status === "stopped") {
    if (stepId === "trust" || stepId === "runtime") return "done";
    return "pending";
  }
  if (project.status === "waiting") {
    if (stepId === "trust" || stepId === "runtime" || stepId === "services") return "done";
    return "active";
  }
  if (project.status === "installing") {
    if (stepId === "trust") return "done";
    if (stepId === "runtime") return "active";
    return "pending";
  }
  if (project.status === "starting") {
    if (stepId === "trust" || stepId === "runtime" || stepId === "services") return "done";
    if (stepId === "application") return "active";
    return "pending";
  }
  return "done";
}

const LIFECYCLE_INDICATOR: Record<LifecycleState, {
  dot: string;
  ring: string;
  text: string;
}> = {
  done: {
    dot: "bg-success",
    ring: "ring-success/30",
    text: "text-success",
  },
  active: {
    dot: "bg-info",
    ring: "ring-info/30 motion-safe:animate-[nerd-pulse_1.6s_ease-in-out_infinite]",
    text: "text-info",
  },
  pending: {
    dot: "bg-text-faint",
    ring: "ring-text-faint/20",
    text: "text-text-faint",
  },
  failed: {
    dot: "bg-destructive",
    ring: "ring-destructive/30",
    text: "text-destructive",
  },
};

function LifecycleStrip({ project }: { project: Project }) {
  return (
    <ol className="space-y-2">
      {LIFECYCLE_STEPS.map((step) => {
        const state = lifecycleStateFor(project, step.id);
        const indicator = LIFECYCLE_INDICATOR[state];
        return (
          <li
            key={step.id}
            className="flex items-center gap-3 text-sm"
          >
            <span
              aria-hidden="true"
              className={cn(
                "relative inline-flex h-2 w-2 rounded-full ring-4",
                indicator.dot,
                indicator.ring,
              )}
            />
            <span
              className={cn(
                state === "active" && "font-semibold text-foreground",
                state !== "active" && indicator.text,
              )}
            >
              {step.label}
            </span>
            <span className="ml-auto font-mono text-[10px] uppercase tracking-[0.1em] text-text-faint">
              {state}
            </span>
          </li>
        );
      })}
    </ol>
  );
}

function ProjectDetail({ project }: { project: Project }) {
  const { setSelectedProjectId } = useRoute();
  const runtime = findRuntime(project.runtimeId);
  const meta = STATUS_META[project.status];
  const Icon = meta.Icon;

  const actionLabel =
    project.status === "running"
      ? "Stop"
      : project.status === "stopped"
        ? "Start"
        : project.status === "degraded" || project.status === "failed"
          ? "Retry"
          : "Start";

  const actionIcon =
    project.status === "running"
      ? Square
      : project.status === "stopped"
        ? Play
        : RefreshCw;

  return (
    <div className="flex h-full flex-col">
      <div className="flex flex-wrap items-center gap-3 border-b border-border/40 px-8 py-3">
        <Button
          variant="ghost"
          size="sm"
          onClick={() => setSelectedProjectId(null)}
        >
          ← Projects
        </Button>
        <div className="h-5 w-px bg-border/40" />
        <h2 className="text-lg font-semibold tracking-[-0.014em]">
          {project.name}
        </h2>
        <Badge variant={meta.tone} className="gap-1">
          <Icon className="h-3 w-3" />
          {meta.label}
        </Badge>
        {project.statusDetail && (
          <span className="text-xs text-muted-foreground">
            — {project.statusDetail}
          </span>
        )}
        <div className="ml-auto flex items-center gap-2">
          <Button variant="ghost" size="sm">
            <ArrowUp className="h-3.5 w-3.5" />
            Open
          </Button>
          <Button variant="ghost" size="sm">
            <ArrowDown className="h-3.5 w-3.5" />
            Copy domain
          </Button>
          <Button variant="default" size="sm">
            {(() => {
              const ActionIcon = actionIcon;
              return <ActionIcon className="h-3.5 w-3.5" />;
            })()}
            {actionLabel}
          </Button>
        </div>
      </div>

      <div className="grid flex-1 grid-cols-1 gap-0 overflow-auto lg:grid-cols-[360px_1fr]">
        <aside className="flex flex-col gap-6 border-r border-border/40 p-8">
          <section>
            <h3 className="mb-3 text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
              Configuration
            </h3>
            <dl className="grid grid-cols-[120px_1fr] gap-y-2 text-sm">
              <dt className="text-text-muted">Domain</dt>
              <dd className="font-mono text-xs tabular-nums">
                https://{project.domain}
              </dd>
              <dt className="text-text-muted">Framework</dt>
              <dd>{project.frameworkLabel}</dd>
              <dt className="text-text-muted">Runtime</dt>
              <dd className="font-mono text-xs tabular-nums">
                {runtime ? `Node ${runtime.version}` : "—"}
              </dd>
              <dt className="text-text-muted">Package manager</dt>
              <dd className="font-mono text-xs uppercase">
                {project.packageManager}
              </dd>
              <dt className="text-text-muted">Source</dt>
              <dd className="capitalize">{project.source}</dd>
              <dt className="text-text-muted">Trust</dt>
              <dd className="capitalize">{project.trust}</dd>
              <dt className="text-text-muted">Path</dt>
              <dd className="truncate font-mono text-xs">{project.path}</dd>
            </dl>
          </section>

          {project.services.length > 0 && (
            <section>
              <h3 className="mb-3 text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
                Services
              </h3>
              <ul className="space-y-1.5">
                {project.services.map((service) => (
                  <li
                    key={`${service.kind}-${service.port}`}
                    className="flex items-center justify-between font-mono text-xs"
                  >
                    <span className="text-foreground">
                      {service.kind}@{service.version}
                    </span>
                    <span className="tabular-nums text-muted-foreground">
                      :{service.port}
                    </span>
                  </li>
                ))}
              </ul>
            </section>
          )}

          <Separator className="bg-border/40" />

          <section>
            <h3 className="mb-3 text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
              Lifecycle
            </h3>
            <LifecycleStrip project={project} />
          </section>
        </aside>

        <section className="flex min-h-0 flex-col">
          <div className="flex items-center justify-between border-b border-border/40 bg-muted/30 px-6 py-2 font-mono text-[10px] uppercase tracking-[0.1em] text-muted-foreground">
            <span>Log · {project.logs.length} lines</span>
            <Button variant="ghost" size="sm">
              Copy tail
            </Button>
          </div>
          <div className="flex-1 overflow-auto bg-log p-4 font-mono text-[12px] leading-relaxed text-log-text">
            {project.logs.length === 0 ? (
              <div className="text-text-faint">No output yet.</div>
            ) : (
              project.logs.map((line, index) => (
                <div
                  key={`${index}-${line.ts}`}
                  className="flex gap-3 whitespace-pre-wrap"
                >
                  <span className="shrink-0 text-text-faint">{line.ts}</span>
                  <span
                    className={cn(
                      line.stream === "stderr" && "text-destructive",
                      line.stream === "system" && "text-warning",
                    )}
                  >
                    {line.text}
                  </span>
                </div>
              ))
            )}
          </div>
        </section>
      </div>
    </div>
  );
}

export function ProjectsScreen() {
  const { selectedProjectId } = useRoute();
  if (selectedProjectId) {
    const project = PROJECTS.find((p) => p.id === selectedProjectId);
    if (project) return <ProjectDetail project={project} />;
  }
  return <ProjectsList />;
}
