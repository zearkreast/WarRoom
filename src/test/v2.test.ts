import { describe, expect, it } from "vitest";
import type { Campaign, Planet, War } from "../api/types";
import { isStalled, type FrontActivity } from "../domain/campaignFilter";
import { dispatchBody, dispatchKind } from "../domain/dispatch";
import { busiestPlanets, parseCommunityPlanets, superEarthPlanets } from "../domain/factionPlanets";
import { sestDate, warDay } from "../domain/format";
import { sectorTerritory } from "../domain/galaxy";
import { buildRegions } from "../domain/regions";
import { controlShares, groupBySector, matchesFilter, matchesSearch } from "../domain/planetIndex";
import { playersByEnemy, toLive, toPlanet, toStatic, toWarSample } from "../domain/snapshot";
import { buildWarHistory, defenseEvents, fmtLongDuration } from "../domain/warHistory";
import communityText from "../data/community-planets.txt?raw";
import campaignsJson from "./fixtures/campaigns.json";
import planetsJson from "./fixtures/planets.json";
import warJson from "./fixtures/war.json";

const planets = planetsJson as unknown as Planet[];
const campaigns = campaignsJson as unknown as Campaign[];
const H = 3_600_000;
const D = 24 * H;

describe("war history", () => {
  const start = 0;
  const now = 100 * D;
  const changes = [
    { planet: 1, planetName: "X", from: "Automaton", to: "Humans", at: 2 * D },
    { planet: 1, planetName: "X", from: "Humans", to: "Automaton", at: 55 * D },
    { planet: 1, planetName: "X", from: "Automaton", to: "Humans", at: 108 * D }, // after now: still counted by time order
  ];

  it("splits the record into owner segments and counts captures", () => {
    const h = buildWarHistory({
      changes: changes.slice(0, 2),
      defenses: [],
      currentOwner: "Automaton",
      recordStart: start,
      now,
    });
    expect(h.segments.map((s) => s.owner)).toEqual(["Automaton", "Humans", "Automaton"]);
    expect(h.liberated).toBe(1);
    expect(h.lost).toBe(1);
    expect(h.heldFreePct).toBeCloseTo(53, 0);
  });

  it("numbers battles in time order and ends the log with the record start", () => {
    const defenses = defenseEvents([50 * D, 50 * D + H, 54 * D], changes, null, now);
    const h = buildWarHistory({
      changes: changes.slice(0, 2),
      defenses,
      currentOwner: "Automaton",
      recordStart: start,
      now,
    });
    expect(h.log[h.log.length - 1]).toMatchObject({ kind: "start", owner: "Automaton" });
    const battles = h.log.flatMap((e) => (e.kind === "start" ? [] : [e.battle]));
    expect(battles).toEqual([...battles].sort((a, b) => b - a));
    expect(h.log[0]).toMatchObject({ kind: "change", to: "Automaton" });
  });

  it("groups defending samples into events and marks the lost one", () => {
    // The planet falls the moment its last defense ends, just after the last defending sample.
    const events = defenseEvents([10 * D, 10 * D + H, 54 * D, 55 * D - 60_000], changes, null, now);
    expect(events).toHaveLength(3); // 54 D and 55 D are more than two hours apart
    expect(events[0].held).toBe(true);
    expect(events[1].held).toBe(true);
    expect(events[2].held).toBe(false);
  });

  it("marks the latest event ongoing while the planet is under attack", () => {
    const events = defenseEvents([now - H], [], now + 5 * H, now);
    expect(events[0].ongoing).toBe(true);
  });

  it("formats long durations", () => {
    expect(fmtLongDuration(3_551_077_816)).toMatch(/^1C 12Y/);
    expect(fmtLongDuration(12 * 3600)).toBe("12H");
    expect(fmtLongDuration(0)).toBe("0H");
  });
});

describe("map regions", () => {
  it("outlines enemy territory only, one or more loops per enemy faction", () => {
    const regions = buildRegions(planets);
    const owners = new Set(regions.map((r) => r.owner));
    expect(owners.has("Humans")).toBe(false);
    for (const r of regions) expect(r.path.startsWith("M")).toBe(true);
    const enemies = new Set(planets.filter((p) => p.currentOwner !== "Humans").map((p) => p.currentOwner));
    expect(owners).toEqual(enemies);
  });
});

