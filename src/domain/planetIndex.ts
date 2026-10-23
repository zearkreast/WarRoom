import type { Faction, Planet } from "../api/types";

export type PlanetFilter = "all" | "active" | "pinned" | Faction;
export type SectorSort = "az" | "players" | "active";

export interface SectorGroup {
  name: string;
  planets: Planet[];
  active: number;
  players: number;
  /** Faction holding most of the sector's planets. */
  owner: Faction;
  hottest: boolean;
}

/** Search across planet, sector, biome and hazard names; every word must match somewhere. */
export function matchesSearch(p: Planet, query: string): boolean {
  const words = query.toLowerCase().split(/\s+/).filter(Boolean);
  if (!words.length) return true;
  const hay = [p.name, p.sector, p.biome?.name ?? "", ...p.hazards.map((h) => h.name)].join(" ").toLowerCase();
  return words.every((w) => hay.includes(w));
}

export function matchesFilter(
  p: Planet,
  filter: PlanetFilter,
  active: Set<number>,
  pinned: ReadonlySet<number> = new Set(),
): boolean {
  if (filter === "all") return true;
  if (filter === "active") return active.has(p.index);
  if (filter === "pinned") return pinned.has(p.index);
  return p.currentOwner === filter;
}

/**
 * Groups planets by sector for the planet index. Inside a sector, fronts come first, then
 * the busiest planets. The sector with the most Helldivers on active fronts is the hottest.
 */
export function groupBySector(planets: Planet[], active: Set<number>, sort: SectorSort): SectorGroup[] {
  const bySector = new Map<string, Planet[]>();
  for (const p of planets) bySector.set(p.sector, [...(bySector.get(p.sector) ?? []), p]);

  const groups: SectorGroup[] = [...bySector].map(([name, list]) => {
    const counts = new Map<Faction, number>();
    for (const p of list) counts.set(p.currentOwner, (counts.get(p.currentOwner) ?? 0) + 1);
    return {
      name,
      planets: [...list].sort(
        (a, b) =>
          Number(active.has(b.index)) - Number(active.has(a.index)) ||
          b.statistics.playerCount - a.statistics.playerCount ||
          a.name.localeCompare(b.name),
      ),
      active: list.filter((p) => active.has(p.index)).length,
      players: list.reduce((n, p) => n + p.statistics.playerCount, 0),
      owner: [...counts].sort((a, b) => b[1] - a[1])[0][0],
      hottest: false,
    };
  });

  const hot = groups.filter((g) => g.active > 0).sort((a, b) => b.players - a.players)[0];
  if (hot) hot.hottest = true;

  const cmp: Record<SectorSort, (a: SectorGroup, b: SectorGroup) => number> = {
    az: (a, b) => a.name.localeCompare(b.name),
    players: (a, b) => b.players - a.players || a.name.localeCompare(b.name),
    active: (a, b) => b.active - a.active || b.players - a.players || a.name.localeCompare(b.name),
  };
  return groups.sort(cmp[sort]);
}

/** Share of planets per faction, 0-100, in a fixed faction order. */
export function controlShares(
  planets: Planet[],
): { faction: Faction; planets: number; pct: number; players: number }[] {
  const order: Faction[] = ["Humans", "Terminids", "Automaton", "Illuminate"];
  const total = planets.length || 1;
  return order.map((f) => {
    const owned = planets.filter((p) => p.currentOwner === f);
    return {
      faction: f,
      planets: owned.length,
      pct: (owned.length / total) * 100,
      players: owned.reduce((n, p) => n + p.statistics.playerCount, 0),
    };
  });
}
