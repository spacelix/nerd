import {
  CheckCircle2,
  ChevronRight,
  FileArchive,
  Play,
  XCircle,
} from "lucide-react";
import * as React from "react";
import { cn } from "@/lib/utils";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
} from "@/components/ui/dialog";
import {
  GhostButton,
  PrimaryButton,
} from "@/components/actions/action-dialogs";
import { diagnosticProbes } from "@/mocks/data";
import type { ProbeStatus } from "@/lib/types";

const statusMeta: Record<
  ProbeStatus,
  { tone: "success" | "warning" | "danger" | "muted"; label: string }
> = {
  pass: { tone: "success", label: "pass" },
  warn: { tone: "warning", label: "warn" },
  fail: { tone: "danger", label: "fail" },
  idle: { tone: "muted", label: "idle" },
  "unsupported-policy": { tone: "warning", label: "unsupported policy" },
  "foreign-conflict": { tone: "warning", label: "foreign conflict" },
};

const bundleFiles: string[] = [
  "nerd-version.txt",
  "daemon-state.json (redacted)",
  "probes.json",
  "logs/nerd-daemon.jsonl (redacted)",
  "diagnostics-summary.json",
];

function ProbeIcon({ status }: { status: ProbeStatus }) {
  if (status === "pass") {
    return <CheckCircle2 className="size-4 text-success" />;
  }
  if (status === "fail") {
    return <XCircle className="size-4 text-danger" />;
  }
  return (
    <span className="flex size-4 items-center justify-center rounded-full border border-warning text-[10px] font-bold text-warning">
      !
    </span>
  );
}

interface DiagnosticsScreenProps {
  selectedId: string | null;
  onSelect: (id: string) => void;
}

function DiagnosticsScreen({ selectedId, onSelect }: DiagnosticsScreenProps) {
  const [exportOpen, setExportOpen] = React.useState(false);
  const [exported, setExported] = React.useState(false);
  return (
    <div className="flex min-h-full flex-col gap-5 px-10 py-10">
      <header className="flex flex-col gap-2">
        <span
          data-mono
          className="text-[11px] tracking-[0.18em] text-muted-foreground/60 uppercase"
        >
          Route · diagnostics · N5
        </span>
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <h1 className="text-xl font-semibold tracking-tight text-foreground">
              Diagnostics
            </h1>
            <p className="mt-0.5 max-w-xl text-xs text-muted-foreground">
              Probes run against the loopback environment. Repairs are safe and
              reversible.
            </p>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <button
              type="button"
              onClick={() => {
                setExported(false);
                setExportOpen(true);
              }}
              className="inline-flex items-center gap-1.5 rounded-md border border-border/60 bg-card/40 px-2.5 py-1.5 text-xs font-medium text-foreground transition-colors hover:border-border hover:bg-accent/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            >
              <FileArchive className="size-3" />
              Export support bundle
            </button>
            <button
              type="button"
              className="inline-flex items-center gap-1.5 rounded-md bg-primary px-2.5 py-1.5 text-xs font-medium text-primary-foreground transition-opacity hover:opacity-90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            >
              <Play className="size-3" />
              Run all
            </button>
          </div>
        </div>
      </header>

      <Dialog open={exportOpen} onOpenChange={setExportOpen}>
        <DialogContent>
          {exported ? (
            <>
              <DialogHeader>Support bundle exported</DialogHeader>
              <p className="rounded-md border border-border/50 bg-background/40 px-2 py-1.5 text-[10px] leading-relaxed text-muted-foreground/70">
                Local-only archive. Previewed before creation; credentials,
                request/mail bodies, private keys, and raw authorization/cookies
                are excluded or redacted. Never uploaded automatically.
              </p>
              <DialogFooter>
                <PrimaryButton onClick={() => setExportOpen(false)}>
                  Done
                </PrimaryButton>
              </DialogFooter>
            </>
          ) : (
            <>
              <div className="flex flex-col gap-0.5 pr-6">
                <DialogHeader>Export support bundle</DialogHeader>
                <DialogDescription className="text-[11px] text-muted-foreground">
                  Local archive for support or recovery. Nothing leaves the
                  machine.
                </DialogDescription>
              </div>
              <p className="rounded-md border border-border/50 bg-background/40 px-2 py-1.5 text-[10px] leading-relaxed text-muted-foreground/70">
                Includes daemon logs, state snapshot, probe output, and versions.
                Redaction is applied before writing: no secrets, mail or request
                bodies, private keys, or raw credentials.
              </p>
              <div className="flex flex-col gap-1 rounded-md border border-border/50 bg-background/40 px-2.5 py-2">
                <span className="text-[10px] font-semibold tracking-[0.16em] text-muted-foreground/70 uppercase">
                  Bundle preview
                </span>
                {bundleFiles.map((f) => (
                  <div key={f} className="flex items-center gap-2">
                    <CheckCircle2 className="size-3 text-success" />
                    <span data-mono className="truncate text-[10px] text-muted-foreground">
                      {f}
                    </span>
                  </div>
                ))}
                <span className="mt-1 text-[10px] leading-relaxed text-muted-foreground/60">
                  Excluded: request/mail bodies, private keys, and raw
                  authorization/cookies.
                </span>
              </div>
              <DialogFooter>
                <DialogClose>
                  <GhostButton>Cancel</GhostButton>
                </DialogClose>
                <PrimaryButton onClick={() => setExported(true)}>
                  Create bundle
                </PrimaryButton>
              </DialogFooter>
            </>
          )}
        </DialogContent>
      </Dialog>

      <div className="flex flex-col gap-1.5">
        <span data-mono className="text-[11px] text-muted-foreground/60">
          {diagnosticProbes.length} probes
        </span>
        {diagnosticProbes.map((p) => {
          const meta = statusMeta[p.status];
          const isSelected = selectedId === p.id;
          return (
            <button
              key={p.id}
              type="button"
              onClick={() => onSelect(p.id)}
              className={cn(
                "flex items-center gap-3 rounded-lg border px-4 py-3 text-left transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                isSelected
                  ? "border-primary/40 bg-surface-active"
                  : "border-border/60 bg-card/40 hover:border-border",
              )}
            >
              <ProbeIcon status={p.status} />
              <span className="min-w-0 flex-1">
                <span className="block text-sm font-medium text-foreground">
                  {p.name}
                </span>
                <span className="block truncate text-xs text-muted-foreground">
                  {p.summary}
                </span>
              </span>
              <span
                data-mono
                className={cn(
                  "text-[11px]",
                  meta.tone === "success" && "text-success",
                  meta.tone === "warning" && "text-warning",
                  meta.tone === "danger" && "text-danger",
                  meta.tone === "muted" && "text-muted-foreground/60",
                )}
              >
                {meta.label}
              </span>
              <ChevronRight className="size-3.5 text-muted-foreground/40" />
            </button>
          );
        })}
      </div>
    </div>
  );
}

export { DiagnosticsScreen };