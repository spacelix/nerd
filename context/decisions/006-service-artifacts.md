# ADR 006: Managed Service Artifact Sources

- Status: Proposed, blocking Feature 10
- Date: 2026-08-14

## Context

Nerd promises native Windows services without Docker or WSL. Artifact availability and redistribution differ by engine.

## Findings

### PostgreSQL

PostgreSQL.org directs Windows users to EDB. EDB provides an advanced-user ZIP explicitly described for inclusion with another application installer. Exact supported desktop Windows versions, URL stability, signatures/checksums, and redistribution terms still need approval.

Source: https://www.postgresql.org/download/windows/

### MySQL

Oracle/MySQL publishes Windows Community downloads, including archive distributions, but automated metadata access, checksum retrieval, supported-version policy, and redistribution terms need exact verification before implementation.

Source: https://dev.mysql.com/downloads/mysql/

### Redis

Redis documentation does not provide an official Redis OSS native Windows binary. It directs native Windows use to Memurai, an official Windows compatibility partner, or to Redis under WSL. WSL is excluded by Nerd architecture.

Source: https://redis.io/docs/latest/operate/oss_and_stack/install/archive/install-redis/install-redis-on-windows/

Garnet is an MIT-licensed Microsoft RESP-compatible cache with Windows ReadyToRun artifacts, but it is not Redis and must be named/configured as Garnet rather than silently substituted.

Source: https://github.com/microsoft/garnet

## Proposed Decision

- PostgreSQL and MySQL remain MVP candidates pending exact artifact and legal verification.
- Managed Redis is blocked.
- Do not use WSL, unofficial abandoned Redis ports, or a silent Garnet substitution.
- Product choice required: license Memurai, replace MVP Redis with explicitly named Garnet, move managed Redis post-MVP while retaining external Redis connections, or remove cache service from MVP.

## Acceptance Gate

Each promoted engine records source API, versions, architecture, license, checksum/signature, archive layout, configuration, readiness, shutdown, backup, update, and Windows 10 support in `context/library-docs.md`.
