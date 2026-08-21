# Library Docs

This file records approved third-party dependencies and project-specific usage constraints. It is not a substitute for current official documentation.

## Before Adding Or Using A Dependency

1. Check available installed skills and MCP documentation.
2. Read current official docs for the exact installed version.
3. Confirm standard library or existing dependency cannot solve the problem cleanly.
4. Record package, version policy, purpose, security boundary, and project usage here.
5. Add license and vulnerability checks to CI when relevant.

No dependency is approved merely because it appears in this planning document.

## Approved Stack Decisions

| Area | Decision | Notes |
|---|---|---|
| Core language | Rust | Native daemon, CLI, helper |
| Async model | Tokio | Exact version chosen during foundation feature |
| Desktop shell | Tauri 2 | Must retain WebView2 and small-bundle model |
| UI | React + TypeScript | UI only; daemon owns state |
| Persistence | SQLite | Exact Rust driver chosen during foundation feature |
| Styling | Tailwind CSS v4 or token-equivalent CSS | Must honor `ui-tokens.md` |

## Selection Constraints

### DNS

- Must support UDP and TCP.
- Must parse safely and answer authoritative local `.test` records only.
- Must not introduce public DNS forwarding.
- Fuzzable parser preferred.

### HTTP Proxy

- Must support HTTP/1.1, streaming, SSE, WebSocket upgrades, TLS, and backpressure.
- HTTP/2 client-facing support may follow after MVP unless framework behavior requires it.
- Inspector must observe streams without unconditional buffering.

### TLS And Certificates

- Must generate local CA and leaf certificates.
- Private-key material must support DPAPI-protected storage.
- Windows trust-store changes remain in helper boundary.

### SQLite

- Must support transactions and migrations.
- Prefer a mature driver without embedding business logic in macros that obscure SQL behavior.
- Connection access stays inside daemon repositories.

### Filesystem Watching

- Must use Windows native change notifications.
- Recursive watching is unnecessary for parked roots; only immediate project detection is required.
- Events must be debounced and followed by state reconciliation.

### Process Management

- Must support Windows Job Objects or allow a small isolated Windows implementation.
- Generic cross-platform process termination alone is insufficient.

### MIME And SMTP

- Parser must enforce size and nesting limits.
- HTML mail rendering must be sandboxed in UI.
- Remote images are blocked by default.

### Archives And Downloads

- ZIP extraction must reject traversal and escaping links.
- HTTP client must support proxy environment variables and explicit timeouts.
- Checksums are mandatory before promotion.

## Node Distribution Rules

- Release metadata: official Node.js release index.
- Artifacts: official Windows x64 ZIP.
- Integrity: official checksum list over HTTPS.
- npm comes from Node distribution.
- Corepack is bundled only with Node 14.19 through versions before 25. Nerd must manage a separate isolated Corepack for Node 25+ when pnpm or Yarn is requested.
- Respect the project's `packageManager` field and pinned version.

## Feature 01 Dependencies

Versions are exact for the foundation implementation. Updating one requires reading the new official documentation and rerunning all Feature 01 checks.

### tokio

- Version policy: `=1.53.1`
- Feature: daemon runtime, named-pipe I/O, signals, bounded channels, and timers
- Enabled features: `io-util`, `macros`, `net`, `rt`, `signal`, `sync`, `time`
- Official docs: https://docs.rs/tokio/1.53.1/tokio/
- Security boundary: named-pipe servers use explicit security attributes, reject remote clients, and enforce bounded frames and connections
- Allowed modules: `nerd-daemon`, `nerd-cli`
- Prohibited usage: unbounded channels, detached tasks, and blocking SQLite work on the runtime thread
- Verification: IPC concurrency, cancellation, shutdown, and idle-resource tests

### rusqlite

- Version policy: `=0.40.2`
- Feature: daemon-owned SQLite state and transactional migrations
- Enabled features: `bundled`; default features disabled
- Official docs: https://docs.rs/rusqlite/0.40.2/rusqlite/
- Security boundary: one dedicated database worker owns the connection; SQL remains inside daemon state repositories
- Allowed modules: `nerd-daemon::state`
- Prohibited usage: direct access from CLI, helper, desktop, or async runtime tasks
- Verification: foreign-key, unsupported-version, rollback, and repository integration tests

