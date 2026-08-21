import {
  Folder,
  Globe,
  PanelRight,
  Search,
  TerminalSquare,
  Volume2,
  Wifi,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { NerdTray } from "@/components/desktop/tray";
import type { Route } from "@/components/shell/app-sidebar";

interface TaskbarProps {
  className?: string;
  onNavigate: (route: Route) => void;
}

interface TaskbarIconProps {
  icon: React.ComponentType<{ className?: string }>;
  label: string;
  active?: boolean;
}

function TaskbarIcon({ icon: Icon, label, active }: TaskbarIconProps) {
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      className={cn(
        "group relative inline-flex size-10 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-foreground/10 hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
      )}
    >
      <Icon className="size-5" />
      <span
        aria-hidden="true"
        className={cn(
          "absolute inset-x-2 bottom-0.5 h-0.5 rounded-full transition-opacity",
          active ? "bg-primary opacity-100" : "bg-foreground opacity-0",
        )}
      />
    </button>
  );
}

function Taskbar({ className, onNavigate }: TaskbarProps) {
  return (
    <div
      role="toolbar"
      aria-label="Taskbar"
      data-slot="taskbar"
      className={cn(
        "pointer-events-auto fixed inset-x-0 bottom-0 z-10 flex h-14 items-end justify-center border-t px-3 pb-2",
        className,
      )}
      style={{
        background: "var(--desktop-taskbar-bg)",
        borderColor: "var(--desktop-taskbar-border)",
        backdropFilter: "blur(40px) saturate(180%)",
        WebkitBackdropFilter: "blur(40px) saturate(180%)",
      }}
    >
      <div className="flex items-center gap-1">
        <TaskbarIcon icon={Search} label="Search" />
        <TaskbarIcon icon={Folder} label="File Explorer" />
        <TaskbarIcon icon={Globe} label="Browser" />
        <TaskbarIcon icon={TerminalSquare} label="Terminal" />
      </div>

      <div
        aria-hidden="true"
        className="mx-2 h-8 w-px bg-border/60"
      />

      <div className="flex items-center gap-1">
        <TaskbarIcon icon={Folder} label="app.test" active />
      </div>

      <div className="ml-auto flex items-center gap-1">
        <NerdTray onNavigate={onNavigate} />
        <TaskbarIcon icon={Wifi} label="Network" />
        <TaskbarIcon icon={Volume2} label="Volume" />
        <TaskbarIcon icon={PanelRight} label="Inspector" />
      </div>
    </div>
  );
}

export { Taskbar };
