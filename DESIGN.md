---
name: GGUF Pilot
description: A machine-room control cabinet for local GGUF inference — matte graphite panels, engraved plates, and instrument-grade readouts.
colors:
  primary: "#9edc72"
  primary-deep: "#19351e"
  secondary: "#f0b95c"
  secondary-deep: "#3a2b13"
  tertiary: "#ef7b72"
  tertiary-deep: "#46211f"
  tertiary-lit: "#ffb1aa"
  workbench: "#171a1b"
  rail: "#1d2122"
  shell: "#202425"
  panel: "#222627"
  panel-raised: "#292e2f"
  well: "#111514"
  field: "#191d1e"
  line: "#424849"
  line-soft: "#34393a"
  ink: "#e8e9e4"
  muted: "#9ca3a0"
  plate: "#d5d1c5"
  plate-ink: "#151718"
typography:
  display:
    fontFamily: "Bahnschrift Condensed, Arial Narrow, sans-serif"
    fontSize: "44px"
    fontWeight: 700
    lineHeight: 0.95
    letterSpacing: "-0.015em"
  display-min:
    fontFamily: "Bahnschrift Condensed, Arial Narrow, sans-serif"
    fontSize: "30px"
    fontWeight: 700
    lineHeight: 0.95
    letterSpacing: "-0.015em"
  readout:
    fontFamily: "Bahnschrift Condensed, Arial Narrow, sans-serif"
    fontSize: "82px"
    fontWeight: 700
    lineHeight: 0.8
    letterSpacing: "0.025em"
  readout-min:
    fontFamily: "Bahnschrift Condensed, Arial Narrow, sans-serif"
    fontSize: "48px"
    fontWeight: 700
    lineHeight: 0.8
    letterSpacing: "0.025em"
  instrument:
    fontFamily: "Bahnschrift Condensed, Arial Narrow, sans-serif"
    fontSize: "26px"
    fontWeight: 700
    lineHeight: 1.1
    letterSpacing: "0.025em"
  instrument-compact:
    fontFamily: "Bahnschrift Condensed, Arial Narrow, sans-serif"
    fontSize: "21px"
    fontWeight: 700
    lineHeight: 1.1
    letterSpacing: "0.025em"
  panel-value:
    fontFamily: "Bahnschrift Condensed, Arial Narrow, sans-serif"
    fontSize: "23px"
    fontWeight: 700
    lineHeight: 1.1
    letterSpacing: "0.025em"
  brand:
    fontFamily: "Bahnschrift Condensed, Arial Narrow, sans-serif"
    fontSize: "20px"
    fontWeight: 700
    lineHeight: 1
    letterSpacing: "0.04em"
  title-lg:
    fontFamily: "Bahnschrift Condensed, Arial Narrow, sans-serif"
    fontSize: "17px"
    fontWeight: 700
    lineHeight: 1.1
    letterSpacing: "0.05em"
  title-runtime:
    fontFamily: "Bahnschrift Condensed, Arial Narrow, sans-serif"
    fontSize: "18px"
    fontWeight: 700
    lineHeight: 1.1
    letterSpacing: "0.05em"
  title-hardware:
    fontFamily: "Bahnschrift Condensed, Arial Narrow, sans-serif"
    fontSize: "19px"
    fontWeight: 700
    lineHeight: 1.1
    letterSpacing: "0.05em"
  title:
    fontFamily: "Bahnschrift Condensed, Arial Narrow, sans-serif"
    fontSize: "15px"
    fontWeight: 700
    lineHeight: 1.1
    letterSpacing: "0.06em"
  marker:
    fontFamily: "Bahnschrift Condensed, Arial Narrow, sans-serif"
    fontSize: "16px"
    fontWeight: 700
    lineHeight: 1
    letterSpacing: "0em"
  body:
    fontFamily: "Public Sans, Segoe UI, sans-serif"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: 1.5
  body-row:
    fontFamily: "Public Sans, Segoe UI, sans-serif"
    fontSize: "14px"
    fontWeight: 600
    lineHeight: 1.4
  body-sm:
    fontFamily: "Public Sans, Segoe UI, sans-serif"
    fontSize: "11px"
    fontWeight: 400
    lineHeight: 1.5
  label:
    fontFamily: "Bahnschrift Condensed, Arial Narrow, sans-serif"
    fontSize: "10px"
    fontWeight: 600
    lineHeight: 1.2
    letterSpacing: "0.12em"
  plate:
    fontFamily: "Bahnschrift Condensed, Arial Narrow, sans-serif"
    fontSize: "9px"
    fontWeight: 700
    lineHeight: 1.2
    letterSpacing: "0.08em"
  micro:
    fontFamily: "Bahnschrift Condensed, Arial Narrow, sans-serif"
    fontSize: "8px"
    fontWeight: 700
    lineHeight: 1.4
    letterSpacing: "0.06em"
  mono:
    fontFamily: "Consolas, Cascadia Mono, monospace"
    fontSize: "11px"
    fontWeight: 400
    lineHeight: 1.55
