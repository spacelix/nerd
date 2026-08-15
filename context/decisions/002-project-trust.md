# ADR 002: Explicit Project Trust Before Execution

- Status: Accepted
- Date: 2026-08-14

## Context

Parked directory discovery can expose cloned or malicious repositories. Reading metadata is lower risk than running package scripts or scaffold output.

## Decision

Discovery never executes project code. First dependency installation or project start requires Trust and Start with a preflight showing final command, working directory, runtime, package manager, services, and environment conflicts. Material project identity changes invalidate trust.

## Consequences

- Park Directory remains automatic but execution is not.
- Trust state must bind to stable project and filesystem/repository identity.
- Changed commands require confirmation.
- Project processes remain non-elevated and cannot call helper operations.
