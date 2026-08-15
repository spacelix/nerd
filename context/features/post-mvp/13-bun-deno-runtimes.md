# Feature 13: Bun And Deno Runtimes

## Horizon

Post-MVP v1.x candidate.

## Goal

Extend Nerd runtime management beyond Node.js while preserving one project lifecycle, proxy, logs, services, and UI model.

## User Outcome

Projects can select Node, Bun, or Deno in `nerd.json`; Nerd installs the selected runtime, detects compatible projects, and starts them through the existing supervisor.

## In Scope

- Verified Windows x64 Bun and Deno downloads
- Runtime inventory and version pinning
- `runtime` field evolution in `nerd.json`
- Bun package-manager support
- Deno task support
- Framework adapter capability declarations
- Runtime-aware create-project choices where official scaffolds support them

## Out Of Scope

- Automatic source conversion between runtimes
- Pretending every Node framework supports Bun or Deno
- Global PATH replacement
- Native ARM64 artifacts until Feature 14

## Architecture Rules

- Generalize runtime traits only when this feature begins; do not pre-abstract MVP Node code speculatively.
- Runtime adapter owns release metadata, install layout, executable resolution, and task invocation.
- Framework adapter explicitly states supported runtimes.
- Existing Node behavior and manifest migration remain backward compatible.

## Open Decisions

- Whether Bun acts as runtime, package manager, or both per project.
- Deno permission policy and how explicit permissions appear in `nerd.json`.
- Runtime-specific environment and lockfile precedence.

## Acceptance Criteria

- Bun and Deno smoke projects run through the same `.test`, HTTPS, logging, and stop lifecycle.
- Unsupported framework/runtime pair fails before process launch.
- Existing Node projects require no manifest changes.
- Downloads and updates pass the same integrity rules as Node.
- Runtime choice is visible in GUI, CLI, status, and diagnostics.

## Dependencies

- MVP Features 03-07, 11, and 12