rounded:
  square: "0px"
spacing:
  hairline: "1px"
  xs: "4px"
  sm: "8px"
  md: "13px"
  lg: "16px"
  xl: "18px"
  gutter: "28px"
components:
  app-shell:
    backgroundColor: "{colors.workbench}"
    textColor: "{colors.ink}"
    typography: "{typography.body}"
    rounded: "{rounded.square}"
  topbar:
    backgroundColor: "{colors.shell}"
    textColor: "{colors.ink}"
    rounded: "{rounded.square}"
    padding: "0 28px"
    height: "74px"
  hairline:
    backgroundColor: "{colors.line}"
    height: "1px"
  hairline-soft:
    backgroundColor: "{colors.line-soft}"
    height: "1px"
  button:
    backgroundColor: "#303637"
    textColor: "{colors.ink}"
    rounded: "{rounded.square}"
    padding: "0 14px"
    height: "38px"
  button-hover:
    backgroundColor: "#3a4142"
    textColor: "{colors.ink}"
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "#152012"
    rounded: "{rounded.square}"
    padding: "0 14px"
    height: "38px"
  button-primary-hover:
    backgroundColor: "#b1eb88"
    textColor: "#152012"
  button-danger:
    backgroundColor: "{colors.tertiary-deep}"
    textColor: "{colors.tertiary-lit}"
    rounded: "{rounded.square}"
    padding: "0 14px"
    height: "38px"
  button-secondary:
    backgroundColor: "#282d2e"
    textColor: "{colors.ink}"
    rounded: "{rounded.square}"
    padding: "0 14px"
    height: "38px"
  input:
    backgroundColor: "{colors.field}"
    textColor: "{colors.ink}"
    rounded: "{rounded.square}"
    padding: "0 10px"
    height: "37px"
  input-label:
    textColor: "#a9b0ad"
    typography: "{typography.label}"
  nav-item:
    backgroundColor: "{colors.rail}"
    textColor: "#858e8b"
    typography: "{typography.plate}"
    rounded: "{rounded.square}"
    height: "58px"
  nav-item-active:
    backgroundColor: "#252b28"
    textColor: "{colors.primary}"
  machine-panel:
    backgroundColor: "{colors.panel}"
    textColor: "{colors.ink}"
    rounded: "{rounded.square}"
  panel-title:
    backgroundColor: "{colors.panel-raised}"
    textColor: "{colors.ink}"
    typography: "{typography.title}"
    rounded: "{rounded.square}"
    padding: "0 14px"
    height: "42px"
  spec-row-label:
    textColor: "{colors.muted}"
    typography: "{typography.body}"
  instrument:
    backgroundColor: "#1e2223"
    textColor: "{colors.plate}"
    typography: "{typography.instrument}"
    rounded: "{rounded.square}"
    padding: "16px 18px"
    height: "112px"
  instrument-running:
    backgroundColor: "#202a23"
    textColor: "{colors.primary}"
  state-tag-good:
    backgroundColor: "{colors.primary-deep}"
    textColor: "{colors.primary}"
    typography: "{typography.plate}"
    rounded: "{rounded.square}"
    padding: "4px 6px"
  state-tag-warning:
    backgroundColor: "{colors.secondary-deep}"
    textColor: "{colors.secondary}"
    typography: "{typography.plate}"
    rounded: "{rounded.square}"
    padding: "4px 6px"
  stop-indicator:
    backgroundColor: "{colors.tertiary-deep}"
    textColor: "{colors.tertiary}"
    typography: "{typography.plate}"
    rounded: "{rounded.square}"
    padding: "4px 6px"
  terminal-well:
    backgroundColor: "{colors.well}"
    textColor: "#b8eaa0"
    typography: "{typography.mono}"
    rounded: "{rounded.square}"
    padding: "16px"
  notice-code:
    backgroundColor: "{colors.plate}"
    textColor: "{colors.plate-ink}"
    typography: "{typography.plate}"
    rounded: "{rounded.square}"
    padding: "2px 5px"
  runtime-option-current:
    backgroundColor: "{colors.primary-deep}"
    textColor: "{colors.primary}"
    rounded: "{rounded.square}"
    padding: "0 14px"
    height: "38px"
  backend-plate:
    backgroundColor: "{colors.rail}"
    textColor: "#aab6af"
    typography: "{typography.plate}"
    rounded: "{rounded.square}"
    padding: "3px 5px"
  provider-tab:
    backgroundColor: "{colors.rail}"
    textColor: "#858e8b"
    typography: "{typography.plate}"
    rounded: "{rounded.square}"
    height: "40px"
  provider-tab-active:
    backgroundColor: "{colors.panel}"
    textColor: "{colors.primary}"
  secret-input:
    backgroundColor: "{colors.field}"
    textColor: "{colors.ink}"
    typography: "{typography.mono}"
    rounded: "{rounded.square}"
    padding: "0 10px"
    height: "37px"
  trial-entry-best:
    backgroundColor: "#14201a"
    textColor: "{colors.primary}"
    typography: "{typography.mono}"
    rounded: "{rounded.square}"
    padding: "12px 16px"
  trial-error:
    backgroundColor: "{colors.tertiary-deep}"
    textColor: "{colors.tertiary}"
    typography: "{typography.mono}"
    rounded: "{rounded.square}"
    padding: "8px 10px"
