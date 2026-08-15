# Feature 08: Request Inspector

## Goal

Observe local HTTP traffic at the reverse proxy without requiring framework instrumentation or unbounded buffering.

## User Outcome

Users inspect live requests by project, including method, URL, status, timing, headers, query parameters, and optional bodies.

## In Scope

- Per-project enable/disable
- Live metadata capture
- In-memory ring buffer, default 500 requests per project
- Request and response headers
- Query parameters
- Timing and upstream outcome
- Optional request/response body capture
- 1 MB per-body limit
- Content-type allowlist
- Automatic secret redaction
- Clear/export-safe-metadata actions
- Desktop split-view and CLI stream/list/show

## Out Of Scope

- Full WebSocket frame capture
- Persistent capture by default
- HAR fidelity guarantee
- TLS interception outside Nerd proxy

## Capture Rules

- Metadata capture must not buffer streaming bodies.
- Body capture is off by default.
- Capture only textual/structured allowlisted content types.
- Truncation records original-known size and captured size.
- SSE and WebSocket record lifecycle metadata only.
- Backpressure and application behavior must remain unchanged.

## Redaction

Always redact:

- Authorization and proxy authorization
- Cookie and set-cookie
- Password, passwd, secret, token, api-key, access-key patterns
- Service connection credentials

Redaction occurs before event emission or persistence.

## Acceptance Criteria

- Metadata is correct for success, upstream error, timeout, and aborted client.
- Ring buffer evicts oldest deterministically.
- Bodies larger than 1 MB are truncated without high memory use.
- Secret fixtures never appear in logs, IPC events, UI, or export.
- HMR, SSE, and WebSocket behavior remains intact with inspector enabled.
- Disabled project incurs minimal measurable overhead.

## Dependencies

- Feature 06
- Feature 07 for final desktop UI
