# Design.md — Nerd Desktop

A design context for generating Nerd's Tauri 2 desktop UI. This document is the single source of visual and interaction truth for any agent generating screens. Generated screens must respect every rule below. Pair this with `PRODUCT.md` for product truth and the per-feature files under `context/features/` for scope.

## 1. Product Identity

Nerd is a lightweight, open-source local JavaScript development environment for Windows. It gives Herd-style local domains (`*.test`), HTTPS, multi-version Node management, process supervision, managed services, mail capture, and request inspection for Node projects — without Docker, VMs, Electron, or a system-wide Node install.

Target user: JavaScript and TypeScript developers on Windows 10 (minimum 22H2, build 19045) and Windows 11. They keep the desktop app open alongside their editor and terminal.

Visual mode: **Operate** (per impeccable taxonomy). Visitors complete tasks. Scanability, consistency, and native desktop expectations outrank expression. Brand lives in precise details, not marketing flourish.

## 2. Brand Voice

- Terse, technical, calm — never marketing.
- Active voice. Imperative where it fits ("Trust and start", "Park a folder").
- Errors name the problem and the recovery. They never blame the user.
- Copy in English. No emoji in UI text.
- Voice stays consistent across desktop, CLI, tray, and notifications.

## 3. Visual Direction

Compact Windows developer utility. Stark monochromatic surface with a single restrained accent. Minimalism + Swiss style + developer-tool density. No gradients on UI surfaces, no glassmorphism, no colored border-left accents above 1px, no kicker/eyebrow labels, no emoji icons, no decorative shadows. Borders and 1px hairlines carry structure; shadows appear only on dialogs and flyouts.

The product principle of restraint shows in the chrome: chrome disappears when content needs the room. Density is intentional; whitespace is earned, not added.

## 4. Color Tokens

Use the values below verbatim. Light theme is the default. Dark theme is first-class and must read identically.

### Light theme

```
--color-window:           #f7f7f5   app background
--color-sidebar:          #efefed   sidebar background
--color-surface:          #ffffff   panels, cards
--color-surface-raised:   #ffffff   dialogs, flyouts
--color-surface-hover:    #f5f5f3   hover state
--color-surface-selected: #efefed   selected sidebar item, active surface
--color-border:           #e3e3e0   hairlines, dividers
--color-border-strong:    #c9c9c4   emphasized borders
--color-text:             #0a0a0a   primary text
--color-text-muted:       #5e5e5a   secondary text, descriptions
--color-text-faint:       #8a8a85   metadata, labels
--color-accent:           #4f46e5   brand accent (indigo) — used sparingly
--color-accent-hover:     #4338ca   accent hover
--color-accent-soft:      #eeecff   accent tints for badges, soft fills
--color-success:          #15803d   running, healthy, verified
--color-success-soft:     #ecfdf3
--color-warning:          #b45309   degraded, conflict, foreign listener
--color-warning-soft:     #fef6e7
--color-danger:           #b91c1c   failed, crashed, destructive
--color-danger-soft:      #fef0f0
--color-info:             #1d4ed8   starting, installing, waiting
--color-info-soft:        #eff4ff
--color-log:              #0a0a0a   log panel background
--color-log-text:         #d4d4d0   log panel foreground
```

### Dark theme

```
--color-window:           #0c0c0c
--color-sidebar:          #141414
--color-surface:          #161616
--color-surface-raised:   #1c1c1c
--color-surface-hover:    #1f1f1f
--color-surface-selected: #1f1f1f
--color-border:           #232323
--color-border-strong:    #2e2e2e
--color-text:             #f5f5f3
--color-text-muted:       #9c9c98
--color-text-faint:       #6e6e6a
--color-accent:           #a5b4fc
--color-accent-hover:     #c7d2fe
--color-accent-soft:      #1e1b4b
--color-success:          #4ade80
--color-success-soft:     #052e16
--color-warning:          #fbbf24
--color-warning-soft:     #422006
--color-danger:           #f87171
--color-danger-soft:      #450a0a
--color-info:             #93c5fd
--color-info-soft:        #172554
--color-log:              #050505
--color-log-text:         #d4d4d0
```