---

# Design System: GGUF Pilot

## Overview

**Creative North Star: "The Machine-Room Control Cabinet"**

GGUF Pilot looks like the front face of a rack-mounted control cabinet in a machine room: matte graphite steel, engraved label plates, hard-edged panels butted against each other with a single hairline of separation, and a small number of lamps that mean exactly one thing each. The interface is a physical instrument for a physical job — a 30 GB model is about to occupy real memory on real silicon, and the screen should feel as consequential as that.

Density is a feature, not a compromise. Values live next to the controls that produce them, exact commands are shown rather than summarized, and every number is set in tabular figures so a column of measurements reads as a column. The palette is nearly monochrome so that the three signal colors — green, amber, red — carry real information instead of decoration. Nothing floats: depth comes from tonal steps between regions (`#171a1b` workbench → `#1d2122` rail → `#222627` panel → `#292e2f` raised header), never from drop shadows.

Two worlds are explicitly rejected. It is not a rounded-card SaaS dashboard: no pill buttons, no soft shadows, no pastel status chips, no 24 px radii. It is also not a decorative sci-fi terminal: no scanlines, no glow text, no ambient animation, no fictional telemetry. Every lamp, plate, and readout corresponds to a fact the control plane actually knows.

**Key Characteristics:**

- Zero border radius on every rectangular surface; corners are cut, not eased.
- Condensed uppercase (`Bahnschrift Condensed`) for structure and measurement; `Public Sans` for prose; `Consolas` for paths, commands, and logs.
- Three signal colors, each with one meaning, on an otherwise graphite field.
- 1 px structural lines and tonal region contrast instead of elevation.
- Tabular numerics everywhere a value can change.

## Colors

A graphite machine finish in six tonal steps, with three reserved signal colors and one warm "engraved plate" neutral that stands in for a physically labelled metal tag.

### Primary

