# Progress Tracker

Update this file whenever feature state changes. Only one feature may be `in progress` unless the work is explicitly split into independent tracks.

## Current Status

- Phase: Phase 3 - Runtimes and Package Managers
- In progress: Feature 03 - Node runtime and package managers
- Last completed: Feature 02 - Windows DNS, HTTPS, and privilege setup
- Next: Complete Feature 03 acceptance, then start Feature 04

## Features

| ID | Feature | Specification | Status |
|---|---|---|---|
| 01 | Foundation, daemon, state, IPC | `features/01-foundation-daemon-ipc.md` | complete |
| 02 | Windows DNS, HTTPS, privilege setup | `features/02-windows-dns-https.md` | complete |
| 03 | Node runtime and package managers | `features/03-node-runtime-package-managers.md` | in progress |
| 04 | Project discovery and configuration | `features/04-project-discovery-config.md` | planned |
| 05 | Frameworks and process supervision | `features/05-frameworks-process-supervision.md` | planned |
| 06 | Reverse proxy | `features/06-reverse-proxy.md` | planned |
| 07 | Desktop, tray, CLI | `features/07-desktop-tray-cli.md` | planned |
| 08 | Request Inspector | `features/08-request-inspector.md` | planned |
| 09 | Mail capture | `features/09-mail-capture.md` | planned |
| 10 | Managed services | `features/10-managed-services.md` | blocked |
| 11 | Project creation | `features/11-project-creation.md` | planned |
| 12 | Diagnostics, installer, updates, recovery | `features/12-release-diagnostics.md` | planned |

Allowed status values: `planned`, `in progress`, `blocked`, `complete`.

## Decisions

- Windows 10 minimum, x64 MVP.
- Rust daemon/CLI/helper and Tauri 2 + React/TypeScript desktop.
- Node.js only for MVP.
- Park Directory required in MVP.
- Wildcard `.test` DNS through built-in responder and Windows NRPT.
- Project services are isolated and on demand.
- Existing runtimes and services are `External`, read-only, and never adopted or removed; only verified `Managed` resources may be mutated.
- MIT license, no account, subscription, or telemetry.
- Stable MVP targets `1.0.0`; independent IPC, manifest, and database versions follow `versioning.md`.
- Source code uses trunk-based Git development; application releases use immutable `vX.Y.Z` tags.
- All commits use Conventional Commits 1.0.0.

## Blockers

Feature 01 and 02 are complete with no open decision blockers.

Deferred to release testing: end-to-end UDP/TCP `.test` resolution and after-resume resolution on a clean image without WSL2/`hns` on port 53.

Known upcoming blockers:

- Feature 10: OD-002 MySQL artifacts, OD-003 PostgreSQL artifacts, and OD-004 Redis strategy.
- Public beta: OD-001 final product identity and OD-005 code-signing custody.

Full register: `open-decisions.md`.

## Post-MVP Roadmap

Post-MVP candidates are tracked separately in `roadmap.md`. They are not `planned` MVP work and must not be marked active here until formally promoted.

## Notes

- Feature-level decisions belong in the matching file under `context/features/`.
- `context/features/README.md` defines specification maintenance rules.
