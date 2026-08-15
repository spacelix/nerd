# Code And Application Versioning

## Source-Code Versioning

Nerd uses Git as the source-code versioning system with a trunk-based workflow.

### Branches

- `main` is the single long-lived branch and must remain releasable.
- Work happens on short-lived branches created from current `main`.
- No permanent `develop` branch.
- After the initial repository bootstrap, do not commit or push directly to `main`.
- Push work branches to the remote and integrate them through pull requests; pushing a branch does not publish a release.
- Rebase or update a branch before merge according to repository policy; never rewrite shared history without explicit coordination.

Preferred branch names:

```text
feat/<short-name>
fix/<short-name>
docs/<short-name>
refactor/<short-name>
perf/<short-name>
test/<short-name>
build/<short-name>
ci/<short-name>
chore/<short-name>
release/<version>
```

Examples:

```text
feat/node-version-resolver
fix/websocket-upgrade-headers
docs/post-mvp-roadmap
release/1.0.0
```

### Commits

- Every commit follows Conventional Commits 1.0.0.
- One logical change per commit.
- Commits must build toward an independently reviewable state.
- Generated credentials, runtime binaries, databases, captures, and machine-local state are never committed.
- Lockfiles are committed when the matching ecosystem requires reproducible builds.

### Integration

- Every post-bootstrap change reaches `main` through a pull request, including changes made by a sole maintainer.
- Required `rust` and `dependencies` CI checks must pass before merge.
- Documentation-only pull requests keep those required check contexts but complete them as lightweight no-op checks.
- Resolve review conversations and inspect the complete branch diff before merge.
- Required approval count may remain zero for a sole-maintainer repository, but the pull-request boundary is still mandatory.
- Delete short-lived branches after merge.
- Direct changes to `main` are break-glass recovery only and require an explicitly documented governance decision.
- Required formatting, linting, tests, security checks, and resource gates pass before merge.
- Merge strategy should preserve useful Conventional Commit history or produce a compliant squash commit.
- Release commits contain release metadata only; feature work does not hide inside release commits.

Repository settings must protect `main` by requiring pull requests and the required CI checks, blocking force pushes and deletion, and requiring conversation resolution. If the hosting plan cannot enforce a rule, the limitation is documented and the same rule remains mandatory by process.

### Tags

- Application releases use annotated, preferably signed Git tags.
- Tag format is `vMAJOR.MINOR.PATCH` with optional SemVer prerelease suffix.
- Tag must point at the exact commit used to build published artifacts.
- Tags are immutable after publication. A bad release receives a new patch version; its tag is not moved.

Examples:

```text
v1.0.0-beta.1
v1.0.0-rc.1
v1.0.0
v1.1.0
v1.1.1
```

## Application Version

Nerd follows Semantic Versioning 2.0.0:

```text
MAJOR.MINOR.PATCH[-PRERELEASE]
```

- **MAJOR**: incompatible public behavior, CLI contract, project manifest contract, or supported-platform contract.
- **MINOR**: backward-compatible feature or meaningful capability expansion.
- **PATCH**: backward-compatible bug, security, performance, packaging, or documentation fix.

Desktop, daemon, CLI, helper, installer, and updater belong to one Nerd application release and share the same application version.

### Version Source Of Truth

- Root Rust workspace version is the canonical application version.
- Rust crates use the workspace version rather than independent package versions.
- Desktop package metadata, Tauri bundle metadata, Windows executable metadata, and installer metadata are synchronized from the canonical version by release tooling.
- CI fails when any application component reports a different version.
- Frontend package is private and is not versioned as an independently published npm product.

Every application surface reports the same version:

- `nerd --version`
- daemon status and IPC handshake metadata
- desktop About screen
- Windows Apps & Features
- installer filename and release manifest

Do not manually maintain duplicate version strings when build tooling can derive them.

## Release Stages

| Stage | Version example | Meaning |
|---|---|---|
| Early development | `0.1.0-alpha.1` | Internal foundation, no stability promise |
| Public preview | `1.0.0-beta.1` | MVP feature-complete, validation continues |
| Release candidate | `1.0.0-rc.1` | Release gates pass; only release-blocking fixes |
| MVP stable | `1.0.0` | Features 01-12 complete and supported |
| Post-MVP | `1.1.0`, `1.2.0` | Backward-compatible v1.x roadmap features |
| Platform expansion | `2.0.0` target horizon | Major portability and contract review |

