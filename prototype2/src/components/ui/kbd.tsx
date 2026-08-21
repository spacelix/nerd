import * as React from "react";
import { cn } from "@/lib/utils";

type KbdProps = React.ComponentProps<"kbd">;

function Kbd({ className, ...props }: KbdProps) {
  return (
    <kbd
      data-slot="kbd"
      className={cn(
        "inline-flex h-5 min-w-5 select-none items-center justify-center rounded border border-border bg-muted px-1.5 font-mono text-[10px] font-medium text-muted-foreground",
        className,
      )}
      {...props}
    />
  );
}

export { Kbd };
