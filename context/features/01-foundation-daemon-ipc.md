# Feature 01: Foundation, Daemon, State, And IPC

## Goal

Create the stable control plane every other feature uses: workspace, daemon lifecycle, SQLite state, versioned named-pipe IPC, structured logging, and a minimal CLI.

## User Outcome

After installation or a development build, `nerd status` reports daemon identity, protocol version, uptime, health, and data paths. Daemon starts and stops without leaving locks or background tasks.

## In Scope

- Rust workspace and four initial crates
- Per-user daemon instance guard
- Graceful startup and shutdown
- SQLite migration runner and repository boundary
- Current-user-only named pipe
- Protocol handshake, request IDs, typed errors, and event envelope
- CLI daemon discovery and `nerd status`
- Structured rotating daemon logs
- Transactional schema versioning

## Out Of Scope

- DNS, proxy, Node, projects, services, and desktop UI
- Automatic daemon installation
- Remote IPC

## Required Decisions

- IPC schema lives in `schemas/ipc.schema.json`.
- CLI and desktop are clients; they never read SQLite directly.
- A second daemon instance exits with a typed already-running result.
- Incompatible protocol versions fail before any mutation.
- Long operations use operation IDs and progress event envelopes from the start.

## Implementation Blueprint

- `nerd-daemon` runs in the foreground. The CLI discovers it but does not start it.
- IPC uses bounded UTF-8 JSON frames prefixed by an unsigned 32-bit little-endian payload length.
- `schemas/ipc.schema.json` is canonical; strict Rust wire types are checked against it in contract tests, and outbound frames are rejected unless they round-trip through the strict wire deserializer.
- SQLite uses bundled `rusqlite` behind one dedicated worker and a bounded command channel.
- A protected `Global\\` named mutex enforces the one-active-daemon-per-machine MVP rule.
- The pipe DACL grants access only to the active user SID and LocalSystem, rejects remote clients, and protects the first instance.
- Daemon rejects elevated and LocalSystem tokens before filesystem or IPC mutation. Clients authenticate the pipe server's SID, elevation, and sibling executable path before sending requests.
- SQLite validates its application identity, migration ledger, and expected schema before serving repositories.
- One four-second deadline bounds the complete graceful-shutdown sequence.
- Console signal handlers are registered before the IPC endpoint starts serving.
- Daemon logs use JSON Lines, rotate at 10 MiB, retain five generations, and fall back to stderr plus `OutputDebugStringW`.
- Initial application version is `0.1.0-alpha.1`; initial IPC protocol version is `1`.

## Initial State Entities

- schema metadata
- global settings
- operation history needed for recovery
- installed artifact inventory shell
- project registry shell

Do not create speculative feature tables beyond what migration boundaries require.

## Failure Behavior

- Invalid request envelopes with a valid request ID receive a typed `invalid_request`; uncorrelatable malformed frames fail closed by disconnecting.
- Corrupt or unsupported database version prevents mutation and points to diagnostics/recovery.
- Pipe ACL failure aborts daemon startup.
- Log failure degrades to Windows event/debug output without crashing core operation.
- Shutdown deadline expiration records unfinished task identities before forced exit.

## Security

- Named pipe grants access only to current user and local system where required for setup coordination.
- Daily daemon operation rejects elevated and LocalSystem process tokens.
- Error payloads never include secrets or raw SQL.
- Database and log directories use per-user locations.

## Acceptance Criteria

- `nerd status` succeeds against a running daemon.
- CLI clearly distinguishes daemon absent, protocol mismatch, and daemon unhealthy.
- Concurrent commands retain correct request/response IDs.
- A second user session cannot access the pipe.
- An elevated daemon invocation is rejected before creating user state.
- Migrations roll back on injected failure.
- Daemon exits cleanly with no open child tasks.
- Idle resource baseline is measured and recorded.

## Dependencies

None. This feature must complete before all others.

Third-party implementation dependencies and exact versions are approved under Feature 01 in `../library-docs.md`.
