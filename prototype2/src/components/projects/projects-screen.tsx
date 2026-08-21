import { ChevronRight, Folder, FolderPlus, Link2, Pin, Play, Plus, Search, Square, Trash2 } from "lucide-react";
import * as React from "react";
import { cn } from "@/lib/utils";
import { StatusDot } from "@/components/status/status-dot";
import { Kbd } from "@/components/ui/kbd";
import { WorkingSessionToggle } from "@/components/shell/working-session-toggle";
import { useProjectActions } from "@/hooks/use-project-actions";
import { useProjectPreflight } from "@/hooks/use-project-preflight";
import {
  ActionButton,
  DeleteProjectDialog,
  LinkProjectDialog,
  NewProjectDialog,
  ParkDirectoryDialog,
} from "@/components/actions/action-dialogs";
import { projects } from "@/mocks/data";
import type { Project, ProjectStatus } from "@/lib/types";
import type { WorkingSession } from "@/components/shell/working-session-toggle";

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

interface ProjectsScreenProps {
  workingSession: WorkingSession;
  onWorkingSessionChange: (next: WorkingSession) => void;
  onOpenProject: (id: string) => void;
}

function matchesSession(p: Project, session: WorkingSession): boolean {
  if (session === "all") return true;
  if (session === "active") return p.pinned;
  return !p.pinned;
}