- **Signal Green** (`#9edc72`): the only affirmative color. It marks a running process, a valid configuration, an installed managed runtime, the active navigation item, benchmark result figures, and the single Start action. On a green-lit screen exactly one thing is true: the machine is doing what you asked.
- **Signal Green Deep** (`#19351e`): the recessed field behind green plates and tags, so a lit tag reads as an inset lamp rather than painted text.

### Secondary

- **Caution Amber** (`#f0b95c`): incomplete, blocked, or unverified. Missing shards, user-supplied (not download-verified) runtimes, security-relevant configuration notes, and the focus ring. Amber never means failure — it means *the operator has to look*.
- **Caution Amber Deep** (`#3a2b13`): recessed field behind amber plates, with `#684f23` for the heavier warning band.

### Tertiary

- **Stop Red** (`#ef7b72`) on **Stop Red Deep** (`#46211f`): reserved exclusively for Stop and destructive conditions. Red appears on at most one control per screen. It is never used for validation text, never for emphasis.

### Neutral

- **Workbench** (`#171a1b`): the deepest ground — app background, scrollbar track, and the surface the whole cabinet sits on.
- **Rail** (`#1d2122`) and **Shell** (`#202425`): the chassis. Navigation rail, top bar, and table bodies.
- **Panel** (`#222627`): the standard cabinet face for any bordered instrument or settings group.
- **Panel Raised** (`#292e2f`): panel headers, table heads, and the system notice strip — the engraved band above a set of controls.
- **Well** (`#111514`): log and command surfaces, sunk below every other region because text is being poured into them.
- **Field** (`#191d1e`): input interiors, so a field reads as a machined recess.
- **Line** (`#424849`) and **Line Soft** (`#34393a`): the two structural rules. `line` separates instruments and panels; `line-soft` separates rows inside one panel.
- **Ink** (`#e8e9e4`) and **Muted** (`#9ca3a0`): primary text and supporting text.
- **Engraved Plate** (`#d5d1c5`) on **Plate Ink** (`#151718`): the warm off-white of a stamped metal tag. Used for measured values, the brand mark, and the notice code — the parts of a cabinet that would physically be a label.

### Named Rules

**The One Meaning Rule.** Each signal color carries exactly one meaning across the entire application: green = running/valid/installed, amber = incomplete/unverified/needs attention, red = stop/destructive. A color is never borrowed for emphasis, decoration, or brand flourish.

**The Graphite Majority Rule.** Signal colors occupy under 10% of any screen. If a view reads as colorful, the color has stopped being information.

**The Recessed Lamp Rule.** A colored foreground always sits on its matching deep field with a 1 px border of the same hue family (`#426243` for green, `#6f5425` for amber). Colored text on bare panel is never used for state.

## Typography

**Display / instrumentation font:** `Bahnschrift Condensed`, falling back to `Arial Narrow`, then `sans-serif`. Windows ships Bahnschrift; the condensed narrow face is what makes a dense readout legible at 9–10 px and monumental at 82 px.
**Body / control font:** `Public Sans`, falling back to `Segoe UI`.
**Mono font:** `Consolas`, falling back to `Cascadia Mono`.

**Character:** the pairing reads as engraved-metal labels beside printed operating text. The condensed face is structural — it names, measures, and stamps. `Public Sans` is used only where a human sentence has to be read comfortably. The two never trade jobs.

### Hierarchy

Sizes below are the rendered CSS. Where a role is fluid, the frontmatter records the ceiling of the clamp, because the token format carries a single dimension.

- **Display** (700, `clamp(30px, 3vw, 44px)`, line-height 0.95, `-0.015em`, uppercase): one per screen, the view name. Set tight enough to read as a machined header rather than a marketing headline.
- **Readout** (700, `clamp(48px, 7vw, 82px)`, line-height 0.8, tabular): the single most important measured number on a screen — the benchmark result. Nothing else is allowed at this size.
- **Instrument value** (700, 26 px, `0.025em`, plate color, tabular): the four status instruments and panel-level figures.
- **Title** (700, 15–18 px, `0.05–0.07em`, uppercase): panel headers, `legend`, runtime section titles.
- **Body** (400, 11–12 px, line-height 1.5): descriptions, group notes, help text. The only place sentence case and prose live.
- **Label** (600, 10 px, `0.12em`, uppercase): field labels, instrument captions, table heads.
- **Plate** (700, 8–9 px, `0.06–0.08em`, uppercase): state tags, capability chips, trust badges, provenance lines — text small enough to read as stamped.
- **Mono** (400, 8–11 px, line-height 1.55): paths, exact commands, digests, logs. Always `overflow-wrap: anywhere`, never truncated with an ellipsis when the value is verifiable evidence.

