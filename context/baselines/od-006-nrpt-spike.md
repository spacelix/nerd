# OD-006 NRPT Compatibility Spike

- Status: researching (draft evidence collected; mutation phases pending)
- Date: 2026-08-16
- Branch: `test/nrpt-compatibility`

## Purpose

Produce a supported matrix for wildcard `.test` resolution through Windows NRPT before Feature 02 production implementation. Decision requirement: verify NRPT on Windows 10 Home/Pro, VPN and corporate DNS interaction, browser secure DNS behavior, sleep/resume, and UDP/TCP DNS for `.test`.

## Official References

- NRPT namespace rules are managed with `Get/Add/Set/Remove-DnsClientNrptRule` (DnsClient module): https://learn.microsoft.com/en-us/powershell/module/dnsclient/set-dnsclientnrptrule
- Windows DNS client DoH (Server 2022+; client support equivalent): https://learn.microsoft.com/en-us/windows-server/networking/dns/doh-client-support
- Windows 10 lifecycle: end of support 2025-10-14; last Home/Pro version is 22H2, build 19045: https://learn.microsoft.com/en-us/windows/release-health/release-information

## Read-Only Probe

`tests/windows/od006-nrpt-probe.ps1` is non-elevated and read-only. It collects OS identity, NRPT rules, DNS server addresses, DoH server list, per-interface DoH configuration, active interfaces, DNS client service state, and local DNS policy.

### Environment Measured

| Field | Value |
|---|---|
| OS | Microsoft Windows 11 Home Single Language |
| Display version | 25H2 |
| Build | 10.0.26200 (CurrentBuild 26200) |
| Architecture | x86_64 |
| Probe elevation | non-elevated |
| Machine | DESKTOP-U918DRN |

### DNS Client State

| Block | Result |
|---|---|
| NRPT rules | 2 existing rules, both owned by Tailscale MagicDNS (`100.100.100.100`, `fd7a:115c:a1e0::53`); namespaces are `*.ts.net`, `*.100.in-addr.arpa`, and reverse `ip6.arpa` ranges. No `.test` rule present. |
| NRPT global | QueryPolicy `Disable`, SecureNameQueryFallback `Disable` |
| DNS servers | WiFi interface uses `192.168.18.1` |
| DoH server list | Default Cloudflare, Google, and Quad9 templates present |
| Per-interface DoH | `DnsOverHttpsEnabled` unset on all interfaces (Windows DNS client DoH is not configured) |
| Active interfaces | WiFi (native), Tailscale (virtual), WSL vEthernet (virtual) |
| DNS client service | Running, automatic |
| Local DNSClient policy | None |

### Findings

1. **NRPT coexistence with a VPN (Tailscale)** is already observable: two third-party NRPT rules exist and are scoped to their own namespaces. A `.test` rule does not currently collide with them. This is preliminary evidence that namespaces are independent, but interaction must be re-tested with an actual `.test` rule installed.
2. **Windows DNS client DoH is disabled** on the measured machine, so NRPT-based `.test` queries would currently travel as plain DNS to `127.0.0.1`. The DNS client DoH feature routes through NRPT when the target server is a known DoH server; `127.0.0.1` is not in the known list, so NRPT `.test` queries remain plain-text.
3. **Browser secure DNS bypasses NRPT.** Chrome/Edge Secure DNS and Firefox DoH resolve outside the Windows DNS client. Without explicit handling, a `.test` URL can fail when a browser is configured to resolve via an external DoH provider. Proposed behavior: Nerd detects this condition and reports a diagnostic; it never bypasses browser policy or configures browsers.

## Dimensional Status

| Dimension | Status | Evidence required |
|---|---|---|
| NRPT on Windows 11 Home | Partially verified | Probe shows cmdlets and rules present; full add/resolve/remove cycle pending |
| NRPT on Windows 10 Home | Not verified | Requires Windows 10 22H2 x64 test image |
| NRPT on Windows 10 Pro | Not verified | Requires Windows 10 22H2 Pro test image |
| VPN interaction | Preliminary | Tailscale MagicDNS rules coexist; must re-test with `.test` rule |
| Corporate DNS / Group Policy | Not verified | Requires managed-environment fixture |
| Browser secure DNS | Policy direction set | Behavior test with Chrome/Edge/Firefox pending |
| Sleep/resume | Not verified | Requires user-interactive resume and rule + responder active |
| UDP and TCP `.test` resolution | Not verified | Requires NRPT rule + loopback DNS responder |

## OD-007 Minimum Windows 10 Build

Windows 10 Home/Pro reached end of support on 2025-10-14. The final available Home/Pro release is version 22H2 (build 19045).

| Claim | Value |
|---|---|
| Minimum Windows 10 | 22H2 x64, build `19045` or later |
| Windows 11 | Supported, current releases |
| Test image | Windows 10 22H2 x64 ISO with the latest monthly patch; Windows 11 x64 current release |
| Older build behavior | Installation refused with a clear message, no mutation |

Status: researching. The final claim is written to `compatibility.md` when a Windows 10 22H2 image is exercised.

## Next Steps

1. Mutate a `.test` NRPT rule on an approved machine with full rollback and verify UDP/TCP resolution. Requires explicit UAC approval.
2. Test sleep/resume with the rule and responder active. Requires user interaction.
3. Test browser secure DNS on/off for Chrome, Edge, and Firefox.
4. Test on Windows 10 22H2 Home/Pro once an image is available.
5. Record VPN (Tailscale, plus a corporate VPN if available) interaction.
6. Write the supported matrix into `compatibility.md`, close OD-006, and unpark Feature 02.
