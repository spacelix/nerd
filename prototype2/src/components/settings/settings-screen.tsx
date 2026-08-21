import {
  CloudDownload,
  Database,
  HardDrive,
  Moon,
  Monitor,
  Play,
  RefreshCw,
  Server,
  ShieldCheck,
  Sun,
  Wrench,
} from "lucide-react";
import * as React from "react";
import { cn } from "@/lib/utils";
import { Switch } from "@/components/ui/switch";
import { useTheme, type Theme } from "@/hooks/use-theme";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
} from "@/components/ui/dialog";

const themeOptions: { value: Theme; label: string; icon: React.ReactNode }[] = [
  { value: "dark", label: "Dark", icon: <Moon className="size-3.5" /> },
  { value: "light", label: "Light", icon: <Sun className="size-3.5" /> },
  { value: "system", label: "System", icon: <Monitor className="size-3.5" /> },
];

function SettingsScreen() {
  const { theme, setTheme } = useTheme();
  const [discovery, setDiscovery] = React.useState({
    linkedRepos: true,
    attachServers: true,
    trustRootCa: false,
    nativeNotifications: true,
  });
  const [retention, setRetention] = React.useState({
    requests: "90",
    mail: "500",
    logs: "30",
  });
  const [uninstallOpen, setUninstallOpen] = React.useState(false);

  return (
    <div className="flex min-h-full flex-col gap-5 px-10 py-10">
      <header className="flex flex-col gap-2">
        <span
          data-mono
          className="text-[11px] tracking-[0.18em] text-muted-foreground/60 uppercase"
        >
          Route · settings · N6
        </span>
        <h1 className="text-xl font-semibold tracking-tight text-foreground">
          Settings
        </h1>
        <p className="max-w-xl text-xs text-muted-foreground">
          Appearance, discovery, retention, and about information.
        </p>
      </header>

      <div className="grid grid-cols-1 gap-3 xl:grid-cols-2">
        <SettingsCard
          title="Appearance"
          icon={<Moon className="size-3.5 text-muted-foreground" />}
        >
          <div className="flex gap-1.5">
            {themeOptions.map((opt) => (
              <button
                key={opt.value}
                type="button"
                aria-pressed={theme === opt.value}
                onClick={() => setTheme(opt.value)}
                className={cn(
                  "inline-flex h-7 flex-1 items-center justify-center gap-1.5 rounded-md border text-[11px] font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                  theme === opt.value
                    ? "border-primary/50 bg-surface-active text-foreground"
                    : "border-border/60 bg-card/40 text-muted-foreground hover:border-border hover:text-foreground",
                )}
              >
                {opt.icon}
                {opt.label}
              </button>
            ))}
          </div>
        </SettingsCard>

        <SettingsCard
          title="Discovery"
          icon={<Server className="size-3.5 text-muted-foreground" />}
        >
          <SettingRow
            label="Auto-discover linked repositories"
            help="Scan known workspaces for new `nerd.json` projects at startup."
          >
            <Switch
              checked={discovery.linkedRepos}
              onCheckedChange={(v) =>
                setDiscovery((d) => ({ ...d, linkedRepos: v }))
              }
              label="Auto-discover linked repositories"
            />
          </SettingRow>
          <SettingRow
            label="Attach to running local servers"
            help="Claim loopback ports with Nerd ownership metadata before attach."
          >
            <Switch
              checked={discovery.attachServers}
              onCheckedChange={(v) =>
                setDiscovery((d) => ({ ...d, attachServers: v }))
              }
              label="Attach to running local servers"
            />
          </SettingRow>
          <SettingRow
            label="Trust root CA for .test"
            help="Installs the Nerd development root certificate (requires elevation)."
          >
            <Switch
              checked={discovery.trustRootCa}
              onCheckedChange={(v) =>
                setDiscovery((d) => ({ ...d, trustRootCa: v }))
              }
              label="Trust root CA for .test"
            />
          </SettingRow>
          <SettingRow
            label="Use native change notifications"
            help="No filesystem polling — events come from Windows change watchers."
          >
            <Switch
              checked={discovery.nativeNotifications}
              onCheckedChange={(v) =>
                setDiscovery((d) => ({ ...d, nativeNotifications: v }))
              }
              label="Use native change notifications"
            />
          </SettingRow>
        </SettingsCard>

        <SettingsCard
          title="Retention"
          icon={<ShieldCheck className="size-3.5 text-muted-foreground" />}
        >
          <SettingRow label="Request capture" help="Days before captured requests are pruned.">
            <RetentionSelect
              value={retention.requests}
              onChange={(v) => setRetention((r) => ({ ...r, requests: v }))}
              label="Request capture retention"
            />
          </SettingRow>
          <SettingRow label="Mail messages" help="Cap by count or size before sandboxed mail is pruned.">
            <MailRetentionSelect
              value={retention.mail}
              onChange={(v) => setRetention((r) => ({ ...r, mail: v }))}
              label="Mail retention"
            />
          </SettingRow>
          <SettingRow label="Logs" help="Days before per-project logs are pruned.">
            <RetentionSelect
              value={retention.logs}
              onChange={(v) => setRetention((r) => ({ ...r, logs: v }))}
              label="Log retention"
            />
          </SettingRow>
        </SettingsCard>

        <SettingsCard
          title="Network & HTTPS"
          icon={<ShieldCheck className="size-3.5 text-muted-foreground" />}
        >
          <StatusRow label="DNS listener" value="127.0.0.1:53 (UDP + TCP)" ok />
          <StatusRow label="NRPT rule" value=".test → loopback" ok />
          <StatusRow label="Root CA" value="CurrentUser · DPAPI key" ok />
          <StatusRow label="Proxy ports" value="80 ✓ · 443 ✓" ok />
          <p className="rounded-md border border-border/50 bg-background/40 px-2 py-1.5 text-[10px] leading-relaxed text-muted-foreground/70">
            Setup runs elevated in one batch session via the typed helper
            (`nrpt_add` only). Nerd probes first, rolls back every completed
            mutation on failure, and never touches foreign listeners or
            certificates.
          </p>
          <div className="flex flex-wrap gap-2">
            <button
              type="button"
              className="inline-flex h-7 items-center gap-1.5 rounded-md bg-primary px-2.5 text-[11px] font-medium text-primary-foreground transition-opacity hover:opacity-90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            >
              <Play className="size-3" />
              Run setup (UAC)
            </button>
            <button
              type="button"
              className="inline-flex h-7 items-center gap-1.5 rounded-md border border-border/60 bg-card/40 px-2.5 text-[11px] font-medium text-foreground transition-colors hover:border-border hover:bg-accent/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            >
              <RefreshCw className="size-3" />
              Probe now
            </button>
          </div>
        </SettingsCard>

        <SettingsCard
          title="Storage & safe cleanup"
          icon={<HardDrive className="size-3.5 text-muted-foreground" />}
        >
          <UsageRow label="Request capture" used="18.2 MB" limit="500 req / project" pct={38} />
          <UsageRow label="Mail inbox" used="42 MB" limit="500 msg · 250 MB" pct={62} />
          <UsageRow label="Per-project logs" used="11 MB" limit="rotated · 5 generations" pct={24} />
          <UsageRow label="Download cache" used="120 MB" limit="verified archives" pct={48} />
          <p className="rounded-md border border-border/50 bg-background/40 px-2 py-1.5 text-[10px] leading-relaxed text-muted-foreground/70">
            Cleanup covers expired logs, mail, temp files, and unreferenced verified
            caches only. Backups and project data always require separate
            confirmation.
          </p>
          <button
            type="button"
            className="inline-flex h-7 items-center gap-1.5 rounded-md border border-danger/30 bg-danger/5 px-2.5 text-[11px] font-medium text-danger transition-colors hover:border-danger/50 hover:bg-danger/10 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            <Database className="size-3" />
            Clean up Nerd-owned cache
          </button>
        </SettingsCard>

        <SettingsCard
          title="About"
          icon={<span className="font-mono text-xs text-muted-foreground">i</span>}
        >
          <div className="flex flex-col gap-1">
            <AboutRow label="Version" value="v0.1.0-alpha.1" />
            <AboutRow label="Platform" value="Windows 10 22H2 x64 (min)" />
            <AboutRow label="IPC" value="1 · SQLite 1" />
            <AboutRow label="Build" value="dev (prototype2)" />
            <AboutRow label="License" value="Proprietary" />
          </div>
          <p className="mt-3 text-[11px] leading-relaxed text-muted-foreground/70">
            No telemetry, no account, no remote diagnostics. Secrets stay in the
            Windows DPAPI store and never enter logs, IPC errors, nerd.json, or
            UI events.
          </p>
          <div className="flex flex-wrap gap-2">
            <button
              type="button"
              className="inline-flex h-7 items-center gap-1.5 rounded-md border border-border/60 bg-card/40 px-2.5 text-[11px] font-medium text-foreground transition-colors hover:border-border hover:bg-accent/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            >
              <CloudDownload className="size-3" />
              Check for updates
            </button>
            <button
              type="button"
              onClick={() => {
                localStorage.removeItem("nerd-prototype2-onboarded");
                location.reload();
              }}
              className="inline-flex h-7 items-center gap-1.5 rounded-md border border-border/60 bg-card/40 px-2.5 text-[11px] font-medium text-foreground transition-colors hover:border-border hover:bg-accent/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            >
              <Wrench className="size-3" />
              Replay onboarding
            </button>
            <button
              type="button"
              onClick={() => setUninstallOpen(true)}
              className="inline-flex h-7 items-center gap-1.5 rounded-md border border-danger/30 bg-danger/5 px-2.5 text-[11px] font-medium text-danger transition-colors hover:border-danger/50 hover:bg-danger/10 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            >
              <Database className="size-3" />
              Uninstall Nerd
            </button>
          </div>
          <p className="rounded-md border border-border/50 bg-background/40 px-2 py-1.5 text-[10px] leading-relaxed text-muted-foreground/70">
            Uninstall matches ownership markers and removes NRPT, CA, autostart,
            PATH, and binaries symmetrically. Default keeps project and service
            data.             External Node / databases are never removed.
          </p>
        </SettingsCard>
      </div>
      <UninstallDialog open={uninstallOpen} onOpenChange={setUninstallOpen} />
    </div>
  );
}