### Named Rules

**The Uppercase-Is-Structure Rule.** Condensed uppercase names, labels, and measures. It is never used for a sentence the user has to read for meaning; prose is always `Public Sans` in sentence case.

**The Tabular Rule.** Anything that can change while the user watches — throughput, sizes, ports, counts, sample bars — is set `font-variant-numeric: tabular-nums` so digits do not shift under the eye.

**The Evidence-Is-Mono Rule.** Paths, commands, digests, and log output are always mono and always selectable in full. Truncating a SHA-256 or an exact command would destroy the thing that makes the panel trustworthy.

## Layout

The desktop shell is a two-column grid: an 88 px fixed navigation rail and a flexible workspace. The workspace stacks a 74 px runtime header (product identity left, live runtime plate right), a 35 px system notice strip, and one scrolling screen region padded `26px 28px 34px`. Only the screen region scrolls; the rail, header, and strip are fixed structure.

Screens are built from butted rectangles, not floating cards. Panels sit edge to edge separated by 16 px gutters, and every panel carries a 42–58 px header band in `panel-raised`. Views use asymmetric two-column grids where one side is the working surface and the other is evidence: Control is `minmax(320px, .85fr) / minmax(420px, 1.4fr)` (profile cabinet beside the log well); Profile is `minmax(520px, 1.25fr) / minmax(340px, .75fr)` with a sticky exact-command panel; Runtime is `minmax(520px, 1.4fr) / minmax(310px, .6fr)` with a sticky managed-runtime sidebar; Benchmark is `minmax(300px, .65fr) / minmax(440px, 1.35fr)`.

The Control view opens with a four-cell instrument strip (`1.2fr` primary readout + three equal cells) at 112 px per cell, then hands the remaining vertical field to the profile and log panels via `min-height: calc(100vh - 390px)` so the log well always fills the cabinet rather than floating in whitespace. Inventory is a dense table with a `min-width: 850px` body and a six-column row grid at 58 px per row — it scrolls horizontally rather than reflowing, because a comparison table stops working the moment columns disappear. Settings groups are two-column `fieldset` grids at 13 px gaps, with `.wide` fields spanning both columns.

Spacing follows a coarse rhythm: 1 px structural lines, 4–8 px inside a control, 13–18 px inside a panel, 16 px between panels, 28 px page gutter.

Responsive behavior has two breakpoints, and both are layout changes rather than a scaled-down desktop:

- **≤ 980 px:** the rail narrows to 72 px, the instrument strip becomes 2×2 with internal borders rebalanced, and every asymmetric two-column view collapses to a single column. Sticky evidence panels become static — a sticky panel in a single column is a scroll trap.
- **≤ 680 px:** the shell stops being a grid. The rail becomes a fixed 64 px bottom bar with six equal-width items (`repeat(6, minmax(0, 1fr))`), the brand mark and rail footer are dropped, the runtime plate is dropped from the header, `body` regains scroll, and the screen pads to `22px 16px 110px` so content always clears the navigation bar. Controls grow: buttons 46 px, inputs and toggles 44 px, path actions 42 px. Settings collapse to one column and `.wide` stops spanning.

### Named Rules

**The Butted-Panel Rule.** Panels touch their neighbours across a 16 px gutter with 1 px borders. No panel gets a radius, a shadow, or a margin that makes it look detachable.

**The Clearance Rule.** On mobile, the bottom navigation is fixed, so the screen carries 110 px of bottom padding. At maximum scroll the last content sits clear of the bar rather than under it.

