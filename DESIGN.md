---
name: WarCommand Tracker
description: Helldivers 2 war companion on a dark war-table HUD surface.
colors:
  void: "#04070b"
  ground: "#0b0d18"
  surface-0: "#0b0f16"
  surface-1: "#11151b"
  surface-2: "#171c24"
  surface-3: "#1b2027"
  line: "#283039"
  line-strong: "#33373f"
  text-hi: "#eaf4fa"
  text: "#cfd8de"
  text-dim: "#9fb4be"
  label: "#7f93a0"
  faint: "#56707f"
  cta: "#ffff5a"
  cta-deep: "#ad9a63"
  ink-on-cta: "#06090d"
  good: "#5ce372"
  warn: "#ffb23f"
  bad: "#ff5a5a"
  info: "#9fd4ff"
  impact: "#67e8f9"
  faction-humans: "#46a4ff"
  faction-terminids: "#ff9433"
  faction-automaton: "#ff7171"
  faction-illuminate: "#cd8ae9"
  chart-humans: "#3897f1"
  chart-terminids: "#d07c1a"
  chart-automaton: "#eb5f61"
  chart-illuminate: "#b674d2"
  chart-grid: "#1d232c"
typography:
  display:
    fontFamily: "Archivo Variable, Chakra Petch, sans-serif"
    fontSize: "2.125rem"
    fontWeight: 800
    lineHeight: 1.1
    letterSpacing: "0.04em"
  title:
    fontFamily: "Archivo Variable, Chakra Petch, sans-serif"
    fontSize: "1.3125rem"
    fontWeight: 800
    lineHeight: 1.1
    letterSpacing: "0.04em"
  value:
    fontFamily: "Chakra Petch, Segoe UI, sans-serif"
    fontSize: "1.0625rem"
    fontWeight: 700
    lineHeight: 1.35
    letterSpacing: "0.02em"
  body:
    fontFamily: "Chakra Petch, Segoe UI, sans-serif"
    fontSize: "0.8125rem"
    fontWeight: 400
    lineHeight: 1.35
    letterSpacing: "0.02em"
  body-small:
    fontFamily: "Chakra Petch, Segoe UI, sans-serif"
    fontSize: "0.6875rem"
    fontWeight: 600
    lineHeight: 1.35
    letterSpacing: "0.08em"
  label:
    fontFamily: "Chakra Petch, Segoe UI, sans-serif"
    fontSize: "0.625rem"
    fontWeight: 400
    lineHeight: 1.2
    letterSpacing: "0.14em"
  micro:
    fontFamily: "Chakra Petch, Segoe UI, sans-serif"
    fontSize: "0.5rem"
    fontWeight: 700
    lineHeight: 1.2
    letterSpacing: "0.18em"
rounded:
  none: "0px"
spacing:
  s1: "2px"
  s2: "4px"
  s3: "6px"
  s4: "8px"
  s5: "12px"
  s6: "16px"
  s7: "24px"
  s8: "32px"
components:
  button-cta:
    backgroundColor: "{colors.cta}"
    textColor: "{colors.ink-on-cta}"
    typography: "{typography.body-small}"
    rounded: "{rounded.none}"
    padding: "0 16px"
    height: "36px"
  button-hud:
    backgroundColor: "{colors.surface-1}"
    textColor: "{colors.text}"
    typography: "{typography.body-small}"
    rounded: "{rounded.none}"
    padding: "0 12px"
    height: "36px"
  button-hud-on:
    backgroundColor: "{colors.surface-1}"
    textColor: "{colors.cta}"
  filter-chip:
    backgroundColor: "{colors.surface-1}"
    textColor: "{colors.text}"
    typography: "{typography.body-small}"
    rounded: "{rounded.none}"
    padding: "0 12px"
    height: "30px"
  keycap:
    backgroundColor: "{colors.text-hi}"
    textColor: "{colors.ink-on-cta}"
    typography: "{typography.micro}"
    rounded: "{rounded.none}"
    height: "18px"
  panel:
    backgroundColor: "{colors.surface-1}"
    textColor: "{colors.text}"
    rounded: "{rounded.none}"
  tile:
    backgroundColor: "{colors.surface-2}"
    textColor: "{colors.text-hi}"
    rounded: "{rounded.none}"
    padding: "12px"
  input-search:
    backgroundColor: "{colors.surface-1}"
    textColor: "{colors.text-hi}"
    typography: "{typography.body}"
    rounded: "{rounded.none}"
    padding: "0 12px"
    height: "44px"
