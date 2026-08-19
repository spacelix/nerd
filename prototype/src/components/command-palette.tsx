import * as React from "react";
import {
  Activity,
  Boxes,
  CircleDashed,
  FolderTree,
  Gauge,
  LayoutDashboard,
  Mail,
  Moon,
  Plus,
  Search,
  Settings,
  Sun,
  Trash2,
  TriangleAlert,
} from "lucide-react";
import { useTheme } from "@/app/useTheme";
import { useRoute } from "@/app/RouteContext";
import type { RouteId } from "@/app/router";
import { PROJECTS } from "@/mocks/projects";
import {
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandShortcut,
} from "@/components/ui/command";
import type { ProjectStatus } from "@/lib/types";

type IconComponent = React.ComponentType<{ size?: number; className?: string }>;

const STATUS_ICON: Record<ProjectStatus, IconComponent> = {
  running: Activity as unknown as IconComponent,
  starting: CircleDashed as unknown as IconComponent,
  installing: CircleDashed as unknown as IconComponent,
  waiting: CircleDashed as unknown as IconComponent,
  stopped: CircleDashed as unknown as IconComponent,
  degraded: TriangleAlert as unknown as IconComponent,
  failed: Trash2 as unknown as IconComponent,
};

const ROUTE_ITEMS: {
  id: RouteId;
  label: string;
  Icon: IconComponent;
  keywords: string[];
}[] = [
  {
    id: "overview",
    label: "Overview",
    Icon: LayoutDashboard as unknown as IconComponent,
    keywords: ["dashboard", "status", "home"],
  },
  {
    id: "projects",
    label: "Projects",
    Icon: FolderTree as unknown as IconComponent,
    keywords: ["park", "linked", "workspace"],
  },
  {
    id: "runtimes",
    label: "Runtimes",
    Icon: Boxes as unknown as IconComponent,
    keywords: ["node", "versions", "lts"],
  },
  {
    id: "services",
    label: "Services",
    Icon: Gauge as unknown as IconComponent,
    keywords: ["mysql", "postgres", "redis", "databases"],
  },
  {
    id: "mail",
    label: "Mail",
    Icon: Mail as unknown as IconComponent,
    keywords: ["smtp", "capture", "inbox"],
  },
  {
    id: "inspector",
    label: "Inspector",
    Icon: Search as unknown as IconComponent,
    keywords: ["http", "requests", "debug"],
  },
  {
    id: "diagnostics",
    label: "Diagnostics",
    Icon: Activity as unknown as IconComponent,
    keywords: ["dns", "ca", "daemon", "ports", "doctor"],
  },
  {
    id: "settings",
    label: "Settings",
    Icon: Settings as unknown as IconComponent,
    keywords: ["preferences", "theme", "retention"],
  },
];

const THEME_ITEMS = [
  { id: "light" as const, label: "Theme · Light", Icon: Sun as unknown as IconComponent },
  { id: "dark" as const, label: "Theme · Dark", Icon: Moon as unknown as IconComponent },
  { id: "system" as const, label: "Theme · System", Icon: Gauge as unknown as IconComponent },
];

function isMacPlatform() {
  return (
    typeof navigator !== "undefined" &&
    /Mac|iPhone|iPad/i.test(navigator.platform || navigator.userAgent || "")
  );
}

type CommandPaletteProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
};