### serde and serde_json

- Version policy: `serde = =1.0.229`, `serde_json = =1.0.151`
- Feature: strict IPC serialization and structured state values
- Enabled features: Serde `derive` and `std`; serde_json `std`; default features disabled
- Official docs: https://docs.rs/serde/1.0.229/serde/ and https://docs.rs/serde_json/1.0.151/serde_json/
- Security boundary: all IPC objects deny unknown fields; protocol boundaries do not accept unrestricted `Value`
- Allowed modules: workspace crates where typed serialization is required
- Prohibited usage: secret serialization, unbounded recursion, and protocol types that bypass `ipc.schema.json`
- Verification: round-trip, malformed input, unknown-field, and schema contract tests

### tracing and tracing-subscriber

- Version policy: `tracing = =0.1.44`, `tracing-subscriber = =0.3.23`
- Feature: structured daemon diagnostics
- Enabled features: tracing `std`; subscriber `fmt` and `json`; default features disabled
- Official docs: https://docs.rs/tracing/0.1.44/tracing/ and https://docs.rs/tracing-subscriber/0.3.23/tracing_subscriber/
- Security boundary: fields are explicitly selected and must never contain secrets, raw SQL, credentials, or private request content
- Allowed modules: `nerd-daemon`
- Prohibited usage: environment-controlled filters in privileged contexts and secret-bearing `Debug` values
- Verification: JSON-lines output, 10 MiB rotation, five-generation retention, fallback, and shutdown flush tests

### windows-sys

- Version policy: `=0.61.2`
- Feature: narrow Win32 integration for paths, security descriptors, mutexes, debug output, and process metrics
- Enabled feature families: Foundation, Security, Authorization, COM, Debug, Pipes, Process Status, Threading, and Shell
- Official docs: https://docs.rs/windows-sys/0.61.2/windows_sys/ and linked Microsoft Learn API pages
- Security boundary: all unsafe calls stay in focused Windows modules with owned-handle and allocated-memory cleanup
- Allowed modules: Windows-specific modules in `nerd-daemon` and named-pipe peer verification in `nerd-cli`
- Prohibited usage: project-controlled arguments, default pipe ACLs, inherited handles, and mutation without postcondition checks
- Verification: current-user ACL, cross-user denial, global guard, data-path, cleanup, and resource-metric tests

### uuid

- Version policy: `=1.24.0`
- Feature: daemon instance, request, operation, and connection identities
- Enabled features: `serde`, `std`, `v4`; default features disabled
- Official docs: https://docs.rs/uuid/1.24.0/uuid/
- Security boundary: identifiers are correlation values, never authentication tokens
- Allowed modules: `nerd-core`, `nerd-daemon`, `nerd-cli`
- Prohibited usage: authorization or ownership proof
- Verification: typed serialization and request-correlation tests

### jsonschema

- Version policy: `=0.49.9`, development dependency only
- Feature: validate Rust IPC fixtures against the canonical JSON Schema
- Enabled features: none; default features disabled
- Official docs: https://docs.rs/jsonschema/0.49.9/jsonschema/
- Security boundary: no runtime schema loading, network resolution, or user-provided schemas
- Allowed modules: `nerd-core` contract tests
- Prohibited usage: runtime IPC validation or remote reference resolution
- Verification: every request, response, error, and event variant has valid and invalid fixtures

### cargo-deny

- Version policy: `=0.20.2`, CI/development tool only
- Feature: dependency advisory, license, duplicate, wildcard, and source-policy checks
- Official docs: https://embarkstudios.github.io/cargo-deny/ and https://github.com/EmbarkStudios/cargo-deny/releases/tag/0.20.2
- Security boundary: checks the committed Cargo dependency graph; it is not shipped with Nerd and does not run in the daemon
- Allowed usage: Windows-native CI and explicit local dependency review
- Prohibited usage: runtime dependency resolution, automatic policy exceptions, or unreviewed advisory suppression
- Verification: `cargo deny check advisories bans licenses sources`

## Feature 02 Dependencies

Versions are exact for the Feature 02 implementation. Updating one requires reading the new official documentation and rerunning all Feature 02 checks.

### hickory-server

