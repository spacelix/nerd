# Architecture Decision Records

ADRs capture durable choices that affect multiple features. Feature-specific details remain in `context/features/`.

## Status

- `Proposed`: needs explicit approval or blocked research.
- `Accepted`: implementation must follow it.
- `Superseded`: replaced by another ADR; never delete history.
- `Rejected`: considered and intentionally not selected.

## Index

| ID | Decision | Status |
|---|---|---|
| 001 | Managed and external resource ownership | Accepted |
| 002 | Explicit project trust before execution | Accepted |
| 003 | Per-user application with limited machine setup | Accepted |
| 004 | Canonical ports and foreign-conflict policy | Accepted |
| 005 | Retention and explicit data deletion | Accepted |
| 006 | Managed service artifact sources | Proposed, blocking Feature 10 |
| 007 | Wildcard `.test` through NRPT | Accepted |
| 008 | Signed application update chain | Accepted |

## Maintenance

- Never edit an ADR to hide an old decision.
- Add a new ADR and mark the old one superseded.
- Link relevant ADRs from feature specifications.
- `/architect` is required before accepting or superseding an ADR.
