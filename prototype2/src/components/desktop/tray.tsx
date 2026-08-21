import {
  Activity,
  MonitorX,
  Play,
  Power,
  Square,
  Stethoscope,
  AppWindow,
} from "lucide-react";
import * as React from "react";
import { cn } from "@/lib/utils";
import { StatusDot } from "@/components/status/status-dot";
import { useDaemonStream } from "@/hooks/use-daemon-stream";
import { useProjectActions } from "@/hooks/use-project-actions";
import { useProjectPreflight } from "@/hooks/use-project-preflight";
import { projects } from "@/mocks/data";
import type { Route } from "@/components/shell/app-sidebar";

interface NerdTrayProps {
  onNavigate: (route: Route) => void;
}

function NerdTray({ onNavigate }: NerdTrayProps) {
  const snapshot = useDaemonStream();
  const actions = useProjectActions();
  const preflight = useProjectPreflight();
  const [open, setOpen] = React.useState(false);
  const [confirming, setConfirming] = React.useState<
    "stop-daemon" | "quit-gui" | null
  >(null);
  const [stopped, setStopped] = React.useState(false);
  const ref = React.useRef<HTMLDivElement>(null);
  const runningCount = projects.filter(
    (p) => !actions.isRemoved(p.id) && actions.statusFor(p) === "running",
  ).length;
  const recent = projects.slice(0, 4);

  React.useEffect(() => {
    if (!open) return;
    const onDown = (e: MouseEvent): void => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    const onKey = (e: KeyboardEvent): void => {
      if (e.key === "Escape") setOpen(false);
    };
    window.addEventListener("mousedown", onDown);
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("mousedown", onDown);
      window.removeEventListener("keydown", onKey);
    };
  }, [open]);

  const closeMenu = (): void => {
    setOpen(false);
    setConfirming(null);
  };

  return (
    <div ref={ref} className="relative">
      <button
        type="button"
        aria-label="Nerd tray"
        aria-expanded={open}
        title="Nerd tray — daemon health and quick actions"
        onClick={() => setOpen((o) => !o)}
        className="group relative inline-flex size-10 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-foreground/10 hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
      >
        <Activity className="size-5" />
        <span
          aria-hidden="true"
          className={cn(
            "absolute right-1.5 bottom-1.5 size-1.5 rounded-full",
            snapshot.daemon.connected ? "bg-success" : "bg-danger",
          )}
        />
      </button>

      {open ? (
        <div
          role="menu"
          aria-label="Nerd tray"
          className="absolute right-0 bottom-12 z-20 w-72 overflow-hidden rounded-xl border border-border bg-popover text-popover-foreground shadow-lg"
        >
          <div className="flex items-center justify-between gap-2 border-b border-border/60 px-3 py-2">
            <span
              role="status"
              aria-atomic="true"
              className="flex items-center gap-2 text-xs font-semibold text-foreground"
            >
              <StatusDot
                tone={snapshot.daemon.connected ? "success" : "danger"}
              />
              Nerd daemon
            </span>
            <span data-mono className="text-[10px] text-muted-foreground/60">
              {snapshot.daemon.version} · {runningCount} running
            </span>
          </div>

          {confirming === "stop-daemon" ? (
            <div className="flex flex-col gap-2 p-3">
              <p className="text-[11px] leading-relaxed text-foreground/90">
                Stop the daemon and all managed projects? This is the explicit
                separate action required — closing the window never does this.
              </p>
              <div className="flex items-center justify-end gap-2">
                <button
                  type="button"
                  onClick={() => setConfirming(null)}
                  className="rounded-md px-2.5 py-1 text-[11px] text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                >
                  Cancel
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setStopped(true);
                    setConfirming(null);
                  }}
                  className="rounded-md bg-danger px-2.5 py-1 text-[11px] font-medium text-danger-foreground transition-opacity hover:opacity-90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                >
                  Stop daemon
                </button>
              </div>
            </div>
          ) : confirming === "quit-gui" ? (
            <div className="flex flex-col gap-2 p-3">
              <p className="text-[11px] leading-relaxed text-foreground/90">
                Quit the Nerd GUI? The daemon and projects keep running — you
                can reopen Nerd anytime.
              </p>
              <div className="flex items-center justify-end gap-2">
                <button
                  type="button"
                  onClick={() => setConfirming(null)}
                  className="rounded-md px-2.5 py-1 text-[11px] text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                >
                  Cancel
                </button>
                <button
                  type="button"
                  onClick={closeMenu}
                  className="rounded-md bg-primary px-2.5 py-1 text-[11px] font-medium text-primary-foreground transition-opacity hover:opacity-90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                >
                  Quit GUI
                </button>
              </div>
            </div>
          ) : (
            <>
              <div className="flex flex-col p-1.5">
                <button
                  type="button"
                  role="menuitem"
                  onClick={() => {
                    onNavigate("overview");
                    closeMenu();
                  }}
                  className="flex items-center gap-2 rounded-md px-2 py-1.5 text-left text-xs text-foreground/90 transition-colors hover:bg-accent/60 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                >
                  <AppWindow className="size-3.5 text-muted-foreground" />
                  Open app
                </button>
                <button
                  type="button"
                  role="menuitem"
                  onClick={() => {
                    onNavigate("diagnostics");
                    closeMenu();
                  }}
                  className="flex items-center gap-2 rounded-md px-2 py-1.5 text-left text-xs text-foreground/90 transition-colors hover:bg-accent/60 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                >
                  <Stethoscope className="size-3.5 text-muted-foreground" />
                  Diagnostics
                </button>
                <button
                  type="button"
                  role="menuitem"
                  onClick={() => setConfirming("stop-daemon")}
                  className="flex items-center gap-2 rounded-md px-2 py-1.5 text-left text-xs text-foreground/90 transition-colors hover:bg-accent/60 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                >
                  <Power className="size-3.5 text-muted-foreground" />
                  Stop daemon
                </button>
                <button
                  type="button"
                  role="menuitem"
                  onClick={() => setConfirming("quit-gui")}
                  className="flex items-center gap-2 rounded-md px-2 py-1.5 text-left text-xs text-foreground/90 transition-colors hover:bg-accent/60 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                >
                  <MonitorX className="size-3.5 text-muted-foreground" />
                  Quit GUI
                </button>
              </div>

              <div className="border-t border-border/60" />

              <div className="flex max-h-44 flex-col overflow-y-auto p-1.5">
                {recent.map((p) => {
                  const status = actions.statusFor(p);
                  const running = status === "running" || status === "starting";
                  return (
                    <div
                      key={p.id}
                      className="flex items-center gap-2 rounded-md px-2 py-1.5 hover:bg-accent/40"
                    >
                      <StatusDot tone={running ? "success" : "muted"} />
                      <span className="min-w-0 flex-1 truncate text-xs text-foreground/90">
                        {p.name}
                      </span>
                      <span
                        data-mono
                        className="text-[10px] text-muted-foreground/60"
                      >
                        {status}
                      </span>
                      <button
                        type="button"
                        aria-label={
                          running ? `Stop ${p.name}` : `Start ${p.name}`
                        }
                        onClick={() => {
                          if (running) {
                            actions.setStatus(p.id, "stopped");
                          } else if (actions.isTrusted(p.id)) {
                            actions.setStatus(p.id, "running");
                          } else {
                            preflight.request(p.id);
                            closeMenu();
                          }
                        }}
                        className="rounded p-1 text-muted-foreground/70 transition-colors hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                      >
                        {running ? (
                          <Square className="size-3" />
                        ) : (
                          <Play className="size-3" />
                        )}
                      </button>
                    </div>
                  );
                })}
              </div>

              <p
                role="status"
                aria-live="polite"
                className="border-t border-border/60 px-3 py-2 text-[10px] leading-relaxed text-muted-foreground/70"
              >
                {stopped
                  ? "Daemon stopped. Reopen the app to start it again."
                  : "Closing the window never stops the daemon or projects. Stopping the daemon requires the explicit action above."}
              </p>
            </>
          )}
        </div>
      ) : null}
    </div>
  );
}

export { NerdTray };