---

# Design System: WarCommand Tracker

## Overview

A war table read off a Super Destroyer's HUD. The layout: a top bar with News and Overview, a status strip, a three-tab dock (Galaxy map, Active planets, Hellpad), active-planet cards, a planet detail. Every surface speaks one war-map language: near-black void, hairline borders, square corners, uppercase labels with wide tracking, one hot yellow, and faction colour carrying identity. The Hellpad's planet index and the planet detail use sectioned panels in this same skin.

It is an Operate surface: people glance at it on a second screen mid-game. Density is high, controls are standard (buttons, tabs, selects, steppers), and the world shows up only through type, palette, density and three signature moves: hatch-striped section headers, tick-gapped faction meters, and the glowing Regions map.

Source of truth for every value: `src/ui/theme.css`.

## Colors

- **Void and surfaces.** `void` is the page; `ground` sits behind maps and starfields; `surface-0..3` step up for panels, tiles and raised rows. Never a pure black or a neutral grey: every surface carries the cold blue tint.
- **Text ladder.** `text-hi` for names and values, `text` for prose, `text-dim` for secondary copy, `label` for small caps labels, `faint` for disabled and axis remnants. Values and labels never wear a series colour.
- **One hot colour.** `cta` (#ffff5a) is the only accent: the primary action, focus, current selection, Major Order marks, the DAY chip. If two things on a screen are yellow, one of them is wrong.
- **Faction identity.** `faction-*` tokens mark who owns or attacks something: emblems, card edges, sector glyphs, map contours, enemy meters. Super Earth is blue in every view.
- **Chart steps.** `chart-*` are the faction hues stepped into the dark-mode lightness band (OKLCH L ≤ 0.67) and validated with the dataviz palette checker. Stack order is Terminids, Illuminate, Automatons so orange and red never touch. Use them only for chart fills and lines.
- **Status.** `good`, `warn`, `bad`, `info` mean state (progress rate, stale data, losses, impact multiplier) and always ship with a word or icon.
- **Tints.** Every tint is `color-mix()` at one of four strengths: `--wash` 8%, `--fill` 14%, `--edge` 30%, `--strong` 55%.

## Typography

Two faces. **Archivo Variable at width 125, weight 800** (a free, wide display face) sets display, page titles, panel titles, planet names and the dock dispatch label, always uppercase. **Chakra Petch** sets everything else: body, values, labels, data.

- Fixed rem scale: 34 / 21 / 17 / 13 / 11 / 10 / 8 px. No fluid type except the hero planet name and map title, which clamp.
- Tracking grows as size shrinks: 0.02em body, 0.08em small, 0.14em labels, 0.18em micro.
- Numbers are tabular (`.num`).
- Prose (briefings, dispatches, descriptions) stays mixed case at 13px with 1.5 line height and a 70ch measure.

## Layout

- App shell grid: top bar 52px, status strip 26px, content, dock 60px (64px on phones).
- Pages: `page-inner`, max 1180px, 24px top padding, 16px gaps between panels. The Overview is the exception: from 1280px it is a full-viewport board of three columns that scroll on their own; below that it flows like a page.
- Card grids use `auto-fill` with a min track (340px active planets, 380px planet rows, 320px tiles) so they collapse to one column on phones without breakpoints.
- Breakpoints: 860px (two-column layouts stack, heroes stack sphere-first), 760px (dock compacts, keyboard hints hidden, map HUD rearranges).
- Spacing comes from the 2px-based scale `s1..s8`; panel bodies pad 12px 16px.

## Elevation & Depth

Depth is layered, not floating: `--lip` (1px inner top highlight) plus `--shadow-1` on panels and cards, `--shadow-2` on heroes, toasts and drawers, `--shadow-3` on the radial options menu. Glass (`backdrop-filter: blur(11px)`) only on HUD elements that sit over the live map (hover card, control bar, map drawers, dispatch panel). Glow is reserved for data: meter fills, the liberation arc, map contours and planet halos.

## Shapes

Square everywhere (`rounded.none`). Circles appear only for things that are round in the world: planets, the faction control ring, war-log markers, hazard icon discs, live dots. Corner brackets (`.brackets`, 14px, 2px CTA) frame heroes. Hatch stripes (45°, 3px pitch) mark panel headers, the active dock tab, the meter remainder and news banners.

## Components

- **Panel**: surface-1, hairline border, header strip with an icon box and a hatch band tinted by `--tone` (Super Earth blue by default; CTA yellow only on Major Order panels). Title in Archivo; right slot for counts and timers.
- **Meter**: 7px (14px large, 3px inline) bar, fill masked into 4px segments with 1px gaps and a soft glow; remainder hatched in the enemy colour; optional white marker for the projected end result; chevron variant for the big liberation bars.
- **Active-planet card**: type and ETA above; faction-tinted header with emblem, name and sector; the biome's landscape banner under a scanline veil, with MO/DSS badges and hazard discs; large meter; outlook row; four-cell footer (player share, players, Helldiver impact, enemy regen).
- **Planet row** (Hellpad planet index): 64px, emblem, name, status line, icons and a player pill on a biome-tinted ground; active fronts carry a 2px progress line along the bottom.
- **Galactic Campaign**: emblem and name in the campaign's faction colour, dates and an outcome chip; the campaign reward in a raised box (struck through when lost); the Major Orders as folded rows with outcome chips, opening onto their briefings. The same block fills the Overview panel and each Past campaigns entry.
- **Sectors overlay**: only enemy sectors are painted, as hatched tiles in the enemy colour split by dark seams, as on the game's war table; contested sectors are faint. The Theater of War draws the same layer.
- **The Gloom**: a pre-rendered amber cloud texture masked around the planets in or on the edge of the Gloom, on every overlay; it thins once zoomed in to the planets.
- **Pawns**: small square tags beside a planet once zoomed in: sub-factions in the enemy colour, SEAF in Super Earth blue, points of interest in CTA yellow. Fractured planets are drawn as debris, razed ones scorched.
- **War activity panel** (Overview, under the Theater of War): copied at the user's request from a reference telemetry panel, measured rather than eyeballed, so it keeps its own tokens in `warActivity.css` instead of the app's: a neutral #131316-like surface with a 16px radius and backdrop blur, a hatched header with a yellow pill bar, a kicker and the title; headline readings keyed by a 2px rule in their series colour; smooth stacked fronts (#F6B012, #FF5D5E, #CB5DFF; gradient 45% to 12%, 1px edge) under a 2.5px #FBEB00 online line with peak and low dots and a pinging ring; the impact multiplier as a 2px #67E8F9 line on a right-hand axis with five even ticks; 12px corner brackets; outlined chips with a 6px radius. Neon: the online and impact lines, the dots and the header bar glow in their own colour (a 2px core plus a 7px halo), the front edges softly, and each headline key rule bleeds a little light into the panel. It is the one place where rounded corners, glass, a kicker and a dual axis are allowed.
- **Radial options**: circular glass menu with four options (pin, details, show on map, space station) around a close hub.
- **Overlay**: full-viewport planet detail on grid paper with an ESC key button in a fixed footer.
- **Keycap / KeyButton**: white keycap chip before an uppercase label (ESC CLOSE, H CONTROLS, Z DISPATCHES).
- **Stepper**: ◀ VALUE ▶ with a segmented underline; used for every setting.
- **Status chip**: outlined small caps chip in a state tone (FRONT, STALE, LINK LOST).
- **Toast**: glass panel with a toned icon box, bold title run-in and a dismiss button.

## Do's and Don'ts

Do:
- Keep the CTA yellow for one thing per view.
- Show faction through emblem plus colour, never colour alone.
- Give every live number its freshness: the strip turns STALE after ten minutes without data.
- Split charts that mix units into stacked charts on one time axis.
- Use Lucide icons or authored SVG; the faction emblems in `src/ui/emblems.tsx` are original marks.

Don't:
- Add kickers or eyebrow labels above headings; breadcrumbs are navigation and are fine.
- Draw coloured left borders thicker than 1px on cards, rows or callouts.
- Use dual-axis charts (the one exception, at the user's request, is the activity chart's impact multiplier), rainbow ramps, or a categorical hue beyond the four factions.
- Round corners, or use glass where there is no map underneath.
- Copy game art or licensed game fonts. The one exception, asked for by the user, is the biome landscape banners in `src/assets/biomes/`, credited in About.
