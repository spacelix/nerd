# Feature 16: MCP And AI-Agent Integration

## Horizon

Post-MVP v1.x candidate.

## Goal

Expose safe Nerd inspection and lifecycle tools to local AI agents through Model Context Protocol without creating a broader privilege boundary.

## User Outcome

After explicit opt-in, a local agent can inspect project status, logs, diagnostics, services, mail metadata, and request metadata, then request approved lifecycle actions.

## In Scope

- Local stdio MCP server process
- Explicit enable/disable setting
- Read-only tools by default
- Separately approved start/stop/restart and safe-repair tools
- Structured project, runtime, service, log, mail, inspector, and diagnostic resources
- Confirmation policy for mutation and sensitive-content access
- Audit log of agent-requested mutations

## Out Of Scope

- Remote MCP transport
- Arbitrary shell execution
- Reading project source files through Nerd
- Returning secrets, raw credentials, private keys, or unredacted captures
- Autonomous destructive repair

## Security Rules

- MCP process is a daemon client under the current user, not a privileged service.
- Tool surface maps to typed IPC operations.
- Read access to mail bodies or captured request bodies is separately gated.
- Mutation tools are disabled by default.
- Helper operations never become directly callable through MCP.

## Open Decisions

- Per-tool permission model and confirmation UX.
- Content size limits and redaction for agent context.
- Whether enabled state is global or scoped by agent configuration.

## Acceptance Criteria

- Disabled mode exposes no MCP tools through Nerd command discovery.
- Agent cannot execute arbitrary commands or access decrypted credentials.
- Mutation attempts follow configured confirmation policy and audit trail.
- Existing CLI/desktop IPC authorization remains unchanged.
- Large logs and captures use bounded pagination.

## Dependencies

- MVP Features 01, 07-10, and 12
