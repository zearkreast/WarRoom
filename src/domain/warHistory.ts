import type { OwnerChange } from "./history";

/** A stretch of time with one owner. */
export interface OwnerSegment {
  owner: string;
  from: number;
  to: number;
}

/** A defense event, rebuilt from samples taken while the planet was under attack. */
export interface DefenseEvent {
  start: number;
  end: number;
  /** True while the event is still running. */
  ongoing: boolean;
  held: boolean;
}

export type LogEntry =
  | { kind: "start"; at: number; owner: string }
  | { kind: "change"; at: number; from: string; to: string; battle: number; heldDays: number }
  | { kind: "defense"; at: number; enemy: string; battle: number; ongoing: boolean; held: boolean };

export interface WarHistory {
  recordStart: number;
  liberated: number;
  lost: number;
  defensesHeld: number;
  defensesTotal: number;
  /** Share of the recorded time Super Earth held the planet, 0-100. */
  heldFreePct: number;
  segments: OwnerSegment[];
  defenses: DefenseEvent[];
  /** Newest first. */
  log: LogEntry[];
}

/** Two defending samples further apart than this belong to different events. */
const EVENT_GAP_MS = 2 * 3_600_000;
const DAY_MS = 86_400_000;

/** Groups the timestamps of "defending" samples into events. */
export function defenseEvents(
  defendingAt: number[],
  changes: OwnerChange[],
  activeEventEnd: number | null,
  now: number,
): DefenseEvent[] {
  const sorted = [...defendingAt].sort((a, b) => a - b);
  const events: DefenseEvent[] = [];
  for (const t of sorted) {
    const last = events[events.length - 1];
    if (last && t - last.end <= EVENT_GAP_MS) last.end = t;
    else events.push({ start: t, end: t, ongoing: false, held: true });
  }
  for (const e of events) {
    // Lost if Super Earth gave the planet up during (or right after) the event.
    e.held = !changes.some((c) => c.from === "Humans" && c.at >= e.start && c.at <= e.end + EVENT_GAP_MS);
  }
  const last = events[events.length - 1];
  if (last && activeEventEnd != null && now - last.end <= EVENT_GAP_MS) last.ongoing = true;
  return events;
}

/**
 * Builds a planet's war record from its ownership changes (any order) and defense events.
 * `recordStart` is when tracking began for this planet; earlier changes are ignored.
 */
export function buildWarHistory(opts: {
  changes: OwnerChange[];
  defenses: DefenseEvent[];
  currentOwner: string;
  recordStart: number;
  now: number;
}): WarHistory {
  const changes = opts.changes.filter((c) => c.at >= opts.recordStart).sort((a, b) => a.at - b.at);
  const startOwner = changes[0]?.from ?? opts.currentOwner;

  const segments: OwnerSegment[] = [];
  let owner = startOwner;
  let from = opts.recordStart;
  for (const c of changes) {
    if (c.at > from) segments.push({ owner, from, to: c.at });
    owner = c.to;
    from = c.at;
  }
  if (opts.now > from) segments.push({ owner, from, to: opts.now });

  const total = Math.max(1, opts.now - opts.recordStart);
  const free = segments.filter((s) => s.owner === "Humans").reduce((n, s) => n + (s.to - s.from), 0);

  // Battles are numbered in time order across captures and defenses.
  type Battle = { at: number; entry: LogEntry };
  const battles: Battle[] = [];
  let prevAt = opts.recordStart;
  for (const c of changes) {
    battles.push({
      at: c.at,
      entry: { kind: "change", at: c.at, from: c.from, to: c.to, battle: 0, heldDays: (c.at - prevAt) / DAY_MS },
    });
    prevAt = c.at;
  }
  for (const d of opts.defenses) {
    if (d.start < opts.recordStart) continue;
    const enemy = changes.find((c) => c.from === "Humans" && c.at >= d.start)?.to ?? "";
    battles.push({
      at: d.start,
      entry: { kind: "defense", at: d.start, enemy, battle: 0, ongoing: d.ongoing, held: d.held },
    });
  }
  battles.sort((a, b) => a.at - b.at);
  battles.forEach((b, i) => {
    if (b.entry.kind !== "start") b.entry.battle = i + 1;
  });

  const log: LogEntry[] = [
    ...battles.map((b) => b.entry).reverse(),
    { kind: "start", at: opts.recordStart, owner: startOwner },
  ];

  const finished = opts.defenses.filter((d) => !d.ongoing && d.start >= opts.recordStart);
  return {
    recordStart: opts.recordStart,
    liberated: changes.filter((c) => c.to === "Humans").length,
    lost: changes.filter((c) => c.from === "Humans").length,
    defensesHeld: finished.filter((d) => d.held).length,
    defensesTotal: finished.length,
    heldFreePct: (free / total) * 100,
    segments,
    defenses: opts.defenses,
    log,
  };
}

/** "1C 11Y 5M 3W" style duration for long spans such as time deployed. */
export function fmtLongDuration(seconds: number): string {
  const units: [string, number][] = [
    ["C", 100 * 365 * 86400],
    ["Y", 365 * 86400],
    ["M", 30 * 86400],
    ["W", 7 * 86400],
    ["D", 86400],
    ["H", 3600],
  ];
  let rest = Math.max(0, Math.floor(seconds));
  const parts: string[] = [];
  for (const [u, s] of units) {
    const n = Math.floor(rest / s);
    if (n || parts.length) {
      if (n) parts.push(`${n}${u}`);
      rest -= n * s;
    }
    if (parts.length === 4) break;
  }
  return parts.join(" ") || "0H";
}
