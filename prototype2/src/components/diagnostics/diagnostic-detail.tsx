import { CheckCircle2, ShieldCheck, XCircle } from "lucide-react";
import { cn } from "@/lib/utils";
import type { DiagnosticProbe, ProbeStatus } from "@/lib/types";

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

function ProbeBadge({ status }: { status: ProbeStatus }) {
  const meta = statusMeta[status];
  return (
    <span
      data-mono
      className={cn(
        "rounded border px-1.5 py-0.5 text-[10px]",
        meta.tone === "success" && "border-success/40 text-success",
        meta.tone === "warning" && "border-warning/40 text-warning",
        meta.tone === "danger" && "border-danger/40 text-danger",
        meta.tone === "muted" && "border-border/60 text-muted-foreground",
      )}
    >
      {meta.label}
    </span>
  );
}

interface DiagnosticDetailProps {
  probe: DiagnosticProbe;
}

function DiagnosticDetail({ probe }: DiagnosticDetailProps) {
  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3 px-4 py-3">
      <div className="flex items-center gap-2">
        {probe.status === "pass" ? (
          <CheckCircle2 className="size-4 text-success" />
        ) : probe.status === "fail" ? (
          <XCircle className="size-4 text-danger" />
        ) : null}
        <span className="text-base font-semibold text-foreground">
          {probe.name}
        </span>
        <ProbeBadge status={probe.status} />
      </div>

      <p className="text-xs leading-relaxed text-foreground/90">
        {probe.summary}
      </p>

      <div className="flex flex-col gap-0.5">
        <span className="text-[10px] font-semibold tracking-[0.16em] text-muted-foreground/70 uppercase">
          Probe output
        </span>
        {probe.detail.map((line) => (
          <div
            key={line}
            data-mono
            className="rounded px-1 py-0.5 text-[11px] text-muted-foreground hover:bg-accent/40"
          >
            {line}
          </div>
        ))}
      </div>

      {probe.actionLabel ? (
        <button
          type="button"
          className={cn(
            "mt-auto inline-flex items-center justify-center gap-1.5 rounded-md px-2.5 py-1.5 text-xs font-medium transition-opacity hover:opacity-90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
            probe.actionSafe
              ? "bg-primary text-primary-foreground"
              : "border border-warning/40 text-warning",
          )}
        >
          <ShieldCheck className="size-3.5" />
          {probe.actionLabel}
        </button>
      ) : (
        <p
          data-mono
          className="mt-auto rounded-md border border-border/50 bg-background/40 px-2 py-1.5 text-[10px] text-muted-foreground/70"
        >
          no repair needed
        </p>
      )}
    </div>
  );
}

export { DiagnosticDetail };