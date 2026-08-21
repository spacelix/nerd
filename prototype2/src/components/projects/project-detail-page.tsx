import { ArrowLeft, Copy, ExternalLink, FolderOpen, Link2, Play, Square, Trash2, Unplug } from "lucide-react";
import * as React from "react";
import { StatusDot } from "@/components/status/status-dot";
import { ProjectDetailTabs } from "@/components/shell/project-inspector";
import { useProjectActions } from "@/hooks/use-project-actions";
import { useProjectPreflight } from "@/hooks/use-project-preflight";
import {
  DeleteProjectDialog,
  UnlinkProjectDialog,
  UnparkProjectDialog,
} from "@/components/actions/action-dialogs";
import type { Project, ProjectStatus } from "@/lib/types";

const statusTone: Record<
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

function Chip({ children }: { children: React.ReactNode }) {
  return (
    <span
      data-mono
      className="rounded border border-border/60 bg-card/40 px-1.5 py-0.5 text-[11px] text-muted-foreground"
    >
      {children}
    </span>
  );
}

interface ProjectDetailPageProps {
  project: Project;
  onBack: () => void;
  onDeleted: () => void;
  onOpenMail?: (projectId: string) => void;
}

function ProjectDetailPage({
  project,
  onBack,
  onDeleted,
  onOpenMail,
}: ProjectDetailPageProps) {
  const actions = useProjectActions();
  const preflight = useProjectPreflight();
  const status = actions.statusFor(project);
  const running = status === "running" || status === "starting";
  const [unparkOpen, setUnparkOpen] = React.useState(false);
  const [unlinkOpen, setUnlinkOpen] = React.useState(false);
  const [deleteOpen, setDeleteOpen] = React.useState(false);
  const [copied, setCopied] = React.useState(false);

  const copyDomain = () => {
    navigator.clipboard?.writeText(`http://${project.domain}`).catch(() => {});
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1200);
  };
  return (
    <div className="flex min-h-full flex-col gap-6 px-10 py-10">
      <div>
        <button
          type="button"
          onClick={onBack}
          className="inline-flex items-center gap-1.5 rounded-md px-1 py-0.5 text-xs text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        >
          <ArrowLeft className="size-3.5" />
          Projects
        </button>
      </div>

      <header className="flex flex-col gap-3">
        <span
          data-mono
          className="text-[11px] tracking-[0.18em] text-muted-foreground/60 uppercase"
        >
          Route · projects · {project.domain}
        </span>
        <div className="flex flex-wrap items-center gap-x-3 gap-y-1">
          <StatusDot tone={statusTone[status]} />
          <h1 className="text-2xl font-semibold tracking-tight text-foreground">
            {project.name}
          </h1>
          <span data-mono className="text-sm text-muted-foreground">
            {project.domain}
          </span>
        </div>
        <div className="flex flex-wrap gap-2">
          <Chip>{project.framework}</Chip>
          <Chip>{project.runtime}</Chip>
          <Chip>:{project.port}</Chip>
          <Chip>{project.source}</Chip>
          {project.pinned ? <Chip>pinned</Chip> : null}
          <Chip>{status}</Chip>
        </div>
        {project.failure ? (
          <div
            role="alert"
            aria-live="assertive"
            className="flex items-start gap-2 rounded-md border border-danger/30 bg-danger-soft/60 px-2.5 py-2"
          >
            <span
              aria-hidden="true"
              className="mt-0.5 size-1.5 shrink-0 rounded-full bg-danger"
            />
            <p className="text-[11px] leading-relaxed text-foreground/90">
              Failed at stage “{project.failure.stage}”: {project.failure.cause}
              {project.failure.exitCode !== undefined
                ? ` · exit code ${project.failure.exitCode}`
                : ""}
            </p>
          </div>
        ) : null}
        {project.registry ? (
          <div
            role="alert"
            aria-live="assertive"
            className="flex items-start gap-2 rounded-md border border-warning/30 bg-warning-soft/60 px-2.5 py-2"
          >
            <span
              aria-hidden="true"
              className="mt-0.5 size-1.5 shrink-0 rounded-full bg-warning"
            />
            <p className="text-[11px] leading-relaxed text-foreground/90">
              Registry ({project.registry.kind.replace("-", " ")}):{" "}
              {project.registry.note}
            </p>
          </div>
        ) : null}
        <div className="flex flex-wrap items-center gap-2">
          <button
            type="button"
            onClick={() => {
              if (running) {
                actions.setStatus(project.id, "stopped");
              } else if (actions.isTrusted(project.id)) {
                actions.setStatus(project.id, "running");
              } else {
                preflight.request(project.id);
              }
            }}
            className="inline-flex h-8 items-center gap-1.5 rounded-md bg-primary px-3 text-[11px] font-medium text-primary-foreground transition-opacity hover:opacity-90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            {running ? (
              <Square className="size-3" />
            ) : (
              <Play className="size-3" />
            )}
            {running ? "Stop" : "Start"}
          </button>
          {project.source === "parked" ? (
            <button
              type="button"
              onClick={() => setUnparkOpen(true)}
              className="inline-flex h-8 items-center gap-1.5 rounded-md border border-border/60 bg-card/40 px-3 text-[11px] font-medium text-foreground transition-colors hover:border-border hover:bg-accent/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            >
              <Unplug className="size-3.5" />
              Unpark
            </button>
          ) : (
            <button
              type="button"
              onClick={() => setUnlinkOpen(true)}
              className="inline-flex h-8 items-center gap-1.5 rounded-md border border-border/60 bg-card/40 px-3 text-[11px] font-medium text-foreground transition-colors hover:border-border hover:bg-accent/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            >
              <Link2 className="size-3.5" />
              Unlink
            </button>
          )}
          <button
            type="button"
            onClick={copyDomain}
            title="Copy domain"
            aria-label="Copy domain"
            className="inline-flex h-8 items-center gap-1.5 rounded-md border border-border/60 bg-card/40 px-3 text-[11px] font-medium text-foreground transition-colors hover:border-border hover:bg-accent/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            <Copy className="size-3.5" />
            {copied ? "Copied" : "Copy domain"}
          </button>
          <a
            href={`http://${project.domain}`}
            title="Open in browser"
            aria-label={`Open ${project.domain} in browser`}
            className="inline-flex h-8 items-center gap-1.5 rounded-md border border-border/60 bg-card/40 px-3 text-[11px] font-medium text-foreground transition-colors hover:border-border hover:bg-accent/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            <ExternalLink className="size-3.5" />
            Open
          </a>
          <button
            type="button"
            title="Reveal in Explorer"
            aria-label="Reveal in Explorer"
            className="inline-flex h-8 items-center gap-1.5 rounded-md border border-border/60 bg-card/40 px-3 text-[11px] font-medium text-foreground transition-colors hover:border-border hover:bg-accent/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            <FolderOpen className="size-3.5" />
            Reveal
          </button>
          <button
            type="button"
            onClick={() => setDeleteOpen(true)}
            className="inline-flex h-8 items-center gap-1.5 rounded-md border border-danger/30 bg-danger/5 px-3 text-[11px] font-medium text-danger transition-colors hover:border-danger/50 hover:bg-danger/10 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            <Trash2 className="size-3.5" />
            Delete
          </button>
        </div>
      </header>

      <UnparkProjectDialog
        open={unparkOpen}
        onOpenChange={setUnparkOpen}
        project={project.domain}
        onConfirm={() => actions.setStatus(project.id, "stopped")}
      />
      <UnlinkProjectDialog
        open={unlinkOpen}
        onOpenChange={setUnlinkOpen}
        project={project.domain}
        onConfirm={() => actions.setStatus(project.id, "stopped")}
      />
      <DeleteProjectDialog
        open={deleteOpen}
        onOpenChange={setDeleteOpen}
        project={project.domain}
        onConfirm={() => {
          actions.remove(project.id);
          onDeleted();
        }}
      />

      <ProjectDetailTabs project={project} onOpenMail={onOpenMail} />
    </div>
  );
}

export { ProjectDetailPage };