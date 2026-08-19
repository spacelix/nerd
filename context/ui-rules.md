# UI Rules

## Direction

Nerd is a quiet, dense Windows developer utility. It must not look like a SaaS dashboard, a marketing site, or a settings wizard. It must look like a tool a careful engineer built for themselves: considered typography, restrained color, generous whitespace where the work breathes, dense where data demands it.

The chrome disappears so the projects do not.

The prototype uses [shadcn/ui](https://ui.shadcn.com) as the design system foundation. Components consume shadcn primitives; the chrome (sidebar, header, dialog, button, badge, separator, breadcrumb, tooltip, dropdown, command) comes from `src/components/ui/`. See `ui-registry.md` for the inventory and `ui-tokens.md` for the OKLCH token map.

## Window layout

- Native Tauri 2 window with a 40px draggable title bar.
- Title bar holds the brand mark on the left and global state (daemon health, theme switcher) on the right.
- Left navigation: Overview, Projects, Runtimes, Services, Mail, Inspector, Diagnostics, Settings. Grouped into Workspace / Observability / System.
- Main content area takes the rest of the window. No artificial max width on tables or lists.
- Minimum window width 960px. Below that, sidebar collapses to off-canvas (fully hidden) by default.
- App shell (`App.tsx`) is wrapped in `h-svh w-full flex-col overflow-hidden` so the chrome is sticky and only the content area scrolls.

## Density

- Default controls are 32px high; compact 28px.
- Tables and lists use 38px rows. Comfortable lists use 44px rows.
- Spacing between grouped elements: 16–24px. Spacing between items in a list: 0px (rows are tight, dividers carry the structure).
- One bordered surface at a time. Do not nest cards inside cards.
- Use cards only when a region carries its own status or action boundary.

## Sidebar

- `collapsible="offcanvas"` — sidebar fully hides on collapse (no icon rail).
- Sections labeled with eyebrow text (10px, 0.16em tracking, uppercase, color text-faint).
- Items: icon + label. Active item uses `surface-selected` background, semibold label, and a filled icon background in `--color-surface`. Hover uses `--color-surface-hover`.
- No accent-colored left bar, no border-left accent, no glow. Multi-cue active state only.
- No user footer (Nerd has no authentication). No bottom nav. Quick Create is the only primary action in the sidebar.

## Top toolbar (SiteHeader)

- 48px tall, hairline divider at the bottom (`border-b border-border/40`).
- Left: SidebarTrigger + vertical Separator + Breadcrumb (Workspace › Current page).
- Right: Search button (triggers CommandPalette with `⌘K` hint) + Theme dropdown.
- One primary action per screen. Secondary actions are ghost or outline.

## Status

- Never rely on color alone. Pair color with text or icon.
- Use exact daemon lifecycle states. Do not invent intermediate labels.
- Starting and installing states show progress when measurable (percentage or stage).
- Failures show the failing stage, a one-line cause, and one safe next action.
- `role="status"` with `aria-atomic="true"` for live updates. `role="alert"` for errors.
- Status colors: success / info / warning / danger (Nerd extensions on top of shadcn OKLCH base).

## Actions

- One primary action per toolbar or dialog. Primary is the only filled button.
- Destructive actions require confirmation. Confirmations name the affected resource and any retained data.
- Async buttons disable duplicate submission and retain visible progress (spinner or percentage).
- Open / reveal / copy actions are always ghost.

## Command palette

- Toggle with `Ctrl/Cmd+K` (handled globally in `App.tsx`).
- Items grouped: Navigate, Projects (searchable across name, domain, framework, status, port), Actions, Theme.
- In-palette shortcuts use modifier keys (`Ctrl/Cmd+P/R/,//`) — single-letter shortcuts conflict with the search input.
- Shortcut labels render with a margin between modifier and key (`<span className="mr-1">{mod}</span>P`) so they read as `⌘ P`, not `⌘P`.

## Logs

- Mono font, 12px, 18px line-height.
- Three stream colors: stdout `--color-log-text`, stderr `--color-danger`, system `--color-warning`.
- Default follow-tail only while the user is at the bottom. Pause and show a "Jump to latest" affordance when scrolled up.
- Secrets redacted before the event reaches the UI. Never display authorization, cookies, set-cookie, x-api-key, password, or token values.

## Inspector

- Split view: list on the left, detail on the right.
- Body capture is OFF by default with an explicit toggle in the toolbar.
- When off: bodies are never stored or shown.
- When on: bodies are size-limited (1 MB), content-type filtered, and redacted.
- Detail sections are independent and collapsible: Headers, Query, Body, Timing, Response.
- Live updates do not steal the current selection.

## Mail

- Split view: inbox list on the left, message preview on the right.
- HTML preview is sandboxed in an `iframe sandbox=""` (no allow-same-origin, no allow-scripts, no allow-forms).
- Remote images blocked by default with a per-message reveal toggle.
- Text and source views are always available alongside the HTML view.
- Attachments list with explicit Save / Open actions. No auto-download.

## Forms

- Labels are always visible above the field. Placeholders are examples, not labels.
- Validation lives next to the field and repeats in daemon checks.
- Generated ports, paths, and credentials are read-only with explicit copy actions.
- Secret values hidden by default. Copy is opt-in and shows a one-time reveal.

## Empty states

- State what is absent and provide one direct next action.
- No illustrations. No decorative copy.
- One empty state per list: "No projects in this view — Park a folder", "No runtime installed — Install Node", etc.

## Loading and progress

- Long operations show a determinate progress bar when possible.
- Indeterminate only when the operation has no measurable stages.
- A cancel action is always available on operations longer than 2 seconds.

## Accessibility

- WCAG 2.1 AA across both themes.
- Body text contrast ≥ 4.5:1, large text ≥ 3:1.
- Full keyboard navigation. Tab order matches visual order.
- Visible focus rings on every interactive control.
- Icon-only buttons carry an `aria-label`.
- Status changes announced with `role="status"` or `role="alert"`.
- Targets are 28×28 minimum in compact areas, 32×32 default.
- `prefers-reduced-motion` honored across all motion.
- Honors `prefers-color-scheme` for the default theme.

## Responsive desktop behavior

- This is not a mobile app. No touch-target reflow.
- Below 880px effective width, the sidebar collapses to off-canvas (fully hidden).
- Split views stack below a defined minimum comfortable width.
- Tables never scroll horizontally below 1024px; redesign the column set instead.

## Borders

- Light theme `--border` is `oklch(0.972 0.003 106.5)` — near-white, hairline-only.
- Dark theme `--border` is `oklch(1 0 0 / 8%)` — barely visible.
- Dividers between content areas use `border-border/40` for consistent restraint.
- Never use colored border-left/right above 1px as decoration (anti-pattern from `craft-floor`).
- Cards in primary content use borders, not shadows.

## Component rule

Before building a reusable component, read `ui-registry.md`. Reuse an existing pattern when present — prefer shadcn primitives from `src/components/ui/` before writing custom components. After adding a component, record path, purpose, variants, token usage, and accessibility behavior.

## Anti-patterns (hard bans)

- Hardcoded hex values or raw Tailwind palette utilities in components.
- Colored `border-left` / `border-right` above 1px on cards, list items, callouts, or alerts.
- Gradient text or gradient UI surfaces.
- Glass / blur as decoration (functional blur only on overlays).
- Emoji as icons. Lucide SVG only.
- Kicker / eyebrow labels above headings. The heading carries its own weight.
- Section numbers as decoration.
- Modals for tasks that need neither interruption nor protected focus.
- Sparklines, progress rings, soft-shadowed rounded rectangles standing in for content.
- Monospace as costume for "technical". Mono is for code, data, paths, and measurement.
- Light or dark theme picked by category. Pick from the use scene (system by default).
- Card `shadow-sm` + parent `bg-gradient-to-t from-primary/5` — that's the SaaS-dashboard look. Cards in primary content are flat (border + bg only).
- Analytics charts on the Overview. Nerd is a local dev utility, not an analytics dashboard.
