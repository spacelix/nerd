import { Moon, Search, Sun } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { SidebarTrigger } from "@/components/ui/sidebar";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import { useTheme } from "@/app/useTheme";
import { findRoute, ROUTES } from "@/app/router";

function ThemeButton() {
  const { mode, setMode, resolvedTheme } = useTheme();
  const next = mode === "light" ? "dark" : mode === "dark" ? "system" : "light";
  const Icon = resolvedTheme === "dark" ? Moon : Sun;
  return (
    <Button
      variant="ghost"
      size="icon"
      aria-label={`Theme: ${mode} (${resolvedTheme})`}
      onClick={() => setMode(next)}
      className="size-7"
    >
      <Icon className="h-3.5 w-3.5" />
    </Button>
  );
}

export function SiteHeader({
  route,
  onOpenCommandPalette,
}: {
  route: string;
  onOpenCommandPalette: () => void;
}) {
  const meta = ROUTES.find((r) => r.id === route);
  const sectionTitle = (() => {
    if (
      route === "overview" ||
      route === "projects" ||
      route === "runtimes" ||
      route === "services"
    )
      return "Workspace";
    if (route === "mail" || route === "inspector" || route === "diagnostics")
      return "Observability";
    if (route === "settings") return "System";
    return "";
  })();

  return (
    <header className="flex h-(--header-height) shrink-0 items-center gap-2 border-b border-border/40 transition-[width,height] ease-linear group-has-data-[collapsible=icon]/sidebar-wrapper:h-(--header-height)">
      <div className="flex w-full items-center gap-1 px-4 lg:gap-2 lg:px-6">
        <SidebarTrigger className="-ml-1" />
        <Separator
          orientation="vertical"
          className="mx-2 data-[orientation=vertical]:h-4"
        />
        <Breadcrumb>
          <BreadcrumbList>
            {sectionTitle && (
              <>
                <BreadcrumbItem className="hidden md:block">
                  <BreadcrumbLink href="#" className="text-text-muted">
                    {sectionTitle}
                  </BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator className="hidden md:block" />
              </>
            )}
            <BreadcrumbItem>
              <BreadcrumbPage className="font-semibold">
                {meta?.label ?? findRoute("overview").label}
              </BreadcrumbPage>
            </BreadcrumbItem>
          </BreadcrumbList>
        </Breadcrumb>
        <div className="ml-auto flex items-center gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={onOpenCommandPalette}
            className="gap-2 text-text-muted"
          >
            <Search className="h-3.5 w-3.5" />
            <span>Search</span>
            <kbd className="inline-flex items-center gap-0.5 rounded-sm border border-border bg-background px-1 font-mono text-[10px] tracking-tight text-text-faint">
              <span>⌘</span>
              <span>K</span>
            </kbd>
          </Button>
          <ThemeButton />
        </div>
      </div>
    </header>
  );
}
