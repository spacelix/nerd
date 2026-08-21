import type { ReactNode } from "react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useRequestAnnotations } from "@/hooks/use-request-annotations";
import { cn } from "@/lib/utils";
import type { HighlightColor, RequestDetail } from "@/lib/types";

function JsonPretty({ json }: { json: unknown }) {
  const render = (v: unknown): ReactNode => {
    if (v === null) {
      return <span className="text-muted-foreground">null</span>;
    }
    if (typeof v === "string") {
      return <span className="text-success">"{v}"</span>;
    }
    if (typeof v === "boolean") {
      return <span className="text-warning">{String(v)}</span>;
    }
    if (typeof v === "number") {
      return <span className="text-warning">{v}</span>;
    }
    if (Array.isArray(v)) {
      if (v.length === 0) {
        return <span className="text-muted-foreground">[]</span>;
      }
      return (
        <>
          <span className="text-muted-foreground">[</span>
          <div className="pl-4">
            {v.map((item, i) => (
              <div key={i}>
                {render(item)}
                {i < v.length - 1 ? (
                  <span className="text-muted-foreground">,</span>
                ) : null}
              </div>
            ))}
          </div>
          <span className="text-muted-foreground">]</span>
        </>
      );
    }
    if (typeof v === "object") {
      const entries = Object.entries(v as Record<string, unknown>);
      if (entries.length === 0) {
        return <span className="text-muted-foreground">{"{}"}</span>;
      }
      return (
        <>
          <span className="text-muted-foreground">{"{"}</span>
          <div className="pl-4">
            {entries.map(([k, val], i) => (
              <div key={k}>
                <span className="text-info">"{k}"</span>
                <span className="text-muted-foreground">: </span>
                {render(val)}
                {i < entries.length - 1 ? (
                  <span className="text-muted-foreground">,</span>
                ) : null}
              </div>
            ))}
          </div>
          <span className="text-muted-foreground">{"}"}</span>
        </>
      );
    }
    return String(v);
  };
  return (
    <div className="overflow-x-auto rounded-md border border-border/50 bg-background/40 p-2 font-mono text-[11px] leading-relaxed text-foreground/90">
      {render(json)}
    </div>
  );
}

const highlightDot: Record<HighlightColor, string> = {
  none: "bg-muted-foreground/25",
  red: "bg-danger",
  yellow: "bg-warning",
  green: "bg-success",
  blue: "bg-info",
  purple: "bg-highlight-purple",
};

function statusClass(status: number): string {
  if (status >= 500) return "text-danger";
  if (status >= 400) return "text-warning";
  if (status === 304) return "text-info";
  return "text-success";
}

function Row({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <div className="grid grid-cols-[5.5rem_minmax(0,1fr)] items-baseline gap-2 py-1">
      <span className="text-[11px] text-muted-foreground/70">{label}</span>
      <span className="min-w-0 truncate text-xs text-foreground/90">{value}</span>
    </div>
  );
}

function HeadersList({ title, headers }: { title: string; headers: Record<string, string> }) {
  const entries = Object.entries(headers);
  if (entries.length === 0) {
    return (
      <p className="py-2 text-[11px] text-muted-foreground/60">No {title.toLowerCase()}.</p>
    );
  }
  return (
    <div className="flex flex-col gap-0.5">
      <span data-mono className="pt-1 pb-1 text-[10px] tracking-wide text-muted-foreground/60 uppercase">
        {title}
      </span>
      {entries.map(([k, v]) => (
        <div
          key={k}
          className="grid grid-cols-[minmax(0,1fr)_auto] items-baseline gap-2 rounded px-1 py-0.5 hover:bg-accent/40"
        >
          <span data-mono className="truncate text-[11px] text-foreground/80">{k}</span>
          <span data-mono className="truncate text-[11px] text-muted-foreground">{v}</span>
        </div>
      ))}
    </div>
  );
}

function Body({
  label,
  body,
  contentType,
}: {
  label: string;
  body: string;
  contentType: string | null;
}) {
  if (!body) {
    return (
      <p className="py-1 text-[11px] text-muted-foreground/50">
        {label}: empty
      </p>
    );
  }
  const trimmed = body.trim();
  const looksJson = trimmed.startsWith("{") || trimmed.startsWith("[");
  let parsed: unknown = null;
  let isJson = false;
  if (looksJson) {
    try {
      parsed = JSON.parse(trimmed);
      isJson = true;
    } catch {
      isJson = false;
    }
  }
  return (
    <div className="flex flex-col gap-0.5">
      <span
        data-mono
        className="flex items-center gap-1.5 pt-1 pb-1 text-[10px] tracking-wide text-muted-foreground/60 uppercase"
      >
        {label}
        {isJson ? (
          <span className="rounded border border-border/50 bg-background/40 px-1 py-px normal-case text-muted-foreground">
            {contentType ?? "json"} · formatted
          </span>
        ) : contentType ? (
          <span className="rounded border border-border/50 bg-background/40 px-1 py-px normal-case text-muted-foreground">
            {contentType}
          </span>
        ) : null}
      </span>
      {isJson ? (
        <JsonPretty json={parsed} />
      ) : (
        <pre className="overflow-x-auto rounded-md border border-border/50 bg-background/40 p-2 font-mono text-[11px] leading-relaxed whitespace-pre-wrap text-foreground/80">
          {body}
        </pre>
      )}
    </div>
  );
}

