# Architecture

## Stack

| Layer | Technology | Responsibility |
|---|---|---|
| Core and daemon | Rust | State, DNS, proxy, process and service supervision |
| Async runtime | Tokio | Network, IPC, filesystem events, process I/O |
| Desktop shell | Tauri 2 | Native window, tray, lifecycle, packaging |
| Desktop UI | React + TypeScript | User interface only |
| Local state | SQLite | Registry, settings, metadata, migrations |
| IPC | Windows named pipe | Versioned daemon API and event stream |
| Project config | `nerd.json` + JSON Schema | Reproducible repository configuration |
| Privilege boundary | Separate Rust helper | Typed UAC-only setup and removal operations |

Exact third-party crates and npm packages must be approved in `library-docs.md` before use.

## Process Model

```text
Desktop UI                         nerd CLI
    |                                 |
    +------------ named pipe ---------+
                      |
                nerd-daemon.exe
                      |
       +--------------+----------------+
       |              |                |
   DNS/proxy      project runner   service supervisor
       |              |                |
  127.0.0.1       user process      user process

nerd-daemon.exe --typed request--> nerd-helper.exe --UAC--> Windows
```

### `nerd-daemon.exe`

Per-user, non-elevated, long-running control plane. Owns all mutable application state and runtime processes.

### `nerd.exe`

Thin CLI client. It validates command syntax, calls IPC, renders results, and contains no independent business logic.

### Desktop

Tauri and React client. Closing the window hides or exits the GUI, not the daemon. GUI state is derived from daemon snapshots and events.

### `nerd-helper.exe`

Short-lived elevated helper. It accepts only typed, allowlisted operations:

- Install/remove/probe NRPT rule
- Install/remove/probe Nerd root CA
- Install/remove/probe daemon autostart
- Install/remove Nerd CLI PATH entry if installer cannot do it safely

It never accepts shell text, executable paths supplied by projects, or arbitrary registry mutations.

## Repository Shape

```text
/
├── Cargo.toml
├── package.json
├── AGENTS.md
├── context/
│   ├── decisions/
│   ├── features/
│   └── *.md
├── crates/
│   ├── nerd-core/
│   ├── nerd-daemon/
│   ├── nerd-cli/
│   └── nerd-helper/
├── apps/
│   └── desktop/
│       ├── src/
│       └── src-tauri/
├── schemas/
│   ├── nerd.schema.json
│   └── ipc.schema.json
└── tests/
    ├── fixtures/
    └── windows/
```

Keep implementation in modules inside these crates. Add another crate only for a real binary, privilege, portability, or dependency boundary.

## Dependency Direction

```text
nerd-core <- nerd-daemon
nerd-core <- nerd-cli
nerd-core <- nerd-helper
daemon IPC <- desktop
```

Rules:

- `nerd-core` contains domain types and pure decisions. It performs no UI work and no privileged mutation.
- CLI and desktop never access SQLite or runtime directories directly.
- Helper never imports project execution, package manager, proxy, inspector, or service orchestration code.
- React never invokes OS commands directly.

## Daemon Modules

Initial modules inside `nerd-daemon`:

```text
state/        SQLite repositories and migrations
ipc/          commands, responses, subscriptions, protocol version
projects/     park/link registry, scanner, watcher, manifest merge
runtimes/     Node release index, install, resolve, child environment
processes/    Job Objects, stdout/stderr, readiness, lifecycle
network/      DNS, certificates, HTTP/HTTPS proxy, WebSocket
services/     common lifecycle plus MySQL/Postgres/Redis adapters
mail/         SMTP capture, MIME storage, retention
inspector/    request events, redaction, bounded buffers
diagnostics/  probes and safe repairs
updates/      signed manifest and release staging
```

## Data Layout

```text
%LOCALAPPDATA%\Nerd\
├── nerd.db
├── runtimes\node\<version>\
├── package-managers\
├── services\binaries\<engine>\<version>\
├── services\projects\<project-id>\<engine>\
├── certificates\
├── mail\<project-id>\
├── logs\
└── cache\
```

Generated credentials and private keys are encrypted with Windows DPAPI. SQLite stores encrypted blobs or references, never plaintext secrets.

## State Ownership

SQLite is the daemon-owned source of truth for local state:

- Schema version
- Settings
- Parked roots
- Linked projects
- Local project overrides
- Installed runtime and service inventory
- Project lifecycle intent
- Allocated ports
- Mail metadata
- Backup metadata

`nerd.json` is repository-owned desired configuration. Effective configuration is resolved as:

```text
built-in defaults
<- global user settings
<- auto-detected project metadata
<- nerd.json
<- local project overrides
```

Secrets and generated ports are never written back to `nerd.json`.

## Resource Ownership And Coexistence

Every runtime, service, listener, certificate, resolver rule, autostart entry, and downloaded artifact is classified as `Managed` or `External`.

### Managed

- Created or installed through Nerd.
- Stores stable Nerd ownership ID, type, version, path, and integrity metadata.
- May be mutated only after ownership metadata and expected path/fingerprint/process identity match.
- May be removed by Nerd through a feature-specific safe workflow.

### External

