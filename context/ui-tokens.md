# UI Tokens

Nerd is a compact Windows developer utility, not a marketing dashboard. Use semantic CSS variables so light and dark themes share component code. Components must not contain hardcoded colors or raw Tailwind palette classes.

## Typography

Use local system fonts only. Do not download web fonts.

```css
--font-sans: "Segoe UI Variable", "Segoe UI", sans-serif;
--font-mono: "Cascadia Code", "Cascadia Mono", monospace;
```

| Role | Size | Weight | Line height |
|---|---:|---:|---:|
| Window title | 18px | 600 | 26px |
| Section title | 14px | 600 | 20px |
| Body | 13px | 400 | 20px |
| Label | 12px | 600 | 16px |
| Metadata | 11px | 400 | 16px |
| Code/log | 12px | 400 | 18px |

## Light Theme

```css
:root {
  --color-window: #f4f6f8;
  --color-sidebar: #edf1f4;
  --color-surface: #ffffff;
  --color-surface-raised: #ffffff;
  --color-surface-hover: #f0f4f7;
  --color-surface-selected: #e5f0f7;
  --color-border: #d8e0e6;
  --color-border-strong: #bcc8d1;
  --color-text: #17212b;
  --color-text-muted: #5f6f7d;
  --color-text-faint: #8795a1;
  --color-accent: #087ea4;
  --color-accent-hover: #066b8d;
  --color-accent-soft: #dff3fa;
  --color-success: #16845b;
  --color-success-soft: #def5e9;
  --color-warning: #a86100;
  --color-warning-soft: #fff0d2;
  --color-danger: #c9363e;
  --color-danger-soft: #fde6e8;
  --color-info: #356fd6;
  --color-info-soft: #e3edff;
  --color-log: #111820;
  --color-log-text: #d7e1e8;
}
```

## Dark Theme

```css
[data-theme="dark"] {
  --color-window: #11171d;
  --color-sidebar: #161e25;
  --color-surface: #1b242c;
  --color-surface-raised: #202b34;
  --color-surface-hover: #26333d;
  --color-surface-selected: #173849;
  --color-border: #2f3d47;
  --color-border-strong: #465761;
  --color-text: #edf3f6;
  --color-text-muted: #a8b5bd;
  --color-text-faint: #768691;
  --color-accent: #55b9dc;
  --color-accent-hover: #72c7e4;
  --color-accent-soft: #173849;
  --color-success: #55c995;
  --color-success-soft: #173a2d;
  --color-warning: #efb45a;
  --color-warning-soft: #44331d;
  --color-danger: #f1777d;
  --color-danger-soft: #482429;
  --color-info: #7fa7f5;
  --color-info-soft: #22365b;
  --color-log: #0c1116;
  --color-log-text: #d7e1e8;
}
```

## Shape And Spacing

```css
--radius-sm: 4px;
--radius-md: 7px;
--radius-lg: 10px;
--radius-pill: 999px;

--space-1: 4px;
--space-2: 8px;
--space-3: 12px;
--space-4: 16px;
--space-5: 20px;
--space-6: 24px;
```

## Elevation

```css
--shadow-flyout: 0 8px 28px rgb(0 0 0 / 0.18);
--shadow-dialog: 0 18px 60px rgb(0 0 0 / 0.28);
```

Cards inside primary content usually use borders, not shadows. Shadows are reserved for flyouts, menus, dialogs, and detached overlays.

## Component Measurements

| Element | Measurement |
|---|---:|
| Title bar | 40px |
| Sidebar | 220px |
| Compact sidebar | 56px |
| Toolbar | 44px |
| Input/button default | 32px |
| Input/button compact | 28px |
| Table row | 38px |
| Status dot | 8px |
| Content max width | none; desktop window owns width |

## Status Mapping

| State | Token |
|---|---|
| Running, healthy, ready | success |
| Starting, installing, waiting | info |
| Stopped, disabled | text-muted |
| Degraded, conflict | warning |
| Failed, crashed | danger |

## Invariants

- Never use hardcoded hex values in React components.
- Never use raw Tailwind color utilities such as `bg-blue-500`.
- Logs and code use the mono font.
- Preserve native-looking focus rings and keyboard visibility.
- Theme follows system by default and may be overridden by user.
- Motion duration stays between 100 and 180 ms and respects reduced motion.
