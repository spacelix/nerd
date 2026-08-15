# Trust Model

## Purpose

Nerd executes project-controlled code, manages local network endpoints, stores development data, and performs limited elevated setup. This document defines what is trusted, what is hostile by default, and where mutation is allowed.

## Trust Levels

### Trusted Nerd Code

- Signed Nerd desktop, daemon, CLI, helper, and installer from a verified release
- Versioned IPC schema implemented by matching clients
- Nerd-owned state carrying valid ownership metadata
- Download metadata from an allowlisted source after signature/checksum verification

### User-Approved Code

- A project explicitly trusted by the current user
- Package scripts explicitly approved for first execution
- External runtime or service explicitly registered for read-only use

User approval does not make project code privileged. It runs as the current user with no helper access.

### Untrusted By Default

- Newly discovered, cloned, downloaded, or renamed projects
- `package.json` scripts and framework scaffolding output
- npm packages and native addons
- `nerd.json` from a repository
- HTTP requests, headers, bodies, WebSocket peers, and captured mail
- Downloaded archives before integrity verification
- Paths reached through symlinks, junctions, UNC shares, or removable drives
- External processes, listeners, runtimes, databases, certificates, and registry entries

## Protected Assets

- Current-user files and project source
- Database/service data and backups
- DPAPI-protected credentials and CA private key
- Windows DNS, certificate, PATH, autostart, and firewall configuration
- Daemon IPC authority
- Request and mail content
- Integrity of Nerd binaries and updates

## Boundaries

### Desktop/CLI To Daemon

- Current-user named pipe only
- Version handshake before commands
- Typed commands and errors
- No raw SQL, shell text, or helper passthrough

### Daemon To Project

- Project process is non-elevated
- Selected runtime PATH is child-local
- Project receives only required generated environment values
- Process tree is contained in a Job Object
- Project cannot invoke privileged helper through daemon IPC

### Daemon To Helper

- Typed allowlisted operations only
- Strict arguments, paths, ownership, and postcondition checks
- Helper is short-lived and never executes project-controlled files

### Daemon To Network

- Loopback binding by default
- `.test` DNS answers only local addresses
- Proxy validates host and prevents loops
- Public/LAN exposure is post-MVP, explicit, visible, and per-project

### Daemon To Downloads

- HTTPS and allowlisted origin
- Temporary staging
- Checksum/signature before extraction
- Traversal-safe extraction
- Atomic promotion after verification

## Project Trust Workflow

1. Discovery records metadata but does not execute code.
2. UI/CLI shows runtime, package manager, final command, services, working directory, and environment conflicts.
3. User chooses Trust and Start.
4. Trust is stored against stable project ID plus canonical path and repository identity when available.
5. Material identity change, such as path replacement or repository replacement, requires trust again.

Operations requiring separate confirmation:

- Dependency installation
- First execution of a new or changed command
- Framework scaffolding
- Database restore or destructive reset
- External credential storage
- Body capture
- Elevated setup or repair

## Threats And Controls

| Threat | Required control |
|---|---|
| Malicious package script | Explicit project trust, command preview, non-elevated process |
| Child-process escape | Windows Job Object and verified process identity |
| IPC caller from another user | Current-user pipe ACL |
| Arbitrary privileged mutation | Typed helper operations and ownership verification |
| Archive traversal | Reject absolute, parent, and escaping link entries |
| Supply-chain replacement | Allowlisted source plus checksum/signature |
| Secret leakage | DPAPI, structured redaction, no secrets in manifest/log/errors |
| Mail-based script execution | Sandboxed HTML, scripts disabled, remote content blocked |
| Request capture leakage | Opt-in bodies, size/content limits, redaction before event/storage |
| Symlink/junction escape | Canonicalize final target and enforce approved roots |
| Foreign process termination | Ownership and live identity required before stop/kill |
| Stale PID reuse | PID plus creation identity, executable, and ownership metadata |
| Malicious update | Signed manifest, checksums, tagged build, rollback health check |

## Location Policy

- Local NTFS paths: supported.
- Paths with spaces, Unicode, case variation, and long-path support: required tests.
- UNC, mapped network drives, OneDrive-controlled folders, removable drives, and `\\wsl$`: unsupported in MVP unless a feature decision promotes them.
- Unsupported locations are detected before trust or execution.

## Non-Goals

- Sandboxing arbitrary npm code from the current user's files
- Protecting against a fully compromised current Windows account
- Production-grade secret management
- Malware scanning
- Bypassing corporate policy, antivirus, VPN, or endpoint controls