### Status mapping (semantic, not literal)

| State | Token |
|---|---|
| running, healthy, ready, verified | success |
| starting, installing, waiting, info | info |
| stopped, disabled, idle | text-muted |
| degraded, conflict, foreign listener | warning |
| failed, crashed, destructive | danger |

Status must always carry color **plus** text or icon. Color alone is never a status signal.

## 5. Typography

System fonts only — do not download web fonts. Local fallbacks are acceptable.

- Sans: `"Segoe UI Variable", "Segoe UI", system-ui, -apple-system, BlinkMacSystemFont, sans-serif`
- Mono: `"Cascadia Code", "Cascadia Mono", Consolas, "JetBrains Mono", monospace`

| Role | Size | Weight | Line height | Tracking |
|---|---:|---:|---:|---:|
| Window title | 18px | 600 | 26px | -0.018em |
| Section title | 22px | 600 | 30px | -0.022em |
| Display number | 28–32px | 600 | 36–40px | -0.025em |
| Page heading | 18px | 600 | 26px | -0.014em |
| Body | 13px | 400 | 20px | 0 |
| Body strong | 13px | 600 | 20px | 0 |
| Label | 12px | 600 | 18px | 0 |
| Metadata | 11px | 400 | 16px | 0 |
| Small caps label | 10px | 600 | 14px | 0.14–0.18em, uppercase |
| Mono / log | 12px | 400 | 18px | 0 |
| Code path | 12px | 400 | 18px | -0.005em |

Numerals are tabular (`font-variant-numeric: tabular-nums`) everywhere a number appears. Headers enable `font-feature-settings: "ss01", "cv11", "kern"`.

## 6. Shape, Spacing, Elevation

```
--radius-sm:   5px    inputs, buttons, badges
--radius-md:   8px    cards, dialogs
--radius-lg:  12px    large surfaces, hero cards
--radius-pill: 999px  status pills (optional)

Spacing scale (Tailwind-compatible, base 4px):
--space-1: 4px
--space-2: 8px
--space-3: 12px
--space-4: 16px
--space-5: 20px
--space-6: 24px
--space-8: 32px
--space-10: 40px

Elevation:
--shadow-flyout: 0 1px 2px rgb(10 10 10 / 0.04), 0 12px 32px rgb(10 10 10 / 0.06) — light
                0 1px 2px rgb(0 0 0 / 0.4),  0 12px 32px rgb(0 0 0 / 0.5)  — dark
--shadow-dialog: 0 4px 12px rgb(10 10 10 / 0.08), 0 32px 80px rgb(10 10 10 / 0.12) — light
                0 4px 12px rgb(0 0 0 / 0.6),   0 32px 80px rgb(0 0 0 / 0.7)   — dark
```

Cards in primary content use borders, never shadows. Shadows are reserved for flyouts, menus, dialogs, and detached overlays.

## 7. Component Measurements (desktop window)

| Element | Measurement |
|---|---:|
| Title bar (draggable) | 40px |
| Sidebar expanded | 220px |
| Sidebar collapsed (icon rail) | 52px |
| Top toolbar / page header | 64px |
| Section header | 40px |
| Input / button default | 32px |
| Input / button compact | 28px |
| Table row | 38px |
| List row (compact) | 36px |
| List row (comfortable) | 44px |
| Status dot | 6–8px |
| Content max width | none (window owns width); reading width 65–75ch for prose |
| Minimum window width | 960px (proposed; subject to OD-015) |

## 8. Layout System

Three-region shell, used by every screen:

```
+---------------------------------------------------+
| Title bar (40px)                                  |
+----------+----------------------------------------+
| Sidebar  | Top toolbar (64px)                     |
| 220/52px |----------------------------------------|
|          | Content                                 |
|          |                                         |
|          |                                         |
+----------+-----------------------------------------+
```

