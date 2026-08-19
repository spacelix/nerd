# UI Registry

Living registry for reusable desktop UI patterns. Read before creating a component and update after every new reusable component.

## Entry Format

```md
### ComponentName

- Path:
- Purpose:
- Variants:
- Tokens:
- Keyboard behavior:
- Accessibility:
- Used by:
```

## Design System Foundation

The prototype uses [shadcn/ui](https://ui.shadcn.com) (new-york style) as the design system foundation. All UI surfaces consume shadcn primitives from `src/components/ui/`. Tokens use OKLCH from the shadcn neutral palette, with Nerd-specific extensions for status colors and log surfaces. See `ui-tokens.md` for the full token map.

## Implemented Primitives

Added to the registry only after the first working implementation lands. Path is relative to `prototype/src/`.

### AppShell
- Path: `app/App.tsx` (DashboardPage) + `components/app-sidebar.tsx`
- Purpose: Three-region desktop shell with brand sidebar, breadcrumb header, and scrollable content. Sticky chrome, content-only scroll.
- Tokens: `--background`, `--sidebar`, `--sidebar-border`, `--border`.
- Keyboard: `Ctrl/Cmd+K` opens command palette. `Ctrl/Cmd+P`/`R`/`,`/`/`/` while palette is open.
- Accessibility: Skip link to `#main-content`. `aria-label` on SidebarInset reflects current route.

### Sidebar (App)
- Path: `components/app-sidebar.tsx`
- Purpose: Collapsible (`offcanvas` variant — fully hides when collapsed) primary navigation. Grouped sections (Workspace / Observability / System). Brand header with Logo + version. Quick Create primary button.
- Variants: `offcanvas` (default).
- Tokens: `--sidebar`, `--sidebar-foreground`, `--sidebar-accent`, `--sidebar-primary`.
- Keyboard: `Ctrl/Cmd+B` toggles collapse (shadcn default).

### SiteHeader
- Path: `components/site-header.tsx`
- Purpose: 48px breadcrumb + page actions row. Includes command palette trigger button with `⌘K` hint and theme switcher dropdown.
- Tokens: `--border` at 40% opacity for hairline divider.

### CommandPalette
- Path: `components/command-palette.tsx`
- Purpose: Keyboard-first action launcher (`Ctrl/Cmd+K`). Built on cmdk + shadcn `CommandDialog`. Items grouped: Navigate, Projects (searchable), Actions, Theme. Modifier-key shortcuts (`⌘P`/`⌘R`/`⌘,`/`⌘/`).
- Keyboard: `Ctrl/Cmd+K` toggle, `Esc` close, `↑`/`↓` navigate, `Enter` select.
- Accessibility: Searchable via fuzzy match. `role="listbox"` semantics via cmdk.

### ProjectsScreen
- Path: `components/projects-screen.tsx`
- Purpose: Projects list with filter chips (All / Running / Stopped / Degraded / Failed) + full project table (status dot, name + badge, domain, framework, runtime, port, source). Row click sets `selectedProjectId` and renders `ProjectDetail`.
- Variants: filter `all` / `running` / `stopped` / `degraded` / `failed`.
- Tokens: `--border` at 40% opacity, `--success` / `--info` / `--warning` / `--destructive` (status badges).

### ProjectDetail
- Path: `components/projects-screen.tsx` (internal `ProjectDetail`)
- Purpose: Per-project view with top action bar (Back, status badge, status detail, Open / Copy domain / Stop-Start-Retry), left aside with configuration dl + services list + 6-step lifecycle strip, and right log viewer with mono `bg-log` + stream colors.
- Lifecycle steps: Trust and Start, Resolve runtime, Start services, Start application, Readiness probe, Route enabled. Active step uses `motion-safe:animate-[nerd-pulse_...]`.
- Log colors: stdout `text-log-text`, stderr `text-destructive`, system `text-warning`.
- Tokens: `--log`, `--log-text`, `--success`, `--info`, `--destructive`, `--border` at 40% opacity.

### RuntimesScreen
- Path: `components/runtimes-screen.tsx`
- Purpose: Installed Node runtimes list (managed/external/degraded) with usage count, default badge, action buttons (Set default, Update ref, Remove). Banner for default runtime + warning banner for external runtimes.
- Tokens: `--accent`, `--success`, `--warning` (for banners), `--border` at 40% opacity.

### ServicesScreen
- Path: `components/services-screen.tsx`
- Purpose: MySQL / PostgreSQL / Redis engines list with status (managed/external/blocked). Banner explains OD-002/003/004 blockers.
- Tokens: `--warning`, `--warning-soft` (for blocker banners), `--border` at 40% opacity.

### MailScreen
- Path: `components/mail-screen.tsx`
- Purpose: Split view (inbox list + message preview). Sandboxed preview notice, attachments row, "Save source" + "Reveal remote images" actions.
- Tokens: `--warning`, `--warning-soft` (for sandbox warning), `--border` at 40% opacity.

### InspectorScreen
- Path: `components/inspector-screen.tsx`
- Purpose: Split view (recent requests list + detail with method/path/status/duration/headers). Body capture OFF by default with explicit toggle. Headers redact `authorization` and `cookie` before display.
- Tokens: `--success`, `--destructive` (for status badges), `--border` at 40% opacity.

### DiagnosticsScreen
- Path: `components/diagnostics-screen.tsx`
- Purpose: 4-card grid (DNS resolver / Root CA / Daemon / Ports). Each card: status icon, status badge, detail paragraph, meta dl, action button. Safe repairs section below.
- Tokens: `--success`, `--warning`, `--destructive` (status badges), `--border` at 40% opacity.

### SettingsScreen
- Path: `components/settings-screen.tsx`
- Purpose: Sections — Appearance (theme picker buttons), Discovery (toggle rows), Retention (3 number inputs), About (version + MIT + links).
- Tokens: `--accent` (active theme button), `--border` at 40% opacity, `--muted-foreground`.

### PageHeader
- Path: `components/PageHeader.tsx` + `PageBody`
- Purpose: Stub-screen header (title + subtitle + actions row) plus scrollable body. Used by non-Overview screens that don't have their own custom header.
- Tokens: `--window`, `--border` at 40% opacity.

### OverviewScreen
- Path: `components/overview-screen.tsx`
- Purpose: Three-section dashboard. (1) Hero: daemon health + headline + metrics grid (Uptime/Memory/CPU/Active/Idle). (2) Two-column row: Active projects list + Recent activity. Recent activity capped at 5 events.
- Tokens: `--success`, `--warning`, `--danger`, `--info` for status colors; `--muted-foreground` for labels.

### Logo
- Path: `components/Logo.tsx`
- Purpose: Custom Nerd brand mark. Rounded square with `var(--primary)` background, white stylized `N` letterform, blinking terminal-cursor accent. Larger variant (26px) in sidebar, default 22px.
- Tokens: `--primary`, white at 100% / 8% overlay.

### StatusDot (inline)
- Path: inline in `components/overview-screen.tsx`
- Purpose: Live indicator with optional pulse ring. Color maps to project status tone. Always paired with text for dual encoding.
- Tokens: `--success`, `--info`, `--muted-foreground`, `--warning`, `--danger`.

## shadcn Primitives in Use

From `src/components/ui/` (shadcn new-york style):

| Primitive | Used by |
|---|---|
| `Button` | AppSidebar (Quick Create), ProjectsScreen (lifecycle actions), SiteHeader (search/theme) |
| `Card` | (reserved for future status cards; OverviewScreen uses plain containers) |
| `Separator` | SiteHeader (vertical divider between SidebarTrigger and Breadcrumb) |
| `Sidebar` + family | AppSidebar wrapper |
| `Breadcrumb` + family | SiteHeader |
| `DropdownMenu` | SiteHeader theme switcher |
| `Tooltip` + family | TooltipProvider in App |
| `Dialog` | (used internally by CommandDialog) |
| `Command` + family | CommandPalette |
| `Sonner` | (available for future toasts) |
| `Sheet`, `Drawer`, `Tabs`, `Table`, `Toggle`, `ToggleGroup`, `Select`, `Input`, `Avatar`, `Checkbox`, `Skeleton`, `Popover`, `Slider`, `Chart` | Available, not yet used |

## Planned Primitives

These are planning hints, not approved implementations. Add each only when required by an active feature, and record the actual implementation in the registry.

### Toast
- Purpose: Ephemeral notification. Variants: info (`role="status"`), error (`role="alert"`).
- Tokens: surface-raised, border, status.
- Note: shadcn `sonner` is already installed.

### InspectorDetail
- Purpose: Deeper inspector view (full body, response body, timing breakdown) when body capture is enabled.
- Tokens: log, log-text, success, warning, danger.

### MailViewer (full)
- Purpose: Full HTML mail rendering in sandboxed iframe with text and source view toggles.
- Tokens: surface, log; sandboxed iframe.
- Accessibility: HTML rendered in `sandbox=""` with no `allow-*`.

### ProgressDialog
- Purpose: Modal dialog for long operations with determinate or indeterminate progress.
- Tokens: surface-raised, shadow-dialog, accent.

Add each only when required by an active feature. Do not prebuild a component library.
