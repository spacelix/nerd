# Compatibility Matrix

## Purpose

This file defines support claims and mandatory test dimensions. `Required` means release-blocking. `Experimental` requires explicit UI labeling. `Unsupported` must fail before mutation or execution.

## Windows

| Target | MVP status | Notes |
|---|---|---|
| Windows 10 x64 | Required | Minimum proposal is 22H2 build 19045 (OD-007, researching); Windows 10 Home/Pro reached end of support 2025-10-14 |
| Windows 11 x64 | Required | Current supported releases |
| Windows ARM64 | Unsupported | Post-MVP Feature 14 |
| Windows Server | Unsupported | Not an MVP target |
| Concurrent Windows users | Unsupported | One active daemon per machine, ADR 003 |

## Filesystems And Paths

| Case | MVP status |
|---|---|
| Local NTFS | Required |
| Spaces in path | Required |
| Unicode path | Required |
| Case-only name collision | Required diagnostic |
| Windows long paths | Required when OS policy permits |
| Junction/symlink inside approved project | Required with escape protection |
| UNC or mapped network drive | Unsupported |
| `\\wsl$` | Unsupported |
| OneDrive-controlled project root | Unsupported |
| Removable drive | Unsupported |

## Shells

| Shell | MVP status | Requirement |
|---|---|---|
| PowerShell | Required | Quoting, environment, exit code |
| Command Prompt | Required | Quoting, environment, exit code |
| Git Bash | Required | CLI invocation and path translation smoke test |
| WSL shell | Unsupported | No Linux process orchestration |

## Node

| Channel | MVP status |
|---|---|
| Active LTS | Required |
| Maintenance LTS | Required |
| Current | Decision OD-008 |
| End-of-life | Decision OD-008 |
| x64 Windows ZIP | Required |
| ARM64 | Post-MVP |

## Package Managers

| Tool | MVP status |
|---|---|
| npm | Required |
| pnpm | Required |
| Yarn modern | Required |
| Yarn classic | Required compatibility fixture |
| Multiple conflicting lockfiles | Required diagnostic; never guess |

## Frameworks

| Framework | MVP smoke paths |
|---|---|
| Next.js | HTTP, HTTPS, HMR, production-like error page not required |
| Vite | React, Vue, Svelte, vanilla, HMR |
| Nuxt | HTTP, HTTPS, HMR |
| Astro | HTTP, HTTPS, HMR |
| NestJS | HTTP, WebSocket fixture |
| Express/custom | `PORT`, readiness path, WebSocket/SSE fixture |

Exact versions are selected when Feature 05 begins and recorded in `library-docs.md` test fixtures.

## Network

| Case | MVP status |
|---|---|
| DNS UDP | Required |
| DNS TCP | Required |
| IPv4 loopback | Required |
| IPv6 loopback | OD-014 |
| HTTP/1.1 | Required |
| HTTPS trusted in Edge/Chrome | Required |
| Firefox trust behavior | Required diagnostic/support decision during Feature 02 |
| SSE | Required |
| WebSocket/HMR | Required |
| VPN/corporate DNS | Feature 02 compatibility spike (OD-006 researching; Tailscale coexistence evidence collected, mutation test pending) |
| LAN/public access | Unsupported, post-MVP Feature 15 |

## Services

| Engine | MVP status |
|---|---|
| MySQL | Candidate, blocked by OD-002 |
| PostgreSQL | Candidate, blocked by OD-003 |
| Redis | Blocked by OD-004; no approved native Windows Redis artifact |
| External MySQL/PostgreSQL/Redis | Required read-only registration/probe |
| Garnet | Not Redis; possible explicit product decision only |

## Fault Matrix

Release tests must inject:

- UAC denied
- Mandatory port occupied before startup and raced during startup
- Download interrupted, tampered, and disk full
- Archive traversal entry
- SQLite migration failure and database lock
- Daemon crash and Windows restart
- Stale/reused PID
- Project and service startup timeout
- Child process crash loop
- Sleep/resume
- External runtime/service removed or changed
- Update activation health failure and rollback

## Performance Matrix

Measure on representative Windows 10 and 11 hardware:

- Daemon cold start and idle RAM/CPU
- Desktop cold start and idle RAM
- Park scan with 1,000 child directories
- Watcher idle and burst reconciliation
- Proxy latency and memory with inspector off/on
- Sustained stdout/stderr throughput
- Mail and inspector retention cleanup
- Concurrent project and service startup