describe("map territory", () => {
  it("names a sector held only when the enemy holds every planet in it", () => {
    const territory = sectorTerritory(planets);
    for (const [sector, t] of territory) {
      if (!t) continue;
      const list = planets.filter((p) => p.sector === sector);
      expect(t.held).toBe(list.every((p) => p.currentOwner === t.faction));
    }
    expect([...territory.values()].some((t) => t && !t.held)).toBe(true);
  });
});

describe("planet index", () => {
  const active = new Set(campaigns.map((c) => c.planet.index));

  it("searches planet, sector, biome and hazard names", () => {
    const fury = planets.find((p) => p.name === "FURY")!;
    expect(matchesSearch(fury, "fury")).toBe(true);
    expect(matchesSearch(fury, fury.sector.toLowerCase())).toBe(true);
    expect(matchesSearch(fury, `${fury.biome.name} fury`)).toBe(true);
    expect(matchesSearch(fury, "fury nonsense")).toBe(false);
  });

  it("filters by front and owner", () => {
    expect(planets.filter((p) => matchesFilter(p, "active", active))).toHaveLength(active.size);
    expect(planets.every((p) => !matchesFilter(p, "Terminids", active) || p.currentOwner === "Terminids")).toBe(true);
    const pinned = new Set([planets[3].index]);
    expect(planets.filter((p) => matchesFilter(p, "pinned", active, pinned)).map((p) => p.index)).toEqual([
      planets[3].index,
    ]);
  });

  it("groups by sector with exactly one hottest front and fronts first", () => {
    const groups = groupBySector(planets, active, "players");
    expect(groups.filter((g) => g.hottest)).toHaveLength(1);
    expect(groups.reduce((n, g) => n + g.planets.length, 0)).toBe(planets.length);
    for (const g of groups) {
      const flags = g.planets.map((p) => active.has(p.index));
      expect(flags).toEqual([...flags].sort((a, b) => Number(b) - Number(a)));
    }
    for (let i = 1; i < groups.length; i++) expect(groups[i - 1].players).toBeGreaterThanOrEqual(groups[i].players);
    const az = groupBySector(planets, active, "az").map((g) => g.name);
    expect(az).toEqual([...az].sort((a, b) => a.localeCompare(b)));
  });

  it("control shares add up to the whole galaxy", () => {
    const shares = controlShares(planets);
    expect(shares.reduce((n, s) => n + s.planets, 0)).toBe(planets.length);
    expect(shares.reduce((n, s) => n + s.pct, 0)).toBeCloseTo(100, 6);
  });
});

describe("dispatches", () => {
  it("categorises by headline", () => {
    expect(dispatchKind("<i=3>NEW MAJOR ORDER</i>\nLiberate X.")).toBe("order");
    expect(dispatchKind("<i=3>MAJOR ORDER WON</i>\nWell done.")).toBe("victory");
    expect(dispatchKind("<i=3>STRATEGIC ALERT</i>\nBots on Y.")).toBe("alert");
    expect(dispatchKind("<i=3>CONTROL CENTER ADVISORY</i>\nNote.")).toBe("advisory");
    expect(dispatchKind("Something happened.")).toBe("news");
  });

  it("drops the headline from the body", () => {
    const body = dispatchBody("<i=3>NEW MAJOR ORDER</i>\nLiberate <i=1>FURY</i>.");
    expect(body.map((s) => s.text).join("")).toBe("Liberate FURY.");
  });
});

describe("war samples", () => {
  it("splits players by enemy front, counting defenses for the attacker", () => {
    const fronts = playersByEnemy(campaigns);
    const total = campaigns
      .filter((c) => (c.planet.event?.faction ?? c.planet.currentOwner) !== "Humans")
      .reduce((n, c) => n + c.planet.statistics.playerCount, 0);
    expect(fronts.Terminids + fronts.Automaton + fronts.Illuminate).toBe(total);
    const row = toWarSample(warJson as unknown as War, campaigns, 0);
    expect(row).toMatchObject({
      at: new Date(0).toISOString(),
      players: warJson.statistics.playerCount,
      terminids: fronts.Terminids,
    });
  });

  it("round-trips planet statistics through the trimmed snapshot", () => {
    const fury = planets.find((p) => p.name === "FURY")!;
    const back = toPlanet(toStatic(fury), toLive(fury));
    expect(back.statistics.automatonKills).toBe(fury.statistics.automatonKills);
    expect(back.statistics.missionTime).toBe(fury.statistics.missionTime);
    expect(back.hazards.some((h) => h.name.toLowerCase() === "none")).toBe(false);
  });
});

