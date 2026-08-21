import { ChevronRight, Download, ScanLine, Star, Trash2 } from "lucide-react";
import { cn } from "@/lib/utils";
import {
  ActionButton,
  InstallRuntimeDialog,
  RegisterExternalRuntimeDialog,
  UninstallRuntimeDialog,
} from "@/components/actions/action-dialogs";
import { runtimes } from "@/mocks/data";
import type { RuntimeClass } from "@/lib/types";
import * as React from "react";

const removedRuntimes = new Set<string>();

const classBadge: Record<RuntimeClass, string> = {
  managed: "border-success/40 text-success",
  external: "border-border/60 text-muted-foreground",
  degraded: "border-warning/40 text-warning",
};

interface RuntimesScreenProps {
  selectedId: string | null;
  onSelect: (id: string) => void;
}

function RuntimesScreen({ selectedId, onSelect }: RuntimesScreenProps) {
  const [installOpen, setInstallOpen] = React.useState(false);
  const [registerOpen, setRegisterOpen] = React.useState(false);
  const [uninstallTarget, setUninstallTarget] = React.useState<string | null>(
    null,
  );
  const [filter, setFilter] = React.useState<"all" | RuntimeClass>("all");
  const visible = runtimes.filter(
    (r) => !removedRuntimes.has(r.id) && (filter === "all" || r.class === filter),
  );
  const filterTabs: { value: "all" | RuntimeClass; label: string }[] = [
    { value: "all", label: "All" },
    { value: "managed", label: "Managed" },
    { value: "external", label: "External" },
    { value: "degraded", label: "Degraded" },
  ];
  return (
    <div className="flex min-h-full flex-col gap-5 px-10 py-10">
      <header className="flex flex-col gap-2">
        <span
          data-mono
          className="text-[11px] tracking-[0.18em] text-muted-foreground/60 uppercase"
        >
          Route · runtimes · N5
        </span>
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <h1 className="text-xl font-semibold tracking-tight text-foreground">
              Runtimes
            </h1>
            <p className="max-w-xl text-xs text-muted-foreground">
              Installed Node versions. Managed runtimes are owned by Nerd; external
              ones are system-installed and never mutated.
            </p>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <ActionButton
              label="Register external"
              icon={<ScanLine className="size-3.5" />}
              onClick={() => setRegisterOpen(true)}
            />
            <ActionButton
              label="Install Node"
              icon={<Download className="size-3.5" />}
              onClick={() => setInstallOpen(true)}
            />
          </div>
        </div>
      </header>

      <InstallRuntimeDialog open={installOpen} onOpenChange={setInstallOpen} />
      <RegisterExternalRuntimeDialog
        open={registerOpen}
        onOpenChange={setRegisterOpen}
      />
      <UninstallRuntimeDialog
        open={uninstallTarget !== null}
        onOpenChange={(o) => {
          if (!o) setUninstallTarget(null);
        }}
        version={
          runtimes.find((r) => r.id === uninstallTarget)?.version ?? ""
        }
        onConfirm={() => {
          if (uninstallTarget) removedRuntimes.add(uninstallTarget);
          setUninstallTarget(null);
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
          {visible.length} runtimes
        </span>
        {visible.map((r) => {
          const isSelected = selectedId === r.id;
          const isExternal = r.class === "external";
          return (
            <div
              key={r.id}
              className={cn(
                "flex items-stretch rounded-lg border transition-colors",
                isSelected
                  ? "border-primary/40 bg-surface-active"
                  : "border-border/60 bg-card/40 hover:border-border",
              )}
            >
              <button
                type="button"
                onClick={() => onSelect(r.id)}
                className="flex min-w-0 flex-1 items-center gap-3 px-4 py-3 text-left focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              >
                <span
                  data-mono
                  className="w-24 shrink-0 text-base font-medium text-foreground"
                >
                  v{r.version}
                </span>
                <span
                  data-mono
                  className={cn(
                    "rounded border px-1.5 py-0.5 text-[10px]",
                    classBadge[r.class],
                  )}
                >
                  {r.class}
                </span>
                {r.isDefault ? (
                  <span className="flex items-center gap-1 text-[11px] text-muted-foreground">
                    <Star className="size-3 fill-warning text-warning" />
                    default
                  </span>
                ) : null}
                <span
                  data-mono
                  className="ml-auto hidden text-xs text-muted-foreground sm:block"
                >
                  {r.usageCount} project{r.usageCount === 1 ? "" : "s"}
                </span>
                <ChevronRight className="size-3.5 text-muted-foreground/40" />
              </button>
              <div className="flex shrink-0 items-center border-l border-border/40 pl-2 pr-2.5">
                {isExternal ? (
                  <span
                    title="External runtimes are read-only. Nerd never uninstalls system Node."
                    className="rounded px-1.5 py-0.5 text-[10px] text-muted-foreground/60"
                  >
                    read-only
                  </span>
                ) : (
                  <button
                    type="button"
                    aria-label={`Uninstall v${r.version}`}
                    title="Uninstall"
                    onClick={() => setUninstallTarget(r.id)}
                    className="rounded p-1.5 text-muted-foreground/60 transition-colors hover:bg-danger/10 hover:text-danger focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  >
                    <Trash2 className="size-3.5" />
                  </button>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

export { RuntimesScreen };