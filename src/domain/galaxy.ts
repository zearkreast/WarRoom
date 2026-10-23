import { Delaunay } from "d3-delaunay";
import type { Faction, Planet } from "../api/types";

/** Map units: the API's [-1, 1] galaxy coordinates scaled up, y flipped so north is up. */
export const GALAXY_SCALE = 500;
export const project = (p: { x: number; y: number }): [number, number] => [p.x * GALAXY_SCALE, -p.y * GALAXY_SCALE];

export interface SectorShape {
  name: string;
  owner: Faction;
  /** Closed, smoothed SVG path data (may contain several loops). */
  path: string;
  /** Planet indices in this sector. */
  planets: number[];
  /** Label anchor: the centroid of its planets. */
  center: [number, number];
}

type Pt = [number, number];

const VOID_RING_POINTS = 72;
const VOID_RING_RADIUS = 1.07 * GALAXY_SCALE;
const BOUNDS = 1.25 * GALAXY_SCALE;

/** Sector outlines: one shape per sector, labelled with its majority owner. */
export function buildSectors(planets: Planet[]): SectorShape[] {
  return buildOutlines(planets, (p) => p.sector, 2);
}

/** Region of one enemy faction's contiguous territory; Super Earth space is left open. */
export interface RegionShape extends SectorShape {
  owner: Faction;
}

/** Enemy territory outlines: same-owner cells merged into smooth fronts. */
export function buildRegions(planets: Planet[]): RegionShape[] {
  return buildOutlines(planets, (p) => (p.currentOwner === "Humans" ? null : p.currentOwner), 4);
}

/**
 * Builds one outline per group (`keyOf`) by merging the Voronoi cells of its planets: an
 * edge shared by two cells of the same group is interior and dropped; the remaining edges
 * are chained into loops and smoothed. Planets keyed null belong to no outline. A ring of
 * "void" sites around the galaxy keeps the outer cells from stretching to the bounds.
 */
function buildOutlines(planets: Planet[], keyOf: (p: Planet) => string | null, smoothing: number): SectorShape[] {
  // Several planets can share a position (e.g. at the origin). Delaunay gives duplicates
  // no cell, so only the first planet at a position becomes a site.
  const sites: Pt[] = [];
  const siteSector: (string | null)[] = [];
  const siteOf = new Map<string, number>();
  const planetSite: number[] = [];
  for (const p of planets) {
    const k = `${p.position.x},${p.position.y}`;
    let s = siteOf.get(k);
    if (s == null) {
      s = sites.length;
      siteOf.set(k, s);
      sites.push(project(p.position));
      siteSector.push(keyOf(p));
    }
    planetSite.push(s);
  }
  const planetSites = sites.length;
  for (let i = 0; i < VOID_RING_POINTS; i++) {
    const a = (i / VOID_RING_POINTS) * Math.PI * 2;
    sites.push([Math.cos(a) * VOID_RING_RADIUS, Math.sin(a) * VOID_RING_RADIUS]);
    siteSector.push(null);
  }

  const voronoi = Delaunay.from(sites).voronoi([-BOUNDS, -BOUNDS, BOUNDS, BOUNDS]);
  const key = (p: Pt) => `${p[0].toFixed(3)},${p[1].toFixed(3)}`;
  const edgeKey = (a: Pt, b: Pt) => {
    const ka = key(a);
    const kb = key(b);
    return ka < kb ? `${ka}|${kb}` : `${kb}|${ka}`;
  };

  const edgeOwners = new Map<string, (string | null)[]>();
  const cells: Pt[][] = [];
  for (let i = 0; i < planetSites; i++) {
    const poly = (voronoi.cellPolygon(i) as Pt[] | null) ?? [];
    cells[i] = poly;
    for (let k = 0; k + 1 < poly.length; k++) {
      const ek = edgeKey(poly[k], poly[k + 1]);
      const owners = edgeOwners.get(ek) ?? [];
      owners.push(siteSector[i]);
      edgeOwners.set(ek, owners);
    }
  }

  // Group → its planets (by array position) and the sites it owns.
  const bySector = new Map<string, { members: number[]; sites: Set<number> }>();
  planets.forEach((p, i) => {
    const key = keyOf(p);
    if (key == null) return;
    const entry = bySector.get(key) ?? { members: [], sites: new Set<number>() };
    entry.members.push(i);
    if (siteSector[planetSite[i]] === key) entry.sites.add(planetSite[i]);
    bySector.set(key, entry);
  });

  const shapes: SectorShape[] = [];
  for (const [name, { members, sites: sectorSites }] of bySector) {
    if (!sectorSites.size) continue;
    // Boundary edges of this sector, as directed segments in cell winding order.
    const segments: [Pt, Pt][] = [];
    for (const i of sectorSites) {
      const poly = cells[i];
      for (let k = 0; k + 1 < poly.length; k++) {
        const owners = edgeOwners.get(edgeKey(poly[k], poly[k + 1]))!;
        const interior = owners.filter((o) => o === name).length > 1;
        if (!interior) segments.push([poly[k], poly[k + 1]]);
      }
    }
    const loops = chainLoops(segments, key);
    const path = loops.map((l) => toPath(chaikin(l, smoothing))).join(" ");
    shapes.push({
      name,
      owner: majorityOwner(members.map((i) => planets[i].currentOwner)),
      path,
      planets: members.map((i) => planets[i].index),
      center: centroid([...sectorSites].map((i) => sites[i])),
    });
  }
  return shapes;
}

function chainLoops(segments: [Pt, Pt][], key: (p: Pt) => string): Pt[][] {
  const byStart = new Map<string, number[]>();
  segments.forEach(([a], i) => {
    const k = key(a);
    byStart.set(k, [...(byStart.get(k) ?? []), i]);
  });
  const used = new Set<number>();
  const loops: Pt[][] = [];
  for (let s = 0; s < segments.length; s++) {
    if (used.has(s)) continue;
    const loop: Pt[] = [];
    let cur = s;
    while (cur !== -1 && !used.has(cur)) {
      used.add(cur);
      const [a, b] = segments[cur];
      loop.push(a);
      const next = (byStart.get(key(b)) ?? []).find((i) => !used.has(i));
      cur = next ?? -1;
    }
    if (loop.length >= 3) loops.push(loop);
  }
  return loops;
}

/** Chaikin corner cutting on a closed ring; each pass rounds the corners further. */
export function chaikin(ring: Pt[], passes: number): Pt[] {
  let pts = ring;
  for (let n = 0; n < passes; n++) {
    const out: Pt[] = [];
    for (let i = 0; i < pts.length; i++) {
      const a = pts[i];
      const b = pts[(i + 1) % pts.length];
      out.push([a[0] * 0.75 + b[0] * 0.25, a[1] * 0.75 + b[1] * 0.25]);
      out.push([a[0] * 0.25 + b[0] * 0.75, a[1] * 0.25 + b[1] * 0.75]);
    }
    pts = out;
  }
  return pts;
}

const toPath = (ring: Pt[]) => `M${ring.map((p) => `${p[0].toFixed(1)},${p[1].toFixed(1)}`).join("L")}Z`;

function majorityOwner(owners: Faction[]): Faction {
  const counts = new Map<Faction, number>();
  for (const o of owners) counts.set(o, (counts.get(o) ?? 0) + 1);
  return [...counts].sort((a, b) => b[1] - a[1])[0][0];
}

function centroid(pts: Pt[]): Pt {
  const n = pts.length || 1;
  return [pts.reduce((s, p) => s + p[0], 0) / n, pts.reduce((s, p) => s + p[1], 0) / n];
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
