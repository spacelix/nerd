# Feature 18: macOS And Linux Support

## Horizon

Platform Expansion v2.x candidate.

## Goal

Port Nerd's control plane and desktop experience to macOS and Linux while preserving platform-neutral project behavior and platform-specific safety boundaries.

## User Outcome

Node projects use the same `nerd.json`, CLI concepts, desktop workflows, services, mail, and inspector across supported operating systems.

## In Scope

- Platform abstraction audit and extraction
- macOS Apple Silicon first, Intel policy decided during planning
- Selected Linux distributions and package formats
- Per-platform DNS resolver setup
- Per-platform CA trust
- Privileged-port strategy
- Autostart and privilege helper model
- Unix process-group supervision
- Artifact matrices for runtimes and services
- Platform CI, installers, updates, rollback, and uninstall

## Out Of Scope

- Identical low-level implementation across operating systems
- Supporting every Linux distribution initially
- Weakening Windows behavior to fit a lowest common denominator
- Docker as portability layer

## Architecture Rules

- Shared domain decisions remain in platform-neutral modules.
- Side effects live behind narrow platform traits.
- Each platform has symmetric setup/probe/remove implementations.
- Build-time platform selection is preferred over scattered runtime conditionals.
- `nerd.json` remains portable; local overrides carry platform-specific values.

## Open Decisions

- First supported Linux distributions and init/resolver combinations.
- macOS privileged-port redirect strategy.
- Desktop packaging and code-signing requirements per platform.
- Service engines with equivalent native artifact support.

## Acceptance Criteria

- Same repository manifest works on Windows and each supported new platform.
- DNS, HTTPS, process cleanup, services, update, and uninstall pass platform-specific tests.
- Platform-specific code does not leak through desktop or CLI contracts.
- Unsupported host configuration produces diagnostics rather than unsafe fallback.
- Resource budgets are defined and measured per platform.

## Dependencies

- Stable MVP architecture and Feature 12 release pipeline
- Features 13-17 are not mandatory unless explicitly selected during v2 planning
