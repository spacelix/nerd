# ADR 001: Managed And External Resource Ownership

- Status: Accepted
- Date: 2026-08-14

## Context

Developer machines may already contain Node, databases, certificates, services, ports, and tooling. Automatic adoption creates data-loss and uninstall risk.

## Decision

Every resource is `Managed` or `External`. Nerd may mutate only a Managed resource whose ownership metadata and live identity match. External resources are read-only and used only after explicit registration where supported.

## Consequences

- Existing software coexists with Nerd.
- Detection cannot imply ownership.
- Uninstall and repair need ownership verification.
- External resources cannot receive Nerd backup, update, repair, stop, or removal actions.
- UI and diagnostics must show ownership class.
