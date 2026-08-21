import { Cpu, Network, PanelRight, Server } from "lucide-react";
import { cn } from "@/lib/utils";
import { StatusDot } from "@/components/status/status-dot";
import {
  useDaemonStream,
  type ProjectStatus,
} from "@/hooks/use-daemon-stream";

interface StatusBarProps {
  className?: string;
  inspectorOpen: boolean;
  onToggleInspector: () => void;
}

const projectStatusTone: Record<
  ProjectStatus,
  "success" | "warning" | "muted"
> = {
  running: "success",
  starting: "warning",
  stopped: "muted",
};

function StatusBar({ className, inspectorOpen, onToggleInspector }: StatusBarProps) {
  const stream = useDaemonStream();
  const requests = String(stream.requests).padStart(4, "0");

  return (
    <footer
      data-slot="status-bar"
      className={cn(
        "flex h-7 shrink-0 items-center justify-between gap-3 border-t border-border/60 bg-status-bar-bg px-3 font-mono text-[11px] text-muted-foreground select-none",
        className,
      )}
    >
      <div className="flex items-center gap-3 overflow-hidden">
        <span className="flex items-center gap-1.5">
          <Server className="size-3 shrink-0" />
          <StatusDot tone="success" pulse />
          <span className="text-foreground">daemon</span>
          <span>{stream.daemon.version}</span>
          <span className="text-muted-foreground/60">·</span>
          <span>IPC {stream.daemon.ipc}</span>
        </span>
        <span className="text-muted-foreground/40">·</span>
        <span className="flex items-center gap-1.5 truncate">
          <Network className="size-3 shrink-0" />
          <span className="text-foreground">{stream.project.name}</span>
          <StatusDot tone={projectStatusTone[stream.project.status]} />
          <span>{stream.project.status}</span>
          <span className="text-muted-foreground/60">:</span>
          <span>{stream.project.port}</span>
        </span>
        <span className="hidden items-center gap-1.5 truncate md:inline-flex">
          <span className="text-muted-foreground/40">·</span>
          <Cpu className="size-3 shrink-0" />
          <span>{stream.project.runtime}</span>
        </span>
      </div>
      <div className="flex shrink-0 items-center gap-3">
        <span className="hidden items-center gap-1.5 lg:inline-flex">
          <span className="size-1.5 rounded-full bg-info" />
          <span>REQ {requests}</span>
        </span>
        <span className="hidden items-center gap-1.5 lg:inline-flex">
          <span className="size-1.5 rounded-full bg-info" />
          <span>{stream.services} services</span>
        </span>
        <button
          type="button"
          onClick={onToggleInspector}
          aria-pressed={inspectorOpen}
          aria-label={inspectorOpen ? "Hide inspector" : "Show inspector"}
          title={inspectorOpen ? "Hide inspector" : "Show inspector"}
          className={cn(
            "inline-flex items-center gap-1.5 rounded px-1.5 py-0.5 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
            inspectorOpen
              ? "text-foreground"
              : "text-muted-foreground hover:text-foreground",
          )}
        >
          <PanelRight className="size-3" />
          <span>inspector</span>
          <span
            aria-hidden="true"
            className={cn(
              "ml-0.5 size-1 rounded-full transition-colors",
              inspectorOpen ? "bg-primary" : "bg-muted-foreground/40",
            )}
          />
        </button>
      </div>
    </footer>
  );
}

export { StatusBar };