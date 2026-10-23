import type { Faction } from "../api/types";
import { FACTION_BY_ID } from "./factions";

/** One row of the `front_activity` database function. */
export interface FrontRow {
  bucket: string;
  owner: number;
  /** Set for defenses, which come back per planet; null for a front's summed liberations. */
  planet: number | null;
  players: number;
}

export interface FrontBucket {
  at: number;
  terminids: number;
  automatons: number;
  illuminate: number;
  /** All three fronts together. */
  players: number;
}

export const BUCKET_MS = 15 * 60_000;
const KEYS: Partial<Record<Faction, "terminids" | "automatons" | "illuminate">> = {
  Terminids: "terminids",
  Automaton: "automatons",
  Illuminate: "illuminate",
};

/**
 * Helldivers per front per bucket. A defense counts for whoever `attackerOf` names; one it can't
 * place is left out rather than guessed.
 */
export function frontBuckets(rows: FrontRow[], attackerOf: (planet: number) => Faction | undefined): FrontBucket[] {
  const byTime = new Map<number, FrontBucket>();
  for (const r of rows) {
    const enemy = r.planet == null ? FACTION_BY_ID[r.owner] : attackerOf(r.planet);
    const key = enemy && KEYS[enemy];
    const at = Date.parse(r.bucket);
    const b = byTime.get(at) ?? { at, terminids: 0, automatons: 0, illuminate: 0, players: 0 };
    byTime.set(at, b);
    if (!key) continue;
    b[key] += r.players;
    b.players += r.players;
  }
  return [...byTime.values()].sort((a, b) => a.at - b.at);
}

/** The newest stretch without a missing bucket, so a collector outage doesn't draw a line across it. */
export function latestRun(buckets: FrontBucket[]): FrontBucket[] {
  let start = buckets.length - 1;
  while (start > 0 && buckets[start].at - buckets[start - 1].at <= BUCKET_MS) start--;
  return buckets.slice(Math.max(start, 0));
}
