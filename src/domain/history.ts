import type { Planet } from "../api/types";

export interface OwnerChange {
  planet: number;
  planetName: string;
  from: string;
  to: string;
  at: number;
}

/** Returns every planet whose owner differs from the last owner we recorded for it. */
export function detectOwnerChanges(known: Map<number, string>, planets: Planet[], at: number): OwnerChange[] {
  const changes: OwnerChange[] = [];
  for (const p of planets) {
    const was = known.get(p.index);
    if (was && was !== p.currentOwner) {
      changes.push({ planet: p.index, planetName: p.name, from: was, to: p.currentOwner, at });
    }
  }
  return changes;
}

/** Planets that have a defense event that wasn't in the previous snapshot. */
export function detectNewDefenses(prev: Planet[] | undefined, next: Planet[]): Planet[] {
  if (!prev) return [];
  const had = new Set(prev.filter((p) => p.event).map((p) => p.event!.id));
  return next.filter((p) => p.event && !had.has(p.event.id));
}

/** Planets worth sampling every poll: contested ones. Quiet planets only need owner tracking. */
export function isActive(p: Planet): boolean {
  return p.event != null || p.health < p.maxHealth;
}
