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

From `src/components/ui/` (shadcn new-york style) — **v1**:

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

## V2 Primitives

`prototype2/` runs in parallel with v1. Paths below are relative to `prototype2/`. Each v2 primitive carries a `(v2)` tag in its entry.

### Shell (v2)

Full-height `AppSidebar` on the left + Command Bar at top + two-pane content (Center Stage + Inspector Rail) + persistent Status Bar at the bottom. The earlier inner three-pane project rail was retired in favor of the AppSidebar owning primary navigation.

#### AppSidebar (v2)
- Path: `src/components/shell/app-sidebar.tsx`
- Purpose: Full-height primary navigation. Regions: header (button that opens the command palette — `Search` icon + `⌘K` hint), nav (three grouped sections — Workspace / Observability / System), footer (version `v0.1.0-alpha.1` in mono on the left + theme switcher on the right). Brand (logo + name) lives in the desktop window chrome.
- Variants: 8 routes; `aria-current="page"` on the active item.
- Tokens: `--sidebar` at 70% opacity; `--border` at 40% for hairline dividers; `--surface-active` for the active nav cell; `--success` for unread badge; `--muted` at 40% for the search button background.
- Effects: `backdrop-blur-md` for the acrylic feel.
- Keyboard: every nav button is focusable with visible focus ring. The header button opens the command palette; the palette itself is triggered globally by `Ctrl/Cmd+K`.
- Accessibility: `aria-label="Primary navigation"` on the `<aside>`, `aria-label="Main"` on `<nav>`. Active route carries `aria-current="page"`. Header button has `aria-label="Open search"` and `title`.
- Used by: App root.

#### StatusBar (v2)
- Path: `src/components/shell/status-bar.tsx`
- Purpose: 28px persistent bottom status bar. Always visible across every screen. Mono throughout. Data comes from the `useDaemonStream` mock (heartbeat 1s, request counter, periodic status blip). Left: daemon health + version + IPC protocol. Center: current project + status dot + status + port + runtime. Right: REQ counter + services count + Inspector toggle button.
- Source: `src/hooks/use-daemon-stream.ts` — module-level mock daemon event stream (`subscribeDaemonStream`) with a reducer; emits `daemon.heartbeat`, `project.request`, `project.status`. Status dot tone maps `running→success`, `starting→warning`, `stopped→muted`.
- Tokens: `--status-bar-bg` (distinct surface); `--success` / `--info` for status dots and counters; `--muted-foreground` for separators.
- Props: `inspectorOpen: boolean`, `onToggleInspector: () => void`. The right-side toggle button carries `aria-pressed={inspectorOpen}` and a small status dot (primary when open, muted when closed).
- Accessibility: status communicated via paired text + dot (dual encoding).
- Used by: App root.

#### InspectorScreen (v2)
- Path: `src/components/inspector/inspector-screen.tsx`
- Purpose: Request capture list for the `inspector` route. Rows sorted newest-first: status (colored mono), method chip, URL, duration. Per-row actions: highlight swatch group (red/yellow/green/blue/purple; click active swatch to clear) and a comment button that expands an inline textarea (Save/Cancel). Toolbar under the header: **Capture** switch (`ui/switch.tsx`; off shows a buffered-state placeholder), **Clear** (visually empties the buffer until re-enabled), **Export safe metadata** (transient "Exported" state). Header has a filter (URL / method / status / host). Selection is lifted to App (`selectedRequestId`) so the rail shows the detail.
- Tokens: `--danger` / `--warning` / `--info` / `--success` status colors, `--highlight-purple`, `--surface-active` for the selected row.
- Accessibility: row main area is a `<button>` (selection), the action cluster sits outside it as sibling buttons with `aria-pressed`; swatch group has `role="group"`.
- Used by: App CenterStage (route `inspector`).

#### RequestInspector (v2)
- Path: `src/components/shell/request-inspector.tsx`
- Purpose: Proxyman-style multi-tab request detail rendered inside `InspectorRail`. Header: status + method chip + highlight dot + URL, plus the local comment shown as a warning-tinted note. Tabs via `ui/tabs.tsx`: Overview (host/path/query/protocol/content-type/duration/timing/highlight/project), Headers (request only), Body (request only), **Response** (status/size/duration summary + response headers + response body — F-12/ui-rules), Timing (stacked bars + total). Body tab **pretty-prints JSON**: bodies starting `{`/`[` are parsed and rendered with syntax coloring (`JsonPretty` — keys `text-info`, strings `text-success`, numbers/booleans `text-warning`, punctuation `muted`), badged `content-type · formatted`; non-JSON falls back to a mono `pre`. Reads the same `useRequestAnnotations` store as the list.
- Used by: InspectorRail when a request is selected on the `inspector` route.

