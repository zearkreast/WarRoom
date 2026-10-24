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

type Reading = { at: number; impact: number; players: number };

/**
 * War readings averaged over `windowMs`, stamped at the window's middle. The API's player count swings
 * by thousands from one minute to the next, which turns a narrow chart's line into noise.
 */
export function averageReadings(readings: Reading[], windowMs: number): Reading[] {
  const sums = new Map<number, { impact: number; players: number; n: number }>();
  for (const r of readings) {
    const at = Math.floor(r.at / windowMs) * windowMs;
    const s = sums.get(at) ?? { impact: 0, players: 0, n: 0 };
    s.impact += r.impact;
    s.players += r.players;
    s.n++;
    sums.set(at, s);
  }
  return [...sums]
    .sort(([a], [b]) => a - b)
    .map(([at, s]) => ({ at: at + windowMs / 2, impact: s.impact / s.n, players: Math.round(s.players / s.n) }));
}

/** One moment on the activity chart: the fronts, and the galaxy-wide reading when there is one. */
export interface ActivityPoint extends FrontBucket {
  online: number | null;
  impact: number | null;
}

/**
 * Front buckets joined with the war readings: one row per bucket and per reading, so the online and
 * impact lines keep every reading while the fronts, summed per 15 minutes, are interpolated between
 * their buckets. The bucket still filling at `now` is dropped: its partial sum would draw a fall that
 * isn't there.
 */
export function mergeActivity(
  buckets: FrontBucket[],
  readings: Reading[],
  now: number,
): ActivityPoint[] {
  const done = buckets.filter((b) => b.at + BUCKET_MS <= now);
  if (!done.length) return [];
  const first = done[0].at;
  const last = done[done.length - 1].at;
  const inside = readings.filter((r) => r.at >= first && r.at <= last).sort((a, b) => a.at - b.at);
  const fronts = (t: number): FrontBucket => {
    let i = 0;
    while (i < done.length - 2 && done[i + 1].at <= t) i++;
    const a = done[i];
    const b = done[Math.min(i + 1, done.length - 1)];
    const f = b.at === a.at ? 0 : Math.min(1, Math.max(0, (t - a.at) / (b.at - a.at)));
    const mix = (k: keyof Omit<FrontBucket, "at">) => Math.round(a[k] + (b[k] - a[k]) * f);
    return {
      at: t,
      terminids: mix("terminids"),
      automatons: mix("automatons"),
      illuminate: mix("illuminate"),
      players: mix("players"),
    };
  };
  const rows: ActivityPoint[] = [
    ...done.map((b) => ({ ...b, online: null, impact: null })),
    ...inside.map((r) => ({ ...fronts(r.at), online: r.players, impact: r.impact })),
  ];
  return rows.sort((a, b) => a.at - b.at || (a.online == null ? -1 : 1));
}
