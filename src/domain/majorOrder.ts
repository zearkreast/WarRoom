import type { Assignment, AssignmentTask, Faction, Planet } from "../api/types";
import { phaseName, type EpisodeLive } from "./snapshot";
import { FACTION_BY_ID } from "./factions";
import { byIndex } from "./planets";
import { projectedAt } from "./progress";

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

export type DeadlineOutlook = "done" | "on-track" | "behind" | "measuring";

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
