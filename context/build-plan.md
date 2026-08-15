# Build Plan

## Delivery Rule

Build one feature file at a time. A feature is complete only when implementation, tests, documentation, resource impact, and acceptance criteria all pass. Backend-only phases must expose CLI or diagnostic output so work remains testable before GUI exists.

## Phase 0: Context And Repository

1. Establish context documents and feature specifications.
2. Create Rust workspace and desktop package skeleton.
3. Add formatting, linting, test, license, and dependency-policy CI.

## Phase 1: Control Plane

### Feature 01: Foundation, Daemon, State, And IPC

- Daemon lifecycle and graceful shutdown
- SQLite migrations and repositories
- Versioned named-pipe protocol
- CLI `status`
- Structured logs

## Phase 2: Windows Network Foundation

### Feature 02: Windows DNS, HTTPS, And Privilege Setup

- Complete OD-006 compatibility spike before production implementation
- Typed elevated helper
- NRPT `.test` rule
- DNS UDP/TCP responder
- Local CA and trust
- Ports 80/443
- Setup rollback

## Phase 3: JavaScript Runtime

### Feature 03: Node Runtime And Package Managers

- Release metadata and verified downloads
- Multi-version Node inventory
- Version resolution
- npm, pnpm, Yarn, and Corepack handling
- Isolated child environment

## Phase 4: Projects

### Feature 04: Project Discovery And Configuration

- Park/unpark
- Link/unlink
- Native watcher
- `nerd.json` schema and merge

### Feature 05: Frameworks And Process Supervision

- Framework adapters
- Port allocation
- Job Objects
- Readiness and crash states
- Logs

### Feature 06: Reverse Proxy

- Host routing
- HTTP/HTTPS
- Streaming, SSE, WebSocket, HMR
- Stopped and failed project responses

## Phase 5: Product Interface

### Feature 07: Desktop, Tray, And CLI

- Tauri shell and tray
- Project and runtime screens
- CLI parity
- Keyboard and accessibility paths

## Phase 6: Developer Observability

### Feature 08: Request Inspector

- Metadata capture
- Redaction
- Optional bounded bodies
- Live UI and CLI

### Feature 09: Mail Capture

- Project SMTP endpoint
- MIME storage and retention
- Safe mail viewer

## Phase 7: Data Services

### Feature 10: Managed Services

- Resolve ADR 006 and OD-002 through OD-004 before adapter implementation
- Shared binary cache
- Per-project MySQL, PostgreSQL, Redis
- Credentials, health, backup, restore

## Phase 8: Reproducible Creation

### Feature 11: Project Creation

- Official scaffolding adapters
- Desktop wizard and `nerd create`
- `nerd.json` generation
- Automatic registration and first start

## Phase 9: Release Readiness

### Feature 12: Diagnostics, Installer, Updates, And Recovery

- `nerd doctor`
- Safe repair operations
- Signed installer and updates
- Clean uninstall and rollback
- Windows 10/11 and performance gates

## Global Release Gates

- No critical or high security findings.
- Daemon idle RAM below 20 MB.
- Desktop idle RAM below 80 MB.
- Near-zero daemon idle CPU.
- Initial installer below 30 MB.
- Clean Windows 10 x64 install and uninstall pass.
- All downloaded artifacts verify integrity.
- No unrelated DNS, certificate, PATH, or process mutation.

## Post-MVP

Post-MVP features are intentionally excluded from this active build sequence. See `roadmap.md` and individual specifications under `features/post-mvp/`. Promote a candidate into this build plan only through the workflow defined in `roadmap.md`.