**The No-Reflow-Table Rule.** Dense comparison tables scroll horizontally at `min-width: 850px`. They are never reflowed into stacked cards, because losing column alignment loses the table's only purpose.

## Elevation & Depth

There are no shadows in the resting interface. Depth is entirely tonal: six graphite steps from `#111514` (sunk log well) through `#171a1b` (workbench), `#1d2122` (rail), `#202425` (shell), `#222627` (panel face), to `#292e2f` (raised header band). A surface reads as nearer because it is lighter and separated by a hairline, not because it casts light.

Only two lighting effects exist, and both are lamps rather than elevation:

- **Live signal glow** (`box-shadow: 0 0 10px rgba(158, 220, 114, .35)`): the rail's status dot when a server is running. This is a lamp that is on.
- **Unlit signal inset** (`box-shadow: inset 0 0 0 1px #787f7d`): the same dot when idle, reading as a dark bulb behind a metal ring.

### Named Rules

**The Flat-Cabinet Rule.** No `box-shadow` on any panel, button, input, dialog, or hovered surface. If something needs to feel closer, move it one tonal step lighter and give it a 1 px `line` border.

**The Lamps-Only Rule.** Glow is permitted only on a state indicator that is genuinely on. It is never used for hover, emphasis, or ambience.

## Shapes

Every rectangle in the system has `border-radius: 0`. Buttons, inputs, selects, panels, tags, chips, plates, wells, and the brand mark are all hard-cornered — including form controls, which explicitly reset `border-radius: 0` against browser defaults. The only curve in the product is a circle: status dots and plate lights at `border-radius: 50%`, which read as physical indicator lamps rather than rounded UI.

Borders do the structural work. `1px solid #424849` bounds an instrument or panel; `1px solid #34393a` divides rows inside one; brighter hue-matched borders (`#426243`, `#6f5425`, `#824a46`) bound lit tags and destructive controls. The brand mark is a 48 × 48 square with a 2 px plate-colored stroke and a 6 px green square notched into its top-right corner — a plate with a lamp in it, and the one piece of pure identity geometry in the product.

Disclosure uses typographic markers, not chevrons: `+` when closed, `−` when open, set in the condensed display face. Native `<details>` markers are removed so the marker reads as an engraved symbol on the panel band.

### Named Rules

**The Zero-Radius Rule.** No radius on anything rectangular, including inputs and selects. The only round things in GGUF Pilot are indicator lamps.

**The Border-Is-Structure Rule.** Separation is always a 1 px line in `line` or `line-soft`. Gaps alone never imply a boundary, and a boundary is never implied by a shadow.

## Components

### Buttons

- **Shape:** hard-cornered (0 px radius), 38 px tall on desktop and 46 px on mobile, `0 14px` padding, 8 px icon-to-label gap.
- **Default:** `#303637` face, `#596061` border, ink label at 12 px/700. Hover lightens to `#3a4142` with a `#707879` border; there is no transform, no shadow, no lift.
- **Primary:** signal-green face (`#9edc72`) with a `#b9ef93` border and near-black label (`#152012`), hovering to `#b1eb88`. One primary per screen — it is the Start switch.
- **Danger:** `#46211f` face, `#ffb1aa` label, `#824a46` border. Stop only.
- **Secondary:** `#282d2e` face for the quieter of two adjacent actions.
- **Disabled:** `opacity: .42` with `cursor: not-allowed`. Disabled controls stay visible and legible so the operator can see what would be possible.
- **Text button:** borderless, transparent, signal-green label at 11 px/700 — used for a single inline escape hatch inside a panel.

### Inputs and Fields

- **Style:** 37 px tall (44 px on mobile), `#191d1e` recessed interior, `#4b5253` border, 0 px radius, 11 px value text, label above at 10 px/600 in `#a9b0ad`.
- **Focus:** the border turns signal green; the global `:focus-visible` ring is a 2 px amber outline at 2 px offset. Focus is always visible — it is never removed for aesthetics.
- **Disabled:** value text drops to `#8d9491`; the field keeps its recess so the layout does not shift.
- **Help text:** 9 px/400 in `#7f8885` directly under the control, never in a tooltip. A setting that needs explanation gets it inline.
- **Toggle line:** a bordered 37 px row (44 px mobile) in `#1d2122` with a 15 px green-accented checkbox and its label — a switch on a panel, not a floating checkbox.
- **Path bar:** a 45 px bordered strip in `#252a2b` holding an icon, a borderless mono path input, a count, and a picker action. The path is always editable text *and* pickable.