- Discovered on the system or registered by the user without Nerd ownership metadata.
- Read-only from Nerd's perspective.
- May be probed for version, reachability, and health.
- May be selected explicitly for project use where the feature supports it.
- Is never updated, stopped, reconfigured, backed up, adopted, or removed by Nerd.

Discovery sources may include executable lookup, read-only registry uninstall entries, Windows Service queries, known installation paths, and listening-port inspection. Discovery results are hints and must be verified before display or use.

External credentials and connection details are machine-local. Secrets use DPAPI or remain in the project's ignored environment files; they never enter `nerd.json`.

Ownership checks are mandatory before uninstall, repair, cleanup, process termination, certificate removal, NRPT removal, PATH mutation, or data deletion. Path location alone is not sufficient proof of ownership.

## IPC

- Transport: per-user Windows named pipe
- Pipe ACL: current user only
- Encoding: versioned structured messages
- Request/response commands plus server-pushed events
- Every request has request ID, protocol version, operation, and typed payload
- Unknown fields and operations fail closed
- Long tasks report progress events and support cancellation where safe
- Desktop and CLI perform protocol compatibility handshake before commands

The schema in `schemas/ipc.schema.json` is the contract. Rust and TypeScript types must be generated or checked against it; handwritten divergent copies are prohibited.

## DNS And Routing

### DNS

- Daemon binds UDP and TCP on `127.0.0.1:53`.
- It answers only the configured local TLD, fixed to `.test` for MVP.
- `*.test` resolves to `127.0.0.1`.
- Windows NRPT routes only `.test` queries to Nerd.
- Nerd does not forward public DNS and does not replace adapter DNS settings.
- Corporate policy conflicts become diagnostics; Nerd does not bypass policy.

### HTTP And HTTPS

- Daemon binds loopback ports 80 and 443.
- Routing is based on validated `Host`.
- A project gets a stable public local domain and a dynamic internal port.
- Proxy supports HTTP streaming, SSE, WebSocket, and framework HMR.
- HTTPS certificates are issued on demand by a per-user Nerd CA trusted during setup.
- Foreign listeners on 53, 80, or 443 are reported and never terminated automatically.

## Runtime Resolution

Nerd downloads official Node Windows ZIP archives and verifies official checksums. It prepends the selected runtime and package-manager shims only to the child process environment.

Resolution order:

```text
nerd.json -> .nvmrc -> .node-version -> engines.node -> default
```

Ambiguous ranges resolve to the newest installed compatible LTS by default. Downloads require explicit user-visible progress.

External Node runtimes may participate only after explicit registration. Their path and observed version are pinned in local state, they remain lower priority than an explicit managed-runtime selection, and Nerd marks them degraded if path or version changes.

## Project Lifecycle

```text
Stopped
  -> Resolving dependencies
  -> Starting services
  -> Starting application
  -> Waiting for readiness
  -> Running
  -> Stopping application
  -> Stopping on-demand services
  -> Stopped
```

Failures retain structured stage and cause. Processes run non-elevated inside Windows Job Objects. Stopping a project terminates the complete process tree.

## Services

- Binary cache shared by engine/version
- Process, data, credentials, and port scoped to project
- Loopback binding only
- Start before application
- Health check before app start continues
- Stop after application unless `keepRunning`
- Backups are explicit, versioned metadata records
- Deleting a project never deletes service data without separate confirmation

External database or Redis connections are connection references, not supervised services. Nerd may perform read-only reachability/health probes but does not own their process, configuration, backup, upgrade, credentials, or data.

## Mail

Each running project receives a local SMTP endpoint. Message metadata lives in SQLite; raw MIME and attachments live under the project mail directory. Retention limits apply by count, age, and total bytes.

## Request Inspector

Instrumentation occurs in the proxy, never through framework injection. Metadata capture uses bounded in-memory buffers. Bodies are opt-in, limited to 1 MB, restricted by content type, and redacted before storage or events.

## Security Invariants

- All network services bind loopback by default.
- Project commands never run elevated.
- Named pipe is restricted to current user.
- Helper operations are typed and allowlisted.
- Download origin and checksum are verified before extraction.
- Archive extraction rejects absolute paths and traversal.
- Proxy rejects malformed hosts and routing loops.
- Inspector redacts authorization, cookies, passwords, tokens, and API keys.
- Private keys and generated credentials use DPAPI.
- Uninstall removes only resources carrying Nerd ownership markers.

Threats, trust levels, project approval, and protected assets are defined in `trust-model.md`. Cross-feature decisions are recorded in `decisions/`; unresolved implementation choices are tracked in `open-decisions.md`.

## Recovery

- SQLite migrations run transactionally.
- Config and runtime downloads use temporary files followed by atomic rename.
- Daemon reconciles persisted lifecycle intent with live PIDs at startup.
- PID identity includes creation metadata, not PID alone.
- Safe repairs are explicit in `nerd doctor`; destructive repairs require confirmation.
- Installer records setup operations for symmetric rollback.

## Performance Invariants

- No filesystem polling.
- No Electron or bundled Chromium.
- No Node runtime inside daemon.
- Bounded queues, logs, inspector buffers, and mail storage.
- Services start on demand.
- GUI lists use pagination or virtualization for large collections.
