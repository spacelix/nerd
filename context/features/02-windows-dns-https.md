# Feature 02: Windows DNS, HTTPS, And Privilege Setup

## Goal

Make wildcard `.test` names and trusted local HTTPS available on Windows 10+ while keeping daily daemon operation non-elevated.

Implementation is gated by compatibility spike OD-006 and ADR 007.

## User Outcome

One-time setup makes any `*.test` query resolve to loopback and allows Nerd-issued HTTPS certificates without browser warnings. Uninstall removes only Nerd-owned changes.

## In Scope

- Typed `nerd-helper.exe` operations
- UAC invocation and strict argument parser
- NRPT add, probe, repair, and remove for `.test`
- UDP and TCP DNS listener on `127.0.0.1:53`
- Authoritative `.test` A responses to `127.0.0.1`
- Root CA generation, DPAPI-protected private key, trust install/probe/remove
- On-demand leaf certificates
- Loopback bind/probe for ports 80 and 443
- Setup transaction journal and rollback

## Out Of Scope

- Public DNS forwarding
- Adapter DNS replacement
- Custom TLD UI
- LAN DNS
- Routing to projects; Feature 06 owns proxy routing

## DNS Rules

- Answer `.test` descendants only.
- Bare `test`, malformed names, and non-`.test` names return an appropriate negative response.
- Support UDP and TCP.
- NRPT sends `.test` only to `127.0.0.1`.
- Corporate Group Policy conflicts are reported, never bypassed.
- Sleep/resume re-probes DNS binding and resolver behavior.
- Browser secure-DNS behavior and VPN interaction must be verified on supported Windows editions before support is claimed.

## Certificate Rules

- One CA per Windows user installation.
- Leaf certificate SAN exactly covers requested validated hostnames.
- Keys never appear in logs or IPC.
- Renewal happens before expiration without replacing trusted CA unnecessarily.
- Remove CA only when fingerprint and Nerd ownership record match.

## Port Conflict Rules

- Never kill or reconfigure foreign listeners.
- Identify owning PID/process where permissions allow.
- DNS cannot silently fall back because NRPT cannot encode a custom port.
- Proxy high-port fallback is not MVP behavior for canonical URLs.

## Acceptance Criteria

- `foo.test` resolves through UDP and TCP on clean Windows 10.
- Public DNS remains unchanged.
- Trusted TLS handshake succeeds for `foo.test`.
- Foreign port conflict produces actionable diagnostic.
- Failed setup rolls back NRPT and CA mutations already performed.
- Repeated setup is idempotent.
- Uninstall preserves unrelated NRPT rules and certificates.
- Windows 10 Home/Pro, Windows 11, VPN, browser secure DNS, and sleep/resume spike results are recorded in `compatibility.md`.

## Dependencies

- Feature 01 IPC, state, logging, and operations
