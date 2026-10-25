import type { Assignment, AssignmentTask, Faction, Planet } from "../api/types";
import { phaseName, type EpisodeLive, type OrderSample } from "./snapshot";
import { FACTION_BY_ID, factionLabel } from "./factions";
import { byIndex } from "./planets";
import { projectedAt, projectedCompletion, slopePerHour } from "./progress";

// valueTypes observed in assignment tasks.
const VT_FACTION = 1;
const VT_GOAL = 3;
const VT_PLANET = 12;

// Task types with known meanings; anything else is shown generically.
const TASK_LABELS: Record<number, string> = {
  2: "Extract samples",
  3: "Eradicate",
  7: "Complete operations",
  9: "Complete operations",
  11: "Liberate",
  12: "Defend",
  13: "Hold",
  15: "Liberate more planets than lost",
};

export interface TaskView {
  kind: "planet" | "count";
  label: string;
  planet?: Planet;
  faction?: Faction;
  /** For count tasks: current / goal. For planet tasks: 0/1 completion flag in `done`. */
  current: number;
  goal: number;
  done: boolean;
}

function valueOf(task: AssignmentTask, valueType: number): number | undefined {
  const i = task.valueTypes.indexOf(valueType);
  return i >= 0 ? task.values[i] : undefined;
}

export function describeTask(task: AssignmentTask, progress: number, planets: Planet[] | undefined): TaskView {
  const planetIndex = valueOf(task, VT_PLANET);
  const factionId = valueOf(task, VT_FACTION);
  const goal = valueOf(task, VT_GOAL) ?? 1;
  const faction = factionId ? FACTION_BY_ID[factionId] : undefined;
  const label = TASK_LABELS[task.type] ?? `Objective (type ${task.type})`;

  const planet = planetIndex != null && planets ? byIndex(planets).get(planetIndex) : undefined;
  if (planet && (task.type === 11 || task.type === 13 || task.type === 12)) {
    return { kind: "planet", label, planet, faction, current: progress, goal: 1, done: progress >= 1 };
  }
  return { kind: "count", label, faction, current: progress, goal, done: progress >= goal };
}

export function describeAssignment(a: Assignment, planets: Planet[] | undefined): TaskView[] {
  return a.tasks.map((t, i) => describeTask(t, a.progress[i] ?? 0, planets));
}

/** A count task's progress as 0-100, held at 99.9 until done so it never reads 100% early. */
export const taskPercent = (t: TaskView) =>
  t.done ? 100 : t.goal > 0 ? Math.min(99.9, Math.max(0, (t.current / t.goal) * 100)) : 0;

const REWARD_LABELS: Record<number, string> = { 1: "Warbond Medals", 2: "Requisition Slips", 3: "Super Credits" };
export const rewardLabel = (type: number) => REWARD_LABELS[type] ?? "Reward";

/**
 * The running order's name, taken from its campaign: the assignment itself is titled only "MAJOR ORDER"
 * and its briefing is worded differently, so a name is given only while exactly one order is running.
 */
export function orderName(episodes: EpisodeLive[] | undefined): string | undefined {
  const running = (episodes ?? []).flatMap((e) => (e.status === "active" ? e.phases.filter((p) => p.status === "active") : []));
  return running.length === 1 ? phaseName(running[0]) : undefined;
}

/** Less history than this and a count task's rate isn't trusted. */
const MIN_ORDER_SPAN_MS = 30 * 60_000;

/** Each task's count per hour, from the readings kept for order `id`; null while there's too little history. */
export function orderTaskRates(samples: OrderSample[] | undefined, id: number, tasks: number): (number | null)[] {
  const mine = (samples ?? []).filter((s) => s.id === id).sort((a, b) => a.at - b.at);
  const enough = mine.length > 1 && mine[mine.length - 1].at - mine[0].at >= MIN_ORDER_SPAN_MS;
  return Array.from({ length: tasks }, (_, i) =>
    enough ? slopePerHour(mine.map((s) => ({ at: s.at, value: s.progress[i] ?? 0 }))) : null,
  );
}

/** "no-front": a target planet nobody can deploy to, so it can't move. */
export type DeadlineOutlook = "done" | "on-track" | "behind" | "measuring" | "no-front";

export const OUTLOOK: Record<DeadlineOutlook, { label: string; tone: string }> = {
  done: { label: "Complete", tone: "var(--good)" },
  "on-track": { label: "Winning", tone: "var(--good)" },
  behind: { label: "Losing", tone: "var(--bad)" },
  measuring: { label: "Measuring", tone: "var(--label)" },
  "no-front": { label: "No front", tone: "var(--bad)" },
};

export const taskName = (t: TaskView) =>
  t.planet ? t.planet.name : `${t.label}${t.faction ? ` ${factionLabel(t.faction)}` : ""}`;

/** The whole order at the current pace: it is won only if every objective makes the deadline. */
export function orderVerdict(tasks: TaskView[], outlooks: TaskOutlook[]): { text: string; tone: string } | null {
  const short = tasks.filter((_, i) => outlooks[i].outlook === "behind" || outlooks[i].outlook === "no-front");
  if (short.length)
    return { tone: "var(--bad)", text: `Losing: ${short.map(taskName).join(", ")} won't make the deadline at this pace` };
  if (outlooks.every((o) => o.outlook === "done" || o.outlook === "on-track"))
    return { tone: "var(--good)", text: "Winning: every objective makes the deadline at this pace" };
  return outlooks.some((o) => o.outlook === "measuring") ? { tone: "var(--label)", text: "Measuring progress rates…" } : null;
}

export interface TaskOutlook {
  outlook: DeadlineOutlook;
  /** Percent it stands at when the order expires, 0-100. */
  end: number | null;
  /** When it reaches 100% at the current rate, if that's before the order expires. */
  eta: number | null;
  /** Trend in %/h. */
  rate: number | null;
}

/** A task's outlook from its percent and its trend in %/h. */
export function taskOutlook(percent: number, rate: number | null, now: number, deadline: number): TaskOutlook {
  const { outlook, end } = deadlineOutlook(percent, rate, now, deadline);
  const eta = outlook === "done" ? null : projectedCompletion(percent, rate, now);
  return { outlook, end, eta: eta != null && eta < deadline ? eta : null, rate };
}

/** Where a target ends up when the order expires if the current rate holds; `end` is clamped to 0-100. */
export function deadlineOutlook(
  percent: number,
  rate: number | null,
  now: number,
  deadline: number,
): { outlook: DeadlineOutlook; end: number | null } {
  if (percent >= 100) return { outlook: "done", end: 100 };
  const end = projectedAt(percent, rate, now, deadline);
  if (end == null) return { outlook: "measuring", end: null };
  return { outlook: end >= 100 ? "on-track" : "behind", end: Math.min(100, Math.max(0, end)) };
}