- The sidebar collapses to an icon rail; the choice persists per user.
- The window is native (Tauri 2) with native window controls; do not draw fake traffic lights.
- A persistent skip link exposes main content to keyboard users.
- Below 880px effective width, the sidebar collapses by default and the layout reflows to a stacked split where applicable.

## 9. Sidebar Navigation

Sidebar groups, top to bottom, with section labels in small caps (10px, tracking 0.18em, color `--color-text-faint`):

- **Workspace** — Overview, Projects, Runtimes, Services
- **Observability** — Mail, Inspector, Diagnostics
- **System** — Settings

Active state cues (apply all of them, never one alone):

- Background `--color-surface-selected` (light: `#efefed`, dark: `#1f1f1f`).
- Icon color `--color-text` (no accent border, no accent left bar).
- Label weight 600 (semibold). Default weight 500.
- Hover background `--color-surface-hover`.

## 10. Primitives

The reusable primitives below are the building blocks. Every screen composes from them.

| Primitive | Purpose | Tokens |
|---|---|---|
| `AppShell` | Three-region frame with title bar, sidebar, content | border, sidebar, window |
| `TitleBar` | Draggable 40px bar with brand mark, version, daemon status, theme switcher | sidebar, border, text |
| `Sidebar` | Collapsible navigation with section groups | sidebar, surface-selected, text |
| `SiteHeader` | Top toolbar with breadcrumbs, page title, primary actions | surface, border, text |
| `StatusBadge` | Dual-encoded status (color + dot + text) | success / info / muted / warning / danger |
| `StatusDot` | Pulsing indicator with optional ping ring | same tokens as StatusBadge |
| `Card` | Surface container with hairline border; padding 20–32px | surface, border, shadow-sm |
| `MetricCard` | Tonal accent card with big tabular number, label, detail, icon | surface, border, accent / status tokens |
| `Button` | Variants: default (accent), secondary (surface + border), ghost, outline, destructive | accent, surface, danger |
| `Badge` | Small caps pill with variants | surface-hover, accent-soft, success-soft, warning-soft, danger-soft |
| `Input` | Single-line text input, 32px high, hairline border | surface, border, accent (focus ring) |
| `Table` | Header + 38px rows, tabular numerals, hover and selected states | surface, surface-hover, surface-selected, border |
| `SplitPane` | List left, detail right; stacked when narrow | surface, border |
| `PropertyGrid` | 140px key column, value column, optional monospace values | text-muted, text |
| `LogViewer` | Mono font, virtualized, stream-colored (stdout / stderr / system), redaction before display | log, log-text, info / warning / danger |
| `Toolbar` | 44–64px high; left: title + subtitle, right: actions | surface, border |
| `EmptyState` | Title, description, one direct next action | surface, text-faint, text-muted |
| `DropdownMenu` / `ContextMenu` | Radix-styled menus with proper a11y | surface-raised, border, shadow-flyout |
| `Dialog` / `Drawer` / `Sheet` | Modal surfaces with overlay and focus trap | surface-raised, shadow-dialog |
| `Tooltip` | On hover, 200ms delay, dark surface in light theme | surface-raised, border |
| `Toast` | `role="status"` for info, `role="alert"` for errors; ephemeral | surface-raised, border, status |
| `ProgressDialog` | Indeterminate or % progress with cancel action | accent, surface-raised |
| `Icon` | Lucide SVG, 16/20px, 1.5–1.75 stroke; never emoji | — |

## 11. Screen Inventory

Eight destinations plus a tray menu and an onboarding wizard. Each is described below.

### 11.1 Overview

The home screen. Three vertical regions stacked in one max-width column (5xl ≈ 1024px, padding `40px`):

