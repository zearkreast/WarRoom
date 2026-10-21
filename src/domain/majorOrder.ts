import type { Assignment, AssignmentTask, Faction, Planet } from "../api/types";
import { FACTION_BY_ID } from "./factions";
import { byIndex } from "./planets";

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
