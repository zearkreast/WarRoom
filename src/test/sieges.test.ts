import { describe, expect, it } from "vitest";
import type { Faction, Planet } from "../api/types";
import { circumvallation, neighbours } from "../domain/sieges";

const planet = (index: number, owner: Faction, waypoints: number[], maxHealth = 1_000_000, regenPct = 1): Planet =>
  ({
    index,
    name: `P${index}`,
    waypoints,
    maxHealth,
    health: maxHealth,
    disabled: false,
    currentOwner: owner,
    regenPerSecond: (maxHealth * regenPct) / 360_000,
    event: null,
  }) as unknown as Planet;

const ring = (planets: Planet[], front: number, fronts: number[]) =>
  circumvallation(planets.find((p) => p.index === front)!, planets, neighbours(planets), new Set(fronts));

describe("circumvallation", () => {
  it("finds the planet a front would leave surrounded, and says when taking it directly is quicker", () => {
    // Zea Rugosia (1.7M, 2.94%/h) would ring Haldus (1M, 1.5%/h): Haldus is the cheaper fight.
    const planets = [
      planet(1, "Illuminate", [2, 3], 1_700_000, 2.94),
      planet(2, "Illuminate", [4], 1_000_000, 1.5),
      planet(3, "Humans", []),
      planet(4, "Humans", []),
    ];
    const c = ring(planets, 1, [1, 2])!;
    expect(c.targets.map((p) => p.index)).toEqual([2]);
    expect(c.worth).toBe(false);
    expect(c.direct.map((p) => p.index)).toEqual([2]);
  });

  it("counts two sieges at once, and a ring is worth closing when the targets cost more together", () => {
    // RD-4 rings both Rogue 5 and Hesoe Prime.
    const planets = [
      planet(1, "Illuminate", [2, 3, 4]),
      planet(2, "Illuminate", [5], 1_000_000, 1),
      planet(3, "Illuminate", [5], 1_000_000, 4),
      planet(4, "Humans", []),
      planet(5, "Humans", []),
    ];
    const c = ring(planets, 1, [1, 2, 3])!;
    expect(c.targets.map((p) => p.index).sort()).toEqual([2, 3]);
    expect(c.worth).toBe(true);
  });

  it("needs a planet on its own: a pair cut off together is no siege", () => {
    const planets = [planet(1, "Automaton", [2]), planet(2, "Automaton", [3]), planet(3, "Automaton", [4]), planet(4, "Humans", [])];
    expect(ring(planets, 1, [1])).toBeNull();
  });

  it("is worth it when the surrounded planet has no campaign of its own", () => {
    const planets = [planet(1, "Terminids", [2], 3_000_000, 3.33), planet(2, "Terminids", [3], 2_000_000, 5), planet(3, "Humans", [])];
    const c = ring(planets, 1, [1])!;
    expect(c.direct).toEqual([]);
    expect(c.worth).toBe(true);
  });

  it("only applies to enemy planets", () => {
    const planets = [planet(1, "Humans", [2]), planet(2, "Automaton", [])];
    expect(ring(planets, 1, [1])).toBeNull();
  });
});
