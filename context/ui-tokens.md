# UI Tokens

Nerd is a quiet, dense, Windows developer utility. The chrome disappears so the projects do not. Tokens are sourced from the [shadcn/ui](https://ui.shadcn.com) new-york style (OKLCH neutral palette) as the canonical base. Nerd-specific extensions layer on top for status colors and log surfaces. Components never use hardcoded colors or raw Tailwind palette classes.

## Typography

System fonts only. Do not download web fonts.

```css
--font-sans:
  "Segoe UI Variable", "Segoe UI", system-ui, -apple-system,
  BlinkMacSystemFont, "Inter", sans-serif;

--font-mono:
  "Cascadia Code", "Cascadia Mono", Consolas, "JetBrains Mono", monospace;
```

### Type scale

| Role | Size | Weight | Line | Tracking |
|---|---:|---:|---:|---:|
| Display | 36px | 600 | 44px | -0.026em |
| Headline | 28px | 600 | 36px | -0.022em |
| Title | 20px | 600 | 28px | -0.018em |
| Subtitle | 16px | 600 | 24px | -0.010em |
| Body | 13px | 400 | 20px | 0 |
| Body strong | 13px | 600 | 20px | 0 |
| Label | 12px | 500 | 16px | 0 |
| Mono | 12px | 400 | 18px | -0.005em |
| Caption | 11px | 400 | 16px | 0 |
| Eyebrow | 10px | 600 | 14px | 0.16em uppercase |

All numeric content uses tabular numerals (`font-variant-numeric: tabular-nums`). Display headings enable `font-feature-settings: "ss01", "cv11", "kern"`.

## Color

The canonical tokens follow the shadcn OKLCH neutral palette. Nerd-specific status colors are layered on top.

### Light theme

```css
:root {
  --background:              oklch(1 0 0);
  --foreground:              oklch(0.153 0.006 107.1);
  --card:                    oklch(1 0 0);
  --card-foreground:         oklch(0.153 0.006 107.1);
  --popover:                 oklch(1 0 0);
  --popover-foreground:      oklch(0.153 0.006 107.1);
  --primary:                 oklch(0.841 0.238 128.85);
  --primary-foreground:      oklch(0.405 0.101 131.063);
  --secondary:               oklch(0.967 0.001 286.375);
  --secondary-foreground:    oklch(0.21 0.006 285.885);
  --muted:                   oklch(0.966 0.005 106.5);
  --muted-foreground:        oklch(0.58 0.031 107.3);
  --accent:                  oklch(0.966 0.005 106.5);
  --accent-foreground:       oklch(0.228 0.013 107.4);
  --destructive:             oklch(0.577 0.245 27.325);
  --border:                  oklch(0.93 0.007 106.5);
  --input:                   oklch(0.93 0.007 106.5);
  --ring:                    oklch(0.737 0.021 106.9);
  --chart-1:                 oklch(0.845 0.143 164.978);
  --chart-2:                 oklch(0.696 0.17 162.48);
  --chart-3:                 oklch(0.596 0.145 163.225);
  --chart-4:                 oklch(0.508 0.118 165.612);
  --chart-5:                 oklch(0.432 0.095 166.913);
  --sidebar:                 oklch(0.988 0.003 106.5);
  --sidebar-foreground:      oklch(0.153 0.006 107.1);
  --sidebar-primary:         oklch(0.648 0.2 131.684);
  --sidebar-primary-foreground: oklch(0.986 0.031 120.757);
  --sidebar-accent:          oklch(0.966 0.005 106.5);
  --sidebar-accent-foreground: oklch(0.228 0.013 107.4);
  --sidebar-border:          oklch(0.93 0.007 106.5);
  --sidebar-ring:            oklch(0.737 0.021 106.9);

  /* Nerd status extensions */
  --color-success:           oklch(0.768 0.16 145);
  --color-success-soft:      oklch(0.967 0.02 145);
  --color-warning:           oklch(0.78 0.16 80);
  --color-warning-soft:      oklch(0.967 0.02 80);
  --color-danger:            oklch(0.628 0.245 27);
  --color-danger-soft:       oklch(0.967 0.02 27);
  --color-info:              oklch(0.6 0.18 250);
  --color-info-soft:         oklch(0.967 0.02 250);
  --color-log:               oklch(0.13 0.005 107);
  --color-log-text:          oklch(0.92 0.005 106);
}
```

### Dark theme

```css
.dark {
  --background:              oklch(0.153 0.006 107.1);
  --foreground:              oklch(0.988 0.003 106.5);
  --card:                    oklch(0.228 0.013 107.4);
  --card-foreground:         oklch(0.988 0.003 106.5);
  --popover:                 oklch(0.228 0.013 107.4);
  --popover-foreground:      oklch(0.988 0.003 106.5);
  --primary:                 oklch(0.768 0.233 130.85);
  --primary-foreground:      oklch(0.405 0.101 131.063);
  --secondary:               oklch(0.274 0.006 286.033);
  --secondary-foreground:    oklch(0.985 0 0);
  --muted:                   oklch(0.286 0.016 107.4);
  --muted-foreground:        oklch(0.737 0.021 106.9);
  --accent:                  oklch(0.286 0.016 107.4);
  --accent-foreground:       oklch(0.988 0.003 106.5);
  --destructive:             oklch(0.704 0.191 22.216);
  --border:                  oklch(1 0 0 / 10%);
  --input:                   oklch(1 0 0 / 15%);
  --ring:                    oklch(0.58 0.031 107.3);
  --chart-1:                 oklch(0.845 0.143 164.978);
  --chart-2:                 oklch(0.696 0.17 162.48);
  --chart-3:                 oklch(0.596 0.145 163.225);
  --chart-4:                 oklch(0.508 0.118 165.612);
  --chart-5:                 oklch(0.432 0.095 166.913);
  --sidebar:                 oklch(0.228 0.013 107.4);
  --sidebar-foreground:      oklch(0.988 0.003 106.5);
  --sidebar-primary:         oklch(0.768 0.233 130.85);
  --sidebar-primary-foreground: oklch(0.274 0.072 132.109);
  --sidebar-accent:          oklch(0.286 0.016 107.4);
  --sidebar-accent-foreground: oklch(0.988 0.003 106.5);
  --sidebar-border:          oklch(1 0 0 / 10%);
  --sidebar-ring:            oklch(0.58 0.031 107.3);

  --color-success:           oklch(0.748 0.18 145);
  --color-success-soft:      oklch(0.28 0.05 145);
  --color-warning:           oklch(0.798 0.18 80);
  --color-warning-soft:      oklch(0.28 0.05 80);
  --color-danger:            oklch(0.704 0.21 22);
  --color-danger-soft:       oklch(0.28 0.08 22);
  --color-info:              oklch(0.66 0.18 250);
  --color-info-soft:         oklch(0.28 0.05 250);
  --color-log:               oklch(0.1 0.005 107);
  --color-log-text:          oklch(0.92 0.005 106);
}
```

### Dark theme activation

Dark theme is activated via `.dark` class on `<html>`, set by `ThemeProvider` from system preference or user override. No `[data-theme="dark"]` selector.

### Status mapping

| State | Token |
|---|---|
| Running, healthy, ready, verified | success |
| Starting, installing, waiting, info | info |
| Stopped, disabled, idle | muted-foreground |
| Degraded, conflict, foreign listener | warning |
| Failed, crashed, destructive | destructive |

Status is always color + text or icon. Never color alone.

## Shape

```css
--radius-sm:   calc(var(--radius) - 4px);   /* default 6.25px */
--radius-md:   calc(var(--radius) - 2px);   /* default 8.25px */
--radius-lg:   var(--radius);               /* default 10px */
--radius-xl:   calc(var(--radius) + 4px);   /* default 14px */
--radius-pill: 9999px;
```

The shadcn `--radius` is `0.625rem` (10px) by default.

## Spacing

Tailwind base unit `4px`. Use semantic Tailwind spacing utilities (`gap-2`, `px-4`, `py-6`, `space-y-3`).

## Elevation

Cards inside primary content use borders, not shadows. Shadows are reserved for flyouts, menus, dialogs, and detached overlays.

```css
--shadow-flyout: 0 1px 2px rgb(0 0 0 / 0.06), 0 12px 32px rgb(0 0 0 / 0.08);
--shadow-dialog: 0 4px 12px rgb(0 0 0 / 0.10), 0 32px 80px rgb(0 0 0 / 0.16);
```

Dark theme multipliers apply automatically.

## Component measurements

| Element | Measurement |
|---|---:|
| Title bar | 40px |
| Sidebar expanded | 220px |
| Sidebar collapsed (icon rail) | 52px |
| Top toolbar | 64px |
| Section header | 40px |
| Input / button default | 32px |
| Input / button compact | 28px |
| Table row | 38px |
| List row (comfortable) | 44px |
| Status dot | 8px |
| Minimum window width | 960px |

## Motion

```css
--motion-fast:    100ms ease-out;
--motion-default: 150ms ease-out;
--motion-slow:    200ms ease-out;
```

Motion respects `prefers-reduced-motion`. Status dots pulse only on live states (running, active sync). Hover transitions are color and opacity only.

## Invariants

- Never hardcode hex values or raw OKLCH triplets in components. Tokens only.
- Never use raw Tailwind palette utilities such as `bg-blue-500`. Tokens only.
- Logs, paths, versions, ports, and timestamps use the mono font.
- Focus rings are visible, 2px, ring color.
- Theme follows system by default; user override available.
- All numerics use tabular numerals.
- Components consume `--background`, `--foreground`, `--card`, `--primary`, `--secondary`, `--muted`, `--accent`, `--destructive`, `--border`, `--input`, `--ring`, `--sidebar-*`, and `--color-*` (status). No other CSS variables are allowed.

## shadcn Primitive → Token Map

shadcn primitives in `src/components/ui/` map to tokens as follows:

| shadcn utility class | Token source |
|---|---|
| `bg-background`, `text-foreground` | `--background`, `--foreground` |
| `bg-card`, `text-card-foreground` | `--card`, `--card-foreground` |
| `bg-popover`, `text-popover-foreground` | `--popover`, `--popover-foreground` |
| `bg-primary`, `text-primary-foreground` | `--primary`, `--primary-foreground` |
| `bg-secondary`, `text-secondary-foreground` | `--secondary`, `--secondary-foreground` |
| `bg-muted`, `text-muted-foreground` | `--muted`, `--muted-foreground` |
| `bg-accent`, `text-accent-foreground` | `--accent`, `--accent-foreground` |
| `bg-destructive` | `--destructive` |
| `border-border` | `--border` (use `/40` opacity for hairline) |
| `border-input` | `--input` |
| `ring-ring` | `--ring` |
| `bg-sidebar`, `text-sidebar-foreground` | `--sidebar`, `--sidebar-foreground` |
| `bg-sidebar-primary` | `--sidebar-primary` |
| `bg-sidebar-accent` | `--sidebar-accent` |
| `border-sidebar-border` | `--sidebar-border` |
| `bg-success`, `bg-warning`, `bg-danger`, `bg-info` (Nerd status extensions) | `--color-success`, `--color-warning`, `--color-danger`, `--color-info` |
| `bg-{success,warning,danger,info}-soft` | `--color-{success,warning,danger,info}-soft` |
| `bg-log`, `text-log-text` | `--color-log`, `--color-log-text` |
