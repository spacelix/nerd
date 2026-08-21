import {
  Activity,
  ArrowDown,
  ArrowUp,
  CornerDownLeft,
  Database,
  Folder,
  FolderPlus,
  Home,
  Layers,
  Link2,
  Mail,
  Monitor,
  Moon,
  PackagePlus,
  Plus,
  Search,
  Settings,
  Sun,
  Wrench,
  type LucideIcon,
} from "lucide-react";
import { Command } from "cmdk";
import * as Dialog from "@radix-ui/react-dialog";
import * as React from "react";
import { cn } from "@/lib/utils";
import { Kbd } from "@/components/ui/kbd";
import type { Route } from "@/components/shell/app-sidebar";
import type { WorkingSession } from "@/components/shell/working-session-toggle";
import { useQuickActions, type QuickAction } from "@/hooks/use-quick-actions";
import { useTheme } from "@/hooks/use-theme";
import { projects } from "@/mocks/data";

type GroupName =
  | "Go to"
  | "Projects"
  | "Runtimes"
  | "Services"
  | "Mail"
  | "Actions"
  | "Theme";

interface PaletteEntry {
  id: string;
  group: GroupName;
  label: string;
  icon: LucideIcon;
  hint?: string;
  keywords?: string[];
  route?: Route;
  projectId?: string;
  action?: QuickAction;
  theme?: "light" | "dark" | "system";
}

const routeEntries: ReadonlyArray<PaletteEntry> = [
  { id: "go-overview", group: "Go to", label: "Overview", icon: Home, route: "overview" },
  { id: "go-projects", group: "Go to", label: "Projects", icon: Folder, route: "projects" },
  { id: "go-runtimes", group: "Go to", label: "Runtimes", icon: Layers, route: "runtimes" },
  { id: "go-services", group: "Go to", label: "Services", icon: Database, route: "services" },
  { id: "go-mail", group: "Go to", label: "Mail", icon: Mail, keywords: ["inbox", "messages"], route: "mail" },
  { id: "go-inspector", group: "Go to", label: "Inspector", icon: Activity, keywords: ["requests", "capture"], route: "inspector" },
  { id: "go-diagnostics", group: "Go to", label: "Diagnostics", icon: Wrench, keywords: ["dns", "certs", "ports"], route: "diagnostics" },
  { id: "go-settings", group: "Go to", label: "Settings", icon: Settings, keywords: ["preferences"], route: "settings" },
];

const resourceEntries: ReadonlyArray<PaletteEntry> = [
  { id: "runtime-node-22", group: "Runtimes", label: "Node 22.11.0", icon: Layers, hint: "default", route: "runtimes" },
  { id: "runtime-node-20", group: "Runtimes", label: "Node 20.18.0", icon: Layers, hint: "managed", route: "runtimes" },
  { id: "runtime-node-18", group: "Runtimes", label: "Node 18.19.0", icon: Layers, hint: "external", route: "runtimes" },
  { id: "service-mysql", group: "Services", label: "MySQL", icon: Database, hint: "8.0 · running", keywords: ["sql", "database"], route: "services" },
  { id: "service-postgres", group: "Services", label: "PostgreSQL", icon: Database, hint: "16 · stopped", keywords: ["sql", "database"], route: "services" },
  { id: "service-redis", group: "Services", label: "Redis", icon: Database, hint: "7 · running", keywords: ["cache"], route: "services" },
  { id: "mail-welcome", group: "Mail", label: "Welcome — confirm your email", icon: Mail, hint: "unread", route: "mail" },
  { id: "mail-invoice", group: "Mail", label: "Payment receipt #4815", icon: Mail, hint: "unread", route: "mail" },
  { id: "mail-cron", group: "Mail", label: "Cron failed: rotate-tokens", icon: Mail, hint: "read", keywords: ["alerts"], route: "mail" },
];

const actionEntries: ReadonlyArray<PaletteEntry> = [
  { id: "action-new-project", group: "Actions", label: "New project", icon: Plus, action: "new-project", keywords: ["scaffold", "create", "wizard"] },
  { id: "action-park", group: "Actions", label: "Park directory", icon: FolderPlus, action: "park-directory", keywords: ["watch", "discover"] },
  { id: "action-link", group: "Actions", label: "Link existing project", icon: Link2, action: "link-project", keywords: ["register"] },
  { id: "action-install-node", group: "Actions", label: "Install Node", icon: Layers, action: "install-node", keywords: ["runtime", "version", "download"] },
  { id: "action-add-service", group: "Actions", label: "Add service", icon: PackagePlus, action: "add-service", keywords: ["mysql", "postgres", "redis", "database"] },
];

const themeEntries: ReadonlyArray<PaletteEntry> = [
  { id: "theme-light", group: "Theme", label: "Light", icon: Sun, theme: "light" },
  { id: "theme-dark", group: "Theme", label: "Dark", icon: Moon, theme: "dark" },
  { id: "theme-system", group: "Theme", label: "System", icon: Monitor, theme: "system" },
];

const groups: ReadonlyArray<GroupName> = [
  "Go to",
  "Projects",
  "Runtimes",
  "Services",
  "Mail",
  "Actions",
  "Theme",
];

interface CommandMenuProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onNavigate: (route: Route, projectId?: string) => void;
  workingSession: WorkingSession;
}

