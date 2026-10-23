import type { Faction, Planet } from "../api/types";

/** Map units: the API's [-1, 1] galaxy coordinates scaled up, y flipped so north is up. */
export const GALAXY_SCALE = 500;
export const project = (p: { x: number; y: number }): [number, number] => [p.x * GALAXY_SCALE, -p.y * GALAXY_SCALE];

type Pt = [number, number];

export interface Landmark {
  name: string;
  sector: string;
  note: string;
  biome: string;
  position: { x: number; y: number };
}

/** Worlds the game's galaxy map shows but the war API leaves out; positions measured off the map. */
export const LANDMARKS: Landmark[] = [
  {
    name: "Mars",
    sector: "Sol",
    note: "Glassed by the Illuminate. No war data is published for Mars.",
    biome: "Scorched",
    position: { x: 0.043, y: 0.035 },
  },
];

export type Anomaly = "wormhole" | "blackhole";

/** Planets the game draws as space anomalies instead of worlds. */
export const ANOMALIES: Record<string, Anomaly> = { MERIDIA: "wormhole", PENTA: "blackhole" };
export const anomalyOf = (p: { name: string }): Anomaly | undefined => ANOMALIES[p.name.toUpperCase()];

// ---------- Sectors ----------

/**
 * The game lays sectors out on concentric rings around Super Earth, each sector an unbroken
 * run of angle within a ring. Ring width and offset are fitted to the planets' distances
 * from the centre; inside one ring, planets of a sector sit next to each other by angle.
 */
export const RING_WIDTH = 0.098;
const RING_OFFSET = 0.0035;
const RINGS = 10;

/** Ring boundary radii, galaxy units, from the core's edge out to the rim. */
export const RING_RADII = Array.from({ length: RINGS }, (_, i) => RING_OFFSET + (i + 1) * RING_WIDTH);

export interface SectorRun {
  sector: string;
  /** Radians, counter-clockwise from +x; a1 > a0 and a1 - a0 <= 2π. */
  a0: number;
  a1: number;
}

export interface PolarRing {
  /** Galaxy units. */
  r0: number;
  r1: number;
  /** Covers the full circle, in angle order. */
  runs: SectorRun[];
}

export interface PolarEdge {
  d: string;
  a: string;
  /** null on the rim. */
  b: string | null;
}

export interface PolarSectors {
  rings: PolarRing[];
  /** Union of each sector's runs, map units. */
  fills: Map<string, string>;
  /** Boundaries between neighbouring runs of different sectors, map units. */
  edges: PolarEdge[];
  /** Disc radius, map units. */
  rim: number;
}

const TAU = Math.PI * 2;
const norm = (a: number) => ((a % TAU) + TAU) % TAU;

/** Point at radius r (galaxy units) and angle a (radians, counter-clockwise from +x), in map units. */
export const polar = (r: number, a: number): Pt => [r * Math.cos(a) * GALAXY_SCALE, -r * Math.sin(a) * GALAXY_SCALE];
const fmt = ([x, y]: Pt) => `${x.toFixed(1)},${y.toFixed(1)}`;

/** SVG arc command from the current point (at a0) to angle a1; sweep 0 runs counter-clockwise on screen. */
export function arcTo(r: number, a0: number, a1: number, sweep: 0 | 1 = 0): string {
  const R = (r * GALAXY_SCALE).toFixed(1);
  return `A${R},${R} 0 ${Math.abs(a1 - a0) > Math.PI ? 1 : 0} ${sweep} ${fmt(polar(r, a1))}`;
}

function wedge(r0: number, r1: number, a0: number, a1: number): string {
  // A full ring is drawn as two halves: an annulus's inner circle would cut a hole in the same sector's inner runs.
  if (a1 - a0 >= TAU - 1e-9) return wedge(r0, r1, a0, a0 + Math.PI) + wedge(r0, r1, a0 + Math.PI, a1);
  const outer = `M${fmt(polar(r1, a0))}${arcTo(r1, a0, a1)}`;
  return r0 === 0 ? `${outer}L0,0Z` : `${outer}L${fmt(polar(r0, a1))}${arcTo(r0, a1, a0, 1)}Z`;
}

