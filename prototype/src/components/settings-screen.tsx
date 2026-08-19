import { Moon, Sun } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { useTheme } from "@/app/useTheme";

function ThemeRow() {
  const { mode, setMode, resolvedTheme } = useTheme();
  const options = ["light", "dark", "system"] as const;
  return (
    <section className="flex flex-col gap-3 rounded-md border border-border/40 bg-surface p-6">
      <header className="flex items-start justify-between gap-4">
        <div>
          <h2 className="text-sm font-semibold tracking-tight">Appearance</h2>
          <p className="mt-1 text-sm text-muted-foreground">
            Choose light, dark, or follow the system. Current resolved theme:{" "}
            <span className="font-mono text-foreground">{resolvedTheme}</span>.
          </p>
        </div>
      </header>
      <div className="flex flex-wrap gap-2">
        {options.map((m) => (
          <Button
            key={m}
            variant={mode === m ? "default" : "outline"}
            size="sm"
            onClick={() => setMode(m)}
            aria-pressed={mode === m}
          >
            {m === "dark" ? (
              <Moon className="h-3.5 w-3.5" />
            ) : (
              <Sun className="h-3.5 w-3.5" />
            )}
            {m.charAt(0).toUpperCase() + m.slice(1)}
          </Button>
        ))}
      </div>
    </section>
  );
}

function Toggle({
  label,
  description,
  defaultChecked,
}: {
  label: string;
  description: string;
  defaultChecked: boolean;
}) {
  return (
    <label className="flex items-start justify-between gap-6 border-b border-border/40 py-4 last:border-b-0">
      <div className="min-w-0">
        <p className="text-sm font-medium text-foreground">{label}</p>
        <p className="mt-1 text-sm text-muted-foreground">{description}</p>
      </div>
      <input
        type="checkbox"
        defaultChecked={defaultChecked}
        className="relative h-5 w-9 shrink-0 cursor-pointer appearance-none rounded-full border border-border/40 bg-muted/40 transition-colors checked:border-accent checked:bg-accent"
      />
    </label>
  );
}

function PreferencesSection() {
  return (
    <section className="flex flex-col gap-3 rounded-md border border-border/40 bg-surface p-6">
      <header>
        <h2 className="text-sm font-semibold tracking-tight">Discovery</h2>
        <p className="mt-1 text-sm text-muted-foreground">
          Detect existing developer tools without taking ownership.
        </p>
      </header>
      <Toggle
        label="Discover external Node runtimes"
        description="Detect Node installations outside Nerd management. Discovered runtimes are added as read-only references."
        defaultChecked={false}
      />
      <Toggle
        label="Discover external services"
        description="Probe MySQL, PostgreSQL, and Redis on standard ports. Discovered services stay external."
        defaultChecked={false}
      />
    </section>
  );
}

function RetentionSection() {
  return (
    <section className="flex flex-col gap-4 rounded-md border border-border/40 bg-surface p-6">
      <header>
        <h2 className="text-sm font-semibold tracking-tight">Retention</h2>
        <p className="mt-1 text-sm text-muted-foreground">
          Cleanup runs only when a project is stopped. No surprise deletion.
        </p>
      </header>
      <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
        <label className="flex flex-col gap-1.5">
          <span className="text-[10px] font-semibold uppercase tracking-[0.1em] text-muted-foreground">
            Inspector buffer
          </span>
          <div className="flex items-center gap-2 rounded-md border border-border/40 bg-background px-3 py-1.5">
            <input
              type="number"
              defaultValue={500}
              className="w-full bg-transparent font-mono text-sm tabular-nums text-foreground outline-none"
            />
            <span className="font-mono text-xs text-muted-foreground">
              / project
            </span>
          </div>
        </label>
        <label className="flex flex-col gap-1.5">
          <span className="text-[10px] font-semibold uppercase tracking-[0.1em] text-muted-foreground">
            Mail retention
          </span>
          <div className="flex items-center gap-2 rounded-md border border-border/40 bg-background px-3 py-1.5">
            <input
              type="number"
              defaultValue={7}
              className="w-full bg-transparent font-mono text-sm tabular-nums text-foreground outline-none"
            />
            <span className="font-mono text-xs text-muted-foreground">days</span>
          </div>
        </label>
        <label className="flex flex-col gap-1.5">
          <span className="text-[10px] font-semibold uppercase tracking-[0.1em] text-muted-foreground">
            Mail size cap
          </span>
          <div className="flex items-center gap-2 rounded-md border border-border/40 bg-background px-3 py-1.5">
            <input
              type="number"
              defaultValue={50}
              className="w-full bg-transparent font-mono text-sm tabular-nums text-foreground outline-none"
            />
            <span className="font-mono text-xs text-muted-foreground">MB</span>
          </div>
        </label>
      </div>
    </section>
  );
}

function AboutSection() {
  return (
    <section className="flex items-center justify-between rounded-md border border-border/40 bg-surface p-6">
      <div>
        <h2 className="text-sm font-semibold tracking-tight">About Nerd</h2>
        <p className="mt-1 font-mono text-xs text-muted-foreground">
          v0.1.0 · prototype · MIT
        </p>
        <p className="mt-3 max-w-xl text-sm text-muted-foreground">
          Lightweight, open-source local JavaScript development
          environment for Windows. Daemon-managed Node runtimes,
          per-project domains, mail capture, and request inspector.
        </p>
      </div>
      <div className="flex items-center gap-2">
        <Button variant="ghost" size="sm">
          Documentation
        </Button>
        <Button variant="ghost" size="sm">
          Release notes
        </Button>
      </div>
    </section>
  );
}

export function SettingsScreen() {
  return (
    <div className="h-full overflow-auto">
      <div className="flex flex-col gap-6 p-8">
        <ThemeRow />
        <Separator className="bg-border/40" />
        <PreferencesSection />
        <Separator className="bg-border/40" />
        <RetentionSection />
        <Separator className="bg-border/40" />
        <AboutSection />
      </div>
    </div>
  );
}
