import * as React from "react";
import * as TabsPrimitive from "@radix-ui/react-tabs";
import { cn } from "@/lib/utils";

const Tabs = TabsPrimitive.Root;

function TabsList({
  className,
  variant = "segmented",
  ...props
}: React.ComponentProps<typeof TabsPrimitive.List> & {
  variant?: "segmented" | "underline";
}) {
  return (
    <TabsPrimitive.List
      data-slot="tabs-list"
      data-variant={variant}
      className={cn(
        "group/tabs inline-flex h-7 shrink-0 items-center justify-start gap-0.5 rounded-md bg-muted/50 p-0.5",
        variant === "underline" &&
          "h-8 items-end gap-1 rounded-none bg-transparent p-0",
        className,
      )}
      {...props}
    />
  );
}

function TabsTrigger({
  className,
  ...props
}: React.ComponentProps<typeof TabsPrimitive.Trigger>) {
  return (
    <TabsPrimitive.Trigger
      data-slot="tabs-trigger"
      className={cn(
        "inline-flex h-[22px] flex-1 items-center justify-center whitespace-nowrap rounded px-2 text-[10px] font-medium transition-colors",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
        "data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-sm",
        "group-data-[variant=underline]/tabs:flex-none group-data-[variant=underline]/tabs:rounded-none group-data-[variant=underline]/tabs:bg-transparent group-data-[variant=underline]/tabs:px-2.5 group-data-[variant=underline]/tabs:text-[11px] group-data-[variant=underline]/tabs:shadow-none group-data-[variant=underline]/tabs:border-b group-data-[variant=underline]/tabs:border-border/40 group-data-[variant=underline]/tabs:data-[state=active]:border-primary group-data-[variant=underline]/tabs:data-[state=active]:border-b-2 group-data-[variant=underline]/tabs:data-[state=active]:bg-transparent group-data-[variant=underline]/tabs:data-[state=active]:text-foreground",
        className,
      )}
      {...props}
    />
  );
}

function TabsContent({
  className,
  ...props
}: React.ComponentProps<typeof TabsPrimitive.Content>) {
  return (
    <TabsPrimitive.Content
      data-slot="tabs-content"
      className={cn(
        "flex-1 overflow-y-auto focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
        className,
      )}
      {...props}
    />
  );
}

export { Tabs, TabsList, TabsTrigger, TabsContent };