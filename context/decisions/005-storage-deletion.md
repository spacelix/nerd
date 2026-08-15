# ADR 005: Retention And Explicit Data Deletion

- Status: Accepted
- Date: 2026-08-14

## Context

Runtimes, service data, backups, logs, mail, and captures can consume significant disk. Unlink, uninstall, and delete have different user expectations.

## Decision

Ephemeral data has bounded retention; user data is retained until an explicit named deletion action. Registration changes never imply data deletion.

Initial defaults:

- Project logs: maximum 100 MB per project through rotation.
- Mail: maximum 500 messages or 250 MB per project, oldest first.
- Inspector: 500 in-memory requests per project; no persistence by default.
- Temporary/incomplete downloads: clean after 24 hours when no operation owns them.
- Backups and service data: no automatic deletion; show disk usage and warnings.

Deletion actions remain distinct:

- Unlink/unpark: remove registration/route only.
- Remove external reference: remove local metadata and Nerd-stored credential copy only.
- Remove managed service: stop and remove instance registration; retain data by default.
- Delete service data: separate confirmation naming project, engine, and path.
- Delete backup: separate confirmation.
- Uninstall: retain project service data/backups by default; optional explicit purge of verified Nerd-owned data.

## Consequences

- Disk usage UI and cleanup commands are required.
- Retention failures cannot crash daemon or project.
- Data purge requires ownership and path verification.
- Secure overwrite is not promised on SSDs.
