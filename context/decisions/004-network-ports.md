# ADR 004: Canonical Ports And Foreign-Conflict Policy

- Status: Accepted
- Date: 2026-08-14

## Context

Wildcard DNS and clean URLs require ports 53, 80, and 443. IIS, VPNs, proxies, Docker tooling, or another Nerd user may already own them.

## Decision

MVP uses loopback 53, 80, and 443. Nerd never kills, stops, reconfigures, or silently bypasses a foreign owner. Setup and doctor report owning process/service where possible and block the affected subsystem.

## Consequences

- Clean canonical URLs remain predictable.
- No silent high-port fallback.
- User must resolve foreign conflicts explicitly.
- Tests cover port takeover races and process identity.