function SettingsCard({
  title,
  icon,
  children,
}: {
  title: string;
  icon: React.ReactNode;
  children: React.ReactNode;
}) {
  return (
    <section className="flex flex-col gap-3 rounded-lg border border-border/60 bg-card/40 p-4">
      <span className="flex items-center gap-2 text-[10px] font-semibold tracking-[0.16em] text-muted-foreground/70 uppercase">
        {icon}
        {title}
      </span>
      <div className="flex flex-col gap-2.5">{children}</div>
    </section>
  );
}

function SettingRow({
  label,
  help,
  children,
}: {
  label: string;
  help: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-center justify-between gap-3">
      <span className="min-w-0">
        <span className="block text-xs text-foreground/90">{label}</span>
        <span className="block text-[11px] text-muted-foreground/70">{help}</span>
      </span>
      {children}
    </div>
  );
}

function RetentionSelect({
  value,
  onChange,
  label,
}: {
  value: string;
  onChange: (v: string) => void;
  label: string;
}) {
  return (
    <select
      aria-label={label}
      value={value}
      onChange={(e) => onChange(e.target.value)}
      className="h-7 rounded-md border border-border/60 bg-card/40 px-2 text-[11px] text-foreground focus:border-primary/50 focus:outline-none focus:ring-2 focus:ring-primary/20"
    >
      <option value="7">7 days</option>
      <option value="30">30 days</option>
      <option value="90">90 days</option>
      <option value="365">365 days</option>
      <option value="forever">Forever</option>
    </select>
  );
}