### Panels and Containers

- **Corner style:** square (0 px).
- **Background:** `#222627` face on a `#424849` border; log and command wells drop to `#111514`.
- **Header band:** 42 px, `#292e2f`, bottom-bordered, holding a 15 px uppercase title and an optional right-aligned state tag.
- **Rows:** 38 px `spec-list` rows with `line-soft` dividers, label left in muted body, value right in 12 px condensed with `0.05em` tracking.
- **Shadow strategy:** none. See Elevation & Depth.

### Navigation

- **Desktop rail:** 88 px wide, `#1d2122`, right-bordered. Items are 58 px icon-over-label stacks with 9 px/700 uppercase labels in `#858e8b`. Hover fills `#252a2b` and lifts the label to ink. Active is signal green on `#252b28` with a `#465048` border — the lit button on a panel.
- **Rail footer:** a status dot plus a condensed 11 px legend, dropped entirely on mobile.
- **Mobile:** the rail becomes a fixed bottom bar, 64 px tall, six equal columns, items 52 px minimum. All six destinations (Control, Inventory, Runtime, Profile, AI Tune, Benchmark) stay visible; nothing collapses into a menu.

### Instrument Strip

The signature component. A single bordered strip is divided by 1 px rules into four cells — a wider primary readout plus three supporting cells — each 112 px tall with a 10 px `0.12em` uppercase caption, a 26 px condensed plate-colored value, and a 10 px muted qualifier. The primary cell sits one tone lighter (`#252a2b`) and, when a server is running, shifts to `#202a23` with a signal-green value. State is read from the whole strip at a glance, before any text is parsed.

### State Tags and Plates

- **Good:** signal green on `#19351e` with a `#426243` border.
- **Warning:** caution amber on `#3a2b13` with a `#6f5425` border.
- **Trust badges:** `managed` uses the green plate; `supplied` uses the amber plate — the same grammar applied to provenance.
- **Capability chips:** `#b8d7aa` on `#202a22` with a `#425945` border, 10 px condensed, wrapped in a flex stack.
- Every tag contains words. Color is a second channel on top of text, never the only channel.

### Terminal Wells

Log and exact-command surfaces share one treatment: `#111514` ground, `#b8eaa0` text for logs and `#d8d8cd` for commands, 11 px/1.55 mono, 16 px padding, `white-space: pre-wrap` with `overflow-wrap: anywhere`. The log well flexes to fill its panel; the command well is capped at 360–390 px and scrolls. The empty state is a centered two-line block — a bold reason plus a 10 px recovery action — never a blank rectangle.

### Disclosure Groups

Advanced settings live in one `details` zone with a `#53605a` border and a 58 px summary in signal green, marked by an absolutely positioned `+`/`−` at 20 px on the right. Inside, nested `option-group` panels each carry a 42 px summary with an inline `+`/`−` that turns green when open, and a two-column 14 px grid of controls. Depth of nesting stops at two: zone, then group.

### Setup Steps

A three-cell bordered strip of 60 px steps, each a 22 px square number badge beside a 11 px title and 9 px caption. Active fills `#292d25` and inverts the badge to green-on-dark; complete keeps a green outlined badge. First-run progress is structure, not a modal. The AI Tune screen reuses the strip as a readiness gauge (provider → model → tune).

### Runtime Option States

An official build row is never a permanent call to action; its button states the row's true relationship to the runtime in use, and only one relationship can be true at a time:

- **Up to date** — the active executable *is* this package at the current release. The row lifts to `#1f2622`, the tag reads `ACTIVE · b<build>`, and the button becomes a recessed green lamp (`is-current`: green label on `#19351e`, `#426243` border, `opacity: 1`, `cursor: default`). It is disabled because there is nothing to do, not because something is wrong.
- **Update to b<tag>** — the active runtime is this backend but an older build. Amber tag `UPDATE FROM b<old>`, primary green button. This is the one case where a row earns the primary action.
- **Use this build** — already downloaded into the managed folder but not active. Secondary button; activating it is a switch, not a download.
- **Install** — not present on this PC. Primary only when recommended, otherwise secondary.

