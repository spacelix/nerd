# Prototype Progress Tracker

Source of truth for `prototype/` UI work. Mirrors `progress-tracker.md` style but scoped to the prototype, not the production Nerd desktop. A prototype milestone is `complete` only when the source files compile (`tsc --noEmit`), the dev server boots without errors, and the relevant slice of the design system is reviewable in light + dark themes.

Allowed status values: `planned`, `in progress`, `complete`, `blocked`.

## Environment Note

The host is Fedora Linux (no WSL mount of Windows). `prototype/` is a browser-only web app — no Tauri, no Rust, no native Windows artifacts. Linux `node`/`npm` is used for install and dev. Lockfiles for the prototype stay inside `prototype/`; they never touch the parent Nerd workspace.

## Design System

- **Foundation**: [shadcn/ui](https://ui.shadcn.com) new-york style. Installed via `npx shadcn@latest add`. OKLCH neutral palette as canonical tokens.
- **Icons**: [lucide-react](https://lucide.dev) (replaced tabler after first shadcn install).
- **Build path**: code-first per impeccable `init` decision.

## Milestones

| ID | Milestone | Status | Notes |
|---|---|---|---|
| M0 | Foundations: scaffold, tokens, empty AppShell | complete | Vite + React 18 + TS strict + Tailwind v4; `theme.css` with shadcn OKLCH tokens + Nerd extensions. AppShell with title bar 40px + 220px sidebar. `npm install` clean, `tsc --noEmit` clean, dev server boots ~500ms. |
| M1 | Navigation: AppSidebar 8 routes + collapse + skip link | complete | `app-sidebar.tsx` with all 8 Nerd routes grouped into Workspace / Observability / System. Logo + version header. Quick Create primary button. `collapsible="offcanvas"` variant. Active state via `aria-current="page"`. Custom `Logo.tsx` (rounded square + stylized N + blinking cursor accent). |
| M2 | Design system reset (shadcn OKLCH + indigo accent) | complete | Replaced custom warm-cream palette with shadcn neutral OKLCH (`#3d348b` indigo accent). Light + dark via `.dark` class. Nerd-specific tokens (success/warning/danger/info + soft, log) layered on top. Border token lightened (`oklch(0.972 0.003 106.5)`) for hairline feel. |
| M3 | shadcn dashboard-01 layout primitives | complete | Installed shadcn `dashboard-01` blocks: `app-sidebar.tsx`, `site-header.tsx`, `card.tsx`, `separator.tsx`, `breadcrumb.tsx`, `label.tsx`, `chart.tsx`, `select.tsx`, `tabs.tsx`, `table.tsx`, `toggle-group.tsx`, `badge.tsx`, `checkbox.tsx`, `dropdown-menu.tsx`, `drawer.tsx`, `input.tsx`, `avatar.tsx`, `sheet.tsx`, `sonner.tsx`, `tooltip.tsx`, `sidebar.tsx`. Replaced `@tabler/icons-react` with `lucide-react`. |
| M4 | OverviewScreen polished for Nerd needs | complete | Removed generic shadcn chart (visitor analytics — not relevant for Nerd). Hero (daemon status + headline + metrics grid) + Active projects list + Recent activity capped at 5 events. Active projects + Recent activity side-by-side. Sticky shell, content-only scroll via `h-svh overflow-hidden` wrapper in `App.tsx`. |
| M5 | ProjectsScreen + ProjectDetail | complete | Projects route shows full table with filter chips (All/Running/Stopped/Degraded/Failed) + 7 columns (status dot, name + status badge, domain, framework, runtime, port, source). Row click sets `selectedProjectId` and renders `ProjectDetail` with top action bar (Back/Stop-Start-Retry/Open/Copy), left aside (configuration dl + services list + 6-step lifecycle strip with pulse animation), and right log viewer with mono `bg-log` + stream colors (stdout/stderr/system). |
| M6 | CommandPalette (cmdk + shadcn Command) | complete | `components/command-palette.tsx`. Toggled by `⌘K`/`Ctrl+K`. Items grouped: Navigate, Projects (searchable across name/domain/framework/status/port), Actions (Start a project, Refresh, Settings, Stop all running), Theme (Light/Dark/System). Modifier-key shortcuts: `⌘P` (Start a project), `⌘R` (Refresh), `�,` (Settings), `⌘/` (toggle theme). Label rendered with `<span className="mr-1">{mod}</span>P` for proper visual separation. |
| M7 | Token polish + border restraint | complete | Light `--border` reduced from `oklch(0.93)` to `oklch(0.972 0.003 106.5)` for hairline visibility. Dark `--border` from `oklch(1 0 0 / 10%)` to `oklch(1 0 0 / 8%)`. All custom dividers use `border-border/40` for consistent subtlety. Card shadow removed (flat). |
| M8 | Sidebar refinement | complete | Removed `NavUser` footer (Nerd has no authentication). Removed bottom nav (Settings/Help/Search — duplicates nav). Removed Quick Links section (redundant with main nav + command palette). Sidebar now contains: brand header, Quick Create button, 8 grouped nav items. |
| M9 | Remaining screens (Runtimes, Services, Mail, Inspector, Diagnostics, Settings) | complete | All 8 routes implemented with Nerd-specific content. Runtimes: managed/external list + usage. Services: 3 engines with OD-002/003/004 blocker status. Mail: split inbox + sandboxed preview. Inspector: request list + redacted headers. Diagnostics: 4 status cards + safe repairs. Settings: theme + discovery toggles + retention inputs + about. |
| M10 | Per-screen layout polish (no PageHeader; full-width) | complete | Removed `PageHeader` from every screen — SiteHeader breadcrumb already shows page title. All screens now full-width shell (no `max-w-* mx-auto` containers). Actions inline at top of content area. |
| M11 | Audit + polish + /review | planned | Run impeccable `audit` (5 dimensions: Accessibility, Performance, Theming, Responsive, Implementation Integrity). Pre-delivery checklist. Keyboard-only walk-through. Verify all primitives recorded in `ui-registry.md`. |

## Files Added Per Milestone

Tracked here to keep history auditable. Do not delete old entries.

## Decisions

- Platform (init): `web` — Vite + React in browser.
- Build path (init): `code` — direct to code; no comp-first image-driven round.
- Voice: English.
- PRODUCT.md captured at `/PRODUCT.md`; build path at `/.impeccable/config.json`.
- Prototype lives at `prototype/`. Promotion to `apps/desktop/` is a separate concern and is not part of this tracker.
- **Design system**: shadcn/ui new-york style + OKLCH neutral palette + lucide-react icons.
- **No authentication** — Nerd is local-only. No user footer in sidebar.
- **No analytics charts** — Overview shows daemon status, project counts, recent activity only.
- **Sticky shell + content-only scroll** — title bar + sidebar + header stay fixed; only content area scrolls.
- **No per-page PageHeader** — SiteHeader breadcrumb carries page title. Actions inline at content top.
- **Full-width content** — no `max-w-* mx-auto` containers; every screen uses full shell width.
- **6-step project lifecycle** — Trust and Start → Resolve runtime → Start services → Start application → Readiness probe → Route enabled. State derived from project status.

## Blockers

None open for the prototype scope.

## Review Rule

A milestone is `complete` only after source compiles (`tsc --noEmit`), dev server boots, light/dark renders correctly, focus ring is visible, and the relevant primitives are recorded in `context/ui-registry.md`.
