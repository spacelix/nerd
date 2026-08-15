# Project Overview

## Product

Nerd is a lightweight, open-source local JavaScript development environment for Windows. It provides Herd/Yerd-style local domains, HTTPS, runtime management, process supervision, services, mail capture, and request inspection for Node.js projects without Docker, virtual machines, Electron, or a system-wide Node dependency.

Nerd is feature-equivalent in workflow, not a source, visual, naming, or branding copy of Laravel Herd or Yerd.

## Target User

JavaScript and TypeScript developers on Windows who want projects to run at stable local URLs without manually coordinating Node versions, ports, certificates, databases, and background processes.

## Platform

- Minimum OS: Windows 10
- Also supported: Windows 11
- MVP architecture: x86_64
- Native ARM64: post-MVP
- License: MIT
- Account, subscription, and telemetry: none
- Code versioning: Git with trunk-based development and immutable release tags
- Application versioning: SemVer 2.0.0; stable MVP targets `1.0.0`
- Commit convention: Conventional Commits 1.0.0

## Product Principles

1. Lightweight while idle. Daemon target is less than 20 MB RAM and near-zero idle CPU.
2. Native control plane. Nerd must keep working when Node is absent, broken, or being replaced.
3. Rootless daily use. Elevation is limited to typed setup and uninstall operations.
4. Reproducible projects. Team-safe configuration lives in `nerd.json`; secrets do not.
5. Explicit lifecycle. Projects start and stop on command, with optional per-project autostart.
6. Safe defaults. Bind locally, redact secrets, verify downloads, and never take over foreign ports.
7. One control plane. Desktop and CLI use the same daemon and IPC contract.
8. Coexistence. Existing developer tools remain untouched unless the user explicitly registers one for read-only external use.
9. Explicit trust. Discovery may read metadata, but no project or dependency script runs before Trust and Start approval.

## Core User Flow

1. Install Nerd and complete one-time setup for `.test` DNS, local CA trust, CLI, and daemon autostart.
2. Park a parent directory, link an existing project, or create a new project.
3. Nerd detects framework, Node version, package manager, and `nerd.json`.
4. User starts the project.
5. Nerd installs missing runtime/service binaries after confirmation, starts project services, runs the development script, waits for readiness, and enables the local route.
6. User opens `https://project.test` and inspects logs, HTTP requests, or captured mail from GUI or CLI.
7. User stops the project; project process and on-demand services stop cleanly.

## MVP Features

Feature specifications live in `context/features/` and are the source of truth for feature-level behavior.

1. Foundation, daemon, state, and IPC
2. Windows DNS, HTTPS, and privilege setup
3. Node runtime and package manager management
4. Parked and linked project discovery plus `nerd.json`
5. Framework detection and process supervision
6. HTTP/HTTPS reverse proxy
7. Desktop app, system tray, and CLI
8. Request Inspector
9. Mail capture
10. MySQL, PostgreSQL, and Redis
11. Project creation wizard and CLI
12. Diagnostics, installer, updates, and recovery

## Frameworks

MVP adapters:

- Next.js
- Vite, including React, Vue, Svelte, and vanilla projects
- Nuxt
- Astro
- NestJS
- Express and custom servers through the `PORT` environment variable

Users may override command, working directory, readiness path, and port strategy.

## Runtime And Tooling

- Node.js only for MVP
- Multiple installed versions
- Per-project version resolution
- npm from the Node distribution
- pnpm and Yarn through isolated package-manager tooling
- Corepack where available; separately managed Corepack for Node 25 and newer
- No mutation of the user's global Node installation

Node version precedence:

```text
nerd.json
-> .nvmrc
-> .node-version
-> package.json engines.node
-> Nerd default
```

## Project Registration

### Park Directory

Every immediate child containing `package.json` becomes a project. Nerd uses native filesystem events, not polling.

### Link Project

One directory is registered under an explicit or derived project name.

### Project Manifest

`nerd.json` is safe to commit and may define:

- Project name
- Node version
- Development script
- HTTPS
- Framework override
- Service versions
- Environment placeholders
- Inspector settings

It must never contain generated credentials, local ports, access tokens, or private keys.

## Managed Services

- MySQL
- PostgreSQL
- Redis

These are desired MVP services, not yet approved artifact claims. Feature 10 is blocked by ADR 006 until MySQL/PostgreSQL sources are approved and the native Windows Redis strategy is resolved.

Service binaries are shared by engine/version. Process, port, credentials, and data remain isolated per project. Services start on demand with the project and stop with it unless `keepRunning` is enabled.

## Existing Software And Ownership

Nerd must coexist with Node.js, MySQL, PostgreSQL, Redis, web servers, and other developer tools already installed on the machine.

Every discovered or configured resource has one ownership class:

- **Managed**: installed or created by Nerd and carrying verified ownership metadata. Nerd may start, stop, update, back up, repair, or remove it within feature rules.
- **External**: installed or created outside Nerd. Nerd may detect, display, probe, or use it only after explicit user choice. Nerd never updates, repairs, stops, reconfigures, adopts, or removes it.

Rules:

- Discovery is read-only and never changes PATH, Windows Services, registry entries, data directories, or configuration files.
- Existing Node may be registered as an external runtime, but Nerd-managed Node remains recommended for reproducibility.
- Existing databases may be registered as external connections. Credentials stay in local DPAPI-protected storage or the project's own ignored `.env`, never `nerd.json`.
- Nerd-managed services use separate processes, dynamic ports, credentials, and data directories, even when the same engine already exists globally.
- A missing or changed external resource becomes `degraded`; Nerd offers a managed alternative but never repairs the external installation.
- Uninstall removes only verified Nerd-owned resources. External resources and data always remain untouched.

## Observability

### Logs

Capture project stdout/stderr with bounded buffers and rotating persistence.

### Request Inspector

Capture method, URL, status, duration, headers, and query parameters. Body capture is optional, size-limited, content-type-aware, and redacted.

### Mail

Run a project-scoped local SMTP sink. Captured messages never leave the machine.

## Performance Budgets

| Metric | MVP budget |
|---|---:|
| Daemon idle RAM | less than 20 MB |
| Desktop idle RAM | less than 80 MB |
| Idle CPU | near 0% |
| Initial installer | less than 30 MB |
| Default inspector buffer | 500 requests per project |
| Captured body limit | 1 MB per body |

Downloaded Node and service binaries are excluded from installer size.

## Out Of Scope

These are outside MVP. Approved candidates and release horizons are documented in `roadmap.md`.

- Bun and Deno
- macOS and Linux
- Native ARM64 release
- Docker or WSL orchestration
- Public tunnels and LAN sharing
- MCP and AI-agent integration
- Cloud accounts, sync, teams, billing, or licensing servers
- Production deployment

## Success Criteria

- Clean Windows 10 user reaches a trusted `https://app.test` project without manually editing hosts or certificates.
- Multiple projects can use different Node and service versions without global conflicts.
- Stopping a project terminates its complete process tree and on-demand services.
- GUI can close while daemon and started projects continue running.
- Uninstall restores Nerd-owned NRPT, CA, autostart, and PATH changes without touching unrelated settings.
- Performance budgets pass on a clean Windows 10 x64 test machine.
- New and materially changed projects cannot execute before trust preflight.
