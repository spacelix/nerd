import { useState } from "react";
import { Activity, Eye, EyeOff, Search } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";

type Request = {
  id: string;
  method: "GET" | "POST" | "PUT" | "DELETE";
  path: string;
  project: string;
  status: number;
  durationMs: number;
  startedAt: string;
  host: string;
  headers: { name: string; value: string }[];
};

const REQUESTS: Request[] = [
  {
    id: "1",
    method: "GET",
    path: "/api/products",
    project: "shop-web",
    status: 200,
    durationMs: 42,
    startedAt: "10:42:08.214",
    host: "api.shop.test",
    headers: [
      { name: "Host", value: "api.shop.test" },
      { name: "User-Agent", value: "Mozilla/5.0 (Windows NT 10.0)" },
      { name: "Accept", value: "application/json" },
      { name: "Authorization", value: "Bearer ****[redacted]" },
      { name: "Cookie", value: "[redacted]" },
    ],
  },
  {
    id: "2",
    method: "POST",
    path: "/api/checkout",
    project: "shop-web",
    status: 201,
    durationMs: 184,
    startedAt: "10:42:11.092",
    host: "api.shop.test",
    headers: [
      { name: "Host", value: "api.shop.test" },
      { name: "Content-Type", value: "application/json" },
      { name: "Content-Length", value: "412" },
      { name: "Authorization", value: "Bearer ****[redacted]" },
    ],
  },
  {
    id: "3",
    method: "GET",
    path: "/api/orders/2042",
    project: "shop-web",
    status: 500,
    durationMs: 1208,
    startedAt: "10:42:14.001",
    host: "api.shop.test",
    headers: [
      { name: "Host", value: "api.shop.test" },
      { name: "Accept", value: "application/json" },
    ],
  },
  {
    id: "4",
    method: "GET",
    path: "/health",
    project: "api-server",
    status: 200,
    durationMs: 4,
    startedAt: "10:43:02.018",
    host: "api.test",
    headers: [{ name: "Host", value: "api.test" }],
  },
];

type StatusTone = "success" | "destructive";

const STATUS_TONE: Record<number, StatusTone> = {
  200: "success",
  201: "success",
  500: "destructive",
};

function StatusBadge({ status }: { status: number }) {
  const tone = STATUS_TONE[status];
  if (!tone) {
    return <Badge variant="outline">{status}</Badge>;
  }
  return <Badge variant={tone}>{status}</Badge>;
}

function MethodBadge({ method }: { method: Request["method"] }) {
  const tone =
    method === "GET"
      ? "secondary"
      : method === "POST"
        ? "default"
        : method === "DELETE"
          ? "destructive"
          : "outline";
  return (
    <Badge variant={tone} className="w-14 justify-center">
      {method}
    </Badge>
  );
}

function RequestRow({
  request,
  active,
  onSelect,
}: {
  request: Request;
  active: boolean;
  onSelect: () => void;
}) {
  return (
    <li>
      <button
        type="button"
        onClick={onSelect}
        className={`grid w-full grid-cols-[60px_minmax(0,1fr)_60px_70px_70px] items-center gap-3 border-b border-border/40 px-3 py-2 text-left font-mono text-[12px] transition-colors hover:bg-accent/30 ${
          active ? "bg-accent/30" : ""
        }`}
      >
        <MethodBadge method={request.method} />
        <span className="truncate text-foreground">{request.path}</span>
        <span className="text-text-muted">{request.project}</span>
        <StatusBadge status={request.status} />
        <span className="text-right tabular-nums text-text-muted">
          {request.durationMs} ms
        </span>
      </button>
    </li>
  );
}

function DetailView({ request }: { request: Request }) {
  return (
    <div className="flex h-full flex-col">
      <div className="border-b border-border/40 px-6 py-4">
        <div className="flex items-center gap-2">
          <MethodBadge method={request.method} />
          <span className="font-mono text-sm">{request.path}</span>
          <StatusBadge status={request.status} />
          <span className="ml-auto font-mono text-xs tabular-nums text-muted-foreground">
            {request.durationMs} ms · {request.startedAt}
          </span>
        </div>
        <p className="mt-1 font-mono text-xs text-muted-foreground">
          {request.host} · {request.project}
        </p>
      </div>
      <div className="flex-1 overflow-auto px-6 py-4">
        <h3 className="text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
          Headers
        </h3>
        <dl className="mt-2 font-mono text-[12px] leading-relaxed">
          {request.headers.map((header) => (
            <div
              key={header.name}
              className="grid grid-cols-[180px_1fr] gap-2 border-b border-border/40 py-1.5 last:border-b-0"
            >
              <dt className="text-text-muted">{header.name}</dt>
              <dd className="truncate text-text">{header.value}</dd>
            </div>
          ))}
        </dl>

        <h3 className="mt-6 text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
          Timing
        </h3>
        <dl className="mt-2 grid grid-cols-2 gap-2 font-mono text-[12px] tabular-nums">
          <div className="rounded-md border border-border/40 bg-surface p-3">
            <dt className="text-[10px] font-sans uppercase tracking-[0.1em] text-text-faint">
              Total
            </dt>
            <dd className="mt-1 text-foreground">{request.durationMs} ms</dd>
          </div>
          <div className="rounded-md border border-border/40 bg-surface p-3">
            <dt className="text-[10px] font-sans uppercase tracking-[0.1em] text-text-faint">
              Status
            </dt>
            <dd className="mt-1 text-foreground">{request.status}</dd>
          </div>
        </dl>

        <h3 className="mt-6 text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
          Body capture
        </h3>
        <p className="mt-2 text-sm text-muted-foreground">
          Bodies are off by default. Turn on body capture to store request and
          response bodies (1 MB cap, content-type filtered, redacted before
          display).
        </p>
      </div>
    </div>
  );
}

export function InspectorScreen() {
  const [activeId, setActiveId] = useState<string>(REQUESTS[0]?.id ?? "");
  const active = REQUESTS.find((r) => r.id === activeId);

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center gap-2 border-b border-border/40 px-6 py-2 font-mono text-[11px] tabular-nums text-muted-foreground">
        <span>{REQUESTS.length} recent</span>
        <span aria-hidden="true">·</span>
        <span>500 / project buffer</span>
        <span className="ml-auto flex items-center gap-2">
          <Button variant="ghost" size="sm">
            <EyeOff className="h-3.5 w-3.5" />
            Body capture off
          </Button>
          <Button variant="outline" size="sm">
            <Eye className="h-3.5 w-3.5" />
            Clear buffer
          </Button>
        </span>
      </div>

      <div className="grid min-h-0 flex-1 grid-cols-1 lg:grid-cols-[420px_1fr]">
        <aside className="flex h-full min-h-0 flex-col border-r border-border/40">
          <div className="flex items-center gap-2 border-b border-border/40 px-3 py-2">
            <Search className="h-3.5 w-3.5 text-muted-foreground" />
            <span className="text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
              Recent
            </span>
            <span className="ml-auto font-mono text-[10px] tabular-nums text-muted-foreground">
              {REQUESTS.length}
            </span>
          </div>
          <ul className="flex-1 overflow-auto">
            {REQUESTS.map((request) => (
              <RequestRow
                key={request.id}
                request={request}
                active={request.id === activeId}
                onSelect={() => setActiveId(request.id)}
              />
            ))}
          </ul>
        </aside>
        <section className="min-h-0 overflow-hidden bg-background">
          {active ? (
            <DetailView request={active} />
          ) : (
            <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
              <Activity className="mr-2 h-4 w-4" />
              No request selected.
            </div>
          )}
        </section>
      </div>
    </div>
  );
}
