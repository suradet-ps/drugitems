# DrugItems - Design System (UI)

> Design language for the DrugItems desktop app - a read-only MySQL
> integrity checker where the spreadsheet snapshot is the source of truth.
>
> Stark monochrome canvas (black CTA / white surfaces), pill-shaped
> buttons/tabs/badges, vibrant accent colors reserved for identity moments.

## Overview

DrugItems stages itself as a disciplined integrity tool with a confident,
technical voice. Working surfaces anchor in stark white canvas with
deep-black typographic emphasis - the tone is precise, editorial, built for
dense data. One identity moment carries the vibrant accent: the **snapshot
card** in coral - the snapshot IS the heart of the product, the source of
truth every comparison is judged against. Semantic statuses (ok / changed /
added / missing) use restrained green / amber / blue / red washes.

**Key Characteristics:**
- Stark monochrome palette - black (`{colors.primary}`) and white (`{colors.canvas}`) - broken open by the coral snapshot identity card
- Distinct semantic encoding: success green, warning amber, added blue, missing red; coral reserved for the source-of-truth moment
- DM Sans for Latin across the entire system (Thai falls back to Sarabun - the app is Thai-only)
- Pill-shaped buttons (`{rounded.full}`) and pill-shaped tabs everywhere; rectangular forms only inside data tables
- Dense documentation-style surfaces: 14px body, generous 1.50 line-height, flat cards with hairline borders
- Desktop tool layout: top nav (white, hairline) + left control rail + right results canvas
- Flat-with-borders is the default; elevation only for modal overlays

## Colors

> Tokens distilled from the reference design language and adapted for
> DrugItems' Thai hospital context. Brand accents must never appear on
> generic controls.

### Brand & Accent
- **Brand Coral** (`{colors.brand-coral}`): The single product-identity accent. Used on the snapshot card, the "SOURCE OF TRUTH" pill, and the app logo mark. Carries the product's most attention-grabbing energy - the source of truth is the hero.
- **Brand Blue** (`{colors.brand-blue}`): Form-control activation (2px focus ring), link emphasis.
- **Brand Blue 200** (`{colors.brand-blue-200}`): "ใหม่ใน DB" badge background, info-tag backgrounds.

### Surface
- **Canvas White** (`{colors.canvas}`): Primary page background and card surface.
- **Surface** (`{colors.surface}`): Sidebar rail, search-pill rest state, table-header background.
- **Surface Soft** (`{colors.surface-soft}`): Quieter section divisions, diff-row body background.
- **Hairline** (`{colors.hairline}`): 1px card border and primary divider.
- **Hairline Soft** (`{colors.hairline-soft}`): Quieter row divider and secondary section break.

### Text
- **Ink** (`{colors.ink}`): Primary headlines, row codes, CTA text - the near-black anchor.
- **Charcoal** (`{colors.charcoal}`): Body text on light surfaces, pressed button state.
- **Slate** (`{colors.slate}`): Secondary text.
- **Steel** (`{colors.steel}`): Tertiary text, hints, table headers.
- **Stone** (`{colors.stone}`): Muted captions, chevrons.
- **Muted** (`{colors.muted}`): De-emphasized labels, disabled text.

### Semantic
- **Success Background / Text** (`{colors.success-bg}` / `{colors.success-text}`): Pale-green wash for "ตรงกัน" verdict, badges, and confirmations.
- **Warning Background / Text** (`{colors.warn-bg}` / `{colors.warn-text}`): Amber wash for "ต่าง" verdict and schema notes.
- **Error** (`#d45656`): "หายจาก DB" text, error borders and messages, disconnected dot.

## Typography

### Font Family
**DM Sans** (primary): Geometric variable sans-serif for Latin text (400-700).
**Sarabun** (Thai): Bundled fallback - Thai-only UI means every Thai glyph
renders in Sarabun while Latin (codes, numbers, column names) uses DM Sans.
No other typeface enters the brand canvas; monospace (system) appears only
for codes, `icode`, and column names.

### Hierarchy

