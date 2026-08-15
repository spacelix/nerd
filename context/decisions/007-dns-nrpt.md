# ADR 007: Wildcard `.test` Through NRPT

- Status: Accepted
- Date: 2026-08-14

## Context

Park Directory requires wildcard local names. Hosts entries cannot express a wildcard. Replacing adapter DNS would interfere with public, VPN, and corporate DNS.

## Decision

Run authoritative `.test` DNS on loopback port 53 and install a Windows NRPT namespace rule directing only `.test` to loopback. Do not forward public DNS or replace adapter resolver settings.

## Consequences

- Port 53 is mandatory.
- Windows 10 Home/Pro, VPN, secure DNS browser, sleep/resume, and corporate policy need a pre-implementation spike.
- Unsupported Group Policy is reported, never bypassed.
- Setup, probe, repair, and uninstall are symmetric typed helper operations.