1. **System hero card** — full-bleed card (`rounded-lg`, padding 32px). Top-left: small caps "DAEMON HEALTHY" with pulsing green dot. Headline: `{N} running, {M} idle.` in 32px semibold. Description: one sentence about managed runtimes and foreign listeners. Top-right: a three-column mono dl of Uptime / RAM / CPU in tabular numerals.
2. **Status row** — four `MetricCard`s in a `grid-cols-4` row. Labels: Running / Stopped / Degraded / Failed. Each shows a big mono number, an icon in a colored-soft square, and a one-line detail. Tones map to the status mapping in §4.
3. **Two-column** — left: "Running now" Card with `Badge` count + list of running projects as `Card` rows. Right: "Recent activity" Card with `Badge` count + last 6 events from project logs as mono-timestamped rows.

The hero card uses the system status copy from `feature 07` — never rephrase. Daemon health wording: "Daemon healthy", "Daemon starting", "Daemon degraded", "Daemon stopped".

### 11.2 Projects

Split view. Left: filterable list of projects. Right: project detail.

- **Toolbar** (top): title "Projects", subtitle `N of M`, right side: filter chip group (All / Running / Stopped / Degraded / Failed) with tabular counts, Refresh icon button, primary "Park folder" button.
- **Project rows**: 40px, columns: status dot, name (semibold), domain (mono), framework badge (mono), runtime, port (mono). Selected row: `--color-surface-selected` background. Hover: `--color-surface-hover`. Click selects, populating the right pane.
- **Project detail (right)**: header with status badge, name, domain (mono), optional status detail sentence. Action row: contextual button — Start (primary) when stopped, Stop (secondary) when running, Retry (secondary) when degraded/failed. Always show Open (ghost) and Copy (ghost).
- **Configuration grid**: PropertyGrid with Domain / Framework / Runtime / Package manager / Port / Source / Path / Trust / Services.
- **Lifecycle strip**: a vertical list of stages (Trust and Start / Resolve runtime / Start services / Start application / Readiness probe / Route enabled) with a small dot per stage (success / info / danger / muted). Active stage is bold.
- **Log viewer**: mono, last 12 lines, color-coded by stream, sticky header with line count.

### 11.3 Runtimes

Tabular inventory of installed Node versions.

- Toolbar: title "Runtimes", right: "Install Node" (primary), "Register external" (ghost).
- Rows: status dot, version (mono), channel (small caps), source ("managed" / "external" / "degraded"), default badge, "X projects" usage count (mono), action menu (Set default / Open in explorer / Remove reference). External runtimes carry a `Degraded` warning when path or version changes; show a one-line cause and an "Open repair" link.
- A separate "External runtimes" section below if any exist, with the warning treatment above.

### 11.4 Services

Status-first inventory for MySQL, PostgreSQL, Redis.

- Each row: engine label, status badge (`Managed` / `External` / `Blocked`), short note explaining state. For `Blocked`, the note references the open decision (OD-002 / OD-003 / OD-004) verbatim — do not paraphrase or invent.
- "Manage" action only for `Managed` engines; disabled for `Blocked`. No manage action at all for `External` — only "Register".

### 11.5 Mail

Split view. Left: inbox list per project, ordered newest first. Right: message preview.

- Mail viewer is **sandboxed**: HTML rendering uses `iframe sandbox=""` (no allow-same-origin, no allow-scripts, no allow-forms). Remote images are blocked by default; a toggle reveals them per message. Text and source view always available.
- Attachment list with explicit "Save" / "Open" actions. No auto-download.
- Retention note in the toolbar: "Retention: 7 days · 1000 messages · 50 MB". Cleanup is explicit, never automatic surprise.

### 11.6 Inspector

Split view. Left: recent HTTP requests (tabular). Right: request detail.

- Metadata capture is on by default. **Body capture is OFF by default** with an explicit toggle; when off, no bodies are stored or shown. When on, bodies are size-limited to 1 MB, content-type filtered, and redacted before display.
- Detail sections: Headers, Query, Body (redacted), Timing, Response. Each section is collapsible.
- Authorization, cookie, set-cookie, and any header whose name matches `authorization|cookie|set-cookie|x-api-key|password|token` is replaced with `[redacted]` before the event leaves the proxy.

