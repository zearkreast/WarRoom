import { Delaunay } from "d3-delaunay";
import type { Faction, Planet } from "../api/types";
import { GALAXY_SCALE, polar, project, sectorAt, sectorTerritory, type PolarSectors } from "./galaxy";

export interface FactionMark {
  faction: Faction;
  /** Map units. */
  x: number;
  y: number;
}

type Overlay = "regions" | "sectors" | "galaxy";
type Pt = [number, number];

const ENEMIES: Faction[] = ["Terminids", "Automaton", "Illuminate"];

/** Groups of items joined, directly or through others, by `linked`. */
function components<T>(items: T[], linked: (a: T, b: T) => boolean): T[][] {
  const seen = new Set<number>();
  const out: T[][] = [];
  items.forEach((_, start) => {
    if (seen.has(start)) return;
    seen.add(start);
    const group = [start];
    for (let k = 0; k < group.length; k++)
      items.forEach((other, j) => {
        if (!seen.has(j) && linked(items[group[k]], other)) {
          seen.add(j);
          group.push(j);
        }
      });
    out.push(group.map((i) => items[i]));
  });
  return out;
}

const mean = (pts: Pt[]): Pt => [
  pts.reduce((n, p) => n + p[0], 0) / pts.length,
  pts.reduce((n, p) => n + p[1], 0) / pts.length,
];

/**
 * Sectors overlay: the enemy's largest run of neighbouring sectors, by area. The mark goes at its
 * area-weighted centre, or at the middle of its biggest cell where that centre falls outside.
 */
function sectorMarks(planets: Planet[], geo: PolarSectors): FactionMark[] {
  const territory = sectorTerritory(planets);
  const cells = geo.rings.flatMap((ring) =>
    ring.runs.map((run) => {
      const r = (ring.r0 + ring.r1) / 2;
      const a = (run.a0 + run.a1) / 2;
      return { sector: run.sector, at: polar(r, a), area: (ring.r1 ** 2 - ring.r0 ** 2) * (run.a1 - run.a0) };
    }),
  );
  const touching = new Set(geo.edges.flatMap((e) => (e.b ? [`${e.a}|${e.b}`, `${e.b}|${e.a}`] : [])));
  return ENEMIES.flatMap((faction) => {
    const sectors = [...territory].flatMap(([s, t]) => (t?.faction === faction ? [s] : []));
    if (!sectors.length) return [];
    const blobs = components(sectors, (a, b) => touching.has(`${a}|${b}`)).map((group) => {
      const own = cells.filter((c) => group.includes(c.sector));
      return { group, own, area: own.reduce((n, c) => n + c.area, 0) };
    });
    const blob = blobs.reduce((m, b) => (b.area > m.area ? b : m));
    const [x, y] = [0, 1].map((k) => blob.own.reduce((n, c) => n + c.at[k] * c.area, 0) / blob.area);
    const inside = blob.group.includes(sectorAt(geo, x / GALAXY_SCALE, -y / GALAXY_SCALE) ?? "");
    const [fx, fy] = inside ? [x, y] : blob.own.reduce((m, c) => (c.area > m.area ? c : m)).at;
    return [{ faction, x: fx, y: fy }];
  });
}

/**
 * Regions and galaxy overlays: the enemy's largest group of planets, joined as that overlay joins them
 * (neighbouring territory cells, or supply lines). The mark goes at the group's centre, moved onto its
 * nearest member planet where the centre lies in someone else's territory.
 */
function planetMarks(planets: Planet[], overlay: Overlay): FactionMark[] {
  const pts = planets.map((p) => project(p.position));
  const delaunay = Delaunay.from(pts);
  const at = new Map(planets.map((p, i) => [p.index, i]));
  const neighbours = (i: number) => new Set(delaunay.neighbors(i));
  const linked =
    overlay === "regions"
      ? (a: Planet, b: Planet) => neighbours(at.get(a.index)!).has(at.get(b.index)!)
      : (a: Planet, b: Planet) => a.waypoints.includes(b.index) || b.waypoints.includes(a.index);
  return ENEMIES.flatMap((faction) => {
    const held = planets.filter((p) => p.currentOwner === faction);
    if (!held.length) return [];
    const blob = components(held, linked).reduce((m, g) => (g.length > m.length ? g : m));
    const centre = mean(blob.map((p) => project(p.position)));
    if (blob.includes(planets[delaunay.find(centre[0], centre[1])])) return [{ faction, x: centre[0], y: centre[1] }];
    const dist = (p: Planet) => Math.hypot(project(p.position)[0] - centre[0], project(p.position)[1] - centre[1]);
    const [x, y] = project(blob.reduce((m, p) => (dist(p) < dist(m) ? p : m)).position);
    return [{ faction, x, y }];
  });
}

/** Where each enemy's emblem goes: on its largest connected holding, as the chosen overlay draws it. */
export function factionMarks(planets: Planet[], overlay: Overlay, geo: PolarSectors | undefined): FactionMark[] {
  return overlay === "sectors" && geo ? sectorMarks(planets, geo) : planetMarks(planets, overlay);
}
