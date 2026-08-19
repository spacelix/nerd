import { useState } from "react";
import { Inbox, Mail as MailIcon, Paperclip } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

type Message = {
  id: string;
  project: string;
  from: string;
  subject: string;
  preview: string;
  receivedAt: string;
  hasAttachments: boolean;
};

const MESSAGES: Message[] = [
  {
    id: "1",
    project: "shop-web",
    from: "Stripe <receipts@stripe.com>",
    subject: "Your invoice INV-2026-0042",
    preview:
      "Thanks for your business. Your latest invoice is attached as a PDF.",
    receivedAt: "10:42",
    hasAttachments: true,
  },
  {
    id: "2",
    project: "auth-mock",
    from: "Mailtrap <noreply@mailtrap.io>",
    subject: "Welcome to Mailtrap — confirm your account",
    preview: "Click the button below to verify your email address.",
    receivedAt: "10:38",
    hasAttachments: false,
  },
  {
    id: "3",
    project: "api-server",
    from: "Postmark <hello@postmarkapp.com>",
    subject: "Your server tokens have rotated",
    preview: "For your security, server tokens rotated at 10:35 UTC.",
    receivedAt: "10:35",
    hasAttachments: false,
  },
];

function MessageRow({
  message,
  active,
  onSelect,
}: {
  message: Message;
  active: boolean;
  onSelect: () => void;
}) {
  return (
    <li>
      <button
        type="button"
        onClick={onSelect}
        className={`flex w-full items-start gap-3 border-b border-border/40 px-4 py-3 text-left transition-colors hover:bg-accent/30 ${
          active ? "bg-accent/30" : ""
        }`}
      >
        <MailIcon className="mt-0.5 h-4 w-4 shrink-0 text-muted-foreground" />
        <div className="min-w-0 flex-1">
          <div className="flex items-baseline gap-2">
            <span className="truncate text-sm font-medium text-foreground">
              {message.from}
            </span>
            <span className="ml-auto shrink-0 font-mono text-[11px] tabular-nums text-muted-foreground">
              {message.receivedAt}
            </span>
          </div>
          <p className="truncate text-sm text-foreground">{message.subject}</p>
          <p className="truncate text-xs text-muted-foreground">
            {message.preview}
          </p>
          <div className="mt-1.5 flex items-center gap-1.5">
            <Badge variant="outline">{message.project}</Badge>
            {message.hasAttachments && (
              <span className="inline-flex items-center gap-1 text-[11px] text-muted-foreground">
                <Paperclip className="h-3 w-3" />
                Attachment
              </span>
            )}
          </div>
        </div>
      </button>
    </li>
  );
}

function MailPreview({ message }: { message: Message }) {
  return (
    <div className="flex h-full flex-col overflow-hidden">
      <div className="border-b border-border/40 px-6 py-4">
        <div className="flex items-center gap-2 text-sm">
          <span className="font-semibold tracking-tight">{message.subject}</span>
          <span className="ml-auto font-mono text-xs tabular-nums text-muted-foreground">
            {message.receivedAt}
          </span>
        </div>
        <p className="mt-1 text-xs text-muted-foreground">{message.from}</p>
        <div className="mt-3 flex flex-wrap items-center gap-1.5">
          <Badge variant="outline">{message.project}</Badge>
          <Button variant="ghost" size="sm">
            Save source
          </Button>
          <Button variant="ghost" size="sm">
            Reveal remote images
          </Button>
        </div>
      </div>
      <div className="flex-1 overflow-auto bg-background px-6 py-6">
        <div className="mx-auto max-w-2xl rounded-md border border-warning/30 bg-warning-soft/30 p-3 text-xs text-muted-foreground">
          Sandboxed preview. Remote images and scripts are blocked. Open the
          raw source for untrusted content.
        </div>
        <div className="mx-auto mt-4 max-w-2xl font-mono text-[12px] leading-relaxed text-text">
          <p>
            Subject: <span className="font-semibold">{message.subject}</span>
          </p>
          <p className="mt-3 text-muted-foreground">{message.preview}</p>
          <p className="mt-3 text-muted-foreground">
            This preview is rendered inside an <code>&lt;iframe&gt;</code> with{" "}
            <code>sandbox=""</code>. No scripts, no same-origin, no forms.
          </p>
        </div>
      </div>
    </div>
  );
}

export function MailScreen() {
  const [activeId, setActiveId] = useState<string | null>(
    MESSAGES[0]?.id ?? null,
  );
  const active = MESSAGES.find((m) => m.id === activeId);

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center gap-2 border-b border-border/40 px-6 py-2 font-mono text-[11px] tabular-nums text-muted-foreground">
        <span>{MESSAGES.length} captured</span>
        <span aria-hidden="true">·</span>
        <span>7 days retention</span>
      </div>

      <div className="grid min-h-0 flex-1 grid-cols-1 lg:grid-cols-[360px_1fr]">
        <aside className="flex h-full min-h-0 flex-col border-r border-border/40">
          <div className="flex items-center gap-2 border-b border-border/40 px-4 py-3">
            <Inbox className="h-4 w-4 text-muted-foreground" />
            <span className="text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
              Inbox
            </span>
            <span className="ml-auto font-mono text-[10px] tabular-nums text-muted-foreground">
              {MESSAGES.length}
            </span>
          </div>
          <ul className="flex-1 overflow-auto">
            {MESSAGES.map((message) => (
              <MessageRow
                key={message.id}
                message={message}
                active={message.id === activeId}
                onSelect={() => setActiveId(message.id)}
              />
            ))}
          </ul>
        </aside>
        <section className="min-h-0 overflow-hidden">
          {active ? (
            <MailPreview message={active} />
          ) : (
            <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
              No message selected.
            </div>
          )}
        </section>
      </div>
    </div>
  );
}
