import type { Faction, Planet } from "../api/types";

/** Most planets listed when hovering a faction on the control ring. */
export const LIST_SIZE = 6;

export interface CommunityPlanets {
  fixed: string[];
  pool: string[];
}

/** One row of the hover list. Mars is not in the API's planet list, so it carries no planet. */
export interface ListedPlanet {
  name: string;
  planet?: Planet;
  lore?: string;
}

/** Parses community-planets.txt: `[fixed]` and `[pool]` sections, one name per line, `#` comments. */
export function parseCommunityPlanets(text: string): CommunityPlanets {
  const out: CommunityPlanets = { fixed: [], pool: [] };
  let section: keyof CommunityPlanets | null = null;
  for (const raw of text.split(/\r?\n/)) {
    const line = raw.replace(/#.*/, "").trim();
    if (!line) continue;
    const header = /^\[(fixed|pool)\]$/i.exec(line);
    if (header) section = header[1].toLowerCase() as keyof CommunityPlanets;
    else if (section) out[section].push(line.toUpperCase());
  }
  return out;
}

/** A faction's planets, busiest first. */
export function busiestPlanets(planets: Planet[], faction: Faction, limit = LIST_SIZE): ListedPlanet[] {
  return planets
    .filter((p) => p.currentOwner === faction)
    .sort((a, b) => b.statistics.playerCount - a.statistics.playerCount || a.name.localeCompare(b.name))
    .slice(0, limit)
    .map((p) => ({ name: p.name, planet: p }));
}

/**
 * Super Earth's list: Super Earth, Mars, the fixed community planets, then random picks from
 * the pool. Only planets Super Earth holds are listed; the pool tops up any gaps.
 */
export function superEarthPlanets(
  planets: Planet[],
  community: CommunityPlanets,
  random: () => number = Math.random,
): ListedPlanet[] {
  const held = new Map(planets.filter((p) => p.currentOwner === "Humans").map((p) => [p.name.toUpperCase(), p]));
  const list: ListedPlanet[] = [];
  const add = (name: string) => {
    const p = held.get(name);
    if (p && !list.some((l) => l.planet === p)) list.push({ name: p.name, planet: p });
  };
  add("SUPER EARTH");
  list.push({ name: "MARS", lore: "Sol system" });
  for (const n of community.fixed) add(n);

  const pool = community.pool.filter((n) => held.has(n) && !list.some((l) => l.planet?.name.toUpperCase() === n));
  while (list.length < LIST_SIZE && pool.length) {
    add(pool.splice(Math.floor(random() * pool.length), 1)[0]);
  }
  return list.slice(0, LIST_SIZE);
}