Backend identity is a neutral plate (`#aab6af` on `#1d2122`) reading e.g. `CUDA 13 · FROM DLLS`, beside the managed/supplied trust badge. Installed managed builds list as 30 px uppercase condensed rows in the sidebar; the current one is the green lamp.

### Provider Tabs

A bordered strip of equal-width 40 px tabs (`#1d2122`, 10 px condensed uppercase, `#858e8b`). The active tab is signal green on the panel face with a 2 px green bottom rule — a border, not a shadow. On mobile the strip reflows to a 2 × 2 grid of 44 px tabs. Below it, the credential field is a mono `password` input with `0.12em` tracking so the masked glyphs read as a code, paired with a **Store** button; help text names the storage location (Windows Credential Manager) inline. Connection proof is a bordered mono line: amber plate while unknown or failing, green plate once the provider has answered.

### Trial Ledger

The tuning log is a terminal well (`#111514`) of 12 px-padded entries divided by `#1f2526` rules. Each entry heads with a 26 × 20 px bordered index chip (`T0`, `T1`…), the changed fields as `key=value` mono, and the measured tok/s as a 15 px condensed figure in signal green. The best entry lifts to `#14201a` with a green index chip; a failed entry prints `FAILED` in stop red and its error in a red-plated mono block capped at 120 px. Trial bars reuse the benchmark bars: best in green, others in `#5d6a61`, failures as a red-deep stub. The ledger is evidence — nothing in it is summarized away.

## Do's and Don'ts

### Do:

- **Do** set `border-radius: 0` on every new rectangular surface, including inputs and selects.
- **Do** convey state with a bordered text tag on a deep field, so it survives grayscale and colorblind viewing.
- **Do** give measured values `font-variant-numeric: tabular-nums` and the condensed display face.
- **Do** separate regions with `1px solid var(--line)` (between panels) or `var(--line-soft)` (within a panel).
- **Do** step tone to signal depth: `#111514` well → `#171a1b` ground → `#1d2122` rail → `#222627` panel → `#292e2f` header.
- **Do** print paths, exact commands, and digests in full mono text the user can select.
- **Do** keep exactly one primary green action per screen, and reserve red for Stop.
- **Do** put explanatory help inline under the control at 9 px, not in a hover tooltip.
- **Do** keep mobile targets at 44 px or more, and hold 110 px of bottom clearance above the fixed navigation bar.
- **Do** write empty states that name the reason and the recovery action.
- **Do** make a button say what is true about its row (`Up to date`, `Update to b10752`, `Use this build`, `Install`) instead of repeating one verb everywhere.

### Don't:

- **Don't** add `box-shadow` to a panel, button, input, or hover state; the cabinet is flat.
- **Don't** introduce a fourth signal color, or reuse green, amber, or red for decoration or emphasis.
- **Don't** set body prose in condensed uppercase, or set labels and readouts in `Public Sans`.
- **Don't** hide a disabled control; drop it to `opacity: .42` and leave it readable — except a satisfied state (`is-current`), which stays at full opacity as a lamp.
- **Don't** ever render a stored secret; show a masked suffix and the storage location, and keep the input a `password` field.
- **Don't** truncate a digest, path, or exact command with an ellipsis — wrap it with `overflow-wrap: anywhere`.
- **Don't** reflow the inventory table into stacked cards; let it scroll at `min-width: 850px`.
- **Don't** keep a sticky evidence panel once the layout is single-column; make it static at ≤ 980 px.
- **Don't** animate for ambience. Motion is limited to a 0.9 s linear spin on an icon representing work actually in progress, and `prefers-reduced-motion` collapses it.
- **Don't** rely on a chevron or icon-only affordance for disclosure; use the engraved `+` / `−`.
- **Don't** remove the amber `:focus-visible` ring.
