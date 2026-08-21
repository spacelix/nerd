import { Check, Plus, ShieldCheck } from "lucide-react";
import * as React from "react";
import { cn } from "@/lib/utils";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
} from "@/components/ui/dialog";

const inputClass =
  "h-7 w-full rounded-md border border-border/60 bg-card/40 px-2 text-[11px] text-foreground placeholder:text-muted-foreground/60 focus:border-primary/50 focus:outline-none focus:ring-2 focus:ring-primary/20";

const selectClass = cn(inputClass, "appearance-none");

function Field({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: React.ReactNode;
}) {
  return (
    <label className="flex flex-col gap-1">
      <span className="text-[10px] font-semibold tracking-[0.14em] text-muted-foreground/70 uppercase">
        {label}
      </span>
      {children}
      {hint ? <span className="text-[10px] text-muted-foreground/60">{hint}</span> : null}
    </label>
  );
}

function PrimaryButton({
  children,
  onClick,
  className,
}: {
  children: React.ReactNode;
  onClick?: () => void;
  className?: string;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        "inline-flex h-7 items-center justify-center gap-1.5 rounded-md bg-primary px-3 text-[11px] font-medium text-primary-foreground transition-opacity hover:opacity-90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
        className,
      )}
    >
      {children}
    </button>
  );
}

function GhostButton({
  children,
  onClick,
}: {
  children: React.ReactNode;
  onClick?: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="inline-flex h-7 items-center rounded-md px-2 text-[11px] text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
    >
      {children}
    </button>
  );
}

function DoneState({ note }: { note: string }) {
  return (
    <div className="flex flex-col items-center gap-2 py-6 text-center">
      <span className="flex size-8 items-center justify-center rounded-full bg-success/15">
        <Check className="size-4 text-success" />
      </span>
      <p className="text-xs text-foreground/90">{note}</p>
      <p data-mono className="text-[10px] text-muted-foreground/60">
        prototype · no state mutated
      </p>
    </div>
  );
}

function ActionButton({
  label,
  icon,
  onClick,
}: {
  label: string;
  icon: React.ReactNode;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="inline-flex h-8 items-center gap-1.5 rounded-md border border-border/60 bg-card/40 px-2.5 text-[11px] font-medium text-foreground transition-colors hover:border-border hover:bg-accent/40 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
    >
      {icon}
      {label}
    </button>
  );
}

const frameworks = ["Next.js", "Vite (React)", "Vite (Vue)", "Vite (Svelte)", "Vite (vanilla)", "Nuxt", "Astro", "NestJS"];
const nodeVersions = ["22.11.0 (default)", "20.18.0", "24.4.0 (LTS)"];
const exactNodeVersion: Record<string, string> = {
  "22.11.0 (default)": "22.11.0",
  "20.18.0": "20.18.0",
  "24.4.0 (LTS)": "24.4.0",
};
const packageManagers = ["npm", "pnpm", "yarn"];

function NewProjectDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (o: boolean) => void;
}) {
  const [done, setDone] = React.useState(false);
  const [location, setLocation] = React.useState("C:\\Code\\app");
  const [node, setNode] = React.useState<string>(nodeVersions[0] ?? "22.11.0 (default)");
  const [gitInit, setGitInit] = React.useState(true);
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        {done ? (
          <>
            <DoneState note="app.test scaffolded, registered, and ready to start." />
            <DialogFooter>
              <GhostButton onClick={() => setDone(false)}>Create another</GhostButton>
              <PrimaryButton onClick={() => onOpenChange(false)}>Done</PrimaryButton>
            </DialogFooter>
          </>
        ) : (
          <>
            <div className="flex flex-col gap-0.5 pr-6">
              <DialogHeader>Create a new project</DialogHeader>
              <DialogDescription className="text-[11px] text-muted-foreground">
                Official scaffold CLI · nerd.json generated without secrets.
              </DialogDescription>
            </div>
            <div className="flex flex-col gap-2.5">
              <Field label="Location" hint="Parent of the new project directory. OneDrive and UNC paths are rejected.">
                <input
                  className={inputClass}
                  value={location}
                  onChange={(e) => setLocation(e.target.value)}
                />
              </Field>
              <Field label="Framework" hint="Exact version shown before execution.">
                <select className={selectClass} defaultValue={frameworks[0]}>
                  {frameworks.map((f) => (
                    <option key={f}>{f}</option>
                  ))}
                </select>
              </Field>
              <div className="grid grid-cols-2 gap-2.5">
                <Field label="Node">
                  <select
                    className={selectClass}
                    value={node}
                    onChange={(e) => setNode(e.target.value)}
                  >
                    {nodeVersions.map((n) => (
                      <option key={n}>{n}</option>
                    ))}
                  </select>
                </Field>
                <Field label="Package manager">
                  <select className={selectClass} defaultValue={packageManagers[0]}>
                    {packageManagers.map((p) => (
                      <option key={p}>{p}</option>
                    ))}
                  </select>
                </Field>
              </div>
              <p data-mono className="text-[10px] text-muted-foreground/70">
                Exact version: v{exactNodeVersion[node] ?? node} · managed Nerd runtime
              </p>
              <Field label="Initialize Git">
                <button
                  type="button"
                  aria-pressed={gitInit}
                  onClick={() => setGitInit((v) => !v)}
                  className={cn(
                    "inline-flex h-7 items-center gap-1.5 rounded-md border px-2.5 text-[11px] font-medium transition-colors",
                    gitInit
                      ? "border-primary/40 bg-primary/10 text-foreground"
                      : "border-border/60 bg-background/40 text-muted-foreground hover:text-foreground",
                  )}
                >
                  {gitInit ? "Enabled" : "Disabled"}
                </button>
              </Field>
              <Field label="Services">
                <div className="flex flex-wrap gap-1.5">
                  {["MySQL", "PostgreSQL", "Redis"].map((s) => (
                    <button
                      key={s}
                      type="button"
                      aria-pressed="false"
                      className="rounded border border-border/60 bg-background/40 px-2 py-1 text-[11px] text-muted-foreground transition-colors hover:text-foreground"
                    >
                      + {s}
                    </button>
                  ))}
                </div>
              </Field>
            </div>
            <p className="flex items-start gap-1.5 rounded-md border border-border/50 bg-background/40 px-2 py-1.5 text-[10px] leading-relaxed text-muted-foreground/70">
              <ShieldCheck className="mt-0.5 size-3 shrink-0 text-success" />
              Scaffold and dependency scripts run only after explicit approval. Cancellation is safe before the trust step (OD-029).
            </p>
            <DialogFooter>
              <DialogClose>
                <GhostButton>Cancel</GhostButton>
              </DialogClose>
              <PrimaryButton onClick={() => setDone(true)}>
                <Plus className="size-3" />
                Create project
              </PrimaryButton>
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}

function ParkDirectoryDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (o: boolean) => void;
}) {
  const [done, setDone] = React.useState(false);
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        {done ? (
          <>
            <DoneState note="C:\Code parked. Immediate-child projects with package.json will appear after the first change-notification cycle." />
            <DialogFooter>
              <PrimaryButton onClick={() => onOpenChange(false)}>Done</PrimaryButton>
            </DialogFooter>
          </>
        ) : (
          <>
            <div className="flex flex-col gap-0.5 pr-6">
              <DialogHeader>Park a directory</DialogHeader>
              <DialogDescription className="text-[11px] text-muted-foreground">
                Immediate children with a root package.json become parked projects. Empty roots stay registered.
              </DialogDescription>
            </div>
            <Field label="Directory">
              <input className={inputClass} defaultValue="C:\Code" placeholder="C:\Code" />
            </Field>
            <p className="rounded-md border border-border/50 bg-background/40 px-2 py-1.5 text-[10px] leading-relaxed text-muted-foreground/70">
              Discovery reads metadata only and never executes project code. Changes appear via native watchers — no polling. Unsupported locations (OneDrive, \\wsl$, UNC) are rejected before registration.
            </p>
            <DialogFooter>
              <DialogClose>
                <GhostButton>Cancel</GhostButton>
              </DialogClose>
              <PrimaryButton onClick={() => setDone(true)}>Park directory</PrimaryButton>
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}

function LinkProjectDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (o: boolean) => void;
}) {
  const [done, setDone] = React.useState(false);
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        {done ? (
          <>
            <DoneState note="Linked. The project starts untrusted until you approve it for execution." />
            <DialogFooter>
              <PrimaryButton onClick={() => onOpenChange(false)}>Done</PrimaryButton>
            </DialogFooter>
          </>
        ) : (
          <>
            <div className="flex flex-col gap-0.5 pr-6">
              <DialogHeader>Link an existing project</DialogHeader>
              <DialogDescription className="text-[11px] text-muted-foreground">
                Registers one individual project without parking its parent.
              </DialogDescription>
            </div>
            <Field label="Project path">
              <input className={inputClass} defaultValue="C:\Code\storefront" placeholder="C:\Code\storefront" />
            </Field>
            <Field label="Alias (optional)" hint="An explicit unique name resolves a parked-name conflict without moving files.">
              <input className={inputClass} placeholder="storefront" />
            </Field>
            <DialogFooter>
              <DialogClose>
                <GhostButton>Cancel</GhostButton>
              </DialogClose>
              <PrimaryButton onClick={() => setDone(true)}>Link project</PrimaryButton>
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}

function InstallRuntimeDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (o: boolean) => void;
}) {
  const [done, setDone] = React.useState(false);
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        {done ? (
          <>
            <DoneState note="Node v22.11.0 installed and checksum-verified." />
            <DialogFooter>
              <PrimaryButton onClick={() => onOpenChange(false)}>Done</PrimaryButton>
            </DialogFooter>
          </>
        ) : (
          <>
            <div className="flex flex-col gap-0.5 pr-6">
              <DialogHeader>Install a Node runtime</DialogHeader>
              <DialogDescription className="text-[11px] text-muted-foreground">
                Official Windows x64 ZIP · checksum verified before extraction.
              </DialogDescription>
            </div>
            <Field label="Version" hint="System Node is never touched.">
              <select className={selectClass} defaultValue={nodeVersions[0]}>
                {nodeVersions.map((n) => (
                  <option key={n}>{n}</option>
                ))}
              </select>
            </Field>
            <div className="flex flex-col gap-1">
              <Step label="Download" state="pending" />
              <Step label="Verify checksum" state="pending" />
              <Step label="Extract atomically" state="pending" />
              <Step label="Register inventory" state="pending" />
            </div>
            <p className="rounded-md border border-border/50 bg-background/40 px-2 py-1.5 text-[10px] leading-relaxed text-muted-foreground/70">
              Allow official Node artifact origins only. Tampered archives and archive traversal are rejected.
            </p>
            <DialogFooter>
              <DialogClose>
                <GhostButton>Cancel</GhostButton>
              </DialogClose>
              <PrimaryButton onClick={() => setDone(true)}>Install</PrimaryButton>
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}

function Step({ label, state }: { label: string; state: "pending" | "done" }) {
  return (
    <div className="flex items-center gap-2">
      <span
        className={cn(
          "flex size-3.5 items-center justify-center rounded-full border text-[9px]",
          state === "done"
            ? "border-success text-success"
            : "border-border/60 text-muted-foreground/50",
        )}
      >
        {state === "done" ? "✓" : ""}
      </span>
      <span className="text-[11px] text-muted-foreground">{label}</span>
    </div>
  );
}

function AddServiceDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (o: boolean) => void;
}) {
  const [engine, setEngine] = React.useState("MySQL");
  const [done, setDone] = React.useState(false);
  const blocker = engine === "MySQL" ? "OD-002" : engine === "PostgreSQL" ? "OD-003" : "OD-004";
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        {done ? (
          <>
            <DoneState note={`${engine} instance allocated on a dynamic loopback port.`} />
            <DialogFooter>
              <PrimaryButton onClick={() => onOpenChange(false)}>Done</PrimaryButton>
            </DialogFooter>
          </>
        ) : (
          <>
            <div className="flex flex-col gap-0.5 pr-6">
              <DialogHeader>Add a service</DialogHeader>
              <DialogDescription className="text-[11px] text-muted-foreground">
                Per-project isolated instance · shared binary cache by engine/version.
              </DialogDescription>
            </div>
            <Field label="Engine">
              <select
                className={selectClass}
                value={engine}
                onChange={(e) => setEngine(e.target.value)}
              >
                <option>MySQL</option>
                <option>PostgreSQL</option>
                <option>Redis</option>
              </select>
            </Field>
            <p className="flex items-start gap-1.5 rounded-md border border-warning/30 bg-warning-soft/60 px-2 py-1.5 text-[10px] leading-relaxed text-muted-foreground">
              <span data-mono className="font-medium text-warning">{blocker}</span>
              Bundled-binary lifecycle is still an open decision — the instance will be informational until it resolves.
            </p>
            <DialogFooter>
              <DialogClose>
                <GhostButton>Cancel</GhostButton>
              </DialogClose>
              <PrimaryButton onClick={() => setDone(true)}>Add service</PrimaryButton>
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}

function UnparkProjectDialog({
  open,
  onOpenChange,
  project,
  onConfirm,
}: {
  open: boolean;
  onOpenChange: (o: boolean) => void;
  project: string;
  onConfirm: () => void;
}) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <div className="flex flex-col gap-0.5 pr-6">
          <DialogHeader>Unpark {project}</DialogHeader>
          <DialogDescription className="text-[11px] text-muted-foreground">
            Removes the project from the parked root. Data on disk is never deleted.
          </DialogDescription>
        </div>
        <p className="rounded-md border border-border/50 bg-background/40 px-2 py-1.5 text-[10px] leading-relaxed text-muted-foreground/70">
          The parked root stays registered. The project stops routing until re-parked or linked; its services and data remain untouched.
        </p>
        <DialogFooter>
          <DialogClose>
            <GhostButton>Cancel</GhostButton>
          </DialogClose>
          <PrimaryButton
            className="bg-danger text-danger-foreground"
            onClick={() => {
              onConfirm();
              onOpenChange(false);
            }}
          >
            Unpark {project}
          </PrimaryButton>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function UnlinkProjectDialog({
  open,
  onOpenChange,
  project,
  onConfirm,
}: {
  open: boolean;
  onOpenChange: (o: boolean) => void;
  project: string;
  onConfirm: () => void;
}) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <div className="flex flex-col gap-0.5 pr-6">
          <DialogHeader>Unlink {project}</DialogHeader>
          <DialogDescription className="text-[11px] text-muted-foreground">
            Removes the link. The directory and files are never modified.
          </DialogDescription>
        </div>
        <p className="rounded-md border border-border/50 bg-background/40 px-2 py-1.5 text-[10px] leading-relaxed text-muted-foreground/70">
          Trust and local overrides stay bound to the project identity; the project simply stops routing.
        </p>
        <DialogFooter>
          <DialogClose>
            <GhostButton>Cancel</GhostButton>
          </DialogClose>
          <PrimaryButton
            className="bg-danger text-danger-foreground"
            onClick={() => {
              onConfirm();
              onOpenChange(false);
            }}
          >
            Unlink {project}
          </PrimaryButton>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function RegisterExternalRuntimeDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (o: boolean) => void;
}) {
  const [done, setDone] = React.useState(false);
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        {done ? (
          <>
            <DoneState note="External Node registered. Re-probed before every launch; system Node is never touched." />
            <DialogFooter>
              <PrimaryButton onClick={() => onOpenChange(false)}>Done</PrimaryButton>
            </DialogFooter>
          </>
        ) : (
          <>
            <div className="flex flex-col gap-0.5 pr-6">
              <DialogHeader>Register an external Node</DialogHeader>
              <DialogDescription className="text-[11px] text-muted-foreground">
                Read-only discovery of an existing installation. Nerd never copies, repairs, updates, or uninstalls it.
              </DialogDescription>
            </div>
            <Field label="Executable path">
              <input className={inputClass} defaultValue="C:\Program Files\nodejs\node.exe" />
            </Field>
            <p className="rounded-md border border-border/50 bg-background/40 px-2 py-1.5 text-[10px] leading-relaxed text-muted-foreground/70">
              Version probe is read-only. Missing path, changed binary identity, or incompatible architecture marks the runtime degraded.
            </p>
            <DialogFooter>
              <DialogClose>
                <GhostButton>Cancel</GhostButton>
              </DialogClose>
              <PrimaryButton onClick={() => setDone(true)}>Register external</PrimaryButton>
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}

function RegisterExternalServiceDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (o: boolean) => void;
}) {
  const [done, setDone] = React.useState(false);
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        {done ? (
          <>
            <DoneState note="External connection registered. Informational until health probes confirm engine identity." />
            <DialogFooter>
              <PrimaryButton onClick={() => onOpenChange(false)}>Done</PrimaryButton>
            </DialogFooter>
          </>
        ) : (
          <>
            <div className="flex flex-col gap-0.5 pr-6">
              <DialogHeader>Register an external service</DialogHeader>
              <DialogDescription className="text-[11px] text-muted-foreground">
                Connect to a machine-local listener. Nerd never starts, stops, or reconfigures it.
              </DialogDescription>
            </div>
            <Field label="Engine">
              <select className={selectClass} defaultValue="PostgreSQL">
                <option>MySQL</option>
                <option>PostgreSQL</option>
                <option>Redis</option>
              </select>
            </Field>
            <Field label="Endpoint" hint="Loopback only. Credentials go to DPAPI or the project .env — never nerd.json.">
              <input className={inputClass} defaultValue="127.0.0.1:5432" />
            </Field>
            <DialogFooter>
              <DialogClose>
                <GhostButton>Cancel</GhostButton>
              </DialogClose>
              <PrimaryButton onClick={() => setDone(true)}>Register external</PrimaryButton>
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}

function DeleteProjectDialog({
  open,
  onOpenChange,
  project,
  onConfirm,
}: {
  open: boolean;
  onOpenChange: (o: boolean) => void;
  project: string;
  onConfirm: () => void;
}) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <div className="flex flex-col gap-0.5 pr-6">
          <DialogHeader>Delete {project}</DialogHeader>
          <DialogDescription className="text-[11px] text-muted-foreground">
            Removes the project from Nerd. Files on disk are never deleted.
          </DialogDescription>
        </div>
        <p className="rounded-md border border-border/50 bg-background/40 px-2 py-1.5 text-[10px] leading-relaxed text-muted-foreground/70">
          Only Nerd's registration (parked/linked entry, overrides, pinned state) is
          removed. The directory, services, and data stay untouched so the project
          can be re-added later.
        </p>
        <DialogFooter>
          <DialogClose>
            <GhostButton>Cancel</GhostButton>
          </DialogClose>
          <PrimaryButton
            className="bg-danger text-danger-foreground"
            onClick={() => {
              onConfirm();
              onOpenChange(false);
            }}
          >
            Delete {project}
          </PrimaryButton>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function UninstallRuntimeDialog({
  open,
  onOpenChange,
  version,
  onConfirm,
}: {
  open: boolean;
  onOpenChange: (o: boolean) => void;
  version: string;
  onConfirm: () => void;
}) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <div className="flex flex-col gap-0.5 pr-6">
          <DialogHeader>Uninstall Node v{version}</DialogHeader>
          <DialogDescription className="text-[11px] text-muted-foreground">
            Removes the managed runtime Nerd owns. System Node is never touched.
          </DialogDescription>
        </div>
        <p className="rounded-md border border-border/50 bg-background/40 px-2 py-1.5 text-[10px] leading-relaxed text-muted-foreground/70">
          Projects pinned to this version fall back to the default runtime. The
          download cache entry for this version is also removed.
        </p>
        <DialogFooter>
          <DialogClose>
            <GhostButton>Cancel</GhostButton>
          </DialogClose>
          <PrimaryButton
            className="bg-danger text-danger-foreground"
            onClick={() => {
              onConfirm();
              onOpenChange(false);
            }}
          >
            Uninstall v{version}
          </PrimaryButton>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function RemoveExternalServiceDialog({
  open,
  onOpenChange,
  service,
  onConfirm,
}: {
  open: boolean;
  onOpenChange: (o: boolean) => void;
  service: string;
  onConfirm: () => void;
}) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <div className="flex flex-col gap-0.5 pr-6">
          <DialogHeader>Remove {service}</DialogHeader>
          <DialogDescription className="text-[11px] text-muted-foreground">
            Drops the external connection. The remote listener is never touched.
          </DialogDescription>
        </div>
        <p className="rounded-md border border-border/50 bg-background/40 px-2 py-1.5 text-[10px] leading-relaxed text-muted-foreground/70">
          Projects lose this endpoint from their configuration. Stored credentials
          under DPAPI are revoked and never logged.
        </p>
        <DialogFooter>
          <DialogClose>
            <GhostButton>Cancel</GhostButton>
          </DialogClose>
          <PrimaryButton
            className="bg-danger text-danger-foreground"
            onClick={() => {
              onConfirm();
              onOpenChange(false);
            }}
          >
            Remove {service}
          </PrimaryButton>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

export {
  ActionButton,
  PrimaryButton,
  GhostButton,
  DoneState,
  NewProjectDialog,
  ParkDirectoryDialog,
  LinkProjectDialog,
  InstallRuntimeDialog,
  AddServiceDialog,
  UnparkProjectDialog,
  UnlinkProjectDialog,
  RegisterExternalRuntimeDialog,
  RegisterExternalServiceDialog,
  DeleteProjectDialog,
  UninstallRuntimeDialog,
  RemoveExternalServiceDialog,
};