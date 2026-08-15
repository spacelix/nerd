# ADR 003: Per-User Application With Limited Machine Setup

- Status: Accepted
- Date: 2026-08-14

## Context

Nerd owns per-user projects and credentials but `.test` NRPT, trusted CA, PATH, and port ownership interact with Windows-wide state. Multiple active user daemons would conflict on 53, 80, and 443.

## Decision

Application binaries, state, runtimes, services, credentials, and daemon are per-user. Elevated helper performs only typed system setup. MVP supports one active Nerd daemon per machine; concurrent multi-user operation is unsupported and diagnosed.

## Consequences

- Daily work remains non-elevated.
- Installer records machine mutations and their owning user/install ID.
- Second active user receives actionable conflict status.
- Multi-user coordination requires a future architecture decision.
