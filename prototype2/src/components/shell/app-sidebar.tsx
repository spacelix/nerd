import {
  Activity,
  Database,
  Folder,
  Home,
  Layers,
  Mail,
  Monitor,
  Moon,
  Search,
  Settings,
  Sun,
  Wrench,
} from "lucide-react";
import * as React from "react";
import { cn } from "@/lib/utils";
import { useTheme, type Theme } from "@/hooks/use-theme";
import { Kbd } from "@/components/ui/kbd";

export type Route =
  | "overview"
  | "projects"
  | "runtimes"
  | "services"
  | "mail"
  | "inspector"
  | "diagnostics"
  | "settings";

interface NavItem {
  route: Route;
  icon: React.ComponentType<{ className?: string }>;
  label: string;
  badge?: string;
}

interface NavSection {
  label: string;
  items: NavItem[];
}

const sections: ReadonlyArray<NavSection> = [
  {
    label: "Workspace",
    items: [
      { route: "overview", icon: Home, label: "Overview" },
      { route: "projects", icon: Folder, label: "Projects" },
      { route: "runtimes", icon: Layers, label: "Runtimes" },
      { route: "services", icon: Database, label: "Services" },
    ],
  },
  {
    label: "Observability",
    items: [
      { route: "mail", icon: Mail, label: "Mail", badge: "2" },
      { route: "inspector", icon: Activity, label: "Inspector" },
    ],
  },
  {
    label: "System",
    items: [
      { route: "diagnostics", icon: Wrench, label: "Diagnostics" },
      { route: "settings", icon: Settings, label: "Settings" },
    ],
  },
];

interface AppSidebarProps {
  active: Route;
  onNavigate?: (route: Route) => void;
  onOpenCommand?: () => void;
  className?: string;
}

function ThemeButton({ theme, cycle }: { theme: Theme; cycle: () => void }) {
  const Icon = theme === "dark" ? Moon : theme === "light" ? Sun : Monitor;
  const label =
    theme === "dark" ? "Dark" : theme === "light" ? "Light" : "System";
  return (
    <button
      type="button"
      onClick={cycle}
      aria-label={`Theme: ${label}. Click to change.`}
      className="inline-flex h-7 shrink-0 items-center gap-1 rounded-md px-1.5 text-[11px] text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
    >
      <Icon className="size-3" />
      <span>{label}</span>
    </button>
  );
}

function AppSidebar({
  active,
  onNavigate,
  onOpenCommand,
  className,
}: AppSidebarProps) {
  const { theme, cycle } = useTheme();

  return (
    <aside
      data-slot="app-sidebar"
      aria-label="Primary navigation"
      className={cn(
        "flex h-full w-full flex-col border-r border-border/40 bg-sidebar/70 backdrop-blur-md",
        className,
      )}
    >
      <div className="shrink-0 border-b border-border/40 px-3 py-3">
        <button
          type="button"
          onClick={onOpenCommand}
          aria-label="Open search"
          title="Open search (⌘K)"
          className="flex h-8 w-full items-center gap-2 rounded-md border border-border/60 bg-background/60 px-2.5 text-sm text-muted-foreground/70 transition-colors hover:bg-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        >
          <Search
            aria-hidden="true"
            className="size-3.5 shrink-0 text-muted-foreground"
          />
          <span className="flex-1 text-left">Search</span>
          <Kbd className="pointer-events-none">⌘K</Kbd>
        </button>
      </div>

      <nav
        aria-label="Main"
        className="flex-1 overflow-y-auto px-2 py-3"
      >
        {sections.map((section, idx) => (
          <div
            key={section.label}
            className={cn(idx > 0 ? "mt-4" : "")}
          >
            <div className="mb-1 px-2 text-[10px] font-semibold tracking-[0.16em] text-muted-foreground/70 uppercase">
              {section.label}
            </div>
            <ul className="space-y-0.5">
              {section.items.map((item) => {
                const isActive = item.route === active;
                const Icon = item.icon;
                return (
                  <li key={item.route}>
                    <button
                      type="button"
                      onClick={() => onNavigate?.(item.route)}
                      aria-current={isActive ? "page" : undefined}
                      className={cn(
                        "flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-sm transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                        isActive
                          ? "bg-surface-active text-foreground"
                          : "text-muted-foreground hover:bg-accent/50 hover:text-foreground",
                      )}
                    >
                      <Icon className="size-4 shrink-0" />
                      <span className="flex-1 text-left">{item.label}</span>
                      {item.badge ? (
                        <span
                          data-mono
                          className="inline-flex h-4 min-w-4 items-center justify-center rounded-full bg-success-soft px-1 text-[10px] font-medium text-success"
                        >
                          {item.badge}
                        </span>
                      ) : null}
                    </button>
                  </li>
                );
              })}
            </ul>
          </div>
        ))}
      </nav>

      <div className="flex shrink-0 items-center justify-between gap-1 border-t border-border/40 px-3 py-2">
        <span
          data-mono
          className="shrink-0 text-[10px] text-muted-foreground/70"
        >
          v0.1.0-alpha.1
        </span>
        <ThemeButton theme={theme} cycle={cycle} />
      </div>
    </aside>
  );
}

export { AppSidebar };
