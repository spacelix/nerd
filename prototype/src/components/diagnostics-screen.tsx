import { Activity, CircleAlert, CircleCheck, PlayCircle, ShieldCheck } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import type { LucideIcon } from "lucide-react";

type Status = "ok" | "warn" | "fail";

type Card = {
  id: string;
  label: string;
  Icon: LucideIcon;
  status: Status;
  detail: string;
  meta: { label: string; value: string }[];
  action: { label: string; primary?: boolean };
};

const STATUS_META: Record<Status, {
  tone: "success" | "warning" | "destructive";
  label: string;
  Icon: LucideIcon;
}> = {
  ok: { tone: "success", label: "verified", Icon: CircleCheck },
  warn: { tone: "warning", label: "warning", Icon: CircleAlert },
  fail: { tone: "destructive", label: "action needed", Icon: CircleAlert },
};

const CARDS: Card[] = [
  {
    id: "dns",
    label: "DNS resolver",
    Icon: ShieldCheck,
    status: "ok",
    detail:
      "NRPT rule for .test installed in CurrentUser. Browser secure DNS overrides fall back correctly.",
    meta: [
      { label: "Rule", value: "nrpt.test" },
      { label: "Scope", value: "CurrentUser" },
      { label: "Verified", value: "14:22" },
    ],
    action: { label: "Run probe", primary: false },
  },
  {
    id: "ca",
    label: "Root CA",
    Icon: ShieldCheck,
    status: "ok",
    detail:
      "Nerd root CA installed in CurrentUser\\Root. SHA-256 fingerprint matches the bundled artifact.",
    meta: [
      { label: "Store", value: "CurrentUser\\Root" },
      { label: "Algorithm", value: "ECDSA P-256" },
      { label: "Expires", value: "in 364 d" },
    ],
    action: { label: "Run probe", primary: false },
  },
  {
    id: "daemon",
    label: "Daemon",
    Icon: Activity,
    status: "ok",
    detail:
      "Nerd daemon healthy. IPC endpoint bound, watcher subscribed, foreign listeners reported only.",
    meta: [
      { label: "PID", value: "12 482" },
      { label: "Uptime", value: "14h 22m" },
      { label: "Endpoint", value: "\\\\.\\pipe\\nerd" },
      { label: "Restarts", value: "0" },
    ],
    action: { label: "View logs", primary: false },
  },
  {
    id: "ports",
    label: "Ports",
    Icon: CircleAlert,
    status: "warn",
    detail:
      "Foreign listener detected on port 53. Nerd will report but never terminate the existing process.",
    meta: [
      { label: "53", value: "hns (foreign)" },
      { label: "80", value: "nerd" },
      { label: "443", value: "nerd" },
    ],
    action: { label: "Resolve conflict", primary: true },
  },
];

function StatusCard({ card }: { card: Card }) {
  const meta = STATUS_META[card.status];
  const Icon = card.Icon;
  return (
    <article className="flex flex-col gap-4 rounded-md border border-border/40 bg-surface p-5">
      <header className="flex items-start justify-between gap-3">
        <div className="flex items-center gap-2">
          <span className="grid h-7 w-7 place-items-center rounded-md bg-muted/40 text-muted-foreground">
            <Icon className="h-3.5 w-3.5" />
          </span>
          <span className="text-sm font-semibold tracking-tight">
            {card.label}
          </span>
        </div>
        <Badge variant={meta.tone}>{meta.label}</Badge>
      </header>
      <p className="text-xs leading-relaxed text-muted-foreground">
        {card.detail}
      </p>
      <dl className="grid grid-cols-2 gap-y-1.5 border-t border-border/40 pt-4 font-mono text-[12px] tabular-nums">
        {card.meta.map((m) => (
          <div key={m.label} className="flex items-baseline justify-between gap-2 pr-2">
            <dt className="text-[10px] font-sans font-semibold uppercase tracking-[0.1em] text-text-faint">
              {m.label}
            </dt>
            <dd className="text-text">{m.value}</dd>
          </div>
        ))}
      </dl>
      <Button
        variant={card.action.primary ? "default" : "ghost"}
        size="sm"
        className="self-start"
      >
        <PlayCircle className="h-3.5 w-3.5" />
        {card.action.label}
      </Button>
    </article>
  );
}

export function DiagnosticsScreen() {
  return (
    <div className="h-full overflow-auto">
      <div className="flex flex-col gap-6 p-8">
        <div className="flex items-center justify-end gap-2">
          <span className="mr-auto font-mono text-xs tabular-nums text-muted-foreground">
            4 foundations verified
          </span>
          <Button variant="default" size="sm">
            Run nerd doctor
          </Button>
        </div>

        <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
          {CARDS.map((card) => (
            <StatusCard key={card.id} card={card} />
          ))}
        </div>

        <section className="flex flex-col gap-3 rounded-md border border-border/40 bg-surface p-6">
          <header>
            <h2 className="text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
              Safe repairs
            </h2>
            <p className="mt-1 text-sm text-muted-foreground">
              Nerd doctor runs only repairs that have a verified rollback
              path. Destructive actions require explicit confirmation.
            </p>
          </header>
          <ul className="space-y-1.5 text-sm">
            <li className="flex items-center justify-between border-b border-border/40 py-2">
              <span>Reinstall NRPT rule for .test</span>
              <Button variant="ghost" size="sm">
                Repair
              </Button>
            </li>
            <li className="flex items-center justify-between border-b border-border/40 py-2">
              <span>Reinstall Nerd root CA in CurrentUser\Root</span>
              <Button variant="ghost" size="sm">
                Repair
              </Button>
            </li>
            <li className="flex items-center justify-between py-2">
              <span>Restart daemon (preserves running projects)</span>
              <Button variant="ghost" size="sm">
                Repair
              </Button>
            </li>
          </ul>
        </section>
      </div>
    </div>
  );
}
