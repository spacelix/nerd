# Post-MVP Roadmap

## Purpose

This roadmap records product capabilities intentionally deferred beyond MVP. It prevents useful ideas from being lost without allowing them to expand current MVP scope.

Roadmap horizons are directional, not release dates or commitments. Priority changes require `/architect`, an updated feature specification, and an explicit tracker status change.

## Release Language

- **MVP**: first stable Windows 10 x64 release, targeted as `1.0.0`, defined by Features 01-12.
- **Post-MVP v1.x**: incremental Windows and workflow expansion after MVP stability and performance gates pass.
- **Platform Expansion v2.x**: larger portability work that changes platform architecture and release matrix.
- **Candidate**: documented idea not yet approved for implementation.

Exact application versions follow `versioning.md`. Roadmap horizons do not replace SemVer decisions.

## Entry Gates

No post-MVP feature starts until:

- Features 01-12 are complete.
- Windows 10/11 release gates pass.
- Installer, update, rollback, and uninstall paths are proven.
- Performance budgets remain satisfied on representative projects.
- Feature has an active owner, acceptance plan, and approved dependency review.

## Post-MVP v1.x

| ID | Feature | Specification | State |
|---|---|---|---|
| 13 | Bun and Deno runtimes | `features/post-mvp/13-bun-deno-runtimes.md` | candidate |
| 14 | Native Windows ARM64 | `features/post-mvp/14-windows-arm64.md` | candidate |
| 15 | LAN and public sharing | `features/post-mvp/15-sharing-tunnels.md` | candidate |
| 16 | MCP and AI-agent integration | `features/post-mvp/16-mcp-ai-integration.md` | candidate |
| 17 | Additional managed services | `features/post-mvp/17-additional-services.md` | candidate |

## Platform Expansion v2.x

| ID | Feature | Specification | State |
|---|---|---|---|
| 18 | macOS and Linux support | `features/post-mvp/18-macos-linux.md` | candidate |

## Explicit Non-Commitments

These remain outside current roadmap unless separately approved:

- Cloud accounts or synchronization
- Paid license enforcement
- Team administration portal
- Production deployment or hosting
- Docker and VM orchestration
- Remote diagnostic upload or telemetry
- Mobile applications

## Promotion Workflow

To promote a candidate:

1. Run `/architect` against its current specification.
2. Validate user need and platform constraints.
3. Resolve open decisions and dependency/license risks.
4. Move feature into `context/build-plan.md` under a named phase.
5. Add it to `context/progress-tracker.md` as `planned`.
6. Keep all earlier release gates green during implementation.