| Token | Size | Weight | Line Height | Use |
|---|---|---|---|---|
| `{typography.start-title}` | 22px | 600 | 1.30 | First-run setup screen title |
| `{typography.modal-title}` | 18px | 600 | 1.35 | Modal headers |
| `{typography.topbar-title}` | 16px | 600 | 1.40 | Top-bar brand line |
| `{typography.panel-title}` | 14px | 600 | 1.40 | Panel headers, button labels |
| `{typography.body-md}` | 14px | 400 | 1.50 | Primary body text |
| `{typography.body-md-medium}` | 14px | 500 | 1.50 | Secondary emphasis |
| `{typography.body-md-bold}` | 14px | 700 | 1.50 | Body emphasis |
| `{typography.body-sm}` | 13px | 400 | 1.50 | Hints, help text, table cells |
| `{typography.caption}` | 12px | 400 | 1.60 | Form labels, summary labels, chips |
| `{typography.caption-bold}` | 12px | 600 | 1.50 | Badge labels, table-header text |
| `{typography.summary-num}` | 24px | 700 | 1.20 | Summary stat numbers |
| `{typography.button}` | 14px | 600 | 1.40 | Pill button labels |

### Principles
- **Weight discipline:** 400 (body), 500 (medium emphasis), 600 (headings/buttons), 700 (stat numbers). Heavier weights are not used.
- **Generous body leading** (1.50) keeps dense tables and Thai text comfortable; captions push to 1.60-1.70.
- **Single sans voice:** DM Sans (Latin) + Sarabun (Thai) only; mono only for codes.

## Layout

### Spacing System
- **Base unit**: 4px (8px primary increment).
- **Tokens**: `{spacing.xxs}` (4px) · `{spacing.xs}` (8px) · `{spacing.sm}` (12px) · `{spacing.md}` (16px) · `{spacing.lg}` (20px) · `{spacing.xl}` (24px) · `{spacing.xxl}` (32px).
- **Density**: tool surfaces stay tight - cards use `{spacing.xl}` (24px) padding, table rows `{spacing.md}` (16px), the sidebar rail `{spacing.xl}`.

### Grid & Container
- App shell: top nav (56px) + two-column body - left control rail (380px) + right results canvas (fluid, max 960px content).
- Diff list: stacked quiet white cards, each expandable to a full data table.
- Modal dialogs: 520px (680px wide variant for settings).

### Whitespace Philosophy
Tools breathe with whitespace: generous canvas padding (`{spacing.xxl}`) around results, tight internal card rhythm. The first-run setup screen gets the most air (8vh top padding) - it is the app's hero moment.

## Elevation & Depth

The system runs predominantly flat. Elevation is reserved for modal overlays.

| Level | Treatment | Use |
|---|---|---|
| 0 (flat) | No shadow; `{colors.hairline}` border | Default cards, table rows, form inputs |
| 2 (card) | `rgba(0, 0, 0, 0.08) 0px 4px 6px 0px` | First-run setup card |
| 4 (modal) | `rgba(36, 36, 36, 0.08) 0px 12px 16px -4px` | Modals, dialogs |

### Decorative Depth
- The coral snapshot card carries its identity via color alone - no shadow needed; the color does the work.

## Shapes

### Border Radius Scale

| Token | Value | Use |
|---|---|---|
| `{rounded.xs}` | 4px | Inline chips |
| `{rounded.sm}` | 6px | Compact controls |
| `{rounded.md}` | 8px | Inputs, search pill, data tables |
| `{rounded.lg}` | 12px | Diff cards, summary stat cards |
| `{rounded.xl}` | 16px | Panels, modal dialogs |
| `{rounded.xxxl}` | 24px | First-run setup card |
| `{rounded.full}` | 9999px | All buttons, all pill tabs, badges, status pill |

### Identity Geometry
- The coral "SOURCE OF TRUTH" pill uses `{rounded.full}` - pill is the brand signature.
- Panels use 16px corners; the setup card softens to 24px for the hero moment.

## Components

> Per the no-hover policy, hover states are minimal - default and
> pressed/active states only.

### Buttons

**`button-primary`** - Black pill primary CTA, the dominant action.
- Background `{colors.primary}`, text `{colors.on-primary}`, typography `{typography.button}`, padding `11px 24px`, rounded `{rounded.full}`.
- Pressed/hover lifts to `{colors.charcoal}`.
- Disabled state uses `{colors.hairline}` background and `{colors.muted}` text.

**`button-secondary`** - Outlined pill secondary action.
- Background transparent, text `{colors.ink}`, border `1px solid {colors.ink}`, rounded `{rounded.full}`.

**`button-danger`** - Outlined pill destructive action ("ลบการตั้งค่าทั้งหมด").
- Background transparent, text `#d45656`, border `1px solid {colors.error-border}`, rounded `{rounded.full}`.

