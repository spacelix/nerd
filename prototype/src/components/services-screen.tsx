import { CircleAlert, Plus } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { SERVICES } from "@/mocks/projects";
import type { Service } from "@/lib/types";

type StatusVariant = "default" | "secondary" | "destructive";

const STATUS_VARIANT: Record<Service["status"], StatusVariant> = {
  managed: "default",
  external: "secondary",
  blocked: "destructive",
};

const STATUS_LABEL: Record<Service["status"], string> = {
  managed: "managed",
  external: "external",
  blocked: "blocked",
};

function ServiceRow({ service }: { service: Service }) {
  const isBlocked = service.status === "blocked";
  return (
    <li className="flex items-start gap-4 border-b border-border/40 px-2 py-4 last:border-b-0">
      <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-muted/40 text-[10px] font-semibold uppercase tracking-[0.08em] text-muted-foreground">
        {service.id === "mysql" ? "MY" : service.id === "postgres" ? "PG" : "RD"}
      </div>
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium text-foreground">
            {service.label}
          </span>
          <Badge variant={STATUS_VARIANT[service.status]}>
            {STATUS_LABEL[service.status]}
          </Badge>
        </div>
        <p className="mt-1 text-xs leading-relaxed text-muted-foreground">
          {service.note}
        </p>
        {isBlocked && (
          <p className="mt-2 inline-flex items-center gap-1.5 rounded-md border border-border/40 bg-warning-soft/30 px-2 py-1 text-[11px] text-muted-foreground">
            <CircleAlert className="h-3 w-3 text-warning" />
            Adapter artifact record pending open decision.
          </p>
        )}
      </div>
      <div className="flex items-center gap-2">
        <Button
          variant="ghost"
          size="sm"
          disabled={isBlocked}
          aria-label={isBlocked ? "Managed adapter blocked" : "Manage"}
        >
          {isBlocked ? "Blocked" : "Manage"}
        </Button>
        {service.status === "external" && (
          <Button variant="ghost" size="sm">
            Register
          </Button>
        )}
      </div>
    </li>
  );
}

export function ServicesScreen() {
  return (
    <div className="h-full overflow-auto">
      <div className="flex flex-col gap-8 p-8">
        <div className="flex items-center justify-end gap-2">
          <span className="mr-auto font-mono text-xs tabular-nums text-muted-foreground">
            {SERVICES.length} configured
          </span>
          <Button variant="default" size="sm" disabled>
            <Plus className="h-3.5 w-3.5" />
            Add service
          </Button>
        </div>

        <div className="flex items-start gap-3 rounded-md border border-border/40 bg-warning-soft/30 px-4 py-3">
          <CircleAlert className="mt-0.5 h-4 w-4 shrink-0 text-warning" />
          <div className="text-sm">
            <p className="font-medium text-foreground">
              Service adapters pending open decisions
            </p>
            <p className="mt-0.5 text-xs text-muted-foreground">
              MySQL (OD-002), PostgreSQL (OD-003), and Redis (OD-004)
              artifacts are not yet approved. Nerd-managed instances stay
              blocked until each open decision is resolved.
            </p>
          </div>
        </div>

        <section>
          <header className="mb-3 flex items-baseline justify-between">
            <h2 className="text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
              Engines
            </h2>
            <span className="font-mono text-[10px] tabular-nums text-muted-foreground">
              {SERVICES.length}
            </span>
          </header>
          <ul className="overflow-hidden rounded-md border border-border/40 bg-surface">
            {SERVICES.map((service) => (
              <ServiceRow key={service.id} service={service} />
            ))}
          </ul>
        </section>

        <section>
          <header className="mb-3 flex items-baseline justify-between">
            <h2 className="text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
              External connections
            </h2>
            <span className="font-mono text-[10px] tabular-nums text-muted-foreground">
              0
            </span>
          </header>
          <div className="rounded-md border border-border/40 bg-surface p-6 text-sm text-muted-foreground">
            <p>
              External databases are read-only references. Nerd probes
              reachability but never adopts, repairs, or removes external
              services.
            </p>
          </div>
        </section>
      </div>
    </div>
  );
}