function MailRetentionSelect({
  value,
  onChange,
  label,
}: {
  value: string;
  onChange: (v: string) => void;
  label: string;
}) {
  return (
    <select
      aria-label={label}
      value={value}
      onChange={(e) => onChange(e.target.value)}
      className="h-7 rounded-md border border-border/60 bg-card/40 px-2 text-[11px] text-foreground focus:border-primary/50 focus:outline-none focus:ring-2 focus:ring-primary/20"
    >
      <option value="500">500 messages</option>
      <option value="1000">1000 messages</option>
      <option value="250mb">250 MB cap</option>
      <option value="forever">Forever</option>
    </select>
  );
}

function AboutRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="grid grid-cols-[5rem_minmax(0,1fr)] items-baseline gap-2 py-0.5">
      <span className="text-[11px] text-muted-foreground/70">{label}</span>
      <span data-mono className="text-xs text-foreground/90">{value}</span>
    </div>
  );
}

function StatusRow({
  label,
  value,
  ok,
}: {
  label: string;
  value: string;
  ok: boolean;
}) {
  return (
    <div className="flex items-center justify-between gap-3">
      <span className="flex min-w-0 items-center gap-2 text-xs text-foreground/90">
        <span
          aria-hidden="true"
          className={cn(
            "size-1.5 shrink-0 rounded-full",
            ok ? "bg-success" : "bg-danger",
          )}
        />
        <span className="truncate">{label}</span>
      </span>
      <span data-mono className="shrink-0 text-[11px] text-muted-foreground">
        {value}
      </span>
    </div>
  );
}

