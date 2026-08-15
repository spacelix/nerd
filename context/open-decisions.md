# Open Decisions

This register contains unresolved choices that can change implementation. Do not guess around a blocking decision. Run `/architect`, record the answer in an ADR or feature specification, then close the item here.

## Status Values

- `open`: decision needed before named feature begins
- `researching`: evidence is being gathered
- `blocked`: feature cannot proceed
- `deferred`: intentionally outside current feature/release
- `closed`: resolved elsewhere; include link

## MVP Blockers

| ID | Decision | Blocks | Status | Required outcome |
|---|---|---|---|---|
| OD-001 | Final product name, trademark, domain, executable, repository, and package-name availability | Public release | open | Legal/namespace check and naming ADR |
| OD-002 | MySQL Windows archive metadata, checksum, version matrix, and download/redistribution terms | Feature 10 MySQL | researching | Approved artifact record in `library-docs.md` |
| OD-003 | PostgreSQL EDB ZIP URL/API, checksum/signature, license, and Windows 10 support | Feature 10 PostgreSQL | researching | Approved artifact record in `library-docs.md` |
| OD-004 | Redis strategy: Memurai, explicit Garnet, external-only Redis, or removal from MVP | Feature 10 cache | blocked | Accepted replacement/update to ADR 006 and Feature 10 |
| OD-005 | Windows code-signing provider and private-key custody | Public beta, Feature 12 | open | Signing runbook and CI identity policy |
| OD-006 | NRPT compatibility spike on Windows 10 Home/Pro, VPN, browser secure DNS, and sleep/resume | Feature 02 | researching | Spike report and supported matrix; read-only probe evidence committed, mutation/browser/sleep phases pending approval |
| OD-007 | Exact minimum Windows 10 build | Installer and support | researching | Proposal: 22H2 build 19045 x64; pending Windows 10 22H2 test-image verification |

## Decisions Before Relevant Feature

| ID | Decision | Before | Status |
|---|---|---|---|
| OD-008 | Node Current/EOL installation and warning policy | Feature 03 | open |
| OD-009 | External-tool discovery during onboarding versus on-demand screens | Feature 03/07/10 | open |
| OD-010 | Exact project trust invalidation identity for replaced directories/repositories | Feature 04/05 | open |
| OD-011 | Monorepo workspace selection and command model | Feature 04/05 | open |
| OD-012 | `.env`, shell environment, `nerd.json`, generated values, and local override precedence | Feature 04/05/10 | open |
| OD-013 | Framework dependency-install prompt and missing-lockfile behavior | Feature 05/11 | open |
| OD-014 | Proxy IPv6 policy; IPv4-only is current recommendation | Feature 02/06 | open |
| OD-015 | Minimum desktop window size and narrow split-view behavior | Feature 07 | open |
| OD-016 | Exact configurable retention limits and disk-warning thresholds | Feature 07/09/12 | open |
| OD-017 | Supported corporate proxy/custom CA behavior for downloads | Feature 03/10/12 | open |
| OD-018 | Logical versus physical backup method per database/version | Feature 10 | open |
| OD-019 | Windows notifications policy for crash/update/long operations | Feature 07/12 | open |
| OD-020 | Diagnostic bundle file list and path-redaction policy | Feature 12 | open |
| OD-026 | Project rename behavior for domain, stable ID, services, mail, certificates, and backups | Feature 04 | open |
| OD-027 | Domain alias data model without implementing alias UI in MVP | Feature 04/06 | open |
| OD-028 | Service minor/major update and data-upgrade policy | Feature 10 | open |
| OD-029 | Cancellation safety classes for downloads, scaffolding, initialization, backup, and restore | Features 03/10/11 | open |

## Deferred Beyond MVP

| ID | Decision | Status | Roadmap link |
|---|---|---|---|
| OD-021 | WSL project execution model | deferred | Not currently planned |
| OD-022 | UNC, mapped drive, OneDrive, and removable-drive support | deferred | Platform expansion candidate |
| OD-023 | Concurrent multi-user daemon coordination | deferred | Not currently planned |
| OD-024 | Public tunnel provider | deferred | Feature 15 |
| OD-025 | MCP permission model | deferred | Feature 16 |

## Closed

| ID | Decision | Resolution |
|---|---|---|
| CD-001 | Core implementation stack | Rust daemon/CLI/helper and Tauri 2 + React/TypeScript |
| CD-002 | Runtime MVP | Node.js first |
| CD-003 | Park Directory | Required from MVP |
| CD-004 | Local DNS | Wildcard `.test` through NRPT, ADR 007 |
| CD-005 | Resource ownership | Managed versus External, ADR 001 |
| CD-006 | Installation scope | Per-user app with limited system setup, ADR 003 |
| CD-007 | Project execution | Explicit Start/Stop and project trust, ADR 002 |
| CD-008 | Commit/version policy | `versioning.md` |

## Review Rule

- `progress-tracker.md` must name any open item blocking the active feature.
- A feature cannot be marked complete with unresolved acceptance-blocking items.
- Closing an item requires a durable link, not only a chat decision.
