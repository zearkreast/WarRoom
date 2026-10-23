# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

(A Tauri 2 desktop + mobile app whose UI is a React web view; the design language is the same web UI on every OS.)

## Users

Primarily the author: an experienced Helldivers 2 player who checks the Galactic War between and during sessions. Also distributed to other players, so a first-time player must be able to find their way without prior knowledge of the app (icons legend, short first-run hints), while the default density stays tuned for experienced players.

## Product Purpose

A companion for the live Galactic War: what is being fought, where, how fast it is moving, what the Major Order needs, and what changed. Success is answering "where should I dive right now, and is it working?" in one glance, and being told when something important changes.

## Positioning

It keeps its own planet ownership history (collector → Supabase snapshots), so it can show how fronts moved over time, which live-only companions cannot. It runs as a native desktop/mobile app with system alerts.

## Operating Context

Glanced at on a second monitor or phone during play; opened after an alert (new Major Order, dispatch, planet captured/lost, defense started). Read-only: the app never writes game data.

## Capabilities and Constraints

- Data: a collector polls api.helldivers2.dev (planets, campaigns, war, assignments, dispatches) into Supabase; the app reads Supabase only. New data types (e.g. space stations) need a collector job and a migration.
- Views: Campaigns (default), Galaxy Map, Hellpad (Planets, Space Stations, G.W.W./History), High Command Overview, News; settings and alerts.
- Terminology follows the game: Major Order, Liberation, Defense, Sector, Supply line, Dispatch, DSS, SEST time, Day N.
- Not affiliated with Arrowhead Game Studios; no game assets or licensed fonts may be copied.
- Undecided: Warbonds and Boosters modules (need a static dataset).

## Brand Commitments

- Visual style pinned by the user: a dark war-map HUD (tokens in src/ui/theme.css).
- Name: WarCommand Tracker. Voice: in-world Super Earth deadpan for flavour lines; plain and clear for errors and states.

## Evidence on Hand

- Live game data via Supabase; no testimonials or usage numbers exist and none may be invented.

## Product Principles

1. The war state is readable in one glance; detail is one click away, never in the way.
2. Live means visibly live: every number shows its freshness, and stale or offline data says so.
3. History is the differentiator: wherever a value changes over time, show its recent trajectory.
4. Game vocabulary over app vocabulary.
