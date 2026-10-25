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

/** A galaxy-wide war reading: the online total, the multiplier and Helldivers per front. */
export interface Reading {
  at: number;
  impact: number;
  players: number;
  terminids: number;
  automatons: number;
  illuminate: number;
}

const FRONTS = ["terminids", "automatons", "illuminate"] as const;

/**
 * War readings averaged over `windowMs`, stamped at the window's middle. The API's player count swings
 * by thousands from one minute to the next, which turns a narrow chart's line into noise.
 */
export function averageReadings(readings: Reading[], windowMs: number): Reading[] {
  const sums = new Map<number, Reading & { n: number }>();
  for (const r of readings) {
    const at = Math.floor(r.at / windowMs) * windowMs;
    const s = sums.get(at) ?? { at, impact: 0, players: 0, terminids: 0, automatons: 0, illuminate: 0, n: 0 };
    s.impact += r.impact;
    s.players += r.players;
    for (const k of FRONTS) s[k] += r[k];
    s.n++;
    sums.set(at, s);
  }
  return [...sums.values()]
    .sort((a, b) => a.at - b.at)
    .map((s) => ({
      at: s.at + windowMs / 2,
      impact: s.impact / s.n,
      players: Math.round(s.players / s.n),
      terminids: Math.round(s.terminids / s.n),
      automatons: Math.round(s.automatons / s.n),
      illuminate: Math.round(s.illuminate / s.n),
    }));
}

/** One moment on the activity chart: the fronts, and the galaxy-wide reading when there is one. */
export interface ActivityPoint extends FrontBucket {
  online: number | null;
  impact: number | null;
}

/** Readings covering at least this long are drawn on their own. */
const READINGS_ALONE_MS = 3_600_000;

/**
 * The chart's rows. Each war reading is a row with its own split by front, as fine-grained as the
 * online line. The 15-minute front buckets fill in only while the readings cover less than an hour:
 * they count the fronts from planet samples, which comes out lower than the readings' split, so
 * drawing both would put a false step where one hands over to the other. A reading without a split
 * (older rows) takes the fronts interpolated from the buckets. The bucket still filling at `now` is
 * dropped: its partial sum would draw a fall that isn't there.
 */
export function mergeActivity(buckets: FrontBucket[], readings: Reading[], now: number): ActivityPoint[] {
  const done = buckets.filter((b) => b.at + BUCKET_MS <= now);
  const sorted = [...readings].sort((a, b) => a.at - b.at);
  const start = Math.min(done[0]?.at ?? Infinity, sorted[0]?.at ?? Infinity);
  if (!Number.isFinite(start)) return [];
  const inside = sorted.filter((r) => r.at >= start);
  const firstReading = inside[0]?.at ?? Infinity;
  const fronts = (t: number): FrontBucket | null => {
    if (!done.length) return null;
    let i = 0;
    while (i < done.length - 2 && done[i + 1].at <= t) i++;
    const a = done[i];
    const b = done[Math.min(i + 1, done.length - 1)];
    const f = b.at === a.at ? 0 : Math.min(1, Math.max(0, (t - a.at) / (b.at - a.at)));
    const mix = (k: keyof Omit<FrontBucket, "at">) => Math.round(a[k] + (b[k] - a[k]) * f);
    return { at: t, terminids: mix("terminids"), automatons: mix("automatons"), illuminate: mix("illuminate"), players: mix("players") };
  };
  const alone = inside.length > 1 && inside[inside.length - 1].at - firstReading >= READINGS_ALONE_MS;
  const rows: ActivityPoint[] = alone
    ? []
    : done.filter((b) => b.at < firstReading).map((b) => ({ ...b, online: null, impact: null }));
  for (const r of inside) {
    const split = r.terminids + r.automatons + r.illuminate;
    const own = { at: r.at, terminids: r.terminids, automatons: r.automatons, illuminate: r.illuminate, players: split };
    rows.push({ ...(split > 0 ? own : (fronts(r.at) ?? own)), online: r.players, impact: r.impact });
  }
  return rows;
}

export type FrontKey = "terminids" | "automatons" | "illuminate";
/** A line or band the activity chart can switch off. */
export type ActivitySeries = FrontKey | "online" | "impact";

/** An activity row as drawn, with the switched-off series parked. */
export interface PlotPoint extends ActivityPoint {
  plotTerminids: number;
  plotAutomatons: number;
  plotIlluminate: number;
  plotOnline: number | null;
  plotImpact: number | null;
}

/** The chart's rows with switched-off series parked rather than dropped, so they can morph away and back. */
export function plotActivity(
  rows: ActivityPoint[],
  off: ReadonlySet<ActivitySeries>,
  hasOnline: boolean,
  flatImpact: number,
): PlotPoint[] {
  const front = (r: ActivityPoint, k: FrontKey) => (off.has(k) ? 0 : r[k]);
  return rows.map((r) => {
    const plotTerminids = front(r, "terminids");
    const plotAutomatons = front(r, "automatons");
    const plotIlluminate = front(r, "illuminate");
    const online = hasOnline ? r.online : r.players;
    return {
      ...r,
      plotTerminids,
      plotAutomatons,
      plotIlluminate,
      plotOnline: online == null || !off.has("online") ? online : plotTerminids + plotAutomatons + plotIlluminate,
      plotImpact: r.impact == null || !off.has("impact") ? r.impact : flatImpact,
    };
  });
}
