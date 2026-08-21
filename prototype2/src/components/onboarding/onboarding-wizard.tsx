import { CheckCircle2, ChevronLeft, ChevronRight, Rocket } from "lucide-react";
import * as React from "react";
import { cn } from "@/lib/utils";

const steps = [
  {
    key: "welcome",
    title: "Welcome to Nerd",
    description: "A few quick checks before your local development loop is ready.",
    checks: [
      { label: "Windows 10 x64 (minimum)", ok: true },
      { label: "Daemon reachable · IPC v1", ok: true },
      { label: "Loopback ports available (80, 443)", ok: true },
    ],
  },
  {
    key: "network",
    title: "DNS & HTTPS for .test",
    description: "One-time elevated setup. Nerd owns every change and rolls back on failure.",
    checks: [
      { label: "NRPT rule .test → 127.0.0.1", ok: true },
      { label: "DNS listener 127.0.0.1:53 (UDP + TCP)", ok: true },
      { label: "Root CA trusted in CurrentUser store", ok: true },
      { label: "Proxy ports 80 / 443 free", ok: true },
    ],
  },
  {
    key: "tools",
    title: "External tools",
    description: "Read-only discovery. Nerd never adopts or mutates these.",
    checks: [
      { label: "Node v18.19.0 found (system)", ok: true },
      { label: "pnpm / npm / yarn available", ok: true },
      { label: "MySQL 8.0 listening on 127.0.0.1:3306", ok: true },
    ],
  },
  {
    key: "runtime",
    title: "Default Node",
    description: "A managed runtime powers projects unless overridden.",
    checks: [
      { label: "Node v22.11.0 (managed) set as default", ok: true },
      { label: "Verified official checksum", ok: true },
    ],
  },
  {
    key: "park",
    title: "Park a directory",
    description: "Immediate-child scan with native watchers — no polling, no surprises.",
    checks: [
      { label: "Scanned for nerd.json projects", ok: true },
      { label: "No untrusted project executed", ok: true },
    ],
  },
  {
    key: "project",
    title: "Your first project",
    description: "Create from a scaffold or link an existing directory.",
    checks: [
      { label: "Scaffold templates ready (F-11)", ok: true },
      { label: "Link existing directory ready (F-04)", ok: true },
    ],
  },
];

interface OnboardingWizardProps {
  onComplete: () => void;
}

function OnboardingWizard({ onComplete }: OnboardingWizardProps) {
  const [index, setIndex] = React.useState(0);
  const step = steps[index]!;
  const last = index === steps.length - 1;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-background/60 p-4 backdrop-blur-sm">
      <div
        role="dialog"
        aria-modal="true"
        aria-label={step.title}
        className="flex w-[min(92vw,26rem)] flex-col overflow-hidden rounded-xl border border-border bg-popover text-popover-foreground shadow-xl"
      >
        <div className="flex items-center gap-2 border-b border-border/60 px-4 py-3">
          <span className="grid size-6 place-items-center rounded-md bg-primary text-primary-foreground">
            <Rocket className="size-3.5" />
          </span>
          <span className="text-sm font-semibold tracking-tight">
            Set up Nerd
          </span>
          <span data-mono className="ml-auto text-[10px] text-muted-foreground/60">
            {index + 1} / {steps.length}
          </span>
        </div>

        <div className="flex flex-col gap-3 px-4 py-4">
          <div>
            <h2 className="text-base font-semibold tracking-tight text-foreground">
              {step.title}
            </h2>
            <p className="mt-0.5 text-xs leading-relaxed text-muted-foreground">
              {step.description}
            </p>
          </div>
          <div className="flex flex-col gap-1.5">
            {step.checks.map((c) => (
              <div
                key={c.label}
                className="flex items-center gap-2 rounded-md border border-border/50 bg-background/40 px-2.5 py-1.5"
              >
                <CheckCircle2 className="size-3.5 shrink-0 text-success" />
                <span className="text-xs text-foreground/90">{c.label}</span>
              </div>
            ))}
          </div>
          <p className="rounded-md border border-border/50 bg-background/40 px-2 py-1.5 text-[10px] leading-relaxed text-muted-foreground/70">
            Prototype flow · steps run real probes and setup in the product. Mock
            here, mirroring Features 01–07 (trust, DPAPI, rollback, no polling).
          </p>
        </div>

        <div className="flex items-center justify-between gap-2 border-t border-border/60 px-4 py-3">
          <button
            type="button"
            onClick={onComplete}
            className="text-[11px] text-muted-foreground transition-colors hover:text-foreground"
          >
            Skip
          </button>
          <div className="flex items-center gap-2">
            {index > 0 ? (
              <button
                type="button"
                onClick={() => setIndex((i) => i - 1)}
                aria-label="Previous step"
                className="inline-flex h-7 items-center gap-1 rounded-md border border-border/60 bg-card/40 px-2.5 text-[11px] font-medium text-foreground transition-colors hover:border-border hover:bg-accent/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              >
                <ChevronLeft className="size-3" />
                Back
              </button>
            ) : null}
            <button
              type="button"
              onClick={() => (last ? onComplete() : setIndex((i) => i + 1))}
              className={cn(
                "inline-flex h-7 items-center gap-1 rounded-md px-3 text-[11px] font-medium text-primary-foreground transition-opacity hover:opacity-90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                last ? "bg-success" : "bg-primary",
              )}
            >
              {last ? "Done" : "Continue"}
              {!last ? <ChevronRight className="size-3" /> : null}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

export { OnboardingWizard };