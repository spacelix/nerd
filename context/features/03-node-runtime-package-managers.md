# Feature 03: Node Runtime And Package Managers

## Goal

Install, resolve, and execute multiple Node.js versions without depending on or mutating system Node.

## User Outcome

Users install Node versions, assign one per project, and run npm, pnpm, or Yarn with deterministic project tooling.

## Decisions

- **D1 Persistence**: runtime inventory lives in SQLite (`runtimes` table via schema migration). New repository methods on `StateClient`: list/register/remove/set-health.
- **D2 Download**: `reqwest =0.13.4` + `zip =8.6.0` + `sha2 =0.11.0`. Official Node origins only. Verify SHASUMS256 checksum before extraction. Reject archive traversal. Extract to a staging directory, then atomic rename to the target version directory. Failed downloads never enter inventory.
- **D3 Resolution**: `version.rs` parses and resolves declarations to one concrete version with a source trace (`nerd.json | nvmrc | node-version | engines | default`). Prefer an installed compatible LTS before downloading.
- **D4 Package managers**: `node.rs` (install/resolve) plus `package_manager.rs` (npm/corepack/pnpm/yarn). npm ships with the Node distribution. pnpm/Yarn resolve through `packageManager` and run via Corepack. Node 25+ uses Nerd-managed Corepack. Child env is built by `exec.rs` with an isolated PATH.
- **D5 Re-probe gating**: the daemon re-probes external runtimes before project launch (authoritative). Missing path, changed binary identity, incompatible architecture, or changed version marks the runtime `degraded`, blocks launch with a typed error, and offers a managed alternative.

## In Scope

- Official Node release index client and cache
- Windows x64 ZIP download and checksum verification
- Atomic install/uninstall and inventory
- Exact, major, LTS, and compatible range resolution
- Resolution from `nerd.json`, `.nvmrc`, `.node-version`, `engines.node`, default
- Isolated child `PATH`
- npm from Node distribution
- pnpm and Yarn through project `packageManager`
- Corepack handling, including separate Corepack for Node 25+
- Read-only discovery of existing Node installations
- Explicit registration and removal of external Node references
- `Managed`, `External`, and `Degraded` runtime status
- CLI install/list/remove/default/use commands

## Out Of Scope

- Bun, Deno, native addons toolchain installation, and global npm package management
- Modifying system `PATH` for each Node version
- Replacing nvm/fnm outside Nerd-managed processes
- Updating, repairing, uninstalling, or adopting external Node installations

## Resolution Rules

- Explicit `nerd.json` wins.
- Invalid or conflicting declarations stop startup with source-specific error.
- Prefer installed compatible LTS before downloading.
- Download requires visible user consent/progress unless project setup explicitly approved it.
- Never silently upgrade a pinned exact version.

## Download Security

- Allow official Node artifact origins only.
- Verify checksum before extraction.
- Reject archive traversal.
- Promote temporary directory atomically.
- Keep failed downloads out of installed inventory.

## Package Manager Rules

- Respect `packageManager` exact version when present.
- Do not rewrite `package.json` without explicit action.
- Cache package-manager artifacts under Nerd data.
- Never use globally installed pnpm or Yarn implicitly.

## Existing Node Rules

- Discovery uses read-only executable lookup, version probes, registry hints, and known install paths.
- Discovery never changes global PATH or runs package installation.
- External Node is used only after explicit user selection.
- Persist canonical executable path, observed version, architecture, and external ownership class in local state.
- Re-probe before project launch. Missing path, changed binary identity, incompatible architecture, or changed version marks runtime `degraded` and blocks launch.
- Removing an external runtime from Nerd removes only its local reference.
- Nerd may offer to install an equivalent managed version but never copies, repairs, updates, or uninstalls the external runtime.

## Acceptance Criteria

- Two projects run different Node versions concurrently.
- Child process sees selected Node first; parent/global environment remains unchanged.
- Tampered archive is rejected.
- Interrupted installation resumes safely or restarts cleanly.
- Node 24 and Node 25+ package-manager workflows both pass.
- Version-source diagnostics identify which file selected Node.
- Existing system Node remains unchanged after discovery, registration, project execution, reference removal, and Nerd uninstall.
- Missing or changed external Node produces degraded status and managed-runtime remediation.

## Dependencies

- Feature 01
