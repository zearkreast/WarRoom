import { describe, expect, it } from "vitest";
import type { Planet, Region } from "../api/types";
import { citiesOf, cityStars, sizeLabel } from "../domain/cities";
import planetsJson from "./fixtures/planets.json";

const planets = planetsJson as unknown as Planet[];
const superEarth = planets.find((p) => p.index === 0)!;

const region = (id: number, name: string, size: string, more: Partial<Region> = {}): Region => ({
  id,
  name,
  description: "",
  health: null,
  maxHealth: 400000,
  size,
  regenPerSecond: null,
  isAvailable: false,
  players: 0,
  ...more,
});

describe("cities", () => {
  it("marks size classes with one to four stars", () => {
    expect(["Settlement", "Town", "City", "MegaCity", "Megafactory"].map(cityStars)).toEqual([1, 2, 3, 4, 0]);
    expect(sizeLabel("MegaCity")).toBe("Mega City");
  });

  it("tells Super Earth's capitals apart by what the invasion left of them", () => {
    const names = ["EAGLEOPOLIS", "ADMINISTRATIVE CENTER 02", "REMEMBRANCE", "YORK SUPREME", "PORT MERCY", "PROSPERITY CITY", "EQUALITY-ON-SEA"];
    const p = { ...superEarth, regions: names.map((n, i) => region(i, n, "MegaCity")) };
    expect(citiesOf(p).map((c) => [c.name, c.state, c.formerly])).toEqual([
      ["EAGLEOPOLIS", "destroyed", undefined],
      ["ADMINISTRATIVE CENTER 02", "destroyed", undefined],
      ["GUN", "rebuilt", "REMEMBRANCE"],
      ["NEW YORK SUPREME", "rebuilt", "YORK SUPREME"],
      ["PORT MERCY", "destroyed", undefined],
      ["PROSPERITY CITY", "intact", undefined],
      ["EQUALITY-ON-SEA", "intact", undefined],
    ]);
  });

  it("counts an enemy-held city's liberation down from its health", () => {
    const enemy = planets.find((p) => p.currentOwner !== "Humans" && p.index !== 0)!;
    const p = {
      ...enemy,
      event: null,
      regions: [
        region(0, "OPEN", "City", { health: 100000, isAvailable: true, players: 40 }),
        region(1, "SHUT", "Town", { health: 200000, maxHealth: 200000 }),
        region(2, "QUIET", "Settlement", { description: "null" }),
      ],
    };
    expect(citiesOf(p).map((c) => [c.state, c.percent, c.description])).toEqual([
      ["liberating", 75, ""],
      ["locked", 0, ""],
      ["held", null, ""],
    ]);
  });
});