### 11.7 Diagnostics

Top-level status cards for the four foundations Nerd manages.

- **DNS resolver**: card with NRPT rule status, last verified time, "Run probe" action.
- **CA trust**: card with Nerd root CA fingerprint, install location (CurrentUser), trust state.
- **Daemon**: card with PID, uptime, version, IPC endpoint (`\\.\pipe\nerd`), restart history (count, last reason).
- **Ports**: card listing 53 / 80 / 443 status. Foreign listeners are reported with `Warning` tone and never terminated automatically — copy: "Foreign listener detected on port N. Report only; not terminating.".

Cards are aligned in a 2×2 grid on wide windows, 1×4 on narrow. Footer: "Run nerd doctor" primary action.

### 11.8 Settings

Grouped list of preferences.

- **Appearance**: theme (Light / Dark / System) radio; accent strength (subtle when user enables).
- **Discovery**: external tool discovery toggle (default OFF per `feature 07`). Label explains: "Detect existing Node, MySQL, PostgreSQL, Redis installations. Off by default. Nerd will never mutate, repair, or remove an external resource."
- **Retention**: number inputs for inspector buffer (default 500 / project), mail retention (default 7 days / 1000 messages / 50 MB).
- **Privacy**: redacted fields preview, "Export redacted diagnostic bundle" action (creates a file with no secrets — see OD-020).
- **About**: version, license (MIT), update channel.

### 11.9 Onboarding (first run)

Five steps, top-progress 5-dot indicator, no back-edit (forward only after Trust).

1. **System check** — show platform, OS build, daemon reachability. Failures become inline error states with one safe next action.
2. **DNS / HTTPS setup** — explain NRPT rule and root CA install. Buttons: "Install (UAC)" primary, "Skip" secondary.
3. **Default Node** — install latest Active LTS, or "Use existing Node" with explicit registration.
4. **Park a directory** — folder picker, preview detected projects.
5. **First project** — pick one detected project, "Trust and Start".

### 11.10 Tray menu

Native Windows tray. Compact, server-rendered to mimic desktop integration.

- Daemon health line with dot.
- "N projects running".
- Recent projects (up to 5) with Start / Stop inline action.
- Open app, Open diagnostics, Quit GUI. Daemon stop is a separate confirm action.

## 12. State and Behavior Rules

- **Lifecycle** uses exact daemon states: `Stopped → Resolving → Starting services → Starting application → Waiting for readiness → Running → Stopping application → Stopping services → Stopped`. UI never invents intermediate labels.
- **Trust and Start** is required before a newly discovered or material-changed project can run. Discovery records metadata but executes no code.
- **External resources are read-only.** UI surfaces their existence and version but offers no "manage", "stop", "update", or "remove" action on them. Nerd may install a managed replacement only with explicit opt-in.
- **Status updates use `role="status"` with `aria-atomic="true"`**, never a bare numeric badge. Error states use `role="alert"`.
- **Empty states** state what is absent and provide one direct next action. No illustrations.
- **Destructive actions** require a confirmation naming the affected resource and any retained data. Unlink, remove service, delete data, delete backup, uninstall are all distinct confirmations.
- **Errors** name the problem and the recovery. Example: "Node 20.10.0 binary missing. Install managed Node 20 or update the external reference."
- **Async actions** show pending / success / failure states; the button disables duplicate submissions.
- **Keyboard**: every action reachable by Tab; visible focus ring on all interactive controls; `prefers-reduced-motion` honored.
- **Animation**: 100–180 ms ease-out; entrance fade-up on screen mount; subtle hover color shift; status dot pulse only for active / live states.

## 13. Accessibility Floor

