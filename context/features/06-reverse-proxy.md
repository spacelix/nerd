# Feature 06: Reverse Proxy

## Goal

Route stable HTTP and HTTPS `.test` hosts to dynamic project development servers while preserving modern development protocols.

## User Outcome

A running project is available at `http://name.test` and `https://name.test` without exposing its internal port.

## In Scope

- Loopback HTTP port 80 and HTTPS port 443
- Exact host routing for linked and parked projects
- HTTP request/response streaming
- SSE
- WebSocket upgrades and framework HMR
- TLS certificate selection and issuance integration
- Proxy headers and client address policy
- Project stopped, starting, failed, unknown-host, and upstream-failure responses
- Routing-loop prevention

## Out Of Scope

- Public/LAN access
- Path-prefix multi-upstream rules
- Caching, compression rewriting, or static asset serving
- Inspector persistence; Feature 08 owns observation

## Routing Rules

- Validate and normalize host, optional port, case, and trailing dot.
- Exact project route only for MVP.
- Unknown `.test` host returns Nerd 404.
- Stopped project returns Nerd 503 with CLI/GUI start guidance.
- Starting project returns Retry-After response.
- Never route to `.test` upstreams or Nerd listener ports.

## Proxy Rules

- Preserve streaming and backpressure.
- Set `X-Forwarded-Host`, `X-Forwarded-Proto`, and `X-Forwarded-For` consistently.
- Do not trust incoming forwarded headers from local clients by default.
- Apply connect, header, and idle timeouts appropriate to dev servers.
- WebSocket and SSE cannot be buffered by inspector.

## Acceptance Criteria

- HTTP and trusted HTTPS work for each framework fixture.
- Vite and Next HMR remain connected through proxy.
- WebSocket echo and SSE streaming pass without full buffering.
- Large streaming response respects bounded memory.
- Malformed hosts and proxy loops are rejected.
- Foreign port conflicts are reported without mutation.

## Dependencies

- Feature 02 network and TLS foundation
- Features 04 and 05 project routes and upstream state
