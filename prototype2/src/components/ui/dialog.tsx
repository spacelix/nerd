import * as React from "react";
import * as DialogPrimitive from "@radix-ui/react-dialog";
import { X } from "lucide-react";
import { cn } from "@/lib/utils";

const Dialog = DialogPrimitive.Root;
const DialogTrigger = DialogPrimitive.Trigger;
const DialogClose = DialogPrimitive.Close;
const DialogDescription = DialogPrimitive.Description;

function DialogContent({
  className,
  children,
  ...props
}: React.ComponentProps<typeof DialogPrimitive.Content>) {
  return (
    <DialogPrimitive.Portal>
      <DialogPrimitive.Overlay className="fixed inset-0 z-50 bg-background/60 backdrop-blur-sm data-[state=open]:animate-[cmdk-overlay-in_150ms_ease-out] data-[state=closed]:animate-[cmdk-overlay-out_150ms_ease-in]" />
      <DialogPrimitive.Content
        data-slot="dialog-content"
        className={cn(
          "fixed top-1/2 left-1/2 z-50 flex w-[min(92vw,24rem)] -translate-x-1/2 -translate-y-1/2 flex-col gap-3 rounded-xl border border-border bg-popover p-4 shadow-lg shadow-black/10 data-[state=open]:animate-[cmdk-content-in_150ms_ease-out] data-[state=closed]:animate-[cmdk-content-out_150ms_ease-in]",
          className,
        )}
        {...props}
      >
        {children}
        <DialogPrimitive.Close
          aria-label="Close"
          className="absolute top-2.5 right-2.5 rounded p-1 text-muted-foreground transition-colors hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        >
          <X className="size-3.5" />
        </DialogPrimitive.Close>
      </DialogPrimitive.Content>
    </DialogPrimitive.Portal>
  );
}

function DialogHeader({
  className,
  ...props
}: React.ComponentProps<typeof DialogPrimitive.Title>) {
  return (
    <DialogPrimitive.Title
      data-slot="dialog-title"
      className={cn("text-sm font-semibold tracking-tight text-foreground", className)}
      {...props}
    />
  );
}

function DialogFooter({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      data-slot="dialog-footer"
      className={cn("flex items-center justify-end gap-2 border-t border-border/50 pt-3", className)}
      {...props}
    />
  );
}

export {
  Dialog,
  DialogTrigger,
  DialogClose,
  DialogContent,
  DialogHeader,
  DialogDescription,
  DialogFooter,
};