function CommandMenu({
  open,
  onOpenChange,
  onNavigate,
  workingSession,
}: CommandMenuProps) {
  const quick = useQuickActions();
  const { theme, setTheme } = useTheme();

  const run = (entry: PaletteEntry): void => {
    if (entry.action) {
      quick.request(entry.action);
    } else if (entry.theme) {
      setTheme(entry.theme);
    } else if (entry.route) {
      onNavigate(entry.route, entry.projectId);
    }
    onOpenChange(false);
  };

  const projectEntries = React.useMemo<ReadonlyArray<PaletteEntry>>(() => {
    const visible = projects.filter((p) => {
      if (workingSession === "all") return true;
      if (workingSession === "active") return p.pinned;
      return !p.pinned;
    });
    return visible.map((p) => ({
      id: `project-${p.id}`,
      group: "Projects",
      label: p.domain,
      icon: Folder,
      hint: `${p.status} · :${p.port}`,
      keywords: [p.name, p.framework, p.runtime],
      route: "projects",
      projectId: p.id,
    }));
  }, [workingSession]);

  const entries = React.useMemo(
    () => [...routeEntries, ...projectEntries, ...resourceEntries, ...actionEntries, ...themeEntries],
    [projectEntries],
  );

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay
          data-slot="command-menu-overlay"
          className="fixed inset-0 z-50 bg-background/60 backdrop-blur-sm data-[state=open]:animate-[cmdk-overlay-in_120ms_ease-out] data-[state=closed]:animate-[cmdk-overlay-out_120ms_ease-in]"
        />
        <Dialog.Content
          data-slot="command-menu"
          aria-label="Command palette"
          className="fixed left-1/2 top-1/2 z-50 -translate-x-1/2 -translate-y-1/2 w-[min(560px,calc(100vw-2rem))] overflow-hidden rounded-xl border border-border/70 bg-popover text-popover-foreground shadow-2xl shadow-black/40 data-[state=open]:animate-[cmdk-content-in_160ms_ease-out] data-[state=closed]:animate-[cmdk-content-out_140ms_ease-in]"
        >
          <Command label="Command palette">
            <div className="flex h-12 items-center gap-2.5 border-b border-border/60 px-4">
              <Search
                aria-hidden="true"
                className="size-4 shrink-0 text-muted-foreground"
              />
              <Command.Input
                autoFocus
                placeholder="Search projects, runtimes, services, mail…"
                aria-label="Search Nerd"
                className="h-full min-w-0 flex-1 bg-transparent text-sm text-foreground outline-none placeholder:text-muted-foreground/60"
              />
              <Kbd>esc</Kbd>
            </div>

            <Command.List className="max-h-[min(50vh,24rem)] overflow-y-auto p-1.5">
              <Command.Empty className="py-10 text-center text-sm text-muted-foreground">
                No results found.
              </Command.Empty>
              {groups.map((group) => {
                const groupEntries = entries.filter((e) => e.group === group);
                if (groupEntries.length === 0) return null;
                return (
                  <Command.Group
                    key={group}
                    heading={group}
                    className="[&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:py-1.5 [&_[cmdk-group-heading]]:text-[10px] [&_[cmdk-group-heading]]:font-semibold [&_[cmdk-group-heading]]:tracking-[0.16em] [&_[cmdk-group-heading]]:text-muted-foreground/70 [&_[cmdk-group-heading]]:uppercase"
                  >
                    {groupEntries.map((entry) => (
                      <Command.Item
                        key={entry.id}
                        value={`${entry.group} ${entry.label} ${entry.hint ?? ""} ${entry.keywords?.join(" ") ?? ""}`}
                        onSelect={() => run(entry)}
                        className={cn(
                          "flex cursor-pointer items-center gap-2.5 rounded-md px-2.5 py-2 text-sm text-muted-foreground",
                          "data-[selected=true]:bg-accent data-[selected=true]:text-accent-foreground",
                          "data-[disabled=true]:cursor-default data-[disabled=true]:opacity-50",
                          "focus:outline-none",
                        )}
                      >
                        <entry.icon
                          aria-hidden="true"
                          className="size-4 shrink-0 text-muted-foreground"
                        />
                        <span className="flex-1 truncate text-left text-foreground">
                          {entry.label}
                        </span>
                        {entry.hint ? (
                          <span
                            data-mono
                            className="shrink-0 text-[10px] text-muted-foreground/70"
                          >
                            {entry.hint}
                          </span>
                        ) : entry.theme ? (
                          <span
                            data-mono
                            className={cn(
                              "shrink-0 text-[10px]",
                              entry.theme === theme
                                ? "text-primary"
                                : "text-muted-foreground/70",
                            )}
                          >
                            {entry.theme === theme ? "active" : ""}
                          </span>
                        ) : null}
                      </Command.Item>
                    ))}
                  </Command.Group>
                );
              })}
            </Command.List>

            <div className="flex h-9 items-center justify-between border-t border-border/60 px-4 text-[10px] text-muted-foreground/70">
              <span className="flex items-center gap-2">
                <span className="flex items-center gap-0.5">
                  <Kbd>
                    <ArrowUp className="size-2.5" />
                  </Kbd>
                  <Kbd>
                    <ArrowDown className="size-2.5" />
                  </Kbd>
                </span>
                navigate
              </span>
              <span className="flex items-center gap-2">
                <span className="flex items-center gap-0.5">
                  <Kbd>
                    <CornerDownLeft className="size-2.5" />
                  </Kbd>
                </span>
                select
              </span>
            </div>
          </Command>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

export { CommandMenu };