#### RuntimesScreen (v2)
- Path: `src/components/runtimes/runtimes-screen.tsx`
- Purpose: Installed Node versions for the `runtimes` route. Header actions: "Register external" (read-only probe dialog) + "Install Node" (wizard dialog). **Ownership filter tabs** (All / Managed / External / Degraded). Rows: mono `v{version}`, class badge (managed/external/degraded), default star, usage count, chevron; right rail shows **Uninstall** (Trash, managed only) or a `read-only` tag (external/degraded). Selecting a row lifts `selectedRuntimeId` to App so the rail shows `RuntimeDetail`.
- Tokens: class badge colors via `--success` / `--warning` / `--muted`.
- Used by: App CenterStage (route `runtimes`).

#### RuntimeDetail (v2)
- Path: `src/components/runtimes/runtime-detail.tsx`
- Purpose: Rail detail for a runtime — class badge, default flag, usage count, "Used by" project list (computed from the `projects` mock via `node-{version}`), and a warning note when it is the default. Read-only.
- Used by: InspectorRail via App `buildRailContent`.

#### ServicesScreen (v2)
- Path: `src/components/services/services-screen.tsx`
- Purpose: Loopback engine list for the `services` route. Header actions: "Register external" (engine + endpoint dialog) + "Add service" (engine pick with OD blocker note). **Ownership filter tabs** (All / Managed / External); external rows show a `external` badge. Rows: status dot (live via `useServiceActions`), name, version · port, OD blocker chip, status, chevron; right rail shows **Remove** (Trash) for services without OD blockers and a `read-only` tag for blocked ones. Selecting lifts `selectedServiceId` to App.
- Tokens: `--warning` blocker chip, status dots via StatusDot.
- Used by: App CenterStage (route `services`).

