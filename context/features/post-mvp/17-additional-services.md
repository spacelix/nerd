# Feature 17: Additional Managed Services

## Horizon

Post-MVP v1.x candidate.

## Goal

Expand the managed-service catalog using the same per-project isolation, verified artifact, lifecycle, health, and backup model as MVP services.

## Candidate Engines

- MariaDB
- MongoDB
- Valkey
- Meilisearch
- Typesense
- MinIO or another approved S3-compatible local object store

Candidate listing is not approval. Each adapter requires source, license, Windows artifact, checksum/signature, lifecycle, and backup review before promotion.

## User Outcome

Approved engines can be declared in `nerd.json`, installed on demand, isolated per project, and exposed through generated environment placeholders.

## In Scope

- Service catalog metadata
- Adapter capability model
- Per-engine artifact and license approval
- Per-project process, data, port, credentials, health, and retention
- Environment URL generation
- UI and CLI installation/status/backup where engine supports it

## Out Of Scope

- Running container-only services
- Redistributing binaries without clear license permission
- Claiming backup support where engine has no safe method
- Automatic cross-major data upgrades

## Architecture Rules

- Do not weaken Feature 10 isolation contract.
- Service-specific behavior stays in adapter modules.
- Unsupported backup or architecture capability is explicit.
- One engine approval does not automatically approve all candidates.

## Acceptance Criteria

- Every promoted adapter passes artifact integrity, isolation, lifecycle, persistence, and cleanup tests.
- License and distribution notes exist in `context/library-docs.md`.
- Existing MySQL, PostgreSQL, and Redis behavior remains unchanged.
- Unavailable engines fail with capability explanation, not generic install errors.

## Dependencies

- MVP Features 10 and 12
