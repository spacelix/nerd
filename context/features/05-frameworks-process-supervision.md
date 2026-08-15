# Feature 05: Frameworks And Process Supervision

## Goal

Detect supported JavaScript frameworks and run their development processes reliably under Nerd lifecycle control.

## User Outcome

Starting a project launches the correct script with selected Node, package manager, internal port, services, logs, and readiness state. Stopping kills the complete process tree.

## In Scope

- Next.js, Vite, Nuxt, Astro, NestJS adapters
- Express/custom `PORT` adapter
- `package.json` dependency and script detection
- Command, working-directory, readiness, and port overrides
- Dynamic internal port allocation
- Windows Job Objects
- stdout/stderr capture and bounded history
- Lifecycle state machine
- Readiness probes and startup timeout
- Crash detection and guarded restart policy
- Optional project autostart after daemon startup
- Trust and Start preflight with final command, runtime, package manager, services, working directory, and environment conflicts
- Explicit confirmation for dependency installation and changed commands

## Out Of Scope

- Reverse proxy transport
- Dependency installation without confirmation
- Production build/start workflows
- Running all monorepo workspaces automatically

## Lifecycle

```text
stopped -> resolving -> starting-services -> starting-app
-> waiting-ready -> running -> stopping -> stopped
```

Any stage may transition to `failed` with stage and typed cause.

## Framework Rules

- Prefer explicit `nerd.json` script and adapter.
- Otherwise detect from dependencies and scripts.
- Never execute package scripts merely during detection.
- Never execute an untrusted project.
- Port injection must use adapter-supported arguments or `PORT`.
- `--strictPort` or equivalent is required where supported.
- Multiple conflicting lockfiles block dependency installation; Nerd never guesses a package manager.
- Native-addon build failures receive diagnostics but Nerd does not install Visual Studio Build Tools or Python automatically.

## Process Rules

- Run as current user, never elevated.
- Assign process tree to a Job Object before treating startup as successful.
- Stop graceful first, then force after deadline.
- Prevent uncontrolled restart loops with bounded attempts and cooldown.
- Logs are redacted, bounded, and optionally rotated.

## Acceptance Criteria

- Smoke fixture for every supported framework reaches ready state.
- Two projects never receive same active internal port.
- Stopping removes complete descendant process tree.
- Crash becomes visible with exit code and last logs.
- Startup timeout leaves no orphan process.
- Autostart applies only to explicitly enabled projects.
- Untrusted and materially changed projects remain stopped until preflight approval.
- Preflight reports effective environment provenance without exposing secret values.

## Dependencies

- Features 01, 03, and 04
- Feature 10 lifecycle hook may be integrated later without changing project state contract
