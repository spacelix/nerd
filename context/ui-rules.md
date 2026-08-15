# UI Rules

## Visual Direction

Compact, calm Windows developer utility. Prioritize scan speed, status clarity, and keyboard operation. Avoid oversized cards, decorative gradients, glassmorphism, marketing-style empty space, and mobile-first page composition.

## Window Layout

- Native Tauri window with 40px title bar.
- Left navigation: Overview, Projects, Runtimes, Services, Mail, Inspector, Diagnostics, Settings.
- Main content uses full available window width.
- Toolbars stay near the content they control.
- Detail views may use split panes for list and inspection content.
- Minimum supported window size must be defined before UI implementation.

## Density

- Default controls are 32px high.
- Tables and project lists use 38px rows.
- Use cards only for grouped status or action boundaries.
- Prefer tables, property grids, and split views for dense data.
- Do not nest more than two bordered surfaces.

## Navigation

- Current destination uses selected surface and accent indicator.
- Every destination has icon and text in expanded mode.
- Sidebar collapse is optional and remembered locally.
- Browser-like back navigation is unnecessary; detail views provide explicit back action.

## Status

- Never rely on color alone. Pair color with text or icon.
- Project lifecycle labels use exact daemon states.
- Starting and installing states show progress when measurable.
- Failures show stage, human-readable cause, and safe next action.

## Actions

- One primary action per toolbar or dialog.
- Destructive actions require confirmation naming affected resource and retained data.
- Start/stop remains available from project row, detail screen, tray, and CLI.
- Async buttons disable duplicate submission and retain visible progress.

## Logs

- Monospace, virtualized, searchable, and selectable.
- Default follows latest output only while user remains at bottom.
- stdout and stderr are distinguishable without sacrificing readability.
- Secrets are redacted before UI receives events.

## Inspector

- Request list and request detail use split view.
- Preserve live updates without stealing current selection.
- Headers, query, body, timing, and response are separate detail sections.
- Body truncation and redaction must be explicit.

## Mail

- Inbox list and message preview use split view.
- HTML preview is sandboxed.
- Remote images blocked by default.
- Text and source views always available.
- Attachments require explicit save/open action.

## Forms

- Labels remain visible; placeholders are examples, not labels.
- Validation occurs near the field and again in daemon.
- Generated ports and credentials are read-only with copy actions.
- Secret values are hidden by default and never copied implicitly.

## Empty States

- State what is absent and provide one direct next action.
- No illustrations required.
- Examples: Park a folder, Install Node, Start a project, Enable Inspector.

## Accessibility

- Full keyboard navigation.
- Visible focus state.
- Semantic labels and accessible names for icon-only actions.
- Minimum target size 28px for compact controls.
- Screen-reader announcements for lifecycle and long-operation completion.
- Respect system text scaling, contrast mode where possible, and reduced motion.

## Responsive Desktop Behavior

- This is not a mobile app.
- At narrow desktop widths, collapse sidebar and secondary columns before reducing control usability.
- Split views may switch to stacked detail navigation below the defined minimum comfortable width.

## Component Rule

Before building a reusable component, read `ui-registry.md`. Reuse an existing pattern when present. After adding a component, record path, purpose, variants, token usage, and accessibility behavior.