describe("Super Earth time", () => {
  it("runs the calendar 160 years ahead, in UTC", () => {
    expect(sestDate(Date.UTC(2026, 9, 23, 1, 0))).toBe("23/10/2186");
  });

  it("counts war days from the start day", () => {
    expect(warDay("2024-02-08T00:00:00Z", Date.UTC(2024, 1, 8, 12))).toBe(1);
    expect(warDay("2024-02-08T00:00:00Z", Date.UTC(2024, 1, 10, 12))).toBe(3);
  });
});

describe("faction planet lists", () => {
  const community = parseCommunityPlanets(communityText);
  const seq = (...values: number[]) => {
    let i = 0;
    return () => values[i++ % values.length];
  };

  it("parses the community file", () => {
    expect(community.fixed).toEqual(["MALEVELON CREEK", "CALYPSO"]);
    expect(community.pool.length).toBeGreaterThanOrEqual(4);
    expect(parseCommunityPlanets("# c\n[fixed]\nA # note\n\n[pool]\nb\nc")).toEqual({ fixed: ["A"], pool: ["B", "C"] });
  });

  it("lists a faction's busiest planets, at most six", () => {
    const list = busiestPlanets(planets, "Automaton");
    expect(list.length).toBeLessThanOrEqual(6);
    const players = list.map((l) => l.planet!.statistics.playerCount);
    expect(players).toEqual([...players].sort((a, b) => b - a));
    expect(list.every((l) => l.planet!.currentOwner === "Automaton")).toBe(true);
  });

  it("leads Super Earth's list with Super Earth and Mars, then the fixed planets", () => {
    const held = planets.map((p) => ({ ...p, currentOwner: "Humans" as const }));
    const list = superEarthPlanets(held, { fixed: ["FURY"], pool: ["ESKER", "MORT", "USTOTU"] }, seq(0));
    expect(list.map((l) => l.name)).toEqual(["SUPER EARTH", "MARS", "FURY", "ESKER", "MORT", "USTOTU"]);
    expect(list[1]).toMatchObject({ lore: "Sol system" });
  });

  it("draws the last picks at random and tops up a lost fixed planet from the pool", () => {
    const held = planets.map((p) => ({
      ...p,
      currentOwner: p.name === "FURY" ? ("Automaton" as const) : ("Humans" as const),
    }));
    const list = superEarthPlanets(held, { fixed: ["FURY", "MORT"], pool: ["ESKER", "USTOTU", "K"] }, seq(0.99));
    expect(list.map((l) => l.name)).toEqual(["SUPER EARTH", "MARS", "MORT", "K", "USTOTU", "ESKER"]);
  });
});

describe("stalled campaigns", () => {
  const front = (players: number, rate: number | null, extra: Partial<FrontActivity> = {}): FrontActivity => ({
    players,
    rate,
    inMajorOrder: false,
    pinned: false,
    ...extra,
  });

  it("hides fronts under 5k divers that gain less than +0.4%/h", () => {
    expect(isStalled(front(2_800, null))).toBe(true); // holding for reinforcement
    expect(isStalled(front(41, 0.002))).toBe(true); // progress on paper only
    expect(isStalled(front(4_999, 0.39))).toBe(true);
    expect(isStalled(front(3_000, -1.2))).toBe(true);
  });

  it("shows fronts that are moving, or busy enough to matter", () => {
    expect(isStalled(front(1_200, 0.4))).toBe(false); // e.g. +6%/h against -5.6%/h regen
    expect(isStalled(front(5_000, 0))).toBe(false);
    expect(isStalled(front(40_000, null))).toBe(false);
  });

  it("always shows Major Order targets and pinned planets", () => {
    expect(isStalled(front(100, null, { inMajorOrder: true }))).toBe(false);
    expect(isStalled(front(100, -2, { pinned: true }))).toBe(false);
  });
});
