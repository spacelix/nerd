# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Stack

Vite + React 18 + TypeScript strict + Tailwind v4 (per `context/library-docs.md` approved stack). Prototype scope; final Nerd delivery is Tauri 2 desktop shell around the same React UI.

## Users

Primary: JavaScript and TypeScript developers on Windows 10/11 who want projects to run at stable local URLs without manually coordinating Node versions, ports, certificates, databases, and background processes.

## Product Purpose

Nerd is a lightweight, open-source local JavaScript development environment for Windows. It provides Herd/Yerd-style local domains, HTTPS, Node runtime management, process supervision, services, mail capture, and request inspection for Node.js projects — without Docker, virtual machines, Electron, or a system-wide Node dependency.

Success means a clean Windows 10 user reaches a trusted `https://app.test` project without editing hosts or certificates manually, and can stop the project with the complete process tree torn down.

## Positioning

Native control plane that keeps working when Node is absent or being replaced; rootless daily use; explicit trust before running discovered project code; per-resource ownership (Managed vs External) that never mutates or removes foreign processes or system resources. Workflow-equivalent to category peers without copying their source, visuals, naming, or branding.

## Operating Context

Windows 10 x64 minimum (build 19045), Windows 11 x64 supported. Compact desktop developer utility kept open alongside editor and terminal. Single per-user daemon, CLI, and optional GUI; closing the GUI does not stop the daemon or running projects.

## Capabilities and Constraints

- Project registration via Park directory, Link project, and `nerd.json` (committed, never carries secrets).
- Multi-version Node runtime management with per-project resolution order (`nerd.json` → `.nvmrc` → `.node-version` → `engines.node` → default).
- Reverse proxy for `.test` HTTPS via NRPT routing.
- Request Inspector (metadata + opt-in body capture) and Mail capture (local-only SMTP sink).
- Managed services (MySQL, PostgreSQL, Redis): blocked pending `OD-002`, `OD-003`, `OD-004`.
- Hard constraints: loopback binding by default; project processes never elevated; secrets under DPAPI; no Electron, Docker, or bundled Chromium; no telemetry; Windows native filesystem events instead of polling.
- Open decisions: see `context/open-decisions.md`; not all are blocking MVP.

## Brand Commitments

Name: Nerd. Voice: terse, technical, calm — never marketing. License: MIT. No account, subscription, or telemetry. UI tokens, typography, color, spacing, and density are bound by `context/ui-tokens.md`, `context/ui-rules.md`, and `context/ui-registry.md`. System fonts only (Segoe UI Variable + Cascadia Mono); semantic CSS variable tokens; no hardcoded colors or raw Tailwind palette classes in components.

## Evidence on Hand

- `context/project-overview.md` — full product spec and MVP feature list.
- `context/architecture.md` — system architecture, process model, data layout.
- `context/trust-model.md` — trust levels, threats, project approval workflow.
- `context/versioning.md` — code and application versioning, Conventional Commits.
- `context/compatibility.md` — support matrix.
- `context/ui-tokens.md`, `context/ui-rules.md`, `context/ui-registry.md` — UI design system.
- `context/features/` — feature specifications 01–12.
- `context/decisions/` — accepted ADRs.
- `schemas/ipc.schema.json` — IPC contract.

## Product Principles

1. Lightweight while idle; native control plane stays working without Node.
2. Rootless daily use; elevation limited to typed setup only.
3. Reproducible projects via `nerd.json`; secrets never committed.
4. Explicit lifecycle and trust before executing discovered code.
5. Coexistence: never mutate, repair, or remove foreign resources.

## Accessibility & Inclusion

WCAG 2.1 AA for desktop UI. Full keyboard navigation, visible focus, screen reader labels for icon-only controls, minimum 28px compact target, system theme following with manual override, `prefers-reduced-motion` honored, status conveyed by color plus text plus icon (never color alone).