#### ServiceDetail (v2)
- Path: `src/components/services/service-detail.tsx`
- Purpose: Rail detail for a service — engine/version/port/status/class rows, OD-002/003/004 blocker banner (warning tint), bound projects, and a lifecycle note. **Manageable** services (class `managed`, no OD blocker — MySQL/Redis) show a keepRunning `ui/switch.tsx` + **Start / Stop / Restart** buttons driven by a new module store `hooks/use-service-actions.ts` (`statusFor`/`start`/`stop`/`restart` with a transient `starting` state on stop→start so the UI never silently botches the lifecycle), plus **Backup**/**Restore** buttons (mock, transient `role="status"` feedback), and a project-scoped lifecycle note (F-10). Blocked/read-only services keep the read-only note. Rows also surface **injected env** per attached project via `ServiceEnvRows` (loopback `127.0.0.1:port` + `NERD_*_URL`, credentials redacted — F-10 connection URLs).
- Used by: InspectorRail via App `buildRailContent`.

#### DiagnosticsScreen (v2)
- Path: `src/components/diagnostics/diagnostics-screen.tsx`
- Purpose: Probe cards for the `diagnostics` route — database / runtimes / services / disk / permissions / DNS / Root CA / Daemon / Ports / Recovery / Group policy / Foreign conflict with pass/warn/fail (and `unsupported-policy` / `foreign-conflict` policy states — F-02/F-12) icons + status label, plus a "Run all" button and an **Export support bundle** button that opens a `ui/dialog.tsx` flow (redaction note → **bundle preview list** — daemon-state.json/probes.json/redacted logs/versions, with an excluded-items note (bodies, private keys, raw credentials) — → "Create bundle" → done state). Selecting lifts `selectedProbeId` to App.
- Tokens: `--success` / `--warning` / `--danger` status colors.
- Used by: App CenterStage (route `diagnostics`).

#### DiagnosticDetail (v2)
- Path: `src/components/diagnostics/diagnostic-detail.tsx`
- Purpose: Rail detail for a probe — status badge (incl. `unsupported-policy` / `foreign-conflict`), summary, probe-output lines, and a safe repair button (`actionSafe` renders primary; otherwise a warning-outline button).
- Used by: InspectorRail via App `buildRailContent`.

#### OverviewScreen (v2)
- Path: `src/components/overview/overview-screen.tsx`
- Purpose: Home dashboard for the `overview` route. Daemon-health hero (status dot + version + IPC + current project, live from `use-daemon-stream`; a **DaemonStateBadge** (`running`/`absent`/`protocol-mismatch`/`unhealthy`, F-01) sits inline; text carries `role="status"` `aria-atomic`), stats row (uptime / requests / services / mail unread), metrics grid (no analytics charts — decision pending N6), Working Session-aware **Active projects** (compact rows open the project detail page; "View all" navigates to Projects), **Recent requests** (top 5 by time), and **Recent operations** (F-01 — install/link/trust-CA/park rows with status dot, operation id + started-at, `role="progressbar"` progress bar colored by state, percent, and a **Cancel** (X) button on running operations; cancel flips the state to `cancelled`, clears the bar, and shows a muted "cancelled" label — ui-rules loading; failed rows carry `role="alert"` + `Operation.cause` line, F-05). Contains a `WorkingSessionToggle` in the panel header.
- Used by: App CenterStage (route `overview`).

#### SettingsScreen (v2)
- Path: `src/components/settings/settings-screen.tsx`
- Purpose: Settings for the `settings` route, six cards. **Appearance**: Dark / Light / System buttons bound to `useTheme`. **Discovery**: four toggles (auto-discover linked repos, attach to running local servers, trust root CA, native change notifications) — all local state. **Retention**: native `<select>` for request / log retention; **Mail retention** uses a dedicated `MailRetentionSelect` configurable by count/size (500 / 1000 messages, 250 MB cap, forever — F-09). **Network & HTTPS** (F-02): status rows (DNS listener 127.0.0.1:53 UDP+TCP, NRPT rule, root CA DPAPI, proxy ports 80/443) + Run setup (UAC) / Probe now with rollback note. **Storage & safe cleanup** (F-07/F-12): per-item usage bars + a danger cleanup button. **About**: version, platform, IPC, build, license, no-telemetry/DPAPI note, Check for updates + Replay onboarding buttons, an **Uninstall Nerd** danger button that opens a scoped `UninstallDialog` (cleanup option rows: NRPT rule, root CA, autostart+PATH, binaries+daemon; "keeps project data" note; type "Uninstall Nerd" to confirm, F-12), and an uninstall-semantics note.
- Primitives: `ui/switch.tsx` (native `role="switch"` button, `aria-checked`, compact `h-[18px]`); `ui/dialog.tsx` (UninstallDialog).
- Used by: App CenterStage (route `settings`).

#### OnboardingWizard (v2)
- Path: `src/components/onboarding/onboarding-wizard.tsx`
- Purpose: First-run flow (F-07) — full-screen overlay dialog with 6 steps: system check, DNS & HTTPS setup, external tools, default Node, park directory, first project. Each step lists passing check rows (success check icons) with feature-accurate descriptions (UAC batch, DPAPI, no polling, read-only external tools). Skip / Back / Continue, last step is a green **Done**. Gate: `localStorage["nerd-prototype2-onboarded"]`; replayed from Settings About.
- Tokens: `--success` checks, `--primary` continue.
- Used by: App root (conditional render).

#### MailScreen (v2)
- Path: `src/components/mail/mail-screen.tsx`
- Purpose: Sandboxed inbox for the `mail` route. Two panes: message list (unread dot, from, subject, snippet, paperclip for attachments, time; search filters subject/from/to; unread auto-clears on select) and a preview pane with **underline-variant tabs** (`TabsList variant="underline"` — slim text buttons with a 1px hairline and a `border-b-2 border-primary` active underline, not a segmented pill group) for `Preview / Headers / Source` (raw MIME in a mono `pre`). **Preview** renders untrusted HTML in a `sandbox=""` iframe (`srcDoc`, no `allow-*`, title "Sandboxed HTML preview — scripts and remote content disabled", with a sandboxed note beneath, F-09); non-HTML bodies fall back to a mono `pre`. Toolbar: **Clear inbox** button + (when `projectFilter` is set) a **filter chip** showing the project (`app.test`/`api.app.test`) with a Clear button (F-09 per-project inbox). Preview header has a per-message **Delete** (danger hover) and, next to the project badge, an `SMTP 127.0.0.1:2525` chip (F-09, title notes the env is injected into the project's process). Remote-image reveal: messages flagged `remoteImages` show a warning banner with a compact "Reveal images" button; it swaps the banner for image placeholders (per-message local state). Attachments render as rows with size + a per-attachment **Save** button (transient "Saved" state). Spacing: `gap-3` between header, tab bar, and content.
- Tokens: `--warning`/`--warning-soft` banner, `--primary` unread dot, `--surface-active` selected row.
- Accessibility: message rows are `<button>`s; unread dot carries `aria-label`; search input is labelled.
- Used by: App CenterStage (route `mail`).

#### useRequestAnnotations (v2)
- Path: `src/hooks/use-request-annotations.ts`
- Purpose: Module-level local-only store (`Map<requestId, {highlight, comment}>`) with subscribe/emit; `get`, `setHighlight`, `setComment`. Highlights + comments never persist and are not daemon state.

#### InspectorRail (v2)
- Path: `src/components/shell/inspector-rail.tsx`
- Purpose: Right rail (30% default, collapsible, resizable 20–40%). Children-based shell: App's `buildRailContent` renders `RequestInspector` (route `inspector`), `RuntimeDetail` (`runtimes`), `ServiceDetail` (`services`), or `DiagnosticDetail` (`diagnostics`) based on the selected object; otherwise an empty "select a request, runtime, service, or diagnostic" state. Not driven by the Projects screen: project detail lives on its own page.
- Tokens: `--sidebar` at 30% opacity; `--border` at 60%.
- Accessibility: `aria-label="Contextual inspector"`.
- Used by: App root.

#### Dialog (v2)
- Path: `src/components/ui/dialog.tsx`
- Purpose: Radix Dialog primitive (`Root/Trigger/Close/Content/Header/Description/Footer`). Compact `w-[min(92vw,24rem)]` popover-surface sheet, centered via `translate` utilities, close button top-right, footer separated by a hairline. Reuses the `cmdk-overlay-in/out` + `cmdk-content-in/out` keyframes for open/close transitions; `prefers-reduced-motion` handled globally.
- Tokens: `--color-popover`, `--border`.
- Used by: action dialogs (Projects / Runtimes / Services).

#### ActionDialogs (v2)
- Path: `src/components/actions/action-dialogs.tsx`
- Purpose: Mock, typed action layer surfaced from the product features. Never mutates state ("prototype · no state mutated") — each shows a done state. Contains `ActionButton` (compact header action button) plus:
  - **NewProjectDialog** (F-11): **Location** input (parent directory, OneDrive/UNC rejection note) / **Framework** select (adds "Vite (vanilla)") / **Node** select with an **exact-version readout** (`Exact version: v{…} · managed Nerd runtime`) / **Package manager** / **Initialize Git** toggle / optional services; safety note re approval-gated scaffold scripts + OD-029 cancellation.
  - **ParkDirectoryDialog** (F-04): directory input; immediate-child discovery, native watchers (no polling), unsupported-location rejection note.
  - **LinkProjectDialog** (F-04): path + optional alias; untrusted-until-approved note.
  - **InstallRuntimeDialog** (F-03): version select + download → checksum → extract → register steps; official-origin rule.
  - **RegisterExternalRuntimeDialog** (F-03): executable path; read-only probe note; never touches system Node.
  - **AddServiceDialog** (F-10): engine select (MySQL/PostgreSQL/Redis) with OD-002/003/004 blocker note.
  - **RegisterExternalServiceDialog** (F-10): engine + loopback endpoint; DPAPI/.env credential note.
  - **UnparkProjectDialog** / **UnlinkProjectDialog** (F-04): confirm with danger primary; data on disk never deleted; stops the project.
  - **DeleteProjectDialog**: removes Nerd's registration only (parked/linked entry, overrides, pinned state); files untouched; calls `useProjectActions.remove()`.
  - **UninstallRuntimeDialog**: managed runtimes only; removes owned runtime + download-cache entry; system Node never touched.
  - **RemoveExternalServiceDialog**: drops external connection; remote listener untouched; DPAPI credentials revoked.
- Used by: ProjectsScreen, ProjectDetailPage, RuntimesScreen, ServicesScreen.

#### ProjectActions store (v2)
- Path: `src/hooks/use-project-actions.ts`
- Purpose: Module-level override store with subscription. `statusFor(p)` returns override-or-mock status; `setStatus(id, status)` toggles running/stopped; `isRemoved(id)`/`remove(id)` track deleted projects via a module `Set`; `isTrusted(id)`/`trust(id)` track a module `Set` of trusted projects (F-05/ADR-002 — Start on an untrusted project routes through the preflight dialog). Keeps ProjectsScreen, ProjectDetailPage, and Overview's active list + running count consistent while the user interacts.

#### ServiceActions store (v2)
- Path: `src/hooks/use-service-actions.ts`
- Purpose: Module-level store for service lifecycle (F-10). `statusFor(svc)` returns the override-or-mock status; `start(id)`/`stop(id)`/`restart(id)` set status and, on stop→start and restart, briefly hold a transient `starting` state (timeout) so the UI reflects the in-flight transition instead of silently flipping between stopped and running. Keeps ServicesScreen rows, ServiceDetail buttons, and the tray consistent while the user interacts.

#### DaemonStream (v2)
- Path: `src/hooks/use-daemon-stream.ts`
- Purpose: Module-level mock daemon event stream (`subscribeDaemonStream`) with a reducer; emits `daemon.heartbeat`, `project.request`, `project.status`. `DaemonSnapshot.daemon` now carries a `state: DaemonState` (`running` / `absent` / `protocol-mismatch` / `unhealthy`, F-01); `OverviewScreen` renders a `DaemonStateBadge` (success/warning/danger tone) from it.

#### PreflightDialog (v2)
- Path: `src/components/actions/preflight-dialog.tsx`
- Purpose: Trust-and-Start preflight (F-05 + ADR-002) driven by `hooks/use-project-preflight.ts` (module store holding `pendingId`; `request`/`cancel`). Shows final command, working dir, runtime + `versionSource`, package manager, services, and port with environment conflicts (app.test → PID 8124 on 3000, warning `role="alert"` note that Nerd never stops foreign processes). Footer note: runs as current user, never elevated; trust binds to stable identity; material identity change requires trust again. Buttons: Cancel (ghost) / **Trust and start** (primary → `trust(id)` + `setStatus(id,"running")`). Rendered globally in App; Start from Projects row, ProjectDetailPage header, or tray routes here until the project is trusted.

#### ActionHost + useQuickActions (v2)
- Path: `src/components/actions/action-host.tsx`, `src/hooks/use-quick-actions.ts`
- Purpose: Global host for the command palette's Actions group. `useQuickActions` is a module store (`pending: "new-project" | "park-directory" | "link-project" | "install-node" | "add-service"`; `request`/`clear`). `ActionHost` renders the matching dialog (NewProject / ParkDirectory / LinkProject / InstallRuntime / AddService) from `action-dialogs.tsx`. Rendered once in App so palette actions open anywhere.

#### ProjectsScreen (v2)
- Path: `src/components/projects/projects-screen.tsx`
- Purpose: Projects list for the `projects` route. Header: title + count, `WorkingSessionToggle`, search field (`⌘K` focuses), and an actions row — New project / Park directory / Link existing (open the mock dialogs above). Rows: main area navigates to `ProjectDetailPage` (status dot, name + pin, domain · framework, runtime, port, live status via `useProjectActions`); a right rail holds the **Start/Stop** toggle (Play/Square) and **Delete** (Trash, confirm dialog), both through the same store.
- Tokens: `--border` at 60% with hover border; `--success`/`--warning`/`--danger`/`--muted` status dots.
- Accessibility: rows are `<button>` with a moving chevron affordance; search input has a real `<label htmlFor>`.
- Used by: App CenterStage (route `projects`, list state).

#### ProjectDetailPage (v2)
- Path: `src/components/projects/project-detail-page.tsx`
- Purpose: Full detail page for one project (route `projects` + `projectDetailId`). Back link ("Projects"), header with status dot + name + domain + chips (framework / runtime / port / source / pinned / live status), an optional **failed/crashed alert banner** (`Project.failure` — stage + cause + exit code, F-05) and an optional **registry alert banner** (`Project.registry` — kind + note, F-04), an action row (**Start/Stop** primary button toggling via `useProjectActions`, **Copy domain** with transient "Copied" feedback, **Open** in browser (anchor to `http://domain.test`), **Reveal** in Explorer (mock), **Unpark** (source `parked`) / **Unlink** (source `linked`) confirm dialogs, and a danger **Delete** that removes the registration and navigates back), then the shared `ProjectDetailTabs` (passed `onOpenMail` for the Mail tab). Accessible from the Projects list and the command palette's Projects group.
- Tokens: `--surface-active`; status chip colors via StatusDot; `--danger-soft`/`--warning-soft` alert banners.
- Used by: App CenterStage (route `projects`, detail state).

#### ProjectDetailTabs (v2)
- Path: `src/components/shell/project-inspector.tsx`
- Purpose: Project detail tab set — Radix Tabs (Config / Services / Logs / **Mail** / Activity). Config (domain/framework/runtime + **Version source** (F-03 — `nerd.json`/`.nvmrc`/`.node-version`/`engines.node`/`default`) /package-manager/command/port + **Routing** row derived from live status — running → `active · 80/443 → :port` (success), starting → `starting · Retry-After: 1s` (warning), else `503 · start this project to route traffic` (danger), F-06 — /readiness/restart-policy/autostart/source/pinned/status + **Mail SMTP** row `127.0.0.1:2525 · NERD_MAIL_HOST/PORT` (F-09) + **Registry** row (when `Project.registry`, warning tint, F-04) + **Port adapter** row ("Express process.env.PORT honored · binds loopback proxy", F-05) + a Job-Object/preflight provenance note; fields listed in `Project.overrides` render an `override` chip with a "never written back to nerd.json" title, F-04). Services (`servicesByProject` rows with status dots; attached services also render **ServiceEnvRows** showing injected `NERD_*_URL` env on `127.0.0.1:port`, F-10). Logs (**LogPane** — bounded scroll container, `role="log"` `aria-live="polite"`, follow-tail on mount, `atBottom` detection, floating "Jump to latest" button when scrolled up, ui-rules). **Mail** (per-project captured messages list + "Open inbox in Mail" → `onOpenMail(projectId)`, F-09). Activity (`requests` — status (2xx→success, 3xx→info, 4xx→warning, 5xx→danger), method, url, duration). When `Project.failure`/`Project.registry` exist, a top banner is shown above the tabs.
- Tokens: `--success`/`--warning`/`--danger`/`--info` for status coloring; `--muted-foreground` for mono meta; `--danger-soft`/`--warning-soft` for banners.
- Used by: ProjectDetailPage.

#### WorkingSessionToggle (v2)
- Path: `src/components/shell/working-session-toggle.tsx`
- Purpose: 3-state pill (`Active` / `All` / `Background`). Proxyman-inspired. Default `Active` — only pinned projects are visible. Global filter lifted to App root; buttons use `flex-1` to fill their container.
- Variants: `active` (default) | `all` | `background`.
- Keyboard: each button is focusable; `aria-pressed` reflects state.
- Tokens: `--muted` at 40% for the pill background; `--background` for the active cell; `--foreground` for active label.
- Accessibility: wrapped in `role="group"` with `aria-label="Working session filter"`.
- Used by: ProjectsScreen header (next to the search field); consumed by the Projects list and the command palette's Projects group (`active`=pinned, `background`=unpinned, `all`=all).

### Status (v2)

#### StatusDot (v2)
- Path: `src/components/status/status-dot.tsx`
- Purpose: 8px live indicator with optional pulse ring. Reusable across TitleBar, StatusBar, project rows, request rows.
- Variants: `success` | `info` | `warning` | `danger` | `muted`. Optional `pulse` for live states.
- Tokens: `--color-success`, `--color-info`, `--color-warning`, `--color-danger`, `--muted-foreground`.
- Accessibility: `aria-hidden="true"` — paired with text or icon for dual encoding.
- Used by: TitleBar (daemon), StatusBar (project + services).

### Reusable Primitives (v2)

#### Kbd (v2)
- Path: `src/components/ui/kbd.tsx`
- Purpose: Inline keyboard hint chip (e.g. `⌘K`, `esc`, `↑`/`↓`). Used in the AppSidebar header and the CommandMenu footer.
- Tokens: `--border`, `--muted`, `--muted-foreground`.
- Accessibility: native `<kbd>` element; `font-family: var(--font-mono)`.
- Used by: AppSidebar (⌘K hint), CommandMenu (footer hints).

#### Tabs (v2)
- Path: `src/components/ui/tabs.tsx`
- Purpose: Radix Tabs primitive with two variants — `segmented` (default: pill `bg-muted/50` list, `h-7` list / `h-[22px]` triggers) and `underline` (slim: `h-8` list, `items-end`, triggers are bare `text-[11px]` with a 1px `border-b border-border/40` hairline and a `border-b-2 border-primary` active underline). Compact sizing chosen after mail design review — no oversized button-group.
- Tokens: `--muted` segmented track, `--border` hairline, `--primary` active underline.
- Accessibility: Radix Tabs (`aria-orientation`, keyboard arrows, `roving tabindex`).
- Used by: ProjectDetailTabs, RequestInspector, MailScreen (underline variant), diagnostic/service detail screens.

#### CommandMenu (v2)
- Path: `src/components/shell/command-menu.tsx`
- Purpose: Global command palette (cmdk). Opened via `Ctrl/Cmd+K` or the AppSidebar header button. Groups: Go to (8 routes), Projects (built live from `mocks/data.ts`, filtered by the global working session), Runtimes, Services, Mail, **Actions** (New project / Park directory / Link existing / Install Node / Add service — dispatch through `useQuickActions` so the global `ActionHost` opens the dialog), **Theme** (Light / Dark / System — call `setTheme`, active theme shows a primary "active" hint). Selecting a route navigates; selecting a project entry navigates straight to that project's detail page (`onNavigate(route, projectId)`). Selecting an entry closes.
- Dependency: `@radix-ui/react-dialog` `Dialog.Root/Portal/Overlay/Content` wrapping a cmdk `Command` root (the cmdk `Command.Dialog` wrapper can't expose Presence, so it's used directly to allow exit animations). Centering via `translate` property; open/close animations from `data-[state=open/closed]` + `cmdk-*` keyframes in `theme.css` (fade overlay; fade + scale content).
- Tokens: `--color-popover` surface, `--color-accent` for selected item, `--border` hairline, `--muted` Kbd chips.
- Keyboard: type to fuzzy-filter; `↑`/`↓` (or `vimBindings` j/k) navigate; `↵` selects; `esc` closes. `aria-label="Search Nerd"` on the input; `label="Command palette"` on the dialog.
- Used by: App root (state + `onNavigate` owned there).

#### PlaceholderPane (v2)
- Path: `src/components/shell/placeholder-pane.tsx`
- Purpose: Reusable placeholder for the eight screens while their N2+ milestones land. Renders a milestone badge, title, description, and optional children.
- Tokens: `--border`, `--foreground`, `--muted-foreground`.
- Used by: App CenterStage during N0 (will be replaced by route screens in N2–N6).

### Desktop Chrome (v2)

The whole v2 shell is wrapped in a Windows 11-style desktop mockup: wallpaper background, centered window frame with OS-style chrome, and a bottom taskbar. This signals "desktop app" while keeping the prototype browser-only.

#### Desktop (v2)
- Path: `src/components/desktop/desktop.tsx`
- Purpose: Top-level wrapper. Composes `DesktopBackground` (full-viewport wallpaper), the centered `WindowFrame` containing the Nerd shell, and the `Taskbar` overlaid at the bottom of the viewport.
- Tokens: `--desktop-wallpaper`, `--desktop-window-shadow`, `--desktop-window-border`, `--desktop-taskbar-bg`, `--desktop-taskbar-border`, `--desktop-chrome-bg`.
- Behavior: window is capped at `max-w-[1280px]`, height is `min(88vh, 880px)` with `min-height: 560px`. On narrow viewports the window takes full width with reduced padding.
- Used by: App root.

#### DesktopBackground (v2)
- Path: `src/components/desktop/desktop-background.tsx`
- Purpose: Full-viewport wallpaper. CSS gradient backdrop with two soft blurred blobs (warm + cool) for depth, derived from the same `--desktop-wallpaper` token.
- Tokens: `--desktop-wallpaper`, plus inline soft `oklch(...)` blobs that adjust with theme.
- Accessibility: `aria-hidden="true"`, `-z-10` so it never blocks interaction.
- Used by: Desktop.

#### WindowFrame (v2)
- Path: `src/components/desktop/window-frame.tsx`
- Purpose: Centered window with rounded `rounded-xl` corners, soft multi-layer shadow, and a thin OS-style chrome strip (32px) across the top. The chrome's left drag region (`app-region: drag`) carries the brand block: primary "N" logo square + "Nerd" name. Window controls sit on the right. The Nerd shell (`AppSidebar`, content, `StatusBar`) renders inside.
- Tokens: `--desktop-window-shadow`, `--desktop-window-border`, `--desktop-chrome-bg`.
- Keyboard: the chrome strip carries `app-region: drag` (Tauri-compatible). Window controls are real `<button>` elements with `aria-label`.
- Accessibility: `role="region"`, `aria-label="Nerd desktop application window"`.
- Used by: Desktop.

#### WindowControls (v2)
- Path: `src/components/desktop/window-controls.tsx`
- Purpose: Minimize / Maximize / Close button group, Windows 11-style. Sits on the right of the window chrome. Hover affordances; close button turns red on hover (`bg-danger`).
- Variants: `minimize` (minus glyph), `maximize` (square glyph), `close` (X glyph, danger-hover).
- Keyboard: each control is focusable with visible focus ring.
- Accessibility: `role="group"`, `aria-label="Window controls"`, per-button `aria-label` and `title`.
- Behavior: non-functional in the browser prototype (visual only). Production Tauri would wire to native window commands.
- Used by: WindowFrame.

#### Taskbar (v2)
- Path: `src/components/desktop/taskbar.tsx`
- Purpose: Windows 11-style centered taskbar pinned to the bottom of the viewport. Center icons: Search, File Explorer, Browser, Terminal. Pinned project (app.test) sits in the middle column with an active underline. Right side: **NerdTray** (F-07), Network, Volume, Inspector (decorative; the actual inspector toggle lives in the StatusBar).
- Tokens: `--desktop-taskbar-bg` (translucent), `--desktop-taskbar-border` (hairline above).
- Effects: `backdrop-filter: blur(40px) saturate(180%)` (and WebKit prefix) for the Mica/Acrylic feel.
- Keyboard: every icon is focusable with visible focus ring and `aria-label`.
- Used by: App root (rendered as a sibling of `<Desktop>`).

#### NerdTray (v2)
- Path: `src/components/desktop/tray.tsx`
- Purpose: F-07 tray indicator in the Taskbar right cluster. Icon button (AppWindow) with a daemon-health dot (`snapshot.daemon.connected` → `--success`, else `--danger`), `aria-expanded`. Opens a popover menu (absolute, above the taskbar, `role="menu"`, closes on outside mousedown / Escape) with a **daemon status section** (connected dot + `snapshot.daemon.version` + running count, `role="status"` `aria-atomic`), a **menu** of: Open app (→ Overview via `onNavigate`), Diagnostics (→ Diagnostics via `onNavigate`), **Stop daemon** (two-step confirm: asks "Stop the Nerd daemon?"; confirm sets a `daemonStopped` local state and shows a stopped footer + disabled controls), and **Quit GUI** (two-step confirm: "Quit Nerd GUI?" → closes the menu; prototype only). Below the menu, the 4 most recent projects show live status via `useProjectActions` with Start/Stop quick buttons (Play/Square; untrusted projects route through the preflight dialog). Footer honours the product promise: it states only the explicit Stop daemon action ("Stop the daemon from here; Nerd never kills foreign processes"), not a generic close hint.
- Tokens: `--success` / `--danger` health dot; `--popover` surface; `--warning` for the Stop confirm.
- Accessibility: `role="menu"` / `role="menuitem"`; confirm sub-views are `role="alertdialog"`-style with Cancel / confirm buttons.
- Used by: Taskbar (receives `onNavigate` prop).

## shadcn Primitives in Use — V2

From `prototype2/src/components/ui/` (shadcn new-york style):

| Primitive | Used by |
|---|---|
| `Button` | (reserved — action buttons in dialogs/screens use inline `ActionButton`/`PrimaryButton`/`GhostButton`) |
| `Badge` | (reserved — `PlaceholderPane` removed in N7 audit; no consumers) |
| `Separator` | (reserved) |
| `Kbd` | AppSidebar (⌘K hint), CommandMenu (footer hints), ProjectsScreen (search hint) |
| `Tabs` | ProjectDetailTabs, RequestInspector, MailScreen (underline variant), Services/Diagnostics screens |
| `Switch` | SettingsScreen (Discovery), InspectorScreen (Capture), ServiceDetail (keepRunning) |
| `Dialog` | ActionDialogs (New project / Park / Link / Install Node / Add service / Unpark / Unlink / Delete / Uninstall / Remove external), ActionHost (palette Actions), PreflightDialog (Trust and Start), DiagnosticsScreen (Export support bundle) |
| `ToggleGroup`, `DropdownMenu`, `Tooltip`, `Sheet`, `Input`, `Select`, `Popover` | Available, not yet used (planned for N3–N6) |

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
