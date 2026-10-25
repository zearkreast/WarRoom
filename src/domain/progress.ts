import type { Planet } from "../api/types";

/** A point-in-time reading of a planet, as stored by the history recorder. */
export interface Sample {
  planet: number;
  at: number; // epoch ms
  owner: string;
  /** Liberation % (0-100) for a liberation campaign, defense % during a defense event. */
  percent: number;
  players: number;
  defending: boolean;
}

export function liberationPercent(p: Pick<Planet, "health" | "maxHealth">): number {
  if (!p.maxHealth) return 0;
  return clamp(100 - (p.health / p.maxHealth) * 100, 0, 100);
}

export function defensePercent(p: Planet): number | null {
  if (!p.event || !p.event.maxHealth) return null;
  return clamp(100 - (p.event.health / p.event.maxHealth) * 100, 0, 100);
}

/** The percent the UI tracks for a planet: defense progress if under attack, else liberation. */
export function trackedPercent(p: Planet): number {
  return defensePercent(p) ?? liberationPercent(p);
}

export function toSample(p: Planet, at: number): Sample {
  return {
    planet: p.index,
    at,
    owner: p.currentOwner,
    percent: trackedPercent(p),
    players: p.statistics.playerCount,
    defending: p.event != null,
  };
}

const MIN_WINDOW_MS = 10 * 60_000;
const RATE_WINDOW_MS = 60 * 60_000;

/**
 * Least-squares slope of `value` per hour. The game updates progress in coarse steps, so a slope over
 * every reading is far steadier than comparing the first and last ones.
 */
export function slopePerHour(points: { at: number; value: number }[]): number | null {
  const n = points.length;
  if (n < 2) return null;
  const t0 = points[0].at;
  const meanT = points.reduce((s, x) => s + (x.at - t0), 0) / n;
  const meanV = points.reduce((s, x) => s + x.value, 0) / n;
  let num = 0;
  let den = 0;
  for (const x of points) {
    const dt = x.at - t0 - meanT;
    num += dt * (x.value - meanV);
    den += dt * dt;
  }
  return den ? (num / den) * 3_600_000 : null;
}

/**
 * Percent-per-hour trend over the samples from the last hour. Returns null until there's 10 minutes
 * of comparable data (same owner and defense state, so a capture doesn't produce a spike).
 */
export function ratePerHour(samples: Sample[]): number | null {
  if (samples.length < 2) return null;
  const sorted = [...samples].sort((a, b) => a.at - b.at);
  const last = sorted[sorted.length - 1];
  const window = sorted.filter(
    (s) => s.at >= last.at - RATE_WINDOW_MS && s.owner === last.owner && s.defending === last.defending,
  );
  if (last.at - window[0].at < MIN_WINDOW_MS) return null;
  return slopePerHour(window.map((s) => ({ at: s.at, value: s.percent })));
}

/** Epoch ms when the planet reaches 100% at the current rate, or null if it isn't progressing. */
export function projectedCompletion(percent: number, rate: number | null, now: number): number | null {
  if (rate == null || rate <= 0) return null;
  return now + ((100 - percent) / rate) * 3_600_000;
}

/** Epoch ms when a liberation drops to 0% at the current (negative) rate. */
export function projectedLoss(percent: number, rate: number | null, now: number): number | null {
  if (rate == null || rate >= 0 || percent <= 0) return null;
  return now + (percent / -rate) * 3_600_000;
}

/** Percent the planet will be at by `deadline` if the current rate holds (can exceed 100). */
export function projectedAt(percent: number, rate: number | null, now: number, deadline: number): number | null {
  if (rate == null) return null;
  return percent + rate * ((deadline - now) / 3_600_000);
}

function clamp(n: number, lo: number, hi: number) {
  return Math.min(hi, Math.max(lo, n));
}
