import * as React from "react";
import { useRoute } from "@/app/RouteContext";
import { AppSidebar } from "@/components/app-sidebar";
import { CommandPalette } from "@/components/command-palette";
import { DiagnosticsScreen } from "@/components/diagnostics-screen";
import { InspectorScreen } from "@/components/inspector-screen";
import { MailScreen } from "@/components/mail-screen";
import { OverviewScreen } from "@/components/overview-screen";
import { ProjectsScreen } from "@/components/projects-screen";
import { RuntimesScreen } from "@/components/runtimes-screen";
import { ServicesScreen } from "@/components/services-screen";
import { SettingsScreen } from "@/components/settings-screen";
import { SiteHeader } from "@/components/site-header";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { TooltipProvider } from "@/components/ui/tooltip";
import type { RouteId } from "@/app/router";

const SHELL_STYLE = {
  "--sidebar-width": "calc(var(--spacing) * 60)",
  "--sidebar-width-icon": "calc(var(--spacing) * 12)",
  "--header-height": "calc(var(--spacing) * 12)",
} as React.CSSProperties;

const SCREENS: Record<RouteId, React.ComponentType> = {
  overview: OverviewScreen,
  projects: ProjectsScreen,
  runtimes: RuntimesScreen,
  services: ServicesScreen,
  mail: MailScreen,
  inspector: InspectorScreen,
  diagnostics: DiagnosticsScreen,
  settings: SettingsScreen,
};

export function DashboardPage() {
  const { route } = useRoute();
  const [commandOpen, setCommandOpen] = React.useState(false);

  React.useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      const isMeta = event.metaKey || event.ctrlKey;
      if (isMeta && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setCommandOpen((open) => !open);
      }
      if (event.key === "Escape" && commandOpen) {
        setCommandOpen(false);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [commandOpen]);

  const Screen = SCREENS[route];

  return (
    <div className="flex h-svh w-full flex-col overflow-hidden bg-background">
      <TooltipProvider delayDuration={150}>
        <SidebarProvider style={SHELL_STYLE}>
          <div className="flex h-full min-h-0 flex-1">
            <AppSidebar />
            <SidebarInset>
              <SiteHeader
                route={route}
                onOpenCommandPalette={() => setCommandOpen(true)}
              />
              <div className="min-h-0 flex-1">
                <Screen />
              </div>
            </SidebarInset>
          </div>
        </SidebarProvider>
      </TooltipProvider>
      <CommandPalette open={commandOpen} onOpenChange={setCommandOpen} />
    </div>
  );
}
