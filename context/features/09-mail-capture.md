# Feature 09: Mail Capture

## Goal

Capture development email per project through a local SMTP sink and provide safe inspection without delivering mail externally.

## User Outcome

Running projects receive SMTP environment values. Sent messages appear in a project inbox with HTML, text, source, headers, and attachments.

## In Scope

- Project-scoped loopback SMTP listener and dynamic port
- `NERD_MAIL_HOST` and `NERD_MAIL_PORT`
- SMTP envelope and MIME parsing
- HTML, text, headers, and attachment metadata
- Raw MIME file storage and SQLite metadata
- Count, age, and byte retention limits
- Inbox list, message view, delete, clear, and attachment save
- Sandboxed HTML viewer with remote images blocked
- CLI list/show/clear

## Out Of Scope

- Relaying or forwarding mail
- Receiving public mail
- Full mailbox protocols such as IMAP
- Automatic remote image loading

## Safety Rules

- Bind loopback only.
- Reject messages above configured total size.
- Enforce MIME nesting, header, attachment count, and filename limits.
- Sanitize attachment filenames.
- Never execute or inline active attachment content.
- HTML preview has no Node/Tauri bridge access.

## Project Identity

Each active project receives a dedicated local SMTP port. This avoids ambiguous attribution when clients do not support SMTP authentication.

## Acceptance Criteria

- Text, HTML, multipart, inline image, and attachment fixtures render correctly.
- Captured message never leaves local machine.
- Oversized and malformed messages fail safely.
- Retention deletes metadata and files consistently.
- Default retention enforces 500 messages or 250 MB per project, oldest first, unless user changes settings.
- Remote images are blocked until explicit user action.
- Stopping project closes its SMTP listener without deleting inbox.

## Dependencies

- Features 01, 05, and 07
