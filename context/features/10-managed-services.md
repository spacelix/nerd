# Feature 10: Managed Services

## Goal

Provide isolated, on-demand MySQL, PostgreSQL, and Redis instances for each project while sharing downloaded binaries by engine/version.

Feature implementation is blocked by ADR 006 and OD-002 through OD-004. Redis must not be silently replaced by a compatible but different engine.

## User Outcome

Projects declare services in `nerd.json`; Nerd installs missing binaries, creates local instances, injects connection URLs, starts them before the app, and provides backup/restore controls.

## In Scope

- Common service adapter contract
- MySQL, PostgreSQL, and Redis Windows adapters
- Trusted artifact metadata, download, verification, extraction
- Shared engine/version binary cache
- Per-project configuration, data, credentials, port, process, and health
- Initialization and migrations required by engine itself
- Start, stop, restart, status, install, remove
- `keepRunning`
- Backup and restore
- Environment URLs
- Read-only discovery of existing MySQL, PostgreSQL, and Redis installations/listeners
- Machine-local external connection registration and health probes
- `Managed`, `External`, and `Degraded` service status
- Desktop and CLI management

## Out Of Scope

- MariaDB, MongoDB, SQL Server, Valkey, Meilisearch, MinIO
- Nerd-managed shared global database instances
- Cloud databases
- Automatic major-version data upgrades
- Updating, stopping, reconfiguring, backing up, adopting, or removing external services

## Isolation Rules

- Bind loopback only.
- Data directory belongs to project ID and engine.
- Credentials are generated and DPAPI protected.
- Ports are dynamic unless local override pins one.
- Same binary version may be shared; writable directories never are.
- Removing project registration does not remove service data.

## Existing Service Rules

- Discovery may inspect Windows Services, known installation records, executable versions, and listening ports read-only.
- A discovered service is informational until the user explicitly registers an external connection.
- External connection metadata is local state. Credentials use DPAPI or the project's own ignored `.env`; credentials never enter `nerd.json`.
- Nerd may probe external reachability and engine identity without changing server state.
- Nerd does not start, stop, restart, update, initialize, configure, back up, restore, adopt, or uninstall an external service.
- A missing endpoint, failed health probe, or changed engine identity becomes `degraded` without repair.
- Managed instances always use Nerd-owned process, dynamic port, credentials, and data directory, even when a global engine exists.
- Port conflicts trigger a new managed port allocation when safe; Nerd never terminates the existing listener.

## Lifecycle Rules

- Start services before application process.
- Wait for engine-specific readiness.
- Failed required service blocks app startup with typed cause.
- Stop after app unless `keepRunning`.
- Never kill a process solely because it occupies desired service port.

## Backup Rules

- Prefer engine-supported logical backup where practical.
- Record engine/version and project identity with backup.
- Restore validates compatibility and requires confirmation when replacing data.
- Failed restore preserves previous usable data through staging/swap where possible.
- Normal Nerd update never performs an implicit service major-version data upgrade; OD-028 defines explicit upgrade behavior.
- Initialization, backup, and restore expose cancellation safety according to OD-029 rather than assuming every stage can be interrupted.

## Acceptance Criteria

- Two projects run isolated instances of each engine concurrently.
- Same-version projects share binaries but not ports, credentials, or data.
- Connection environment works from a real Node fixture.
- Restart preserves data.
- Stop leaves no child process.
- Backup/restore round-trip passes per engine.
- Tampered service artifact is rejected.
- Existing service discovery and external registration make no system or database mutation.
- Removing an external connection removes only Nerd's local reference and protected credential record.
- Nerd uninstall leaves external services, external data, and configuration untouched.
- Each managed engine has an approved artifact source, license, checksum/signature, version matrix, Windows 10 support statement, archive layout, lifecycle, and backup record in `library-docs.md`.
- Cache service name and behavior match the actual engine; Redis clients working against Garnet does not permit labeling Garnet as Redis.

## Dependencies

- Features 01, 04, and 05
- Feature 03 download security patterns should be reused