- Version policy: `=0.26.1`
- Feature: authoritative in-memory DNS server for the `.test` zone in the daemon
- Why needed: safe, fuzzable DNS parsing and serving over UDP and TCP; wildcard `*.test` A records are natively supported by the in-memory zone handler
- Official docs: https://docs.rs/hickory-server/0.26.1/hickory_server/
- Security boundary: binds loopback only; serves only the `.test` zone; non-`.test` names receive a negative response; no public forwarding
- Allowed modules: `nerd-daemon::dns`
- Prohibited usage: binding non-loopback addresses, forwarding queries upstream, and dynamic zone reload from untrusted input
- Verification: UDP and TCP `.test` resolution, negative answers, port-conflict diagnostic, and sleep/resume re-probe tests

### rcgen

- Version policy: `=0.14.9`
- Feature: generate the per-user root CA and leaf certificates (default features: `crypto`, `pem`, `ring`)
- Why needed: pure-Rust CA and leaf generation with SAN, key usage, and validity control
- Official docs: https://docs.rs/rcgen/0.14.9/rcgen/
- Security boundary: private-key material is DPAPI-protected before persistence and never enters logs or IPC
- Allowed modules: `nerd-daemon::cert`
- Prohibited usage: persisting keys unencrypted, emitting SANs beyond validated hostnames, and certificate reuse across unrelated identities
- Verification: CA and leaf generation, trust install/probe/remove, and TLS handshake tests

### time

- Version policy: `=0.3.55`, default features disabled
- Feature: certificate validity (`OffsetDateTime`, `Duration`) required by rcgen
- Why needed: rcgen `CertificateParams` uses `time::OffsetDateTime` for `not_before`/`not_after`
- Official docs: https://docs.rs/time/0.3.55/time/
- Security boundary: no untrusted input parsing
- Allowed modules: `nerd-daemon::cert`
- Verification: leaf renewal threshold tests

### windows-sys (Feature 02 feature additions)

- Existing policy: `=0.61.2`; add feature families `Win32_Security_Cryptography` (DPAPI `CryptProtectData`/`CryptUnprotectData`, cert-store APIs) and `Win32_System_Registry` (NRPT rule mutation)
- Feature: CA key DPAPI protection, CurrentUser trust-store install/probe/remove, NRPT registry mutation in the helper
- Official docs: https://docs.rs/windows-sys/0.61.2/windows_sys/ and linked Microsoft Learn API pages
- Security boundary: unsafe calls stay in focused Windows modules; NRPT writes only the Nerd `.test` rule with ownership comment; cert operations only touch the Nerd-owned fingerprint
- Allowed modules: `nerd-daemon::cert`, `nerd-daemon::dns`, `nerd-helper::nrpt`
- Prohibited usage: deleting registry trees outside the Nerd-owned NRPT rule key, and removing certificates without fingerprint ownership match
- Verification: NRPT add/remove postconditions, DPAPI round-trip, and trust-store ownership tests

## Service Distribution Rules

Every service adapter must document before implementation:

- Official or trusted Windows binary source
- Supported versions
- License and redistribution constraints
- Checksum/signature mechanism
- Configuration file generation
- Initialization command
- Readiness probe
- Graceful and forced shutdown
- Upgrade and backup compatibility

Do not assume Linux archive layouts or commands apply to Windows.

## Service Artifact Research Status

### PostgreSQL

- PostgreSQL.org points Windows users to EDB installers and an advanced-user ZIP intended for inclusion with another application installer.
- Official reference: https://www.postgresql.org/download/windows/
- Not approved yet: exact URL/API stability, checksum/signature, redistribution terms, supported desktop Windows 10 versions, and silent portable configuration.

### MySQL

- Oracle/MySQL publishes Windows Community downloads and archives.
- Official reference: https://dev.mysql.com/downloads/mysql/
- Not approved yet: machine-readable release metadata, checksum retrieval, redistribution terms, supported versions, and archive lifecycle behavior.

### Redis

- Redis documentation directs native Windows users to Memurai or Redis under WSL; no official Redis OSS native Windows binary is provided.
- Official reference: https://redis.io/docs/latest/operate/oss_and_stack/install/archive/install-redis/install-redis-on-windows/
- WSL is prohibited by MVP architecture. Managed Redis is blocked pending OD-004.
- Garnet is a separate MIT-licensed RESP-compatible Microsoft engine with Windows artifacts. It may be offered only under its own name after explicit product approval, never as silent Redis substitution.
- Reference: https://github.com/microsoft/garnet