function runAt(ring: PolarRing, a: number): string {
  const start = ring.runs[0].a0;
  const x = start + norm(a - start);
  return (ring.runs.find((r) => x < r.a1) ?? ring.runs[ring.runs.length - 1]).sector;
}

/** Sector drawn at a galaxy point, or null outside the disc. */
export function sectorAt(geo: PolarSectors, x: number, y: number): string | null {
  const r = Math.hypot(x, y);
  const ring = geo.rings.find((g) => r < g.r1);
  return ring ? runAt(ring, Math.atan2(y, x)) : null;
}

type SectorPlanet = Pick<Planet, "index" | "sector" | "position">;

interface Group {
  s: string;
  /** Angles of the run's first and last planet; `first` may be below 0 for the run that wraps past 0. */
  first: number;
  last: number;
}

/** A ring's planets in angle order, consecutive planets of one sector grouped; null for an empty ring. */
function ringGroups(pins: { s: string; a: number; ring: number }[], i: number): Group[] | null {
  const inRing = pins.filter((p) => p.ring === i).sort((x, y) => x.a - y.a);
  if (!inRing.length) return null;
  const groups: Group[] = [];
  for (const p of inRing) {
    const g = groups[groups.length - 1];
    if (g && g.s === p.s) g.last = p.a;
    else groups.push({ s: p.s, first: p.a, last: p.a });
  }
  if (groups.length > 1 && groups[0].s === groups[groups.length - 1].s) {
    const tail = groups.pop()!;
    groups[0].first = tail.first - TAU;
  }
  return groups;
}

/** Ring a galaxy radius falls in, from 1 (just outside the core) to the outermost. */
function ringIndex(r: number): number {
  const i = RING_RADII.findIndex((R) => r < R);
  return i < 0 ? RINGS - 1 : Math.max(1, i);
}

/**
 * Sector shapes from planet positions alone, as the game doesn't publish them. Where two sectors meet
 * in several rings they share one boundary angle wherever their planets allow, so sectors come out as
 * the game's stepped blocks rather than a zigzag that shifts from ring to ring.
 */
