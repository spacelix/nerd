export type RouteId =
  | "overview"
  | "projects"
  | "runtimes"
  | "services"
  | "mail"
  | "inspector"
  | "diagnostics"
  | "settings";

export type Route = {
  id: RouteId;
  label: string;
  description: string;
};

export const ROUTES: readonly Route[] = [
  { id: "overview", label: "Overview", description: "Status of projects, runtimes, and daemon" },
  { id: "projects", label: "Projects", description: "Parked and linked projects" },
  { id: "runtimes", label: "Runtimes", description: "Installed and external Node versions" },
  { id: "services", label: "Services", description: "Managed and external databases" },
  { id: "mail", label: "Mail", description: "Captured mail per project" },
  { id: "inspector", label: "Inspector", description: "Recent HTTP requests" },
  { id: "diagnostics", label: "Diagnostics", description: "DNS, CA, daemon, and ports" },
  { id: "settings", label: "Settings", description: "Theme, retention, discovery" },
];

export const DEFAULT_ROUTE: RouteId = "overview";

export function isRouteId(value: string): value is RouteId {
  return ROUTES.some((route) => route.id === value);
}

export function findRoute(id: RouteId): Route {
  const route = ROUTES.find((r) => r.id === id);
  if (!route) throw new Error(`Unknown route: ${id}`);
  return route;
}
