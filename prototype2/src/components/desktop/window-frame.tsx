import { cn } from "@/lib/utils";
import { WindowControls } from "./window-controls";

interface WindowFrameProps {
  className?: string;
  children: React.ReactNode;
}

function WindowFrame({ className, children }: WindowFrameProps) {
  return (
    <div
      role="region"
      aria-label="Nerd desktop application window"
      className={cn(
        "relative isolate flex w-full flex-col overflow-hidden rounded-xl border bg-background text-foreground",
        className,
      )}
      style={{
        boxShadow: "var(--desktop-window-shadow)",
        borderColor: "var(--desktop-window-border)",
      }}
    >
      <div
        data-slot="window-chrome"
        className="flex h-9 shrink-0 items-center justify-between border-b border-border/60 select-none"
        style={{ background: "var(--desktop-chrome-bg)" }}
      >
        <div className="flex h-full min-w-0 flex-1 items-center gap-2 px-3 [app-region:drag]">
          <span
            aria-hidden="true"
            className="grid size-4 shrink-0 place-items-center rounded-[5px] bg-primary text-[10px] leading-none font-bold text-primary-foreground"
          >
            N
          </span>
          <span className="shrink-0 text-xs font-medium text-foreground">
            Nerd
          </span>
        </div>
        <WindowControls />
      </div>
      <div className="flex min-h-0 flex-1 flex-col">{children}</div>
    </div>
  );
}

export { WindowFrame };
