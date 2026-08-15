# Feature 14: Native Windows ARM64

## Horizon

Post-MVP v1.x candidate.

## Goal

Provide native ARM64 Nerd binaries and managed artifacts on supported Windows on ARM devices.

## User Outcome

ARM64 users install a native Nerd build without x64 emulation and receive compatible Node, service, and framework tooling.

## In Scope

- ARM64 Rust and Tauri release targets
- Signed ARM64 installer and updater channel artifacts
- Architecture-aware artifact inventory
- ARM64 Node downloads
- Capability reporting for service engines
- ARM64 CI, smoke, performance, update, and uninstall tests

## Out Of Scope

- Emulating unavailable third-party service binaries inside Nerd
- Cross-architecture data migration without engine support
- Windows versions below current product minimum

## Architecture Rules

- Architecture is part of every downloaded-artifact identity.
- Never install x64 artifact silently when native ARM64 was requested.
- Explicit emulation fallback may be offered only when Windows supports it and user confirms.
- Service UI reports unavailable architecture instead of hiding the engine.

## Open Decisions

- Which managed services publish acceptable ARM64 Windows binaries.
- Whether mixed native/emulated service processes meet performance and support goals.
- Minimum hardware and Windows build for release testing.

## Acceptance Criteria

- Native daemon, CLI, helper, and desktop run on ARM64.
- Node ARM64 projects pass framework smoke tests.
- Artifact resolver never crosses architecture accidentally.
- Installer/update/rollback/uninstall pass on physical or trusted ARM64 CI hardware.
- Resource budgets are measured separately for ARM64.

## Dependencies

- MVP Feature 12 and all artifact-producing MVP features
