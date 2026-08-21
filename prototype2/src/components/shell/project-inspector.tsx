import { AlertTriangle } from "lucide-react";
import * as React from "react";
import { cn } from "@/lib/utils";
import { StatusDot } from "@/components/status/status-dot";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useProjectActions } from "@/hooks/use-project-actions";
import { logs, mail, requests, servicesByProject } from "@/mocks/data";
import type { LogLevel, Project } from "@/lib/types";

const logTone: Record<LogLevel, string> = {
  info: "text-muted-foreground",
  warn: "text-warning",
  error: "text-danger",
};

function requestStatusTone(status: number): string {
  if (status < 300) return "text-success";
  if (status < 400) return "text-info";
  if (status < 500) return "text-warning";
  return "text-danger";
}

function ConfigRow({
  label,
  value,
  overridden = false,
}: {
  label: string;
  value: React.ReactNode;
  overridden?: boolean;
}) {
  return (
    <div className="flex items-center justify-between gap-3 py-1.5">
      <span className="shrink-0 text-[11px] text-muted-foreground">{label}</span>
      <span data-mono className="flex min-w-0 items-center gap-1.5 text-[11px] text-foreground">
        <span className="truncate">{value}</span>
        {overridden ? (
          <span
            title="Set in a local override — never written back to nerd.json"
            className="shrink-0 rounded border border-border/50 bg-background/40 px-1 text-[9px] text-muted-foreground/70 uppercase"
          >
            override
          </span>
        ) : null}
      </span>
    </div>
  );
}

interface ProjectDetailTabsProps {
  project: Project;
  onOpenMail?: (projectId: string) => void;
}