export function CommandPalette({ open, onOpenChange }: CommandPaletteProps) {
  const { setRoute, setSelectedProjectId } = useRoute();
  const { mode, setMode, resolvedTheme } = useTheme();
  const [query, setQuery] = React.useState("");
  const mac = isMacPlatform();
  const mod = mac ? "⌘" : "Ctrl";

  const running = PROJECTS.filter((p) => p.status === "running");

  const close = React.useCallback(() => {
    onOpenChange(false);
  }, [onOpenChange]);

  React.useEffect(() => {
    if (!open) return;
    const handler = (event: KeyboardEvent) => {
      const meta = event.metaKey || event.ctrlKey;
      if (!meta) return;
      if (event.altKey) return;
      const key = event.key.toLowerCase();
      switch (key) {
        case "p":
          event.preventDefault();
          setSelectedProjectId(null);
          setRoute("projects");
          close();
          break;
        case "r":
          event.preventDefault();
          setSelectedProjectId(null);
          setRoute("overview");
          close();
          break;
        case ",":
          event.preventDefault();
          setSelectedProjectId(null);
          setRoute("settings");
          close();
          break;
        case "/":
          event.preventDefault();
          setMode(mode === "light" ? "dark" : mode === "dark" ? "system" : "light");
          close();
          break;
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [open, mode, setMode, setRoute, setSelectedProjectId, close]);

  const openProject = (id: string) => {
    setSelectedProjectId(id);
    setRoute("projects");
    close();
  };

  return (
    <CommandDialog
      open={open}
      onOpenChange={onOpenChange}
      title="Command Palette"
      description="Navigate Nerd or run quick actions."
    >
      <CommandInput
        placeholder="Search Nerd or type a command…"
        value={query}
        onValueChange={setQuery}
      />
      <CommandList>
        <CommandEmpty>No matches.</CommandEmpty>

        <CommandGroup heading="Navigate">
          {ROUTE_ITEMS.map((item) => (
            <CommandItem
              key={item.id}
              value={`${item.label} ${item.keywords.join(" ")}`}
              onSelect={() => {
                setSelectedProjectId(null);
                setRoute(item.id);
                close();
              }}
            >
              <item.Icon />
              <span>{item.label}</span>
            </CommandItem>
          ))}
        </CommandGroup>

        <CommandGroup heading={`Projects (${PROJECTS.length})`}>
          {PROJECTS.map((project) => {
            const Icon = STATUS_ICON[project.status];
            return (
              <CommandItem
                key={project.id}
                value={`${project.name} ${project.domain} ${project.frameworkLabel} ${project.status} ${project.port}`}
                onSelect={() => openProject(project.id)}
              >
                <Icon />
                <span className="font-medium">{project.name}</span>
                <span className="font-mono text-xs text-muted-foreground">
                  :{project.port}
                </span>
                <CommandShortcut>{project.status}</CommandShortcut>
              </CommandItem>
            );
          })}
        </CommandGroup>

        <CommandGroup heading="Actions">
          <CommandItem
            value="Start a project park folder create"
            onSelect={() => {
              setRoute("projects");
              setSelectedProjectId(null);
              close();
            }}
          >
            <Plus />
            <span>Start a project</span>
            <CommandShortcut>
              <span className="mr-1">{mod}</span>P
            </CommandShortcut>
          </CommandItem>
          <CommandItem
            value="Refresh overview status"
            onSelect={() => {
              setRoute("overview");
              close();
            }}
          >
            <Activity />
            <span>Refresh overview</span>
            <CommandShortcut>
              <span className="mr-1">{mod}</span>R
            </CommandShortcut>
          </CommandItem>
          <CommandItem
            value="Settings preferences"
            onSelect={() => {
              setRoute("settings");
              close();
            }}
          >
            <Settings />
            <span>Settings</span>
            <CommandShortcut>
              <span className="mr-1">{mod}</span>,
            </CommandShortcut>
          </CommandItem>
          {running.length > 0 && (
            <CommandItem
              value="Stop all running projects"
              onSelect={() => {
                setRoute("overview");
                close();
              }}
            >
              <Trash2 />
              <span>Stop all running projects</span>
              <CommandShortcut>
                {running.length} running
              </CommandShortcut>
            </CommandItem>
          )}
        </CommandGroup>

        <CommandGroup heading="Theme">
          {THEME_ITEMS.map((item) => (
            <CommandItem
              key={item.id}
              value={`Theme ${item.label}`}
              onSelect={() => {
                setMode(item.id);
                close();
              }}
            >
              <item.Icon />
              <span>{item.label}</span>
              {mode === item.id && (
                <CommandShortcut>
                  {resolvedTheme === item.id ? "active" : "—"}
                </CommandShortcut>
              )}
            </CommandItem>
          ))}
        </CommandGroup>
      </CommandList>
    </CommandDialog>
  );
}
