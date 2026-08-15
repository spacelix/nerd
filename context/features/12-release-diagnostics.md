# Feature 12: Diagnostics, Installer, Updates, And Recovery

## Goal

Ship a trustworthy Windows product that diagnoses failures, repairs safe conditions, updates atomically, and uninstalls cleanly.

## User Outcome

Users can install Nerd on Windows 10+, run `nerd doctor`, apply safe repairs, receive signed updates, and uninstall without leaving Nerd-owned system changes.

## In Scope

- Signed x64 Windows installer
- Windows 10 and 11 compatibility checks
- Daemon and tray autostart
- CLI PATH entry
- Setup journal and rollback
- `nerd status` and `nerd doctor`
- Probes for daemon, IPC, database, NRPT, DNS, CA, ports, runtime inventory, service health, disk, and permissions
- Ownership verification and external-resource discovery diagnostics
- Safe repair operations
- Signed update manifest, checksums, staging, atomic activation, rollback
- Clean uninstall options for binaries, config, project data, and service data
- Crash recovery and stale process reconciliation
- Performance and release CI gates
- Local support-bundle export with preview and redaction
- Disk usage, retention, orphan, and safe-cleanup diagnostics

## Out Of Scope

- Account-based update channels
- Silent enterprise deployment policy
- Remote diagnostics upload
- Automatic deletion of project service data

## Diagnostic Rules

- Probe before repair.
- Separate healthy, degraded, failed, unsupported-policy, and foreign-conflict states.
- Report ownership class and evidence for every resource that diagnostics may propose mutating.
- External runtime/service probes are read-only and never become repair targets.
- Safe repair must not delete user data or stop foreign processes.
- Destructive repair requires explicit confirmation and names affected paths/resources.
- `--json` output remains stable for automation.
- Support bundles are local-only, previewed before creation, and never uploaded automatically.
- Safe cleanup covers expired logs/mail/temp files and unreferenced verified caches only; backups and service data require separate confirmation.

## Update Rules

- Verify signature and checksum before staging.
- Update never replaces a running binary in place.
- Preserve compatible state and migrate transactionally.
- Failed health check after activation rolls back binaries.
- Newer unsupported state schema prevents unsafe downgrade.

## Uninstall Rules

- Match ownership markers and fingerprints before removal.
- Remove Nerd NRPT, CA, autostart, PATH, and binaries symmetrically.
- Default retains project service data and backups, with explicit deletion option.
- Removing external registrations deletes only Nerd's references and DPAPI-protected copies of credentials stored by Nerd.
- Never remove external Node, database, Redis, data directories, Windows Services, configuration, executables, certificates, or resolver rules.

## Acceptance Criteria

- Clean Windows 10 x64 install reaches trusted `.test` smoke site.
- Injected setup failure rolls back completed mutations.
- Doctor identifies each known fault fixture accurately.
- Update success and forced-failure rollback pass.
- Uninstall leaves unrelated system configuration unchanged.
- Uninstall fixture with registered external Node and databases proves their processes, files, services, data, and configuration remain unchanged.
- Resource and installer budgets pass release gates.
- No telemetry or diagnostic upload occurs.
- Support-bundle fixtures prove credentials, request/mail bodies, private keys, and raw authorization/cookies are excluded or redacted.

## Dependencies

- All previous features