### Panels (Cards)

**`panel`** - Standard tool card.
- Background `{colors.canvas}`, rounded `{rounded.xl}`, padding `{spacing.xl}`, border `1px solid {colors.hairline}`.

**`snapshot-card`** - Identity card for the source of truth.
- Background `{colors.surface-soft}`, dashed `{colors.hairline}` border, rounded `{rounded.lg}`, coral icon + file name in mono.

**`pill-tag--coral`** - "SOURCE OF TRUTH" identity pill.
- Background `{colors.brand-coral}`, text `{colors.on-primary}`, typography `{typography.caption-bold}`, letter-spacing 0.4px, rounded `{rounded.full}`, padding `2px 10px`.

### Inputs & Forms

**`text-input`** - Standard text field.
- Background `{colors.canvas}`, text `{colors.ink}`, border `1px solid {colors.hairline}`, rounded `{rounded.md}`, padding `9px 16px`, height 40px.
- Focused: `2px solid {colors.brand-blue}` (compensate padding to keep height).

**`search-input`** - Results search field.
- Background `{colors.surface}`, text `{colors.steel}`, rounded `{rounded.md}`, height 36px, border `1px solid {colors.hairline}`. Focus: blue border, canvas background.

### Tabs

**`pill-tab`** + **`pill-tab-active`** - Settings dialog tab nav.
- Inactive: background `{colors.canvas}`, text `{colors.steel}`, border `1px solid {colors.hairline}`, padding `6px 18px`, rounded `{rounded.full}`.
- Active: background `{colors.primary}`, text `{colors.on-primary}`, weight 600.

**`segmented-btn`** + **`segmented-btn-active`** - Results filter (ทั้งหมด / ต่าง / ใหม่ใน DB / หายจาก DB).
- Container: `{colors.surface}` pill with `{colors.hairline}` border, 3px padding.
- Inactive: transparent, text `{colors.steel}`. Active: `{colors.primary}` background, `{colors.on-primary}` text.

### Badges & Status

**`badge-success`** - "ตรงกัน" / confirmations.
- Background `{colors.success-bg}`, text `{colors.success-text}`, rounded `{rounded.full}`, padding `3px 10px`, weight 600.

**`badge-warn`** - "ต่าง".
- Background `{colors.warn-bg}`, text `{colors.warn-text}`.

**`badge-added`** - "ใหม่ใน DB".
- Background `{colors.brand-blue-200}`, text `{colors.brand-blue-deep}`.

**`badge-missing`** - "หายจาก DB".
- Background `{colors.error-bg}`, text `#d45656`.

**`status-pill`** - Top-bar connection state.
- Background `{colors.surface}`, border `1px solid {colors.hairline}`, rounded `{rounded.full}`, 8px dot (green / red / gray), 13px 500 text.

### Data Tables

**`data-table`** - Per-column diff table inside an expanded row.
- Background `{colors.canvas}`, text `{colors.ink}`, typography `{typography.body-sm}`, rounded `{rounded.md}`, border `1px solid {colors.hairline}`, clipped overflow.

**`data-table-header`** - Top header row.
- Background `{colors.surface}`, text `{colors.steel}`, typography `{typography.caption-bold}`, padding `8px 16px`.

**`data-table-row`** - Body rows.
- Background `{colors.canvas}`, padding `8px 16px`, bottom border `1px solid {colors.hairline-soft}`.
- Expected values render in `{colors.success-text}`, actual values in `#d45656` (mono).

### Navigation

**Top Navigation** - Sticky white bar.
- Background `{colors.canvas}`, height 56px, bottom border `1px solid {colors.hairline-soft}`.
- Left: coral app mark + "DrugItems" wordmark (+ site label).
- Right: status pill, pill buttons วิธีใช้ / ตั้งค่า.

**Sidebar Rail** - Left control rail.
- Background `{colors.surface}`, right border `{colors.hairline-soft}`, padding `{spacing.xl}`.

### Signature Components

**`verdict-banner`** - The single most important read in the app.
- `verdict--ok`: `{colors.success-bg}` background, `{colors.success-text}` text, "ไม่พบการเปลี่ยนแปลง - ตรงกันทุกคอลัมน์ทุกค่า".
- `verdict--warn`: `{colors.warn-bg}` / `{colors.warn-text}`, "พบ X รายการที่เปลี่ยนแปลง".
- Radius `{rounded.xl}`, weight 600, icon + one line of copy.