function UsageRow({
  label,
  used,
  limit,
  pct,
}: {
  label: string;
  used: string;
  limit: string;
  pct: number;
}) {
  return (
    <div className="flex flex-col gap-1">
      <div className="flex items-baseline justify-between gap-3">
        <span className="text-xs text-foreground/90">{label}</span>
        <span data-mono className="text-[11px] text-muted-foreground">
          {used} · {limit}
        </span>
      </div>
      <div className="h-1 overflow-hidden rounded-full bg-muted/60">
        <div
          className={cn(
            "h-full rounded-full",
            pct >= 80 ? "bg-warning" : "bg-primary/70",
          )}
          style={{ width: `${pct}%` }}
        />
      </div>
    </div>
  );
}

function UninstallDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (o: boolean) => void;
}) {
  const [confirmText, setConfirmText] = React.useState("");
  const [done, setDone] = React.useState(false);
  const scopes = [
    "NRPT .test rule",
    "Development root CA",
    "Autostart + PATH",
    "Nerd binaries + daemon",
  ];
  const ready = confirmText.trim().toLowerCase() === "uninstall nerd";
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        {done ? (
          <div className="flex flex-col items-center gap-2 py-6 text-center">
            <span className="flex size-8 items-center justify-center rounded-full bg-success/15">
              <CloudDownload className="size-4 text-success" />
            </span>
            <p className="text-xs text-foreground/90">
              Nerd uninstalled. Project directories and service data were kept.
            </p>
            <p data-mono className="text-[10px] text-muted-foreground/60">
              prototype · no state mutated
            </p>
          </div>
        ) : (
          <>
            <div className="flex flex-col gap-0.5 pr-6">
              <DialogHeader>Uninstall Nerd</DialogHeader>
              <DialogDescription className="text-[11px] text-muted-foreground">
                Removes Nerd-owned resources symmetrically. Project data stays.
              </DialogDescription>
            </div>
            <div className="flex flex-col gap-1.5">
              {scopes.map((s) => (
                <div
                  key={s}
                  className="flex items-center gap-2 rounded-md border border-border/50 bg-background/40 px-2.5 py-1.5"
                >
                  <span
                    aria-hidden="true"
                    className="size-1.5 rounded-full bg-danger/70"
                  />
                  <span className="text-[11px] text-foreground/90">{s}</span>
                </div>
              ))}
            </div>
            <p className="rounded-md border border-border/50 bg-background/40 px-2 py-1.5 text-[10px] leading-relaxed text-muted-foreground/70">
              Keeps: project directories, linked repositories, service data,
              external Node, and databases. External listeners are never touched.
            </p>
            <input
              className="h-7 w-full rounded-md border border-border/60 bg-card/40 px-2 text-[11px] text-foreground placeholder:text-muted-foreground/60 focus:border-primary/50 focus:outline-none focus:ring-2 focus:ring-primary/20"
              placeholder="Type “Uninstall Nerd” to confirm"
              value={confirmText}
              onChange={(e) => setConfirmText(e.target.value)}
            />
            <DialogFooter>
              <button
                type="button"
                onClick={() => onOpenChange(false)}
                className="inline-flex h-7 items-center rounded-md px-2 text-[11px] text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              >
                Cancel
              </button>
              <button
                type="button"
                disabled={!ready}
                onClick={() => setDone(true)}
                className="inline-flex h-7 items-center gap-1.5 rounded-md bg-danger px-3 text-[11px] font-medium text-danger-foreground transition-opacity hover:opacity-90 disabled:pointer-events-none disabled:opacity-50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              >
                <Database className="size-3" />
                Uninstall
              </button>
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}

export { SettingsScreen };