import { cn } from "@/lib/utils";

interface DesktopBackgroundProps {
  className?: string;
}

function DesktopBackground({ className }: DesktopBackgroundProps) {
  return (
    <div
      aria-hidden="true"
      className={cn(
        "fixed inset-0 -z-10 overflow-hidden",
        className,
      )}
      style={{ background: "var(--desktop-wallpaper)" }}
    >
      <div
        className="absolute -top-32 -right-24 size-[480px] rounded-full opacity-40 blur-3xl"
        style={{ background: "oklch(0.85 0.12 145 / 0.25)" }}
      />
      <div
        className="absolute -bottom-40 -left-32 size-[560px] rounded-full opacity-30 blur-3xl"
        style={{ background: "oklch(0.78 0.10 220 / 0.30)" }}
      />
    </div>
  );
}

export { DesktopBackground };
