import { Paperclip, Search, ImageIcon, Download, Trash2, X } from "lucide-react";
import * as React from "react";
import { cn } from "@/lib/utils";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { mail } from "@/mocks/data";

function formatTime(iso: string): string {
  const d = new Date(iso);
  const now = new Date();
  const sameDay = d.toDateString() === now.toDateString();
  return sameDay
    ? d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })
    : d.toLocaleDateString([], { month: "short", day: "numeric" });
}

interface MailScreenProps {
  projectFilter: string | null;
  onClearFilter?: () => void;
}

function MailScreen({ projectFilter, onClearFilter }: MailScreenProps) {
  const [selectedId, setSelectedId] = React.useState<string | null>(
    mail[0]?.id ?? null,
  );
  const [readIds, setReadIds] = React.useState<Set<string>>(new Set());
  const [query, setQuery] = React.useState("");
  const [revealedIds, setRevealedIds] = React.useState<Set<string>>(
    new Set(),
  );
  const [deletedIds, setDeletedIds] = React.useState<Set<string>>(new Set());
  const [savedIds, setSavedIds] = React.useState<Set<string>>(new Set());

  const deleteMessage = (id: string) => {
    setDeletedIds((prev) => {
      const next = new Set(prev);
      next.add(id);
      return next;
    });
  };

  const deleteAll = () => {
    setDeletedIds((prev) => {
      const next = new Set(prev);
      visible.forEach((m) => next.add(m.id));
      return next;
    });
  };

  const visible = React.useMemo(() => {
    const q = query.trim().toLowerCase();
    const sorted = [...mail].sort(
      (a, b) => b.receivedAt.localeCompare(a.receivedAt),
    );
    return sorted.filter((m) => {
      if (deletedIds.has(m.id)) return false;
      if (projectFilter && m.projectId !== projectFilter) return false;
      if (!q) return true;
      return (
        m.subject.toLowerCase().includes(q) ||
        m.from.toLowerCase().includes(q) ||
        m.to.toLowerCase().includes(q)
      );
    });
  }, [query, projectFilter, deletedIds]);

  const selected =
    visible.find((m) => m.id === selectedId) ?? visible[0] ?? null;

  const select = (id: string) => {
    setSelectedId(id);
    setReadIds((prev) => {
      if (prev.has(id)) return prev;
      const next = new Set(prev);
      next.add(id);
      return next;
    });
  };

  return (
    <div className="flex h-full min-h-0 flex-col gap-5 px-10 py-10">
      <header className="flex flex-col gap-3">
        <span
          data-mono
          className="text-[11px] tracking-[0.18em] text-muted-foreground/60 uppercase"
        >
          Route · mail · N4
        </span>
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <h1 className="text-xl font-semibold tracking-tight text-foreground">
              Mail
            </h1>
            <p className="mt-0.5 text-xs text-muted-foreground">
              Sandboxed inbox · remote images are blocked until revealed
            </p>
          </div>
          <div className="relative w-full max-w-xs">
            <Search className="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-muted-foreground/60" />
            <input
              type="search"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search mail…"
              className="h-8 w-full rounded-md border border-border/60 bg-card/40 pr-3 pl-8 text-xs text-foreground placeholder:text-muted-foreground/60 focus:border-primary/50 focus:outline-none focus:ring-2 focus:ring-primary/20"
            />
          </div>
        </div>
          <div className="flex flex-wrap items-center gap-2">
            <button
              type="button"
              onClick={deleteAll}
              disabled={visible.length === 0}
              className="inline-flex h-7 items-center gap-1.5 rounded-md border border-border/60 bg-card/40 px-2.5 text-[11px] font-medium text-foreground transition-colors hover:border-border hover:bg-accent/40 disabled:pointer-events-none disabled:opacity-50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            >
              <Trash2 className="size-3" />
              Clear inbox
            </button>
            {projectFilter ? (
              <button
                type="button"
                onClick={onClearFilter}
                className="inline-flex h-7 items-center gap-1.5 rounded-md border border-border/60 bg-card/40 px-2.5 text-[11px] font-medium text-foreground transition-colors hover:border-border hover:bg-accent/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              >
                {projectFilter === "p-api" ? "api.app.test" : "app.test"}
                <X className="size-3" />
                Clear filter
              </button>
            ) : null}
          </div>
      </header>

      <div className="grid min-h-0 flex-1 grid-cols-[minmax(0,260px)_minmax(0,1fr)] gap-3">
        <div className="flex min-h-0 flex-col gap-1.5 overflow-y-auto pr-1">
          <span data-mono className="text-[11px] text-muted-foreground/60">
            {visible.length} messages
          </span>
          {visible.map((m) => {
            const unread = m.unread && !readIds.has(m.id);
            const isSelected = selected?.id === m.id;
            return (
              <button
                key={m.id}
                type="button"
                onClick={() => select(m.id)}
                className={cn(
                  "flex flex-col gap-1 rounded-lg border px-3 py-2 text-left transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                  isSelected
                    ? "border-primary/40 bg-surface-active"
                    : "border-border/60 bg-card/40 hover:border-border",
                )}
              >
                <span className="flex items-center gap-1.5">
                  <span
                    aria-label={unread ? "Unread" : "Read"}
                    className={cn(
                      "size-1.5 shrink-0 rounded-full",
                      unread ? "bg-primary" : "bg-transparent",
                    )}
                  />
                  <span
                    className={cn(
                      "min-w-0 flex-1 truncate text-xs",
                      unread
                        ? "font-medium text-foreground"
                        : "text-muted-foreground",
                    )}
                  >
                    {m.from}
                  </span>
                  {m.attachments.length > 0 ? (
                    <Paperclip
                      aria-label="Has attachments"
                      className="size-3 shrink-0 text-muted-foreground/50"
                    />
                  ) : null}
                </span>
                <span
                  className={cn(
                    "truncate text-xs",
                    unread ? "text-foreground/90" : "text-muted-foreground/80",
                  )}
                >
                  {m.subject}
                </span>
                <span
                  className={cn(
                    "truncate text-[11px]",
                    unread
                      ? "text-muted-foreground"
                      : "text-muted-foreground/60",
                  )}
                >
                  {m.bodyText.split("\n")[0]}
                </span>
                <span
                  data-mono
                  className="pt-0.5 text-[10px] text-muted-foreground/50"
                >
                  {formatTime(m.receivedAt)}
                </span>
              </button>
            );
          })}
          {visible.length === 0 ? (
            <p className="rounded-lg border border-dashed border-border/60 px-4 py-8 text-center text-xs text-muted-foreground">
              No messages match.
            </p>
          ) : null}
        </div>

        <div className="flex min-h-0 flex-col overflow-hidden rounded-lg border border-border/60 bg-card/40">
          {selected ? (
            <>
              <div className="flex flex-col gap-2 border-b border-border/40 px-4 py-3">
                <div className="flex items-start justify-between gap-2">
                  <h2 className="text-sm font-semibold text-foreground">
                    {selected.subject}
                  </h2>
                  <button
                    type="button"
                    onClick={() => deleteMessage(selected.id)}
                    aria-label="Delete message"
                    title="Delete message"
                    className="rounded p-1.5 text-muted-foreground/60 transition-colors hover:bg-danger/10 hover:text-danger focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  >
                    <Trash2 className="size-3.5" />
                  </button>
                </div>
                <div className="flex flex-wrap items-center gap-x-2 gap-y-0.5 text-xs text-muted-foreground">
                  <span className="font-medium text-foreground/90">
                    {selected.from}
                  </span>
                  <span aria-hidden="true">→</span>
                  <span data-mono>{selected.to}</span>
                </div>
                <div className="flex items-center gap-2">
                  <span
                    data-mono
                    className="text-[11px] text-muted-foreground/60"
                  >
                    {new Date(selected.receivedAt).toLocaleString([], {
                      dateStyle: "medium",
                      timeStyle: "short",
                    })}
                  </span>
                  <span
                    data-mono
                    className="rounded border border-border/50 bg-background/40 px-1.5 py-0.5 text-[10px] text-muted-foreground"
                  >
                    {selected.projectId === "p-api"
                      ? "api.app.test"
                      : "app.test"}
                  </span>
                  <span
                    data-mono
                    title="SMTP env injected into this project's process"
                    className="rounded border border-border/50 bg-background/40 px-1.5 py-0.5 text-[10px] text-muted-foreground/60"
                  >
                    SMTP 127.0.0.1:2525
                  </span>
                </div>
                {selected.remoteImages &&
                !revealedIds.has(selected.id) ? (
                  <div className="flex items-center justify-between gap-3 rounded-md border border-warning/30 bg-warning-soft/60 px-2.5 py-1.5">
                    <p className="text-[11px] leading-snug text-foreground/90">
                      This message references remote images and trackers. They
                      are blocked until you reveal them.
                    </p>
                    <button
                      type="button"
                      onClick={() =>
                        setRevealedIds((prev) => {
                          const next = new Set(prev);
                          next.add(selected.id);
                          return next;
                        })
                      }
                      className="shrink-0 rounded px-2 py-1 text-[10px] font-medium text-primary-foreground bg-primary transition-opacity hover:opacity-90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                    >
                      Reveal images
                    </button>
                  </div>
                ) : null}
              </div>

              <Tabs
                defaultValue="preview"
                className="flex min-h-0 flex-1 flex-col gap-3 px-4 pb-4"
              >
                <TabsList variant="underline">
                  <TabsTrigger value="preview">Preview</TabsTrigger>
                  <TabsTrigger value="headers">Headers</TabsTrigger>
                  <TabsTrigger value="source">Source</TabsTrigger>
                </TabsList>

                <TabsContent value="preview" className="px-1">
                  <div className="flex flex-col gap-3">
                    {selected.bodyIsHtml && selected.bodyHtml ? (
                      <div className="flex flex-col gap-1.5">
                        <iframe
                          title="Sandboxed HTML preview"
                          sandbox=""
                          aria-label="Sandboxed HTML preview — scripts and remote content disabled"
                          srcDoc={selected.bodyHtml}
                          className="h-64 w-full rounded-md border border-border/50 bg-background"
                        />
                        <span className="text-[10px] text-muted-foreground/60">
                          Sandboxed iframe — no scripts, no remote content,
                          no navigation.
                        </span>
                      </div>
                    ) : (
                      <p className="font-sans text-xs leading-relaxed whitespace-pre-wrap text-foreground/90">
                        {selected.bodyText}
                      </p>
                    )}
                    {selected.remoteImages &&
                    revealedIds.has(selected.id) ? (
                      <div className="grid grid-cols-3 gap-2">
                        {[0, 1, 2].map((i) => (
                          <div
                            key={i}
                            className="flex aspect-video items-center justify-center rounded-md border border-dashed border-border/60 bg-background/40"
                          >
                            <ImageIcon className="size-4 text-muted-foreground/50" />
                          </div>
                        ))}
                      </div>
                    ) : null}
                    {selected.attachments.length > 0 ? (
                      <div className="flex flex-col gap-1.5">
                        <span className="text-[10px] font-semibold tracking-[0.16em] text-muted-foreground/70 uppercase">
                          Attachments
                        </span>
                        {selected.attachments.map((a) => (
                          <div
                            key={a.id}
                            className="flex items-center justify-between gap-2 rounded-md border border-border/50 bg-background/40 px-3 py-2"
                          >
                            <span className="flex min-w-0 items-center gap-2">
                              <Paperclip className="size-3.5 shrink-0 text-muted-foreground/60" />
                              <span className="truncate text-xs text-foreground/90">
                                {a.name}
                              </span>
                            </span>
                            <span className="flex shrink-0 items-center gap-2">
                              <span
                                data-mono
                                className="text-[10px] text-muted-foreground/60"
                              >
                                {a.sizeKb} KB
                              </span>
                              <button
                                type="button"
                                onClick={() =>
                                  setSavedIds((prev) => {
                                    const next = new Set(prev);
                                    next.add(a.id);
                                    return next;
                                  })
                                }
                                aria-label={`Save ${a.name}`}
                                title="Save attachment"
                                className="flex items-center gap-1 rounded px-1 py-0.5 text-[10px] text-muted-foreground transition-colors hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                              >
                                <Download className="size-3.5" />
                                {savedIds.has(a.id) ? "Saved" : "Save"}
                              </button>
                            </span>
                          </div>
                        ))}
                      </div>
                    ) : null}
                  </div>
                </TabsContent>

                <TabsContent value="headers" className="px-1">
                  <div className="flex flex-col gap-0.5">
                    {Object.entries(selected.headers).map(([k, v]) => (
                      <div
                        key={k}
                        className="grid grid-cols-[minmax(0,1fr)_auto] items-baseline gap-2 rounded px-1 py-0.5 hover:bg-accent/40"
                      >
                        <span
                          data-mono
                          className="truncate text-[11px] text-foreground/80"
                        >
                          {k}
                        </span>
                        <span
                          data-mono
                          className="truncate text-[11px] text-muted-foreground"
                        >
                          {v}
                        </span>
                      </div>
                    ))}
                  </div>
                </TabsContent>

                <TabsContent value="source" className="px-1">
                  <pre className="overflow-x-auto rounded-md border border-border/50 bg-background/40 p-2 font-mono text-[11px] leading-relaxed whitespace-pre-wrap text-foreground/80">
                    {selected.raw}
                  </pre>
                </TabsContent>
              </Tabs>
            </>
          ) : (
            <div className="flex flex-1 items-center justify-center p-6">
              <p className="max-w-[240px] text-center text-xs text-muted-foreground">
                No message selected.
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

export { MailScreen };