function TimingBar({
  label,
  ms,
  max,
}: {
  label: string;
  ms: number;
  max: number;
}) {
  return (
    <div className="grid grid-cols-[4.5rem_minmax(0,1fr)_2.5rem] items-center gap-2 py-1">
      <span className="text-[11px] text-muted-foreground/70">{label}</span>
      <div className="h-1.5 overflow-hidden rounded-full bg-muted/60">
        <div
          className="h-full rounded-full bg-primary/70"
          style={{ width: `${max > 0 ? Math.max(2, (ms / max) * 100) : 0}%` }}
        />
      </div>
      <span data-mono className="text-right text-[11px] text-foreground/80">{ms}ms</span>
    </div>
  );
}

interface RequestInspectorProps {
  request: RequestDetail;
  bodyCapture?: boolean;
}

function RequestInspector({ request, bodyCapture = false }: RequestInspectorProps) {
  const annotations = useRequestAnnotations();
  const ann = annotations.get(request.id);
  const maxTiming = Math.max(
    1,
    request.timing.dns,
    request.timing.connect,
    request.timing.ttfb,
    request.timing.download,
  );
  const total =
    request.timing.dns +
    request.timing.connect +
    request.timing.ttfb +
    request.timing.download;

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex flex-col gap-2 border-b border-border/40 px-4 py-3">
        <div className="flex items-center gap-2">
          <span
            data-mono
            className={cn("text-xs font-medium", statusClass(request.status))}
          >
            {request.status}
          </span>
          <span
            data-mono
            className="rounded border border-border/50 bg-background/40 px-1 py-0.5 text-[10px] text-muted-foreground"
          >
            {request.method}
          </span>
          <span
            className={cn(
              "size-2 rounded-full",
              highlightDot[ann.highlight],
            )}
          />
        </div>
        <span
          data-mono
          className="truncate text-xs text-foreground/90"
          title={request.url}
        >
          {request.url}
        </span>
        {ann.comment ? (
          <p className="rounded-md border border-warning/30 bg-warning-soft/60 px-2 py-1.5 text-[11px] leading-relaxed text-foreground/90">
            {ann.comment}
          </p>
        ) : null}
      </div>

      <Tabs defaultValue="overview" className="flex min-h-0 flex-1 flex-col px-3 pb-3">
        <TabsList className="mt-2">
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="headers">Headers</TabsTrigger>
          <TabsTrigger value="body">Body</TabsTrigger>
          <TabsTrigger value="response">Response</TabsTrigger>
          <TabsTrigger value="timing">Timing</TabsTrigger>
        </TabsList>

        <TabsContent value="overview" className="px-1">
          <Row label="Host" value={request.host} />
          <Row label="Path" value={request.path} />
          {Object.keys(request.query).length > 0 ? (
            <Row
              label="Query"
              value={Object.entries(request.query)
                .map(([k, v]) => `${k}=${v}`)
                .join(" · ")}
            />
          ) : null}
          <Row label="Protocol" value={request.protocol} />
          <Row label="Content type" value={request.contentType ?? "—"} />
          <Row label="Duration" value={`${request.durationMs} ms`} />
          <Row label="Timing total" value={`${total} ms`} />
          <Row label="Highlight" value={ann.highlight === "none" ? "none" : ann.highlight} />
          <Row
            label="Project"
            value={request.projectId === "p-api" ? "api.app.test" : "app.test"}
          />
        </TabsContent>

        <TabsContent value="headers" className="px-1">
          <HeadersList title="Request headers" headers={request.requestHeaders} />
        </TabsContent>

        <TabsContent value="body" className="px-1">
          {bodyCapture ? (
            <Body
              label="Request body"
              body={request.requestBody}
              contentType={request.contentType}
            />
          ) : (
            <p className="py-1 text-[11px] text-muted-foreground/60">
              Body capture is off — only request metadata was recorded.
            </p>
          )}
        </TabsContent>

        <TabsContent value="response" className="px-1">
          <div className="flex flex-col gap-1 rounded-md border border-border/50 bg-background/40 px-2.5 py-1.5">
            <Row label="Status" value={request.status} />
            <Row label="Captured size" value={`${request.responseBody.length} bytes`} />
            {request.originalBytes ? (
              <Row
                label="Original size"
                value={`${request.originalBytes} bytes (truncated to capture budget)`}
              />
            ) : null}
            <Row label="Duration" value={`${request.durationMs} ms`} />
          </div>
          <div className="mt-2" />
          <HeadersList title="Response headers" headers={request.responseHeaders} />
          {bodyCapture ? (
            <>
              <div className="mt-2" />
              <Body
                label="Response body"
                body={request.responseBody}
                contentType={request.contentType}
              />
            </>
          ) : null}
        </TabsContent>

        <TabsContent value="timing" className="px-1">
          <div className="flex flex-col py-1">
            <TimingBar label="DNS" ms={request.timing.dns} max={maxTiming} />
            <TimingBar label="Connect" ms={request.timing.connect} max={maxTiming} />
            <TimingBar label="TTFB" ms={request.timing.ttfb} max={maxTiming} />
            <TimingBar label="Download" ms={request.timing.download} max={maxTiming} />
            <Row label="Total" value={`${total} ms`} />
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}

export { RequestInspector };