function LogPane({ lines }: { lines: Array<{ id: string; time: string; level: LogLevel; message: string }> }) {
  const ref = React.useRef<HTMLDivElement>(null);
  const [atBottom, setAtBottom] = React.useState(true);

  React.useEffect(() => {
    const el = ref.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [lines.length]);

  const jumpToLatest = (): void => {
    const el = ref.current;
    if (el) {
      el.scrollTo({ top: el.scrollHeight, behavior: "smooth" });
      setAtBottom(true);
    }
  };

  return (
    <div className="relative">
      <div
        ref={ref}
        role="log"
        aria-live="polite"
        aria-label="Project logs"
        onScroll={(e) => {
          const el = e.currentTarget;
          setAtBottom(
            el.scrollHeight - el.scrollTop - el.clientHeight < 24,
          );
        }}
        className="max-h-48 overflow-y-auto pr-1 font-mono text-[11px] leading-5"
      >
        {lines.map((line) => (
          <div key={line.id} className="flex gap-2">
            <span className="shrink-0 text-muted-foreground/50">
              {line.time}
            </span>
            <span className={cn("w-9 shrink-0 uppercase", logTone[line.level])}>
              {line.level}
            </span>
            <span className="min-w-0 break-words text-muted-foreground">
              {line.message}
            </span>
          </div>
        ))}
      </div>
      {!atBottom ? (
        <button
          type="button"
          onClick={jumpToLatest}
          className="absolute right-0 bottom-0 rounded-md border border-border bg-popover px-2.5 py-1 text-[10px] font-medium text-muted-foreground shadow-sm transition-colors hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        >
          Jump to latest
        </button>
      ) : null}
    </div>
  );
}

function ProjectDetailTabs({ project, onOpenMail }: ProjectDetailTabsProps) {
  const actions = useProjectActions();
  const status = actions.statusFor(project);
  const projectLogs = logs.filter((l) => l.projectId === project.id);
  const projectRequests = requests
    .filter((r) => r.projectId === project.id)
    .slice()
    .sort((a, b) => b.startedAt.localeCompare(a.startedAt));
  const services = servicesByProject[project.id] ?? [];
  const projectMail = mail
    .filter((m) => m.projectId === project.id)
    .slice()
    .sort((a, b) => b.receivedAt.localeCompare(a.receivedAt));

  const routing =
    status === "running"
      ? {
          label: `active · 80/443 → :${project.port}`,
          tone: "text-success",
        }
      : status === "starting"
        ? { label: "starting · Retry-After: 1s", tone: "text-warning" }
        : {
            label: "503 · start this project to route traffic",
            tone: "text-danger",
          };

  return (
    <Tabs defaultValue="config" className="flex min-h-0 flex-1 flex-col">
      {project.failure ? (
        <div
          role="alert"
          aria-live="assertive"
          className="flex items-start gap-2 border-b border-danger/30 bg-danger-soft/60 px-3 py-2"
        >
          <AlertTriangle className="mt-0.5 size-3.5 shrink-0 text-danger" />
          <div className="flex flex-col gap-0.5">
            <span className="text-[11px] font-semibold text-foreground">
              Failed at stage “{project.failure.stage}”
            </span>
            <span className="text-[11px] leading-relaxed text-foreground/90">
              {project.failure.cause}
              {project.failure.exitCode !== undefined
                ? ` · exit code ${project.failure.exitCode}`
                : ""}
            </span>
            <span className="text-[10px] text-muted-foreground/70">
              Suggestion: reconcile the working directory, then start again.
            </span>
          </div>
        </div>
      ) : null}
      {project.registry ? (
        <div
          role="alert"
          aria-live="assertive"
          className="flex items-start gap-2 border-b border-warning/30 bg-warning-soft/60 px-3 py-2"
        >
          <AlertTriangle className="mt-0.5 size-3.5 shrink-0 text-warning" />
          <div className="flex flex-col gap-0.5">
            <span className="text-[11px] font-semibold capitalize text-foreground">
              Registry: {project.registry.kind.replace("-", " ")}
            </span>
            <span className="text-[11px] leading-relaxed text-foreground/90">
              {project.registry.note}
            </span>
          </div>
        </div>
      ) : null}
      <div className="shrink-0 border-b border-border/60 p-2">
        <TabsList className="w-full">
          <TabsTrigger value="config">Config</TabsTrigger>
          <TabsTrigger value="services">Services</TabsTrigger>
          <TabsTrigger value="logs">Logs</TabsTrigger>
          <TabsTrigger value="mail">Mail</TabsTrigger>
          <TabsTrigger value="activity">Activity</TabsTrigger>
        </TabsList>
      </div>

      <TabsContent value="config" className="px-3 pt-2">
        <div className="flex flex-col">
          <ConfigRow label="Domain" value={project.domain} />
          <ConfigRow label="Framework" value={project.framework} />
          <ConfigRow
            label="Runtime"
            value={project.runtime}
            overridden={project.overrides?.includes("runtime")}
          />
          <ConfigRow
            label="Version source"
            value={project.versionSource}
          />
          <ConfigRow
            label="Package manager"
            value={project.packageManager}
            overridden={project.overrides?.includes("packageManager")}
          />
          <ConfigRow
            label="Command"
            value={project.command}
            overridden={project.overrides?.includes("command")}
          />
          <ConfigRow
            label="Port"
            value={project.port}
            overridden={project.overrides?.includes("port")}
          />
          <ConfigRow
            label="Port adapter"
            value="Express process.env.PORT honored · binds loopback proxy"
          />
          <ConfigRow label="Readiness" value={project.readiness} />
          <ConfigRow label="Restart policy" value={project.restartPolicy} />
          <ConfigRow label="Autostart" value={project.autostart ? "yes" : "no"} />
          <ConfigRow label="Source" value={project.source} />
          <ConfigRow label="Pinned" value={project.pinned ? "yes" : "no"} />
          <ConfigRow label="Status" value={project.status} />
          <ConfigRow
            label="Routing"
            value={
              <span role="status" aria-atomic="true" className={routing.tone}>
                {routing.label}
              </span>
            }
          />
          <ConfigRow
            label="Mail SMTP"
            value="127.0.0.1:2525 · NERD_MAIL_HOST/PORT"
          />
          {project.registry ? (
            <ConfigRow
              label="Registry"
              value={
                <span className="text-warning">
                  {project.registry.kind.replace("-", " ")}
                </span>
              }
            />
          ) : null}
          <ServiceEnvRows projectId={project.id} />
          <p
            data-mono
            className="mt-2 rounded-md border border-border/50 bg-background/40 px-2 py-1.5 text-[10px] leading-relaxed text-muted-foreground/70"
          >
            Process runs in a Windows Job Object under the current user, never
            elevated. Preflight reports effective environment provenance without
            exposing secret values.
          </p>
        </div>
      </TabsContent>

      <TabsContent value="services" className="px-3 pt-2">
        {services.length === 0 ? (
          <p className="py-8 text-center text-xs text-muted-foreground">
            No services attached to this project.
          </p>
        ) : (
          <div className="flex flex-col">
            {services.map((svc) => (
              <div
                key={svc.id}
                className="flex items-center gap-2.5 py-1.5"
              >
                <StatusDot
                  tone={
                    svc.status === "running"
                      ? "success"
                      : svc.status === "degraded"
                        ? "warning"
                        : "muted"
                  }
                />
                <span className="flex-1 truncate text-xs text-foreground">
                  {svc.name}
                </span>
                <span data-mono className="text-[11px] text-muted-foreground">
                  {svc.version}
                </span>
                <span
                  data-mono
                  className="text-[11px] text-muted-foreground/70"
                >
                  {svc.status}
                </span>
              </div>
            ))}
          </div>
        )}
      </TabsContent>

      <TabsContent value="logs" className="relative px-3 pt-2">
        {projectLogs.length === 0 ? (
          <p className="py-8 text-center text-xs text-muted-foreground">
            No logs captured for this project.
          </p>
        ) : (
          <LogPane lines={projectLogs} />
        )}
      </TabsContent>

      <TabsContent value="mail" className="px-3 pt-2">
        {projectMail.length === 0 ? (
          <p className="py-8 text-center text-xs text-muted-foreground">
            No captured mail for this project.
          </p>
        ) : (
          <div className="flex flex-col gap-1.5">
            {projectMail.map((m) => (
              <div
                key={m.id}
                className="flex items-center gap-2 rounded-md border border-border/50 bg-background/40 px-2.5 py-1.5"
              >
                <span className="min-w-0 flex-1">
                  <span className="block truncate text-xs text-foreground/90">
                    {m.subject}
                  </span>
                  <span
                    data-mono
                    className="block truncate text-[11px] text-muted-foreground/70"
                  >
                    {m.from} → {m.to}
                  </span>
                </span>
                {m.unread ? (
                  <span
                    aria-label="Unread"
                    className="size-1.5 shrink-0 rounded-full bg-primary"
                  />
                ) : null}
              </div>
            ))}
            {onOpenMail ? (
              <button
                type="button"
                onClick={() => onOpenMail(project.id)}
                className="rounded-md border border-border/60 bg-card/40 px-2.5 py-1.5 text-[11px] font-medium text-foreground transition-colors hover:border-border hover:bg-accent/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              >
                Open inbox in Mail
              </button>
            ) : null}
          </div>
        )}
      </TabsContent>

      <TabsContent value="activity" className="px-3 pt-2">
        {projectRequests.length === 0 ? (
          <p className="py-8 text-center text-xs text-muted-foreground">
            No request traffic for this project.
          </p>
        ) : (
          <div className="flex flex-col font-mono text-[11px]">
            {projectRequests.map((req) => (
              <div key={req.id} className="flex items-center gap-2 py-1.5">
                <span
                  className={cn(
                    "w-9 shrink-0 font-medium",
                    requestStatusTone(req.status),
                  )}
                >
                  {req.status}
                </span>
                <span className="w-10 shrink-0 text-muted-foreground">
                  {req.method}
                </span>
                <span className="min-w-0 flex-1 truncate text-muted-foreground">
                  {req.url}
                </span>
                <span className="shrink-0 text-muted-foreground/60">
                  {req.durationMs}ms
                </span>
              </div>
            ))}
          </div>
        )}
      </TabsContent>
    </Tabs>
  );
}

function ServiceEnvRows({ projectId }: { projectId: string }) {
  const services = servicesByProject[projectId] ?? [];
  if (services.length === 0) return null;
  const envByEngine: Record<string, string> = {
    MySQL: "NERD_MYSQL_URL",
    PostgreSQL: "NERD_POSTGRES_URL",
    Redis: "NERD_REDIS_URL",
    MongoDB: "NERD_MONGO_URL",
    Mailpit: "NERD_SMTP_URL",
  };
  return (
    <div className="flex flex-col gap-1 pt-1">
      <span className="text-[10px] font-semibold tracking-[0.16em] text-muted-foreground/70 uppercase">
        Service env (injected)
      </span>
      {services.map((svc) => (
        <ConfigRow
          key={svc.id}
          label={envByEngine[svc.name] ?? "NERD_SERVICE_URL"}
          value={`127.0.0.1:${svc.port} · redacted credentials`}
        />
      ))}
    </div>
  );
}

export { ProjectDetailTabs };