Roadmap horizons are planning labels. Exact release assignment is decided when a candidate is promoted.

## Application Release Flow

1. Confirm intended commits and Conventional Commit categories since previous release tag.
2. Select next SemVer based on user-visible and breaking impact.
3. Update canonical application version and generated release metadata.
4. Run full Windows 10/11, security, migration, installer, update, rollback, uninstall, and performance gates.
5. Create release commit using `chore(release): prepare vX.Y.Z`.
6. Create annotated signed tag `vX.Y.Z` from that commit.
7. Build all artifacts from tagged commit only.
8. Publish checksums, signatures, compatibility versions, and release notes.
9. Verify updater sees and validates published release.

Artifact filenames include application version and architecture, for example:

```text
Nerd_1.0.0_windows_x64.exe
Nerd_1.0.0_windows_x64.msi
```

## Internal Compatibility Versions

Application version must not be reused for internal protocols or persisted schemas.

### IPC Protocol

- Monotonic positive integer.
- Desktop and CLI handshake before commands.
- Increment when wire compatibility changes.
- Product release may support a bounded compatibility range during safe rolling update.

### `nerd.json` Schema

- Required integer `version` field.
- Increment when accepted file shape or semantics change.
- Older schemas migrate in memory or receive actionable migration guidance.
- Newer unsupported schemas fail clearly and are never rewritten.

### SQLite Migrations

- Ordered, immutable migration IDs.
- Applied transactionally where SQLite permits.
- Never edit an applied migration; add a new migration.
- Database schema version is independent from product, IPC, and manifest versions.

### Artifact Metadata

- Runtime and service artifacts retain upstream version, architecture, source, checksum, and Nerd adapter metadata.
- Never relabel an upstream artifact with Nerd application version.

## Compatibility Policy

- Patch and minor releases preserve existing valid `nerd.json` behavior.
- Persisted state migrations are forward-only and rollback-safe through backup/staging.
- Downgrade across an unsupported database or manifest schema fails safely.
- Breaking CLI flags, JSON output, IPC messages, or manifest semantics require explicit migration notes and normally a major release.
- Security fixes may disable unsafe behavior in a patch release when preserving it would expose users.

## Conventional Commits

Every commit message must follow Conventional Commits 1.0.0.

```text
<type>[optional scope][!]: <description>

[optional body]

[optional footer(s)]
```

### Allowed Types

| Type | Use |
|---|---|
| `feat` | User-visible capability |
| `fix` | Bug or incorrect behavior |
| `docs` | Documentation only |
| `refactor` | Internal change without behavior change |
| `perf` | Measured performance improvement |
| `test` | Tests and fixtures only |
| `build` | Build system or dependencies |
| `ci` | Continuous integration |
| `chore` | Maintenance not covered above |
| `revert` | Revert an earlier commit |

### Preferred Scopes

```text
core, daemon, ipc, cli, helper, desktop, dns, tls, proxy,
runtime, projects, services, mail, inspector, installer, docs
```

Add a scope only when it is stable and meaningful. Do not use feature numbers as scopes.

### Message Rules

- Lowercase type and scope.
- Imperative, concise description without trailing period.
- One logical change per commit.
- Body explains why and tradeoffs when subject is insufficient.
- Reference issues in footers when applicable.
- Never include secrets, generated credentials, or private paths.
- Breaking changes use `!` and a `BREAKING CHANGE:` footer with migration guidance.

### Examples

```text
feat(runtime): install pinned node versions
fix(proxy): preserve websocket upgrade headers
perf(inspector): bound response body buffering
docs(roadmap): define post-mvp release horizons
feat(ipc)!: replace operation envelope format

BREAKING CHANGE: desktop and CLI clients must use IPC protocol 3.
```

## Release Notes

- Conventional Commits are the source for generated changelog sections.
- Release notes group breaking changes, features, fixes, performance, and security.
- Internal-only commit types may be omitted from user-facing notes.
- Every release records application version, supported IPC range, current `nerd.json` schema, and highest SQLite migration.
