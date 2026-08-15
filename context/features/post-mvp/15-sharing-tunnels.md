# Feature 15: LAN And Public Sharing

## Horizon

Post-MVP v1.x candidate.

## Goal

Share selected local projects with trusted LAN devices or through an explicit public tunnel without weakening Nerd's loopback-by-default security.

## User Outcome

Users temporarily expose one running project, see the exact reachable URL and security state, and revoke access immediately.

## In Scope

- Per-project LAN exposure
- Temporary public quick tunnel through an approved provider
- Optional named tunnel after separate credential setup
- Share-session lifecycle, status, expiration, and revocation
- Remote-device local CA guidance for LAN HTTPS where applicable
- Request Inspector attribution for shared traffic
- Explicit firewall-rule setup and symmetric removal when required

## Out Of Scope

- Global exposure of every project
- Hidden background tunnels
- Production hosting or uptime guarantees
- Account creation inside Nerd
- Persisting provider secrets in `nerd.json`

## Security Rules

- Sharing is off by default and enabled per project.
- UI and CLI show public/LAN exposure continuously.
- Public tunnels require explicit start and have safe expiration defaults.
- Local services and databases remain loopback-only.
- Inspector redaction remains active for shared requests.
- Foreign firewall rules and tunnel processes are never modified.

## Open Decisions

- Tunnel provider and distribution/license model.
- Whether named tunnels belong in first release of this feature.
- LAN DNS/bootstrap experience across Windows, mobile, and other devices.

## Acceptance Criteria

- Only selected project becomes reachable.
- Revocation closes listener/tunnel and removes Nerd-owned firewall state.
- Daemon crash does not leave an unmanaged public tunnel.
- Database, Redis, SMTP, IPC, and daemon admin APIs remain unreachable remotely.
- Sharing state is visible in tray, desktop, CLI, and diagnostics.

## Dependencies

- MVP Features 02, 06-09, and 12
