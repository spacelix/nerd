import { Download, MessageSquare, MessageSquarePlus, Search, Trash2, X } from "lucide-react";
import * as React from "react";
import { cn } from "@/lib/utils";
import { Switch } from "@/components/ui/switch";
import { useRequestAnnotations } from "@/hooks/use-request-annotations";
import { requestDetails } from "@/mocks/data";
import type { HighlightColor, RequestDetail } from "@/lib/types";

const highlightDot: Record<HighlightColor, string> = {
  none: "bg-muted-foreground/25",
  red: "bg-danger",
  yellow: "bg-warning",
  green: "bg-success",
  blue: "bg-info",
  purple: "bg-highlight-purple",
};

const swatchColors: Exclude<HighlightColor, "none">[] = [
  "red",
  "yellow",
  "green",
  "blue",
  "purple",
];

function statusTone(status: number): string {
  if (status >= 500) return "text-danger";
  if (status >= 400) return "text-warning";
  if (status === 304) return "text-info";
  return "text-success";
}

interface InspectorScreenProps {
  selectedId: string | null;
  onSelect: (id: string) => void;
  bodyCapture: boolean;
  onBodyCaptureChange: (v: boolean) => void;
  projectFilter: string | null;
  onProjectFilterChange: (id: string | null) => void;
}

