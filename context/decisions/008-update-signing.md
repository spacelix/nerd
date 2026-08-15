# ADR 008: Signed Application Update Chain

- Status: Accepted
- Date: 2026-08-14

## Context

Nerd downloads executable updates and performs privileged setup. HTTPS alone does not protect against compromised release metadata or publishing credentials.

## Decision

Windows installers and executable artifacts are code-signed. Update manifests are separately signed, list immutable artifact checksums, and reference a tagged application version. Builds come only from the tagged commit. Activation is staged and health-checked with rollback.

## Consequences

- Code-signing certificate/key custody must be resolved before public beta.
- CI release identity receives least privilege.
- Update verification does not trust transport alone.
- Failed post-update health check rolls back binaries without downgrading state unsafely.