function ProjectsScreen({
  workingSession,
  onWorkingSessionChange,
  onOpenProject,
}: ProjectsScreenProps) {
  const [query, setQuery] = React.useState("");
  const [newProjectOpen, setNewProjectOpen] = React.useState(false);
  const [parkOpen, setParkOpen] = React.useState(false);
  const [linkOpen, setLinkOpen] = React.useState(false);
  const [deleteTarget, setDeleteTarget] = React.useState<string | null>(null);
  const actions = useProjectActions();
  const preflight = useProjectPreflight();
  const searchRef = React.useRef<HTMLInputElement>(null);

  React.useEffect(() => {
    const handler = (e: KeyboardEvent): void => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        searchRef.current?.focus();
        searchRef.current?.select();
      }
      if (e.key === "Escape" && document.activeElement === searchRef.current) {
        searchRef.current?.blur();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  const filtered = React.useMemo(() => {
    const q = query.trim().toLowerCase();
    return projects.filter((p) => {
      if (actions.isRemoved(p.id)) return false;
      if (!matchesSession(p, workingSession)) return false;
      if (!q) return true;
      return [p.name, p.domain, p.framework]
        .join(" ")
        .toLowerCase()
        .includes(q);
    });
  }, [query, workingSession, actions]);

  return (
    <div className="flex min-h-full flex-col gap-6 px-10 py-10">
      <header className="flex flex-col gap-3">
        <span
          data-mono
          className="text-[11px] tracking-[0.18em] text-muted-foreground/60 uppercase"
        >
          Route · projects · N2
        </span>
        <div className="flex flex-wrap items-end justify-between gap-4">
          <div className="flex flex-col gap-1">
            <h1 className="text-2xl font-semibold tracking-tight text-foreground">
              Projects
            </h1>
            <p className="text-sm text-muted-foreground">
              {filtered.length} of {projects.length} projects
            </p>
          </div>
          <div className="flex items-center gap-3">
            <WorkingSessionToggle
              value={workingSession}
              onValueChange={onWorkingSessionChange}
            />
            <label
              htmlFor="projects-search"
              className="flex h-8 w-64 items-center gap-2 rounded-md border border-border/60 bg-background/60 px-2.5 transition-colors focus-within:border-ring/60 focus-within:bg-background"
            >
            <Search
              aria-hidden="true"
              className="size-3.5 shrink-0 text-muted-foreground"
            />
            <input
              ref={searchRef}
              id="projects-search"
              type="text"
              placeholder="Search projects"
              aria-label="Search projects"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              className="min-w-0 flex-1 bg-transparent text-sm text-foreground placeholder:text-muted-foreground/70 focus:outline-none"
            />
            <Kbd className="pointer-events-none">⌘K</Kbd>
            </label>
          </div>
        </div>
      </header>

      <div className="flex flex-wrap items-center gap-2">
        <ActionButton
          label="New project"
          icon={<Plus className="size-3.5" />}
          onClick={() => setNewProjectOpen(true)}
        />
        <ActionButton
          label="Park directory"
          icon={<FolderPlus className="size-3.5" />}
          onClick={() => setParkOpen(true)}
        />
        <ActionButton
          label="Link existing"
          icon={<Link2 className="size-3.5" />}
          onClick={() => setLinkOpen(true)}
        />
        <span
          data-mono
          className="ml-auto text-[11px] text-muted-foreground/60"
        >
          parked · linked · create (F-04/F-11)
        </span>
      </div>

      <NewProjectDialog open={newProjectOpen} onOpenChange={setNewProjectOpen} />
      <ParkDirectoryDialog open={parkOpen} onOpenChange={setParkOpen} />
      <LinkProjectDialog open={linkOpen} onOpenChange={setLinkOpen} />
      <DeleteProjectDialog
        open={deleteTarget !== null}
        onOpenChange={(o) => {
          if (!o) setDeleteTarget(null);
        }}
        project={
          projects.find((p) => p.id === deleteTarget)?.domain ?? ""
        }
        onConfirm={() => {
          if (deleteTarget) actions.remove(deleteTarget);
          setDeleteTarget(null);
        }}
      />

      <section className="flex flex-col gap-2">
        {filtered.length === 0 ? (
          <div className="flex flex-col items-center gap-2 rounded-lg border border-dashed border-border/60 py-16 text-center">
            <Folder className="size-6 text-muted-foreground/50" />
            <p className="text-sm text-muted-foreground">
              No projects match the current filter.
            </p>
          </div>
        ) : (
          filtered.map((p) => {
            const status = actions.statusFor(p);
            const running = status === "running" || status === "starting";
            return (
              <div
                key={p.id}
                className={cn(
                  "flex items-stretch rounded-lg border border-border/60 bg-card/40 transition-colors hover:border-border",
                )}
              >
                <button
                  type="button"
                  onClick={() => onOpenProject(p.id)}
                  className="group grid w-full min-w-0 flex-1 grid-cols-[minmax(0,1fr)_auto] items-center gap-3 px-4 py-3 text-left focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring sm:grid-cols-[minmax(0,1fr)_auto_auto_auto]"
                >
                  <span className="flex min-w-0 items-center gap-2.5">
                    <StatusDot tone={statusTone[status]} />
                    <span className="min-w-0">
                      <span className="flex items-center gap-1.5 text-sm font-medium text-foreground">
                        {p.name}
                        {p.pinned ? (
                          <Pin
                            role="img"
                            aria-label="Pinned"
                            className="size-3 text-muted-foreground/60"
                          />
                        ) : null}
                      </span>
                      <span className="block truncate text-xs text-muted-foreground">
                        {p.domain} · {p.framework}
                      </span>
                    </span>
                  </span>
                  <span
                    data-mono
                    className="hidden text-xs text-muted-foreground sm:block"
                  >
                    {p.runtime}
                  </span>
                  <span
                    data-mono
                    className="hidden text-xs text-muted-foreground md:block"
                  >
                    :{p.port}
                  </span>
                  <span className="flex items-center gap-1.5">
                    <span
                      data-mono
                      className="text-[11px] text-muted-foreground/80"
                    >
                      {status}
                    </span>
                    <ChevronRight
                      aria-hidden="true"
                      className="size-3.5 text-muted-foreground/40 transition-transform group-hover:translate-x-0.5 group-hover:text-foreground"
                    />
                  </span>
                </button>
                <div className="flex shrink-0 items-center gap-1 border-l border-border/40 pl-2 pr-2.5">
                  <button
                    type="button"
                    aria-label={running ? `Stop ${p.name}` : `Start ${p.name}`}
                    title={running ? "Stop" : "Start"}
                    onClick={() => {
                      if (running) {
                        actions.setStatus(p.id, "stopped");
                      } else if (actions.isTrusted(p.id)) {
                        actions.setStatus(p.id, "running");
                      } else {
                        preflight.request(p.id);
                      }
                    }}
                    className="rounded p-1.5 text-muted-foreground/70 transition-colors hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  >
                    {running ? (
                      <Square className="size-3.5" />
                    ) : (
                      <Play className="size-3.5" />
                    )}
                  </button>
                  <button
                    type="button"
                    aria-label={`Delete ${p.name}`}
                    title="Delete"
                    onClick={() => setDeleteTarget(p.id)}
                    className="rounded p-1.5 text-muted-foreground/60 transition-colors hover:bg-danger/10 hover:text-danger focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  >
                    <Trash2 className="size-3.5" />
                  </button>
                </div>
              </div>
            );
          })
        )}
      </section>
    </div>
  );
}

export { ProjectsScreen };