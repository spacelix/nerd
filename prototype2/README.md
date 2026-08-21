# Nerd v2 Prototype

Browser-only redesign exploration for Nerd desktop. Sibling to `prototype/` (v1). No Tauri, no Rust, no native Windows artifacts.

## Stack

- Vite + React 18 + TypeScript strict
- Tailwind CSS v4
- shadcn/ui (new-york style, neutral OKLCH base, lucide icons)
- `react-resizable-panels` for three-pane shell

## Direction (vs. v1)

Working Session + Status Bar + Full-Height Sidebar + Two-Pane.

- **AppSidebar** — full-height primary navigation with brand header, three grouped sections (Workspace / Observability / System), and a compact footer with search input + theme switcher. Slight transparency (`bg-sidebar/70`) with `backdrop-blur-md` for an acrylic feel. Global `⌘K` focuses the sidebar search; `Esc` blurs it.
- Persistent bottom **Status Bar** (Herd-style daemon / project / services / port readout)
- Two-pane content: Center Stage + Inspector Rail (resizable, hideable). Inner three-pane project rail was retired; the sidebar owns primary navigation.
- **Working Session** toggle: Active / All / Background (Proxyman-style) — kept for project filtering inside the Projects screen in N2.
- OLED-cool dark by default, light supported
- Block-based log viewer (Warp-inspired) — coming in N3+
- Multi-tab Inspector + Mail + Services (Proxyman-inspired) — coming in N3–N4

## Run

```bash
npm install
npm run dev        # http://localhost:5273
npm run typecheck  # strict TS check
npm run build      # production build
```

## Status

N0 in progress. See `context/progress-prototype-v2.md` for milestone tracker.

## Independence

`prototype2/` has its own lockfile, `node_modules/`, and `dist/`. Nothing is shared with `prototype/`. Both directories stay side-by-side during the v2 exploration.
