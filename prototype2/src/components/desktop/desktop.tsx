import { cn } from "@/lib/utils";
import { DesktopBackground } from "./desktop-background";
import { WindowFrame } from "./window-frame";

interface DesktopProps {
  className?: string;
  children: React.ReactNode;
}

function Desktop({ className, children }: DesktopProps) {
  return (
    <div
      data-slot="desktop"
      className={cn(
        "relative flex h-svh w-full flex-col overflow-hidden",
        className,
      )}
    >
      <DesktopBackground />
      <div className="pointer-events-none relative z-0 flex flex-1 items-center justify-center overflow-hidden px-4 pb-16 pt-6 sm:px-8 sm:pt-10">
        <div
          className="pointer-events-auto flex w-full max-w-[1280px] flex-col"
          style={{ height: "min(88vh, 880px)", minHeight: "560px" }}
        >
          <WindowFrame>{children}</WindowFrame>
        </div>
      </div>
    </div>
  );
}

export { Desktop };
