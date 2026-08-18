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
4. **Port 53 UDP is already owned on this machine.** UDP `0.0.0.0:53` is bound by `svchost` (PID 892) hosting the Host Network Service (`hns`) and SharedAccess services, active because WSL2 networking is running. The spike responder cannot bind UDP 53 while this relay is active, and a Nerd daemon binding `127.0.0.1:53` would conflict with it. This confirms the Feature 02 requirement to detect and report port conflicts rather than terminate the foreign listener. `.test` resolution cannot be end-to-end verified on this machine until the conflicting service is stopped or the test runs on a clean image.

## Mutation Test

`tests/windows/od006-nrpt-mutate.ps1` was run elevated on the same Windows 11 Home 25H2 machine. It snapshots existing NRPT rules, adds a temporary `.test` namespace rule pointing to `127.0.0.1`, checks for a port-53 listener, optionally starts the loopback responder, attempts resolution, removes the rule, and verifies the original rule set is restored.

| Step | Status | Detail |
|---|---|---|
| snapshot | ok | 2 existing Tailscale MagicDNS rules captured |
| port-53 | conflict | UDP 53 owned by a foreign listener (`hns` / SharedAccess) |
| add-rule | ok | `.test` -> `127.0.0.1` rule added successfully |
| verify-rule | ok | added rule namespace and name server match expectation |
| responder | skipped | port 53 occupied; responder not started |
| resolve-via-nrpt | fail | no A record (expected without a local responder) |
| resolve-direct | skipped | responder not running |
| remove-rule | ok | temporary rule removed successfully |
| restore-verify | pass | exactly 2 rules after, matching the snapshot |

### Findings

1. **NRPT add/remove/restore cycle works on Windows 11 Home.** The temporary `.test` rule was added and removed without affecting the two coexisting Tailscale MagicDNS rules.
2. **Port-53 conflict detection works.** The spike reports the foreign listener and skips the responder rather than terminating it.
3. **End-to-end `.test` resolution remains blocked on this machine** because the WSL2/`hns` relay owns UDP 53. Verification needs a clean image or explicit user consent to stop the conflicting service.

## Browser Secure DNS

`tests/windows/od006-browser-secure-dns.ps1` is non-elevated and read-only. It inspects Chrome/Edge secure DNS preferences and Firefox `network.trr.mode` to infer whether browser DoH would bypass NRPT for `.test`.

### Measured Browser State

| Browser | Setting | Value | Inference |
|---|---|---|---|
| Chrome | SecureDnsMode | not configured | Profile not present |
| Edge | SecureDnsMode | not configured | Profile not present |
| Brave | SecureDnsMode | not configured | Secure DNS off; NRPT applies |
| Firefox | network.trr.mode | not installed | N/A |

### Finding

On this machine, Brave is installed and its secure DNS setting is off. Chrome and Edge profiles are not present. No measured browser is configured to use its own DoH resolver, so `.test` queries from Brave travel through the Windows DNS client and NRPT. If a user later enables browser secure DNS, those queries will bypass NRPT and likely fail for `.test`. Nerd's planned behavior remains: detect the enabled-browser-DoH condition and report a diagnostic; never reconfigure browser policy.

## Sleep/Resume

`od006-nrpt-mutate.ps1 -TestSleepResume` was run twice with an actual suspend/resume cycle and a manual Enter press after resume.

| Run | Responder | port-53 | resolve-after-sleep-resume | remove-rule | restore-verify |
|---|---|---|---|---|---|
| A (services untouched) | skipped | conflict | fail (no responder) | ok | pass |
| B (attempted `Stop-Service hns, SharedAccess`) | skipped | conflict | fail (no responder) | ok | pass |

### Findings

1. **The temporary `.test` NRPT rule survives sleep/resume.** In both runs the rule was still active after resume and was then removed, with the exact original rule set restored. Rule persistence across sleep/resume is verified.
2. **Resolution after sleep/resume could not be verified because the responder never started.** `Stop-Service` for `hns` and `SharedAccess` did not free UDP 53 while WSL2 was running; the `svchost` listener remained (owner PID changed after service restart). A full WSL2 shutdown (`wsl --shutdown`) from outside the VM, or a clean image, is required for end-to-end verification.
3. **Nerd's Feature 02 port-conflict detection must treat `hns` as persistent while WSL2 is active** and surface a clear diagnostic that the user must stop WSL2 or run on a machine without it.

## Dimensional Status

| Dimension | Status | Evidence required |
|---|---|---|
| NRPT add/remove cycle | Verified on Windows 11 Home | Elevated spike added and removed `.test` rule; original Tailscale rules restored exactly |
| NRPT on Windows 11 Home | Partially verified | Cmdlets/rules present, add/remove cycle verified; end-to-end resolution blocked by `hns` port-53 conflict |
| NRPT on Windows 10 Home | Not verified | Requires Windows 10 22H2 x64 test image |
| NRPT on Windows 10 Pro | Not verified | Requires Windows 10 22H2 Pro test image |
| VPN interaction | Preliminary | Tailscale MagicDNS rules coexist with temporary `.test` rule; no collision observed |
| Corporate DNS / Group Policy | Not verified | Requires managed-environment fixture |
| Browser secure DNS | Verified off on measured machine | Chrome/Edge not configured; Firefox not installed; NRPT applies today. Re-test if user enables browser DoH |
| Sleep/resume | Rule persistence verified; resolution not verified | `.test` rule survives resume and exact rule set is restored; responder blocked by `hns` UDP 53, which survives `Stop-Service` while WSL2 runs |
| UDP and TCP `.test` resolution | Blocked on measurement machine | Requires a clean machine without `hns`/SharedAccess on port 53, or stopping the conflicting service with user consent |
| Port 53 conflict detection | Verified | `hns`/SharedAccess owns UDP 53; spike reports and skips, never terminates, foreign listeners |

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

1. ~~Run `tests/windows/od006-nrpt-mutate.ps1` elevated to verify the add/probe/remove/restore cycle and port-conflict reporting.~~ Done. Add/remove/restore verified; port-conflict detection verified; responder/resolution blocked on this machine by `hns`. End-to-end UDP/TCP `.test` resolution must run on a clean machine or after the conflicting service is stopped with consent.
2. ~~Test sleep/resume with the rule and responder active on a machine where port 53 is free. Requires user interaction.~~ Rule-persistence half done. End-to-end after-resume resolution still needs a machine without `hns`/WSL2 on port 53.
3. ~~Test browser secure DNS on/off for Chrome, Edge, and Firefox.~~ Done for off-state on this machine; re-test when browser DoH is enabled.
4. Test on Windows 10 22H2 Home/Pro once an image is available.
5. Record VPN (Tailscale, plus a corporate VPN if available) interaction.
6. Decide whether Nerd should detect and warn when WSL2/`hns` occupies port 53, and record that in the Feature 02 spec. Resolved: Nerd must detect and report, never terminate, the `hns`/WSL2 listener.
7. Write the supported matrix into `compatibility.md`, close OD-006, and unpark Feature 02.
8. Deferred to release testing: end-to-end UDP/TCP `.test` resolution and after-resume resolution on a clean Windows 10 22H2 or Windows 11 image without WSL2/`hns` on port 53.