export function buildPolarSectors(planets: SectorPlanet[]): PolarSectors {
  const home = planets.find((p) => p.index === 0)?.sector ?? "Sol";
  const rings: PolarRing[] = [{ r0: 0, r1: RING_RADII[0], runs: [{ sector: home, a0: 0, a1: TAU }] }];
  const pins = planets
    .filter((p) => p.index !== 0)
    .map((p) => ({
      s: p.sector,
      a: norm(Math.atan2(p.position.y, p.position.x)),
      ring: ringIndex(Math.hypot(p.position.x, p.position.y)),
    }));
  const perRing = Array.from({ length: RINGS }, (_, i) => (i === 0 ? null : ringGroups(pins, i)));

  // Each pair's gap between its planets, over every ring where the two meet, in the first ring's angle frame.
  const pairs = new Map<string, { ref: number; last: number; first: number }>();
  const bound = (groups: Group[], k: number) => {
    const prev = groups[(k - 1 + groups.length) % groups.length];
    const last = k === 0 ? prev.last - TAU : prev.last;
    return { key: `${prev.s}|${groups[k].s}`, last, first: groups[k].first };
  };
  const shiftTo = (ref: number, a: number) => Math.round((ref - a) / TAU) * TAU;
  for (const groups of perRing) {
    if (!groups || groups.length < 2) continue;
    groups.forEach((_, k) => {
      const { key, last, first } = bound(groups, k);
      const p = pairs.get(key);
      if (!p) return void pairs.set(key, { ref: (last + first) / 2, last, first });
      const shift = shiftTo(p.ref, (last + first) / 2);
      p.last = Math.max(p.last, last + shift);
      p.first = Math.min(p.first, first + shift);
    });
  }

  for (let i = 1; i < RINGS; i++) {
    const groups = perRing[i];
    const [r0, r1] = [RING_RADII[i - 1], RING_RADII[i]];
    if (!groups) {
      rings.push({ r0, r1, runs: rings[i - 1].runs });
      continue;
    }
    if (groups.length === 1) {
      rings.push({ r0, r1, runs: [{ sector: groups[0].s, a0: 0, a1: TAU }] });
      continue;
    }
    const cuts = groups.map((_, k) => {
      const { key, last, first } = bound(groups, k);
      const p = pairs.get(key)!;
      if (p.last >= p.first) return (last + first) / 2;
      return (p.last + p.first) / 2 - shiftTo(p.ref, (last + first) / 2);
    });
    rings.push({
      r0,
      r1,
      runs: groups.map((g, k) => ({
        sector: g.s,
        a0: cuts[k],
        a1: k + 1 < groups.length ? cuts[k + 1] : cuts[0] + TAU,
      })),
    });
  }

  const parts = new Map<string, string[]>();
  const edges: PolarEdge[] = [];
  for (const ring of rings) {
    ring.runs.forEach((run, k) => {
      parts.set(run.sector, [...(parts.get(run.sector) ?? []), wedge(ring.r0, ring.r1, run.a0, run.a1)]);
      const prev = ring.runs[(k - 1 + ring.runs.length) % ring.runs.length];
      if (ring.runs.length > 1 && prev.sector !== run.sector)
        edges.push({
          d: `M${fmt(polar(ring.r0, run.a0))}L${fmt(polar(ring.r1, run.a0))}`,
          a: prev.sector,
          b: run.sector,
        });
    });
  }

  // Arcs between each ring and the next (or the rim): split at both rings' run edges, keep spans whose sides differ.
  rings.forEach((inner, i) => {
    const outer = rings[i + 1];
    const r = inner.r1;
    const cuts = [
      ...new Set([...inner.runs, ...(outer?.runs ?? [])].map((run) => +norm(run.a0).toFixed(9)).concat(0)),
    ].sort((x, y) => x - y);
    cuts.push(TAU);
    let open: { a: string; b: string | null; from: number; to: number } | null = null;
    const flush = () => {
      if (open) edges.push({ d: `M${fmt(polar(r, open.from))}${arcTo(r, open.from, open.to)}`, a: open.a, b: open.b });
      open = null;
    };
    for (let c = 0; c + 1 < cuts.length; c++) {
      const mid = (cuts[c] + cuts[c + 1]) / 2;
      const a = runAt(inner, mid);
      const b = outer ? runAt(outer, mid) : null;
      if (a === b) flush();
      else if (open && open.a === a && open.b === b) open.to = cuts[c + 1];
      else {
        flush();
        open = { a, b, from: cuts[c], to: cuts[c + 1] };
      }
    }
    flush();
  });

  return {
    rings,
    fills: new Map([...parts].map(([s, d]) => [s, d.join("")])),
    edges,
    rim: RING_RADII[RINGS - 1] * GALAXY_SCALE,
  };
}

export interface Territory {
  faction: Faction;
  /** Every planet of the sector is held by this enemy; otherwise the sector is contested. */
  held: boolean;
}

/** The enemy with most planets (held or under attack) in each sector; null where Super Earth holds them all. */
export function sectorTerritory(planets: Planet[]): Map<string, Territory | null> {
  const bySector = new Map<string, Planet[]>();
  for (const p of planets) bySector.set(p.sector, [...(bySector.get(p.sector) ?? []), p]);
  return new Map(
    [...bySector].map(([s, list]) => {
      const counts = new Map<Faction, number>();
      for (const p of list) {
        const enemy = p.currentOwner !== "Humans" ? p.currentOwner : p.event?.faction;
        if (enemy && enemy !== "Humans") counts.set(enemy, (counts.get(enemy) ?? 0) + 1);
      }
      const faction = [...counts].sort((x, y) => y[1] - x[1])[0]?.[0];
      return [s, faction ? { faction, held: list.every((p) => p.currentOwner === faction) } : null];
    }),
  );
}

/** Unique undirected supply-line pairs from planet waypoints. */
export function supplyLines(planets: Planet[]): [number, number][] {
  const seen = new Set<string>();
  const lines: [number, number][] = [];
  for (const p of planets) {
    for (const w of p.waypoints) {
      const k = p.index < w ? `${p.index}-${w}` : `${w}-${p.index}`;
      if (!seen.has(k)) {
        seen.add(k);
        lines.push([p.index, w]);
      }
    }
  }
  return lines;
}

/** Share of planets (by count) currently held by Super Earth, 0-100. */
export function humanControl(planets: Planet[]): number {
  if (!planets.length) return 0;
  return (planets.filter((p) => p.currentOwner === "Humans").length / planets.length) * 100;
}