- WCAG 2.1 AA across both themes. Body text contrast ≥ 4.5:1, large text ≥ 3:1.
- Icon-only buttons carry an `aria-label`. Status uses color + text + icon.
- Touch / click targets minimum 28×28px in compact areas, 32×32px default.
- Honors `prefers-reduced-motion`. Honors `prefers-color-scheme` for the default theme.
- Full keyboard navigation including Sidebar collapse / expand and split-pane focus traversal.
- Screen reader announces lifecycle transitions and long-operation completion.

## 14. Component States to Cover

For every interactive primitive, design the following states:

- Default, hover, active / pressed, focus-visible, disabled, loading.
- Empty (no data), error (with recovery), success (acknowledgement), pending (with progress).

Do not ship a screen that has any of these states missing.

## 15. Anti-Patterns (Hard Bans)

- Hardcoded hex values or raw Tailwind palette utilities (`bg-blue-500`, `text-slate-700`) anywhere in components. Use tokens.
- Colored `border-left` / `border-right` above 1px on cards, list items, callouts, or alerts.
- Gradient text or gradient UI surfaces. The brand mark is allowed to carry a small internal gradient.
- Glass / blur used as decoration. Acceptable only for flyouts where backdrop blur is functional.
- Emoji as icons. Use Lucide.
- Kicker / eyebrow labels above headings. Headings carry their own weight.
- Hard offset shadows (`4px 4px 0` style) outside an explicitly neobrutalist world.
- Section numbers (01 / 02 / 03) as decoration.
- A modal for tasks that need neither interruption nor protected focus. Use inline confirmation.
- Sparklines / progress rings / soft-shadowed rectangles standing in for real content.
- Monospace as costume for "technical". Mono is for code, data, paths, and measurement.
- A system display face (Impact, Arial Black) as display voice.
- Light or dark theme picked by category. Pick it from the use scene (system preference by default, manual override available).
- Marketing flourish on a developer tool: hero-metric big-number templates, section numbers, decorative empty illustrations.

## 16. Reference Patterns (Banned as Defaults, Acceptable When Earned)

These are not bans — they are patterns the model reaches for. Reach for one only when the axis is free:

- Cards of icon + heading + text as the page structure. Use only when grouping demands it.
- A big-number / small-label / supporting-stats card per section. Already used; restrict to status row only.
- Modal dialogs. Use only for tasks requiring protected focus or interruption.
- Tooltips on every icon. Use only when the label alone is ambiguous.

## 17. Out of Scope (MVP)

Do not design for these — they live in `roadmap.md` and are explicitly out of MVP:

- Bun and Deno runtimes.
- macOS, Linux, ARM64 Windows.
- Public tunnels, LAN sharing.
- MCP / AI-agent integration.
- Cloud accounts, sync, teams, billing, licensing servers.
- Production deployment workflows.

## 18. Acceptance Heuristics

A generated screen is review-ready when:

- It uses only the tokens in §4 and §6. No raw hex, no raw palette classes.
- Every status indicator pairs color with text or icon.
- Empty, error, and loading states exist for every list and async surface.
- All interactive elements have focus styles and accessible names.
- The copy is technical and terse; no marketing language.
- The shell (sidebar / top toolbar) and screen content read as one continuous product.
- Light and dark themes read identically. Verified by toggling both.
- The screen would not look out of place inside Linear, Vercel, or Raycast — without copying any of them.

---

Sources of truth referenced by this file:

- `context/project-overview.md` — product spec.
- `context/architecture.md` — system architecture.
- `context/trust-model.md` — trust levels and threats.
- `context/versioning.md` — code and application versioning.
- `context/code-standards.md` — Rust / TypeScript / SQLite / Windows integration standards.
- `context/library-docs.md` — approved dependencies and security boundaries.
- `context/compatibility.md` — support matrix.
- `context/ui-tokens.md` — visual tokens (canonical source for §4–§7).
- `context/ui-rules.md` — UI behavior rules (canonical source for §12–§13).
- `context/ui-registry.md` — primitive registry.
- `context/features/07-desktop-tray-cli.md` — screen scope source.
- `context/decisions/*.md` — ADRs that bind design choices.
