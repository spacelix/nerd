# Code Standards

## General

- Build only the active feature described in `context/features/`.
- Prefer the smallest correct change.
- Keep business decisions pure and isolate OS side effects.
- Every feature must have immediate verification and explicit acceptance criteria.
- No hidden fallback that changes security, ports, or persistence semantics.
- No TODO comments in completed feature code.
- ASCII by default in source and protocol data.

## Rust

- Stable Rust, latest project-pinned toolchain.
- `rustfmt` and strict `clippy` must pass.
- Avoid `unsafe`. Any required `unsafe` must be isolated, documented with invariants, and reviewed.
- Domain failures use typed errors; avoid string matching.
- Libraries return errors; binary boundaries render user-facing messages.
- Do not hold locks across `.await`.
- Every spawned task has ownership, cancellation, and shutdown behavior.
- Every child process has stdout/stderr handling and termination policy.
- Prefer bounded channels over unbounded channels.
- Secrets must not implement or pass through debug logging.

## TypeScript And React

- TypeScript strict mode.
- Never use `any`; use `unknown` and narrow.
- Named exports for components and utilities.
- React components render state and dispatch typed IPC actions; no OS commands or direct persistence.
- Do not mirror daemon state independently. Derive UI state from snapshots and events.
- Avoid `useMemo` and `useCallback` by default; use only for measured need or library contracts.
- Effects require cleanup when they subscribe, schedule, or allocate.
- All user-visible async actions have pending, success, and failure states.

## IPC

- Schema is versioned and shared.
- Every operation has typed request, response, and error variants.
- Never expose Rust internal error strings directly to UI.
- Reject unknown operation names and incompatible protocol versions.
- Progress events include operation ID and monotonically increasing sequence.
- Cancellation must state whether operation is cancellable and what partial state remains.

## SQLite

- All schema changes use migrations.
- Migrations are transactional when SQLite permits.
- Foreign keys enabled.
- Repository functions own SQL; UI, CLI, and network modules do not embed queries.
- Secrets are encrypted before persistence.
- Store timestamps in UTC and render local time at UI boundary.

## Windows Integration

- Centralize Windows-specific code behind narrow traits or modules.
- Use native filesystem notifications, not polling.
- Treat PID plus process creation identity as process identity.
- Project process trees use Job Objects.
- Registry, NRPT, certificate, and PATH writes carry Nerd ownership markers.
- Probe current state before mutation and verify postconditions afterward.
- Every setup mutation has an uninstall inverse and integration test.
- Never infer ownership from executable name, process name, port, or installation path alone.
- Mutations require verified Nerd ownership metadata plus matching live identity or fingerprint where applicable.

## Privileged Helper

- Helper receives enum-like operations, never arbitrary commands.
- Strictly parse all arguments and reject extras.
- Canonicalize and validate paths against allowed Nerd directories where applicable.
- Never execute project-controlled files.
- Log operation type and result, never secrets.
- Elevation is never used for project, Node, package manager, or database processes.

## Downloads And Extraction

- HTTPS only.
- Allowlisted origins per artifact type.
- Verify checksum or signature before use.
- Download to temporary file, verify, extract to temporary directory, then atomically promote.
- Reject archive traversal, absolute paths, links escaping destination, and unexpected file types.
- Partial downloads are resumable only when integrity remains verifiable.

## Networking

- Bind loopback unless a feature specification explicitly says otherwise.
- Validate hostnames, ports, headers, and size limits before routing.
- Preserve streaming and backpressure.
- Set timeouts for upstream connect and readiness checks.
- Do not buffer entire request or response unless body capture is enabled and within limit.
- Never terminate a foreign process to free a port.

## Logging And Privacy

- Structured logs with component and operation IDs.
- Redact credentials, authorization, cookies, tokens, connection strings, and private paths where user-facing.
- Log rotation and retention are mandatory.
- Request and mail content are local only.
- No telemetry or remote analytics.

## UI Styling

- Use tokens from `ui-tokens.md`; no hardcoded colors in components.
- No raw Tailwind palette classes.
- Desktop utility density; avoid oversized marketing layouts.
- Keyboard and screen-reader behavior are required, not follow-up work.
- Update `ui-registry.md` after each new reusable component.

## Tests

- Pure decisions: table-driven unit tests.
- State: migration and repository integration tests.
- IPC: schema, compatibility, and ACL tests.
- Windows setup: clean install, probe, repair, and uninstall tests.
- Network: DNS UDP/TCP, HTTP, HTTPS, SSE, WebSocket, HMR, and conflict tests.
- Processes: crash, cancellation, restart loop, and process-tree cleanup tests.
- Services: version, isolation, health, backup, and recovery tests.
- UI: interaction and accessibility tests for critical workflows.
- Release gates include Windows 10 x64, Windows 11 x64, and performance budgets.

## Documentation Workflow

- Update active feature file when implementation reveals a changed decision.
- Update `progress-tracker.md` when feature status changes.
- Update `ui-registry.md` after reusable UI additions.
- Update `library-docs.md` before introducing a new third-party dependency.
- Record architecture-wide decisions in `architecture.md`, not only feature notes.
- Record durable cross-feature choices as ADRs under `decisions/`.
- Record unresolved choices in `open-decisions.md`; never guess through a blocking item.
- Update `compatibility.md` before making a new support claim.

## Versioning And Commits

- Follow `versioning.md` for Git workflow, application releases, IPC, manifest, and database version boundaries.
- Source code uses trunk-based Git development with short-lived branches.
- Application releases use Semantic Versioning 2.0.0 and immutable `vX.Y.Z` tags.
- Desktop, daemon, CLI, helper, installer, and updater share one application version.
- Do not couple application version to IPC protocol, `nerd.json` schema, or SQLite migration IDs.
- Every commit uses Conventional Commits 1.0.0.
- Commit type and scope must describe the actual logical change.
- Breaking changes require `!` and a `BREAKING CHANGE:` footer with migration guidance.
