import type { Planet } from "../api/types";

/** Supply-line neighbours by planet index; a link listed on either end counts both ways. */
export function neighbours(planets: Pick<Planet, "index" | "waypoints">[]): Map<number, Set<number>> {
  const links = new Map(planets.map((p) => [p.index, new Set<number>()]));
  for (const p of planets)
    for (const w of p.waypoints) {
      links.get(p.index)?.add(w);
      links.get(w)?.add(p.index);
    }
  return links;
}

export interface Circumvallation {
  /** Enemy planets that liberating the front would leave with only Super Earth planets around them. */
  targets: Planet[];
  /** Of the targets, those that are fronts themselves and so could be liberated directly instead. */
  direct: Planet[];
  /** Whether liberating the front costs less than liberating the direct targets. */
  worth: boolean;
}

const enemyHeld = (p: Planet) => p.currentOwner !== "Humans";
const regenPerHour = (p: Planet) => p.regenPerSecond * 3600;

/**
 * Hours to empty a planet's health at `pace` health an hour, its own resistance taken off. The pace is twice
 * the strongest resistance among the planets compared, so the comparison turns on health and resistance alone.
 */
function hoursAt(p: Planet, pace: number): number {
  const net = pace - regenPerHour(p);
  return pace > 0 && net > 0 ? p.health / net : p.health;
}

/**
 * What liberating `front` would do for a SEAF siege: the enemy planets it would leave surrounded, a siege needing
 * a planet on its own. Null when it surrounds none. `fronts` are the planets with a campaign now.
 */
export function circumvallation(
  front: Planet,
  planets: Planet[],
  links: Map<number, Set<number>>,
  fronts: Set<number>,
): Circumvallation | null {
  if (!enemyHeld(front) || front.event) return null;
  const byIndex = new Map(planets.map((p) => [p.index, p]));
  const targets = [...(links.get(front.index) ?? [])].flatMap((i) => {
    const x = byIndex.get(i);
    if (!x || x.disabled || !enemyHeld(x)) return [];
    const around = [...(links.get(i) ?? [])].filter((n) => n !== front.index);
    return around.every((n) => byIndex.get(n)?.currentOwner === "Humans") ? [x] : [];
  });
  if (!targets.length) return null;
  const direct = targets.filter((x) => fronts.has(x.index));
  const pace = 2 * Math.max(...[front, ...targets].map(regenPerHour));
  const worth = !direct.length || hoursAt(front, pace) <= direct.reduce((n, x) => n + hoursAt(x, pace), 0);
  return { targets, direct, worth };
}
