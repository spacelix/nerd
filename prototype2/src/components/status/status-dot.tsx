import * as React from "react";
import { cn } from "@/lib/utils";

interface StatusDotProps extends React.ComponentProps<"span"> {
  tone: "success" | "info" | "warning" | "danger" | "muted";
  pulse?: boolean;
}

const toneClass: Record<StatusDotProps["tone"], string> = {
  success: "bg-success",
  info: "bg-info",
  warning: "bg-warning",
  danger: "bg-danger",
  muted: "bg-muted-foreground/50",
};

function StatusDot({
  tone,
  pulse = false,
  className,
  ...props
}: StatusDotProps) {
  return (
    <span
      data-slot="status-dot"
      data-tone={tone}
      aria-hidden="true"
      className={cn(
        "relative inline-flex size-2 items-center justify-center rounded-full",
        toneClass[tone],
        className,
      )}
      {...props}
    >
      {pulse ? (
        <span
          className={cn(
            "absolute inset-0 rounded-full motion-safe:animate-[nerd-pulse_2s_ease-in-out_infinite]",
            toneClass[tone],
            "opacity-60",
          )}
        />
      ) : null}
    </span>
  );
}

export { StatusDot };
