import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

type PageHeaderProps = {
  title: string;
  subtitle?: ReactNode;
  children?: ReactNode;
};

export function PageHeader({ title, subtitle, children }: PageHeaderProps) {
  return (
    <div className="flex h-16 shrink-0 items-center justify-between gap-4 border-b border-border/40 bg-window px-8">
      <div className="flex min-w-0 items-baseline gap-3">
        <h1 className="text-[20px] font-semibold leading-lg tracking-[-0.018em] text-text">
          {title}
        </h1>
        {subtitle && (
          <span className="font-mono text-[12px] tabular-nums text-text-muted">
            {subtitle}
          </span>
        )}
      </div>
      {children && <div className="flex items-center gap-2">{children}</div>}
    </div>
  );
}

type PageBodyProps = {
  children: ReactNode;
  className?: string;
};

export function PageBody({ children, className }: PageBodyProps) {
  return (
    <div className={cn("flex-1 overflow-auto", className)}>{children}</div>
  );
}
