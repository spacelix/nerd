# Feature 04: Project Discovery And Configuration

## Goal

Register projects through Park Directory and Link Project, then resolve reproducible project configuration from `nerd.json` and local state.

## User Outcome

Parking `C:\Code` discovers immediate child projects with `package.json`; linking registers any individual project. Changes appear without daemon restart.

## In Scope

- Park, unpark, link, unlink, list, and project detail commands
- Initial immediate-child scan
- Native `ReadDirectoryChangesW` watcher with debounce
- Stable local project IDs
- DNS-safe project name derivation and explicit aliases
- Name conflict detection
- `nerd.json` schema v1, parser, validation, and effective-config merge
- Local overrides that never modify repository files silently
- Missing/deleted/renamed path reconciliation
- Untrusted/trusted project state bound to stable identity
- Unsupported-location detection before registration or execution

## Out Of Scope

- Framework execution, proxying, and project creation
- Recursive monorepo discovery
- Auto-starting discovered projects
- Executing package scripts or dependency installation during discovery
- UNC, mapped drive, OneDrive-controlled, removable, or `\\wsl$` project execution

## Discovery Rules

- Only immediate child directories of a parked root are candidates.
- Candidate becomes a project when root `package.json` exists.
- Do not follow directory links outside parked root during discovery.
- Event watcher triggers reconciliation, not direct state mutation.
- Empty parked roots remain registered.
- Discovery reads metadata only and never executes project code.
- New projects begin untrusted, including projects discovered after a filesystem event.
- Replacing directory/repository identity invalidates trust according to OD-010.
- Rename behavior must resolve OD-026 before implementation; stable project data must not be orphaned or rebound by path guesswork.
- Route storage must permit future explicit aliases per OD-027 without enabling implicit wildcard routing.

## `nerd.json` Rules

- Safe to commit.
- Unknown keys rejected with path-specific errors.
- Schema version required.
- Environment values may reference approved `${NERD_*}` placeholders.
- Secrets, private keys, generated credentials, and generated local ports are prohibited.
- Local override wins but remains visibly marked in UI/CLI.

## Conflict Rules

- Names are lowercase DNS labels.
- Duplicate names remain registered as conflicts but neither receives an ambiguous route.
- Linking under explicit unique name resolves a parked-name conflict without moving files.

## Acceptance Criteria

- New child with `package.json` appears after one filesystem event cycle.
- Removed child route disappears without deleting data.
- No idle polling activity.
- Duplicate names produce deterministic conflict state.
- Invalid `nerd.json` blocks project start but not daemon or other projects.
- Effective configuration reports value provenance.
- Unsupported locations fail before watcher, trust, or execution mutation.
- Discovery of a malicious `package.json` fixture executes no scripts.

## Dependencies

- Feature 01
- Feature 03 types for runtime selection fields
