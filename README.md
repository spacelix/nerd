# Nerd

[![CI](https://github.com/spacelix/nerd/actions/workflows/ci.yml/badge.svg)](https://github.com/spacelix/nerd/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Nerd is a lightweight, open-source local JavaScript development environment for Windows. The goal is to provide stable `.test` domains, local HTTPS, isolated Node.js runtimes, project supervision, local services, mail capture, and request inspection without Docker, virtual machines, Electron, or a system-wide Node.js dependency.

> [!IMPORTANT]
> Nerd is in early development. There is no installer or desktop application yet. Phase 1 provides the native control plane, daemon, state store, IPC protocol, and a minimal `nerd status` command; it is not yet a usable replacement for an existing local development environment.

## Current Status

Phase 1, the control-plane foundation, is complete.

Available now:

- Native Rust workspace for the daemon, CLI, core protocol, and helper boundary
- Foreground, non-elevated Windows daemon with graceful shutdown
- Daemon-owned SQLite state with transactional migrations
- Current-user Windows named-pipe IPC with a versioned JSON contract
- Structured rotating JSON Lines logs
- Minimal `nerd status` CLI command
- Windows CI, dependency policy, contract tests, and security-focused smoke tests

Not available yet:

- `.test` DNS and trusted local HTTPS
- Node.js and package-manager installation or selection
- Project discovery, start/stop, process supervision, and reverse proxying
- Desktop application and system tray
- Managed databases, request inspection, mail capture, installer, or updater

See [`context/progress-tracker.md`](context/progress-tracker.md) for the current implementation status and [`context/build-plan.md`](context/build-plan.md) for the delivery sequence.

## Design Goals

- **Lightweight:** less than 20 MiB daemon idle RAM and near-zero idle CPU
- **Native control plane:** Nerd continues to work when Node.js is absent or broken
- **Rootless daily use:** elevation is restricted to typed setup and uninstall operations
- **Safe coexistence:** foreign runtimes, processes, ports, certificates, and services are never silently modified
- **Explicit trust:** newly discovered project code cannot execute before user approval
- **Reproducible projects:** team-safe configuration belongs in `nerd.json`; secrets do not
- **Local by default:** no account, subscription, telemetry, or remote diagnostics

## Architecture

```text
Desktop (planned)                 nerd CLI
       |                             |
       +------ Windows named pipe ---+
                      |
                nerd-daemon.exe
                      |
        state, network, runtimes, projects

nerd-daemon.exe --typed request--> nerd-helper.exe --UAC--> Windows setup
```

The daemon owns mutable application state and runtime processes. CLI and desktop clients communicate through the same versioned IPC contract and never access SQLite directly. The elevated helper accepts typed, allowlisted setup operations only.

More detail is available in [`context/architecture.md`](context/architecture.md) and [`context/trust-model.md`](context/trust-model.md).

## Development Requirements

- Windows 10 or Windows 11, x64
- Rust `1.97.0` with the `x86_64-pc-windows-msvc` target
- Visual Studio Build Tools with the MSVC C++ toolchain
- PowerShell or Command Prompt

The repository may be edited from WSL, but all Rust, package-manager, Tauri, installer, and Windows API build or test commands must run as native Windows processes. Do not create Linux `target` or `node_modules` artifacts for this workspace.

## Build

From a native Windows terminal:

```powershell
cargo build --locked --workspace
```

For optimized binaries:

```powershell
cargo build --locked --workspace --release
```

The binaries are written under `target\debug` or `target\release`:

- `nerd-daemon.exe`
- `nerd.exe`
- `nerd-helper.exe`

## Try The Control Plane

Start the daemon in one non-elevated Windows terminal:

```powershell
cargo run --locked -p nerd-daemon
```

Query it from another terminal:

```powershell
cargo run --locked -p nerd-cli -- status
```

Stop the daemon with `Ctrl+C` or `Ctrl+Break`.

## Verification

Run the standard checks from a native Windows terminal:

```powershell
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace --all-targets
cargo deny check advisories bans licenses sources
```

The Feature 01 process smoke test requires a release build and a clean `%LOCALAPPDATA%\Nerd` fixture location. It refuses to modify an existing Nerd data directory:

```powershell
powershell.exe -NoProfile -File .\tests\windows\feature01-smoke.ps1 -Configuration release
```

Privileged and temporary-user test flags intentionally require explicit approval. See [`context/baselines/feature-01.md`](context/baselines/feature-01.md) for the latest recorded measurements and acceptance evidence.

## Roadmap

| Phase | Capability | Status |
|---|---|---|
| 1 | Control plane: daemon, state, IPC, CLI status | Complete |
| 2 | Windows DNS, HTTPS, and privilege setup | Planned, compatibility decisions required |
| 3 | Node.js runtimes and package managers | Planned |
| 4 | Projects, process supervision, and reverse proxy | Planned |
| 5 | Desktop, tray, and expanded CLI | Planned |
| 6-9 | Observability, services, project creation, and release readiness | Planned |

The complete MVP and post-MVP scope is documented under [`context/features/`](context/features/) and in [`context/roadmap.md`](context/roadmap.md).

## Contributing

Read [`AGENTS.md`](AGENTS.md) and the relevant files under [`context/`](context/) before changing implementation.

Nerd uses trunk-based development:

1. Create a short-lived branch from current `main`.
2. Keep each commit focused and use [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).
3. Push the work branch and open a pull request.
4. Merge only after the required `rust` and `dependencies` checks pass and review conversations are resolved.

Direct post-bootstrap pushes to `main` are prohibited. See [`context/versioning.md`](context/versioning.md) for the complete policy.

## License

Nerd is available under the [MIT License](LICENSE).