function InspectorScreen({
  selectedId,
  onSelect,
  bodyCapture,
  onBodyCaptureChange,
  projectFilter,
  onProjectFilterChange,
}: InspectorScreenProps) {
  const annotations = useRequestAnnotations();
  const [query, setQuery] = React.useState("");
  const [commentingId, setCommentingId] = React.useState<string | null>(null);
  const [draft, setDraft] = React.useState("");
  const [captureEnabled, setCaptureEnabled] = React.useState(true);
  const [cleared, setCleared] = React.useState(false);
  const [exported, setExported] = React.useState(false);

  const projectOptions = React.useMemo(() => {
    const ids = Array.from(new Set(requestDetails.map((r) => r.projectId)));
    return ids.map((id) => ({
      id,
      label: id === "p-api" ? "api.app.test" : "app.test",
    }));
  }, []);

  const visible = React.useMemo(() => {
    const q = query.trim().toLowerCase();
    const sorted = [...requestDetails].sort(
      (a, b) => b.startedAt.localeCompare(a.startedAt),
    );
    const filtered = projectFilter
      ? sorted.filter((r) => r.projectId === projectFilter)
      : sorted;
    if (!q) return filtered;
    return filtered.filter(
      (r) =>
        r.url.toLowerCase().includes(q) ||
        r.method.toLowerCase().includes(q) ||
        String(r.status).includes(q) ||
        r.host.toLowerCase().includes(q),
    );
  }, [query, projectFilter]);

  const beginComment = (r: RequestDetail) => {
    setCommentingId(r.id);
    setDraft(annotations.get(r.id).comment);
  };

  const saveComment = (r: RequestDetail) => {
    annotations.setComment(r.id, draft.trim());
    setCommentingId(null);
    setDraft("");
  };

  return (
    <div className="flex min-h-full flex-col gap-5 px-10 py-10">
      <header className="flex flex-col gap-3">
        <span
          data-mono
          className="text-[11px] tracking-[0.18em] text-muted-foreground/60 uppercase"
        >
          Route · inspector · N3
        </span>
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <h1 className="text-xl font-semibold tracking-tight text-foreground">
              Inspector
            </h1>
            <p className="mt-0.5 text-xs text-muted-foreground">
              Captured traffic across working session · highlight + comment is
              local-only
            </p>
          </div>
          <div className="relative w-full max-w-xs">
            <Search className="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-muted-foreground/60" />
            <input
              type="search"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Filter requests…"
              className="h-8 w-full rounded-md border border-border/60 bg-card/40 pr-3 pl-8 text-xs text-foreground placeholder:text-muted-foreground/60 focus:border-primary/50 focus:outline-none focus:ring-2 focus:ring-primary/20"
            />
          </div>
        </div>
      </header>

      <div className="flex flex-wrap items-center gap-2">
        <label className="flex items-center gap-2 rounded-md border border-border/60 bg-card/40 px-2.5 py-1.5">
          <Switch
            checked={captureEnabled}
            onCheckedChange={(v) => {
              setCaptureEnabled(v);
              if (v) setCleared(false);
            }}
            label="Capture requests"
          />
          <span className="text-[11px] text-muted-foreground">
            Capture {captureEnabled ? "on" : "off"}
          </span>
        </label>
        <label className="flex items-center gap-2 rounded-md border border-border/60 bg-card/40 px-2.5 py-1.5">
          <Switch
            checked={bodyCapture}
            onCheckedChange={onBodyCaptureChange}
            label="Capture bodies"
          />
          <span className="text-[11px] text-muted-foreground">
            Bodies {bodyCapture ? "on" : "off"}
          </span>
        </label>
        <label className="flex items-center gap-2 rounded-md border border-border/60 bg-card/40 px-2.5 py-1.5">
          <span className="text-[11px] text-muted-foreground">Project</span>
          <select
            aria-label="Filter requests by project"
            value={projectFilter ?? "all"}
            onChange={(e) =>
              onProjectFilterChange(
                e.target.value === "all" ? null : e.target.value,
              )
            }
            className="h-6 rounded border border-border/60 bg-background/40 px-1.5 text-[11px] text-foreground focus:border-primary/50 focus:outline-none focus:ring-2 focus:ring-primary/20"
          >
            <option value="all">All projects</option>
            {projectOptions.map((o) => (
              <option key={o.id} value={o.id}>
                {o.label}
              </option>
            ))}
          </select>
        </label>
        <button
          type="button"
          onClick={() => setCleared(true)}
          className="inline-flex h-7 items-center gap-1.5 rounded-md border border-border/60 bg-card/40 px-2.5 text-[11px] font-medium text-foreground transition-colors hover:border-border hover:bg-accent/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        >
          <Trash2 className="size-3" />
          Clear
        </button>
        <button
          type="button"
          onClick={() => setExported(true)}
          className="inline-flex h-7 items-center gap-1.5 rounded-md border border-border/60 bg-card/40 px-2.5 text-[11px] font-medium text-foreground transition-colors hover:border-border hover:bg-accent/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        >
          <Download className="size-3" />
          {exported ? "Exported (safe metadata)" : "Export safe metadata"}
        </button>
      </div>

      <div className="flex flex-col gap-1.5">
        <span
          data-mono
          className="text-[11px] text-muted-foreground/60"
        >
          {captureEnabled && !cleared ? visible.length : 0} requests
        </span>
        {!captureEnabled ? (
          <div className="flex flex-col items-center gap-2 rounded-lg border border-dashed border-border/60 py-16 text-center">
            <p className="text-sm text-muted-foreground">
              Capture is disabled. Existing metadata stays buffered in memory.
            </p>
          </div>
        ) : cleared ? (
          <div className="flex flex-col items-center gap-2 rounded-lg border border-dashed border-border/60 py-16 text-center">
            <p className="text-sm text-muted-foreground">
              Buffer cleared. New requests are captured from now on.
            </p>
          </div>
        ) : (
          visible.map((r) => {
          const ann = annotations.get(r.id);
          const isSelected = selectedId === r.id;
          return (
            <div
              key={r.id}
              className={cn(
                "rounded-lg border transition-colors",
                isSelected
                  ? "border-primary/40 bg-surface-active"
                  : "border-border/60 bg-card/40",
              )}
            >
              <div className="flex items-center gap-2 px-3 py-2">
                <button
                  type="button"
                  onClick={() => onSelect(r.id)}
                  className="flex min-w-0 flex-1 items-center gap-2.5 text-left focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                >
                  <span
                    data-mono
                    className={cn("text-xs font-medium", statusTone(r.status))}
                  >
                    {r.status}
                  </span>
                  <span
                    data-mono
                    className="w-10 shrink-0 text-xs text-muted-foreground"
                  >
                    {r.method}
                  </span>
                  <span className="min-w-0 flex-1 truncate text-xs text-foreground/90">
                    {r.url}
                  </span>
                  <span data-mono className="shrink-0 text-[11px] text-muted-foreground/70">
                    {r.durationMs}ms
                  </span>
                </button>
                <div
                  role="group"
                  aria-label="Highlight color"
                  className="flex shrink-0 items-center gap-1"
                >
                  {swatchColors.map((c) => (
                    <button
                      key={c}
                      type="button"
                      aria-label={`Highlight ${c}`}
                      aria-pressed={ann.highlight === c}
                      onClick={() => {
                        annotations.setHighlight(
                          r.id,
                          ann.highlight === c ? "none" : c,
                        );
                      }}
                      className={cn(
                        "size-2.5 rounded-full transition-transform hover:scale-125 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                        highlightDot[c],
                        ann.highlight === c && "ring-2 ring-ring/40",
                      )}
                    />
                  ))}
                </div>
                <button
                  type="button"
                  aria-label={ann.comment ? "Edit comment" : "Add comment"}
                  aria-pressed={ann.comment !== ""}
                  onClick={() => beginComment(r)}
                  className={cn(
                    "shrink-0 rounded p-1 transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                    ann.comment
                      ? "text-foreground"
                      : "text-muted-foreground/60",
                  )}
                >
                  {ann.comment ? (
                    <MessageSquare className="size-3.5" />
                  ) : (
                    <MessageSquarePlus className="size-3.5" />
                  )}
                </button>
              </div>
              {commentingId === r.id ? (
                <div className="flex flex-col gap-2 border-t border-border/40 px-3 py-2">
                  <textarea
                    autoFocus
                    value={draft}
                    onChange={(e) => setDraft(e.target.value)}
                    placeholder="Local-only note…"
                    rows={2}
                    className="w-full resize-none rounded-md border border-border/60 bg-card/40 p-2 text-xs text-foreground placeholder:text-muted-foreground/60 focus:border-primary/50 focus:outline-none focus:ring-2 focus:ring-primary/20"
                  />
                  <div className="flex items-center gap-2">
                    <button
                      type="button"
                      onClick={() => saveComment(r)}
                      className="rounded-md bg-primary px-2.5 py-1 text-xs font-medium text-primary-foreground transition-opacity hover:opacity-90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                    >
                      Save
                    </button>
                    <button
                      type="button"
                      onClick={() => setCommentingId(null)}
                      className="rounded-md px-2 py-1 text-xs text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                    >
                      Cancel
                    </button>
                  </div>
                </div>
              ) : null}
            </div>
          );
        })
        )}
        {captureEnabled && !cleared && visible.length === 0 ? (
          <div className="flex flex-col items-center gap-1 rounded-lg border border-dashed border-border/60 px-4 py-8 text-center">
            <X className="size-4 text-muted-foreground/50" />
            <p className="text-xs text-muted-foreground">No requests match.</p>
          </div>
        ) : null}
      </div>
    </div>
  );
}

export { InspectorScreen };