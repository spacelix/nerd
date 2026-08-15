# Feature 07: Desktop, Tray, And CLI

## Goal

Provide fast desktop and terminal interfaces over the same daemon contract.

## User Outcome

Users manage projects, runtimes, services, logs, and settings through a compact Tauri desktop app or CLI with equivalent core capabilities.

## In Scope

- Tauri 2 desktop shell
- React + strict TypeScript UI
- System tray status and project quick actions
- Overview, Projects, Project Detail, Runtimes, Services, Diagnostics, Settings shells
- Live IPC subscriptions and reconnect behavior
- CLI parity for non-visual operations
- Copy/open/reveal actions through typed Tauri commands
- Light, dark, and system themes
- Keyboard navigation and accessibility
- Ownership badges and filters for `Managed`, `External`, and `Degraded` resources
- First-run onboarding: system check, DNS/HTTPS setup, external-tool discovery, default Node, park directory, first project
- Disk usage and safe cleanup screen for Nerd-owned caches and retained data

## Out Of Scope

- Mail and Inspector detail UI until their feature implementation
- Mobile/responsive web app
- Independent desktop persistence of daemon state

## UI State Rules

- Initial view loads one daemon snapshot.
- Events update snapshot-derived state.
- Reconnect always reconciles with a fresh snapshot.
- Optimistic UI is limited to reversible visual intent; daemon response remains authoritative.
- Closing window does not stop daemon or projects.

## Tray Rules

- Show daemon health and count of running projects.
- Start/stop recent projects.
- Open app, diagnostics, and quit GUI.
- Explicit separate action required to stop daemon.

## CLI Rules

- Human output by default and stable `--json` where automation matters.
- Exit codes distinguish usage, daemon unavailable, validation, conflict, and operation failure.
- Long operations show progress in TTY and structured events in JSON mode.

## Acceptance Criteria

- GUI reconnects after daemon restart without stale state.
- Core project/runtime actions have CLI equivalents.
- Tray remains useful with window closed.
- Critical flows pass keyboard-only and screen-reader-label checks.
- Desktop idle RAM remains below budget.
- No direct database, registry, or process calls from React.
- External resources are visually distinct and never expose managed-only mutation actions.
- Destructive actions distinguish unlink, remove service, delete data, delete backup, and uninstall.
- Onboarding never selects or mutates an external tool without explicit opt-in.

## Dependencies

- Features 01 through 06
