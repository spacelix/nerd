import { ChevronRight, Plus, ScanLine, Trash2 } from "lucide-react";
import * as React from "react";
import { cn } from "@/lib/utils";
import { StatusDot } from "@/components/status/status-dot";
import {
  ActionButton,
  AddServiceDialog,
  RegisterExternalServiceDialog,
  RemoveExternalServiceDialog,
} from "@/components/actions/action-dialogs";
import { services } from "@/mocks/data";
import { useServiceActions } from "@/hooks/use-service-actions";
import type { ServiceStatus } from "./service-detail";

const removedServices = new Set<string>();

const statusTone: Record<ServiceStatus, "success" | "warning" | "muted"> = {
  running: "success",
  degraded: "warning",
  stopped: "muted",
};

interface ServicesScreenProps {
  selectedId: string | null;
  onSelect: (id: string) => void;
}

function ServicesScreen({ selectedId, onSelect }: ServicesScreenProps) {
  const serviceActions = useServiceActions();
  const [addOpen, setAddOpen] = React.useState(false);
  const [registerOpen, setRegisterOpen] = React.useState(false);
  const [removeTarget, setRemoveTarget] = React.useState<string | null>(null);
  const [filter, setFilter] = React.useState<"all" | "managed" | "external">(
    "all",
  );
  const visible = services.filter(
    (s) =>
      !removedServices.has(s.id) &&
      (filter === "all" || s.class === filter),
  );
  const filterTabs: { value: "all" | "managed" | "external"; label: string }[] = [
    { value: "all", label: "All" },
    { value: "managed", label: "Managed" },
    { value: "external", label: "External" },
  ];
  return (
    <div className="flex min-h-full flex-col gap-5 px-10 py-10">
      <header className="flex flex-col gap-2">
        <span
          data-mono
          className="text-[11px] tracking-[0.18em] text-muted-foreground/60 uppercase"
        >
          Route · services · N5
        </span>
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <h1 className="text-xl font-semibold tracking-tight text-foreground">
              Services
            </h1>
            <p className="max-w-xl text-xs text-muted-foreground">
              Loopback databases and engines. Lifecycle features depend on open
              decisions OD-002 / OD-003 / OD-004.
            </p>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <ActionButton
              label="Register external"
              icon={<ScanLine className="size-3.5" />}
              onClick={() => setRegisterOpen(true)}
            />
            <ActionButton
              label="Add service"
              icon={<Plus className="size-3.5" />}
              onClick={() => setAddOpen(true)}
            />
          </div>
        </div>
      </header>

      <AddServiceDialog open={addOpen} onOpenChange={setAddOpen} />
      <RegisterExternalServiceDialog
        open={registerOpen}
        onOpenChange={setRegisterOpen}
      />
      <RemoveExternalServiceDialog
        open={removeTarget !== null}
        onOpenChange={(o) => {
          if (!o) setRemoveTarget(null);
        }}
        service={
          services.find((s) => s.id === removeTarget)?.name ?? ""
        }
        onConfirm={() => {
          if (removeTarget) removedServices.add(removeTarget);
          setRemoveTarget(null);
        }}
      />

      <div className="flex items-center gap-1">
        {filterTabs.map((t) => (
          <button
            key={t.value}
            type="button"
            aria-pressed={filter === t.value}
            onClick={() => setFilter(t.value)}
            className={cn(
              "rounded-md px-2 py-1 text-[11px] font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
              filter === t.value
                ? "bg-surface-active text-foreground"
                : "text-muted-foreground hover:text-foreground",
            )}
          >
            {t.label}
          </button>
        ))}
      </div>

      <div className="flex flex-col gap-1.5">
        <span data-mono className="text-[11px] text-muted-foreground/60">
          {visible.length} services
        </span>
        {visible.map((s) => {
          const isSelected = selectedId === s.id;
          const removable = !s.blockerId;
          const live = serviceActions.statusFor(s);
          return (
            <div
              key={s.id}
              className={cn(
                "flex items-stretch rounded-lg border transition-colors",
                isSelected
                  ? "border-primary/40 bg-surface-active"
                  : "border-border/60 bg-card/40",
              )}
            >
              <button
                type="button"
                onClick={() => onSelect(s.id)}
                className="flex min-w-0 flex-1 items-center gap-3 px-4 py-3 text-left focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              >
                <StatusDot
                  tone={statusTone[live as ServiceStatus]}
                  pulse={live === "starting"}
                />
                <span className="min-w-0">
                  <span className="block text-sm font-medium text-foreground">
                    {s.name}
                  </span>
                  <span
                    data-mono
                    className="block text-xs text-muted-foreground"
                  >
                    {s.version} · :{s.port}
                  </span>
                </span>
                {s.class === "external" ? (
                  <span
                    data-mono
                    className="rounded border border-border/60 px-1.5 py-0.5 text-[10px] text-muted-foreground"
                  >
                    external
                  </span>
                ) : null}
                {s.blockerId ? (
                  <span
                    data-mono
                    className="rounded border border-warning/40 px-1.5 py-0.5 text-[10px] text-warning"
                  >
                    {s.blockerId}
                  </span>
                ) : null}
                <span
                  data-mono
                  className="ml-auto hidden text-xs text-muted-foreground/80 sm:block"
                >
                  {live}
                </span>
                <ChevronRight className="size-3.5 text-muted-foreground/40" />
              </button>
              <div className="flex shrink-0 items-center border-l border-border/40 pl-2 pr-2.5">
                {removable ? (
                  <button
                    type="button"
                    aria-label={`Remove ${s.name}`}
                    title="Remove"
                    onClick={() => setRemoveTarget(s.id)}
                    className="rounded p-1.5 text-muted-foreground/60 transition-colors hover:bg-danger/10 hover:text-danger focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  >
                    <Trash2 className="size-3.5" />
                  </button>
                ) : (
                  <span
                    title="Removal is blocked by the open decision listed on this service."
                    className="rounded px-1.5 py-0.5 text-[10px] text-muted-foreground/60"
                  >
                    read-only
                  </span>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

export { ServicesScreen };