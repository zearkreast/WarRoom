import { describe, expect, it } from "vitest";
import type { Assignment, Planet } from "../api/types";
import { anomalyOf, buildPolarSectors, humanControl, sectorAt, sectorTerritory, supplyLines } from "../domain/galaxy";
import { recentDispatches } from "../domain/dispatch";
import { isPlaceholder } from "../domain/planets";
import { describeAssignment } from "../domain/majorOrder";
import { parseMarkup, dispatchTitle, fmtDuration } from "../domain/format";
import {
  liberationPercent,
  projectedAt,
  projectedCompletion,
  projectedLoss,
  ratePerHour,
  type Sample,
} from "../domain/progress";
import planetsJson from "./fixtures/planets.json";
import assignmentsJson from "./fixtures/assignments.json";

const planets = planetsJson as unknown as Planet[];
const assignments = assignmentsJson as unknown as Assignment[];
const H = 3_600_000;

const sample = (at: number, percent: number, extra: Partial<Sample> = {}): Sample => ({
  planet: 1,
  at,
  owner: "Automaton",
  percent,
  players: 100,
  defending: false,
  ...extra,
});

describe("fixtures", () => {
  it("has gaps in planet indices, so code must look planets up by index", () => {
    expect(planets.some((p, i) => p.index !== i)).toBe(true);
  });
});

describe("progress", () => {
  it("computes liberation from health", () => {
    expect(liberationPercent({ health: 1_376_398, maxHealth: 2_000_000 })).toBeCloseTo(31.18, 2);
    expect(liberationPercent({ health: 0, maxHealth: 0 })).toBe(0);
  });

  it("needs ten minutes of samples before reporting a rate", () => {
    expect(ratePerHour([sample(0, 10)])).toBeNull();
    expect(ratePerHour([sample(0, 10), sample(9 * 60_000, 10.1)])).toBeNull();
    expect(ratePerHour([sample(0, 10), sample(H / 2, 11)])).toBeCloseTo(2);
  });

  it("fits a trend that holds steady whatever the sampling phase", () => {
    // Polled every minute, but the game steps progress by 0.07% every 3 minutes (1.4%/h).
    // Over 10 minutes the first/last samples alone read anywhere from 1.26 to 1.68%/h.
    const m = 60_000;
    for (let phase = 0; phase < 3; phase++) {
      const s = Array.from({ length: 11 }, (_, i) => sample((phase + i) * m, 31 + Math.floor((phase + i) / 3) * 0.07));
      expect(Math.abs(ratePerHour(s)! - 1.4)).toBeLessThan(0.06);
    }
  });

  it("ignores samples from before a capture", () => {
    const s = [sample(0, 90, { owner: "Automaton" }), sample(10 * 60_000, 0, { owner: "Humans" })];
    expect(ratePerHour(s)).toBeNull();
  });

  it("projects completion, loss and the percent at a deadline", () => {
    expect(projectedCompletion(50, 2, 0)).toBe(25 * H);
    expect(projectedCompletion(50, -1, 0)).toBeNull();
    expect(projectedLoss(20, -2, 0)).toBe(10 * H);
    expect(projectedAt(30, 1.5, 0, 10 * H)).toBeCloseTo(45);
  });

  it("formats durations", () => {
    expect(fmtDuration(47 * H + 19 * 60_000)).toBe("47h 19m");
    expect(fmtDuration(5 * 24 * H)).toBe("5d 0h");
    expect(fmtDuration(-1)).toBe("now");
  });
});

describe("major order", () => {
  it("resolves liberate tasks to planets", () => {
    const tasks = describeAssignment(assignments[0], planets);
    expect(tasks.map((t) => t.planet?.name)).toEqual(["FURY", "ESKER", "ZEA RUGOSIA"]);
    expect(tasks.every((t) => t.kind === "planet" && !t.done)).toBe(true);
  });

  it("falls back to a count task for unknown types", () => {
    const [t] = describeAssignment(
      { ...assignments[0], tasks: [{ type: 3, values: [3, 0, 500000], valueTypes: [1, 0, 3] }], progress: [1234] },
      planets,
    );
    expect(t).toMatchObject({ kind: "count", faction: "Automaton", current: 1234, goal: 500000, done: false });
  });
});

describe("dispatch markup", () => {
  it("splits highlighted spans", () => {
    const spans = parseMarkup("<i=3>NEW MAJOR ORDER</i>\n\nGo to <i=1>Fury</i>.");
    expect(spans).toEqual([
      { text: "NEW MAJOR ORDER", style: 3 },
      { text: "\n\nGo to ", style: null },
      { text: "Fury", style: 1 },
      { text: ".", style: null },
    ]);
    expect(dispatchTitle("<i=3>STRATEGIC ALERT</i>\n\nBody")).toBe("STRATEGIC ALERT");
  });
});

describe("galaxy geometry", () => {
  it("rebuilds the game's ring sectors with every planet inside its own sector", () => {
    const geo = buildPolarSectors(planets.filter((p) => !isPlaceholder(p)));
    expect(geo.rings[0].runs.map((r) => r.sector)).toEqual(["Sol"]);
    for (const ring of geo.rings) expect(ring.runs.every((r) => geo.fills.has(r.sector))).toBe(true);
    for (const p of planets.filter((q) => !isPlaceholder(q)))
      expect(sectorAt(geo, p.position.x, p.position.y)).toBe(p.sector);
    for (const d of geo.fills.values()) {
      expect(d.startsWith("M")).toBe(true);
      expect(d.endsWith("Z")).toBe(true);
      expect(d).not.toContain("NaN");
    }
    // Edges only separate different sectors; the rim counts as one side.
    expect(geo.edges.every((e) => e.a !== e.b)).toBe(true);
    expect(geo.edges.some((e) => e.b === null)).toBe(true);
  });

  it("gives a sector to the enemy holding or attacking most of its planets", () => {
    const territory = sectorTerritory(planets);
    const held = planets.find((p) => p.currentOwner !== "Humans")!;
    expect(territory.get(held.sector)?.faction).toBeDefined();
    expect(territory.get("Sol")).toBeNull();
  });

  it("dedupes supply lines and computes control share", () => {
    const lines = supplyLines(planets);
    const keys = new Set(lines.map(([a, b]) => `${Math.min(a, b)}-${Math.max(a, b)}`));
    expect(keys.size).toBe(lines.length);
    expect(humanControl(planets)).toBeGreaterThan(0);
  });

  it("marks Meridia and Penta as anomalies", () => {
    expect(anomalyOf({ name: "MERIDIA" })).toBe("wormhole");
    expect(anomalyOf({ name: "Penta" })).toBe("blackhole");
    expect(anomalyOf({ name: "FURY" })).toBeUndefined();
  });

  it("drops placeholder worlds stacked on Super Earth", () => {
    expect(isPlaceholder({ index: 0, position: { x: 0, y: 0 } })).toBe(false);
    expect(isPlaceholder({ index: 265, position: { x: 0, y: 0 } })).toBe(true);
    expect(isPlaceholder({ index: 3, position: { x: 0.1, y: 0 } })).toBe(false);
  });

  it("keeps dispatches from the last 72 hours, newest first", () => {
    const now = Date.parse("2026-10-23T12:00:00Z");
    const at = (h: number, id: number) => ({
      id,
      type: 0,
      message: "",
      published: new Date(now - h * H).toISOString(),
    });
    expect(recentDispatches([at(80, 1), at(70, 2), at(1, 3)], now).map((d) => d.id)).toEqual([3, 2]);
  });
});
