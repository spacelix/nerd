# Feature 02: Windows DNS, HTTPS, And Privilege Setup

## Goal

Make wildcard `.test` names and trusted local HTTPS available on Windows 10+ while keeping daily daemon operation non-elevated.

Implementation is gated by compatibility spike OD-006 and ADR 007, both resolved.

## User Outcome

One-time setup makes any `*.test` query resolve to loopback and allows Nerd-issued HTTPS certificates without browser warnings. Uninstall removes only Nerd-owned changes.

## In Scope

- Typed `nerd-helper.exe` operations (`nrpt_add`, `nrpt_remove`) only
- UAC invocation via one batch session and strict argument parser
- NRPT add, probe, repair, and remove for `.test`
- UDP and TCP DNS listener on `127.0.0.1:53` running in the daemon, non-elevated
- Authoritative `.test` A responses to `127.0.0.1`
- Root CA generation, DPAPI-protected private key, trust install/probe/remove in `CurrentUser` store (daemon, non-elevated)
- On-demand leaf certificates
- Loopback bind/probe for ports 80 and 443
- Setup transaction journal and rollback

## Out Of Scope

- Public DNS forwarding
- Adapter DNS replacement
- Custom TLD UI
- LAN DNS
- Routing to projects; Feature 06 owns proxy routing
- Machine (Local Machine) certificate trust; enterprise scenarios are post-MVP

## Decisions

- **D1 Elevation model**: one batch setup session per UAC prompt. Daemon builds a strict operation plan; helper executes the sequence; rollback on failure. Repair and uninstall each use one additional prompt.
- **D2 Helper contract**: daemon writes the plan JSON to a file under the user data directory; helper argv is the plan path only. Helper writes result JSON and journal entries, exits with a status code. No stdin/stdout redirect, no secrets in the process list.
- **D3 DNS ownership**: the DNS server runs in the daemon, non-elevated, on loopback port 53, using a dedicated DNS library. Port conflicts are detected and reported, never resolved by terminating foreign listeners.
- **D4 CA location**: CA generation, DPAPI key, `CurrentUser` trust install/probe/remove, and leaf issuance live in the daemon, non-elevated. The helper does not handle certificates.
- **D5 Journal and rollback**: append-only JSONL journal in the user data directory, writable by daemon and helper (same user SID). The daemon orchestrates rollback: CA steps it performed are removed if the helper step fails. Ownership markers: NRPT rule `-Comment "nerd-…"` plus display name; CA fingerprint in journal and state DB.
- **D6 Helper surface**: helper is minimal — `nrpt_add` and `nrpt_remove`. NRPT probe runs non-elevated in the daemon. Repair probes first, then issues `[add]` idempotently.
- **D7 Certificate policy**: one long-lived CA per user (~10 years), rotated only when fingerprint ownership mismatches, store entry is missing, or the key is lost. Leaf certificates are issued on demand with SAN covering exactly the validated hostnames and renewed before expiry (30-day threshold) without replacing the CA.
- **D8 Sleep/resume re-probe**: deferred. Re-binding the loopback DNS sockets after resume requires a Windows power-setting notification pump (`RegisterPowerSettingNotification` + `WM_POWERBROADCAST`). Tracked as a follow-up; until then `nerd network status` re-probes and the user can re-run setup to restore the listener.

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
- WSL2/`hns` occupying UDP 53 is detected and reported with a clear diagnostic; the listener is never terminated.
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
- OD-006 spike scripts remain as verification fixtures; the production DNS server replaces the spike responder