**`summary-stat-row`** - Six stat cells (ในไฟล์ / ในฐานข้อมูล / ตรงกัน / ต่าง / ใหม่ใน DB / หายจาก DB).
- Each cell: `{colors.canvas}` card, `{rounded.lg}`, hairline border; number in `{typography.summary-num}` (semantic color per status), label in `{typography.caption}` `{colors.steel}`.

**`changed-columns-chips`** - "คอลัมน์ที่ถูกแก้บ่อย" breakdown.
- Pill chips: `{colors.surface}` background, mono text, `{colors.slate}`.

**`diff-list`** - Stacked expandable diff rows.
- Each row: `{colors.canvas}`, `{rounded.lg}`, hairline border; head shows badge + `icode` (mono 600) + name + changed-column count + chevron.
- Expanded body: `{colors.surface-soft}` background, `data-table` inside.

**`schema-note`** - Structural warnings (คอลัมน์ที่มีเพียงฝั่งเดียว).
- `{colors.warn-bg}` background, `{colors.warn-text}` text, `{rounded.md}`.

**`setup-card`** - First-run hero.
- `{colors.canvas}`, `{rounded.xxxl}`, `{elev-2}`, 540px, `{spacing.xxl}` padding, coral mark + 22px title.

**`help-modal`** - Usage manual.
- 520px modal, `{rounded.xl}`, `{elev-4}`; headings `{typography.panel-title}`, body `{typography.body-sm}` `{colors.steel}`.

## Do's and Don'ts

### Do
- Use `{colors.primary}` (black) as the dominant CTA - the brand's most recognizable interactive element.
- Reserve `{colors.brand-coral}` for identity moments: the snapshot card, the "SOURCE OF TRUTH" pill, the app mark. Never for generic buttons.
- Apply `{rounded.full}` to every button, every pill tab, every badge.
- Keep semantic statuses in their own language: green = ตรงกัน, amber = ต่าง, blue = ใหม่, red = หาย.
- Keep tables flat with hairlines; the data does the work.
- Use `{typography.body-md}` (14px) as the default body size in this dense tool.

### Don't
- Don't use brand-coral on body text or large surfaces - it loses meaning when overused.
- Don't soften corners on buttons (anything less than `{rounded.full}`); the pill is a brand signature.
- Don't introduce a second Latin display typeface; DM Sans handles every role.
- Don't apply heavy shadows on white cards; flat-with-borders is the default.
- Don't put gradients on standard buttons; color identity belongs to the snapshot card only.
- Don't weaken the verdict banner - it is the first thing the operator reads.

## Responsive Behavior

### Breakpoints
| Name | Width | Key Changes |
|---|---|---|
| Narrow | < 1000px | Sidebar shrinks; toolbar wraps; summary grid drops to 3-up |
| Compact | < 720px | Summary grid 2-up; diff-table scrolls horizontally |
| Default | ≥ 1000px | Full two-column shell (380px rail + fluid canvas) |

### Touch Targets
- Pill buttons render at 38-44px height.
- Form inputs at 40px.
- Row heads at 40px+ tap targets.

### Collapsing Strategy
- **Summary stat row**: 6-up → 3-up → 2-up on narrow windows.
- **Toolbar**: filter pills and search wrap to two lines.
- **Diff table**: horizontal scroll inside the expanded body (never squeeze columns).

## Iteration Guide

1. Focus on ONE component at a time. The system has high internal consistency.
2. Reference component names and tokens directly (`{colors.primary}`, `{rounded.full}`) - do not paraphrase.
3. Keep Thai-only copy: labels are resolved in the frontend, errors in Thai at the command layer.
4. Add new variants as separate states (`-active`, `-disabled`, `-pressed`).
5. Default to `{typography.body-md}` for body; headings step `start-title → modal-title → topbar-title → panel-title`.
6. Keep brand coral confined to the snapshot identity. If coral appears on a standard button or generic surface, ask whether it earned that surface.
7. Pill-shaped buttons always; squared buttons signal "third-party widget" in this language.

## Known Gaps

- Dark-mode token values are not defined; the app ships light-only.
- Animation timings are not formalized; use 150ms ease for hover/focus transitions.
- A full on-window responsiveness audit against the 1000×680 minimum window is pending.