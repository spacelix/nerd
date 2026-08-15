# Nerd Agent Guide

Nerd is a lightweight Windows local JavaScript development environment. It is not a Next.js web application. Core is Rust; desktop is Tauri 2 with React and TypeScript.

## Development Command Environment

The repository is physically stored on Windows at `D:\Source\nerd` and exposed to the agent through WSL at `/mnt/d/Source/nerd`.

- WSL tools may read, search, and edit repository files.
- All build, test, package-manager, Tauri, installer, and Windows API commands must run as native Windows processes through `powershell.exe` or `cmd.exe`.
- Use Windows executables such as `cargo.exe`, `rustup.exe`, `node.exe`, `npm.cmd`, `pnpm.cmd`, and `yarn.cmd` where applicable.
- Never use Linux Rust or Node toolchains to produce project artifacts.
- Never create Linux `target/`, `node_modules/`, native addons, binaries, or lockfile changes for this Windows target.
- Pass Windows paths to Windows tools. Convert `/mnt/d/Source/nerd` to `D:\Source\nerd` when a command does not inherit the working directory correctly.
- Run Windows commands with `-NoProfile` where possible so user PowerShell profiles cannot change behavior.
- Do not elevate commands or trigger UAC silently. Privileged setup and tests require explicit user approval and safe fixtures.
- Before relying on a Windows tool, verify its executable and version from the same Windows process environment used for the command.

Example:

```bash
powershell.exe -NoProfile -Command 'cargo.exe test --workspace'
```

## Read Before Implementation

Read in this exact order:

1. `context/project-overview.md`
2. `context/architecture.md`
3. `context/trust-model.md`
4. `context/versioning.md`
5. `context/code-standards.md`
6. `context/library-docs.md`
7. `context/compatibility.md`
8. `context/open-decisions.md`
9. `context/build-plan.md`
10. `context/progress-tracker.md`
11. Active feature specification under `context/features/`
12. ADRs directly referenced by active feature under `context/decisions/`

For desktop UI work, also read before editing:

1. `context/ui-tokens.md`
2. `context/ui-rules.md`
3. `context/ui-registry.md`

Do not read every feature specification by default. Read active feature plus directly named dependencies. Use `context/build-plan.md` and `context/progress-tracker.md` to find them.

Do not read every ADR by default. Read `context/decisions/README.md`, then only ADRs referenced by active feature, architecture, or open blocker.

Post-MVP work lives in `context/roadmap.md` and `context/features/post-mvp/`. Do not read or implement it during MVP work unless the user explicitly promotes or asks about a roadmap feature.

## Feature Workflow

1. Confirm active feature in `context/progress-tracker.md`.
2. Read its complete file under `context/features/`.
3. Load `/architect` before changing feature architecture or scope.
4. Verify current installed-library documentation before code.
5. Implement only in-scope behavior.
6. Run feature acceptance tests and relevant global checks.
7. Load `/review` before marking feature complete.
8. Update active feature spec if implementation changed an agreed decision.
9. Update `context/progress-tracker.md` after status changes.
10. Update `context/ui-registry.md` after reusable UI additions, using `/imprint`.

## Invariants

- Windows 10 x64 is minimum MVP platform.
- Daemon, CLI, and helper are Rust. Desktop uses Tauri 2, React, and strict TypeScript.
- No Electron, Docker, VM, bundled Chromium, or Node dependency for Nerd itself.
- Daily operation is non-elevated. Project and service processes never run elevated.
- Elevated helper accepts typed allowlisted operations only, never arbitrary commands.
- Desktop and CLI use daemon IPC. They never read SQLite or manage processes directly.
- Bind local services to loopback unless feature specification explicitly says otherwise.
- Never kill, modify, or remove foreign processes or system resources automatically.
- Resources are either `Managed` or `External`. Nerd mutates, updates, backs up, and removes only resources carrying verified Nerd ownership metadata.
- Verify downloaded artifacts before extraction and reject archive traversal.
- Secrets use DPAPI and never enter logs, IPC errors, `nerd.json`, or UI events.
- No telemetry, account, subscription, or remote diagnostic upload.
- No filesystem polling. Use Windows native change notifications.
- Respect performance budgets in `project-overview.md`.
- No hardcoded colors or raw Tailwind palette classes in components.
- Product releases follow SemVer; IPC, `nerd.json`, and SQLite use independent versions defined in `context/versioning.md`.
- Every commit message follows Conventional Commits 1.0.0.
- Newly discovered project code is untrusted and must not execute before Trust and Start preflight defined in `context/trust-model.md`.
- Blocking items in `context/open-decisions.md` must be resolved before affected feature implementation.

## Dependency Rules

Before adding any third-party crate or npm package:

1. Load matching installed skill when available.
2. Read current official documentation for exact version.
3. Read `context/library-docs.md`.
4. Confirm existing dependency or standard library is insufficient.
5. Record approved dependency and project-specific constraints in `library-docs.md`.

Do not rely on training knowledge for Tauri, React, Rust crates, Windows APIs, Node distribution, or service binary behavior when current docs are available.

## Git Rules

- Use trunk-based development with `main` as the single long-lived, releasable branch.
- Use short-lived branch names defined in `context/versioning.md`.
- Use Conventional Commits for every commit.
- Keep one logical change per commit.
- Use preferred scopes from `context/versioning.md`.
- Mark breaking changes with `!` and a `BREAKING CHANGE:` footer.
- Application releases use immutable annotated, preferably signed `vX.Y.Z` tags.
- Desktop, daemon, CLI, helper, installer, and updater share one application version.
- Do not commit generated credentials, private keys, local databases, runtime binaries, service data, or captured traffic/mail.

## Failure And Recovery

- After one failed corrective attempt for same root problem, stop and load `/recover`.
- Never use destructive Git or filesystem commands to recover unrelated work.
- Preserve user data by default.
- Repairs probe first, mutate second, and verify postconditions.
- Every privileged setup action needs symmetric uninstall and rollback behavior.

## Available Skills

Skills are workflow gates. Load them at the stated trigger, not after work is already complete.

- `/architect`: before any complex feature, architecture change, or scope change. Think through decisions before code.
- `/imprint`: after adding any reusable UI component. Capture established visual and interaction patterns in `context/ui-registry.md`.
- `/review`: before marking any feature complete, before a demo, or when implementation may have drifted from its specification.
- `/recover`: after one failed corrective attempt for the same root problem. Diagnose before trying another fix.
- `/remember save`: when a feature spans sessions. Preserve decisions, progress, blockers, and exact next step.
- `/remember restore`: when returning to multi-session work. Restore saved context before reading or changing implementation.