See ADR 006 before any Feature 10 dependency or adapter work.

## Prototype V2 Dependencies

`prototype2/` is a browser-only exploration of a redesigned UI for the Nerd desktop. It is **not** part of the production Nerd stack. Dependencies below are approved only for use inside `prototype2/`. They never enter `apps/desktop/`, the Rust workspace, the installer, or the updater.

Versions are recorded as the exact installed version at the time the dependency was first added. Updating one requires reading the new official documentation and rerunning all v2 verification steps.

### react-resizable-panels

- Version policy: `^3.0.4`
- Feature: N0 three-pane shell (Project Rail, Center, Inspector Rail). Resizable, collapsible, persistent layout.
- Why needed: production-grade resizable panels with keyboard accessibility, persisted layout via `autoSaveId`, and predictable behavior. Avoids reinventing resize logic and edge handling.
- Official docs: https://react-resizable-panels.vercel.app/
- Security boundary: no IPC, no OS access, no project execution; pure layout primitive.
- Allowed modules: `prototype2/src/App.tsx`, `prototype2/src/components/shell/*`.
- Prohibited usage: never imported by `apps/desktop/`, `crates/*`, or the Tauri shell.
- Verification: collapse a rail below its minimum, drag to resize, reload to confirm `autoSaveId="nerd-v2-layout"` restores sizes.

### @tanstack/react-virtual

- Version policy: `^3.13.6`
- Feature: large-list virtualization (planned for Mail inbox, Request Inspector list, Log blocks in N3+).
- Why needed: bounded memory and 60fps scroll when lists exceed a few hundred items, without coupling to a heavier data-grid.
- Official docs: https://tanstack.com/virtual/latest
- Security boundary: no IPC, no OS access.
- Allowed modules: `prototype2/src/components/{mail,inspector,logs}/*`.
- Prohibited usage: never imported outside `prototype2/`.
- Verification: render a mocked 10,000-item inbox and confirm scroll remains smooth.

### @radix-ui/react-tabs

- Version policy: `^1.1.5`
- Feature: multi-tab panes in Inspector Rail and Mail preview (N3 and N4). Unstyled, accessible primitive — Nerd provides styling through tokens.
- Why needed: keyboard-navigable tabs with proper `role="tablist"` semantics; matches accessibility requirements in `context/ui-rules.md`.
- Official docs: https://www.radix-ui.com/primitives/docs/components/tabs
- Security boundary: no IPC, no OS access.
- Allowed modules: `prototype2/src/components/{shell,inspector,mail}/*`.
- Prohibited usage: never imported outside `prototype2/`.
- Verification: keyboard `Tab` / arrow-key navigation between tabs; focus ring visible.

### @radix-ui/react-toggle-group

- Version policy: `^1.1.4`
- Feature: Working Session toggle (`Active` / `All` / `Background`) in the Project Rail header.
- Why needed: accessible single-select pill with proper `role="group"` semantics; keyboard support for arrow-key navigation between items.
- Official docs: https://www.radix-ui.com/primitives/docs/components/toggle-group
- Security boundary: no IPC, no OS access.
- Allowed modules: `prototype2/src/components/shell/working-session-toggle.tsx`.
- Prohibited usage: never imported outside `prototype2/`.
- Verification: arrow-key navigation cycles through options; `aria-pressed` updates correctly.

### V2 Dropped Dependencies

Removed from the prototype2 `package.json` compared to `prototype/package.json`:

- `@dnd-kit/core`, `@dnd-kit/modifiers`, `@dnd-kit/sortable`, `@dnd-kit/utilities` — N1+ workspace did not need DnD. If a future milestone reintroduces drag-to-reorder (e.g. project list ordering), re-evaluate against the same dependency rule.
- `@tabler/icons-react` — `lucide-react` covers all required icons and is the canonical icon set per `prototype/src` history. No tabler-specific icons are used in v2.

## Pending Dependency Record Template

```md
### package-name

- Version policy:
- Feature:
- Why needed:
- Official docs:
- Security boundary:
- Allowed modules:
- Prohibited usage:
- Verification:
```
