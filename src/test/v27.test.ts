import { describe, expect, it } from "vitest";
import type { WarStatusRaw } from "../api/types";
import { planetTint } from "../domain/biomes";
import { latestDispatches } from "../domain/dispatch";
import { averageReadings, plotActivity, type ActivityPoint } from "../domain/frontActivity";
import { planetEffects } from "../domain/effects";
import { orderName, taskPercent, type TaskView } from "../domain/majorOrder";
import { campaignRecord, toEffectsLive, type EpisodeLive, type EpisodeStatus } from "../domain/snapshot";
import statusJson from "./fixtures/status-effects.json";

const live = toEffectsLive(statusJson as unknown as WarStatusRaw);
const on = (index: number) => planetEffects(live.planets.find((p) => p.index === index)?.ids ?? []);

describe("planet effects", () => {
  it("names each sub-faction once, though the game lists it once per side", () => {
    const fury = on(261);
    expect(fury.map((e) => [e.kind, e.name])).toEqual([
      ["subfaction", "THE INCINERATION CORPS"],
      ["seaf", "Heavy SEAF Presence"],
    ]);
  });

  it("files cities, the training ground and TCS+ towers as points of interest", () => {
    const pois = live.planets.flatMap((p) => planetEffects(p.ids).filter((e) => e.kind === "poi").map((e) => e.name));
    expect(pois).toEqual(
      expect.arrayContaining(["NEW HOPE CITY", "NEW ASPIRATION CITY", "NEW YEARNING CITY", "HELLDIVER TRAINING FACILITIES", "PANDORA BASE"]),
    );
    expect(pois.filter((n) => n === "TERMINID CONTROL SYSTEM+")).toHaveLength(14);
  });

  it("leaves out unnamed ids and bookkeeping like the campaign blocker", () => {
    const zagon = live.planets.find((p) => p.ids.includes(1318))!;
    const names = planetEffects(zagon.ids).map((e) => e.name);
    expect(names).not.toContain("CAMPAIGN BLOCKER");
    expect(planetEffects([1318, 1319])).toEqual([]);
  });
});

describe("map dispatches", () => {
  const now = Date.parse("2026-10-24T12:00:00Z");
  const d = (id: number, daysAgo: number) => ({
    id,
    published: new Date(now - daysAgo * 86_400_000).toISOString(),
    type: 0,
    message: "x",
  });
  const list = [d(1, 9), d(2, 6), d(3, 5), d(4, 0.5)];

  it("lists the newest few whatever their age, or the last 72 hours'", () => {
    expect(latestDispatches(list, "3", now).map((x) => x.id)).toEqual([4, 3, 2]);
    expect(latestDispatches(list, "72h", now).map((x) => x.id)).toEqual([4]);
  });
});

describe("planet pictures", () => {
  it("shows what a destroyed planet has become instead of its old biome", () => {
    expect(planetTint({ name: "MERIDIA", biome: { name: "Supercolony", description: "" } }).art).toMatch(/meridian-black-hole/);
    expect(planetTint({ name: "PENTA", biome: { name: "Deadlands", description: "" } }).art).toMatch(/black-hole/);
    expect(planetTint({ name: "MORADESH", biome: { name: "Desert Dunes", description: "" } }).art).toMatch(/fractured/);
    expect(planetTint({ name: "FURY", biome: { name: "Magma", description: "" } }).art).toMatch(/magma/);
  });
});

describe("activity readings", () => {
  it("averages war readings over each window, stamped at its middle", () => {
    const w = 300_000;
    const r = (at: number, players: number, impact: number, terminids: number) => ({
      at,
      players,
      impact,
      terminids,
      automatons: 0,
      illuminate: 0,
    });
    const rows = averageReadings([r(0, 100, 0.01, 10), r(60_000, 300, 0.03, 30), r(w + 1, 50, 0.02, 5)], w);
    expect(rows).toEqual([r(w / 2, 200, 0.02, 20), r(w + w / 2, 50, 0.02, 5)]);
  });
});

describe("activity chart toggles", () => {
  const row = (online: number | null, impact: number | null): ActivityPoint => ({
    at: 0,
    terminids: 10,
    automatons: 20,
    illuminate: 30,
    players: 60,
    online,
    impact,
  });

  it("draws every series as recorded while all are on", () => {
    const [p] = plotActivity([row(100, 0.012)], new Set(), true, 0.01);
    expect(p).toMatchObject({ plotTerminids: 10, plotAutomatons: 20, plotIlluminate: 30, plotOnline: 100, plotImpact: 0.012 });
  });

  it("flattens a switched-off front so the bands above settle onto the rest", () => {
    const [p] = plotActivity([row(100, 0.012)], new Set(["automatons"]), true, 0.01);
    expect(p).toMatchObject({ plotTerminids: 10, plotAutomatons: 0, plotIlluminate: 30, automatons: 20 });
  });

  it("parks the online line on the fronts still shown, and the multiplier flat", () => {
    const [p] = plotActivity([row(100, 0.012)], new Set(["online", "impact", "terminids"]), true, 0.01);
    expect(p.plotOnline).toBe(50);
    expect(p.plotImpact).toBe(0.01);
  });

  it("keeps gaps as gaps and falls back to everyone on the fronts without readings", () => {
    const [gap] = plotActivity([row(null, null)], new Set(["online", "impact"]), true, 0.01);
    expect([gap.plotOnline, gap.plotImpact]).toEqual([null, null]);
    const [fronts] = plotActivity([row(null, null)], new Set(), false, 0.01);
    expect(fronts.plotOnline).toBe(60);
  });
});

describe("major order name", () => {
  const phase = (title: string, status: EpisodeStatus) => ({ title, briefing: "", outro: "", status, medals: null, issued: null });
  const episode = (status: EpisodeStatus, phases: ReturnType<typeof phase>[]): EpisodeLive => ({
    id: 1,
    title: "CAMPAIGN",
    description: "",
    faction: "Humans",
    status,
    start: 0,
    end: null,
    reward: null,
    phases,
  });

  it("is the running order of the running campaign", () => {
    const list = [episode("won", [phase("OLD", "won")]), episode("active", [phase("FIRST", "won"), phase("HONOR THE FALLEN", "active")])];
    expect(orderName(list)).toBe("HONOR THE FALLEN");
  });

  it("is left out when it can't be told which order is which", () => {
    expect(orderName([episode("active", [phase("A", "active"), phase("B", "active")])])).toBeUndefined();
    expect(orderName(undefined)).toBeUndefined();
  });

  it("is left out while the game hasn't named the order yet", () => {
    expect(orderName([episode("active", [phase("HONOR THE FALLEN", "lost"), phase("MAJOR ORDER", "active")])])).toBeUndefined();
  });
});

describe("major order progress", () => {
  const task = (current: number, goal: number): TaskView => ({ kind: "count", label: "Eradicate", current, goal, done: current >= goal });

  it("is the share of the goal, held to 0-100%", () => {
    expect(taskPercent(task(21_200_000, 300_000_000))).toBeCloseTo(7.07, 2);
    expect(taskPercent(task(-2, 1))).toBe(0);
    expect(taskPercent(task(70, 50))).toBe(100);
  });

  it("reads 100% only once the task is done", () => {
    expect(taskPercent(task(299_900_000, 300_000_000))).toBe(99.9);
    expect(taskPercent(task(5, 0))).toBe(100);
  });
});

describe("campaign record", () => {
  const phase = (status: EpisodeStatus) => ({ title: "MO", briefing: "", outro: "", status, medals: null, issued: null });
  const episode = (start: number, status: EpisodeStatus, orders: EpisodeStatus[]): EpisodeLive => ({
    id: start,
    title: "CAMPAIGN",
    description: "",
    faction: "Humans",
    status,
    start,
    end: start + 1,
    reward: null,
    phases: orders.map(phase),
  });

  it("counts campaigns and Major Orders won", () => {
    const past = [episode(1, "won", ["won", "lost", "won"]), episode(2, "lost", ["lost", "won", "lost"])];
    expect(campaignRecord(past)).toMatchObject({ won: 1, orders: 6, ordersWon: 3 });
  });

  it("streaks the victories since the newest failure, in any order", () => {
    const past = [episode(3, "won", []), episode(1, "won", []), episode(4, "won", []), episode(2, "lost", [])];
    expect(campaignRecord(past).streak).toBe(2);
    expect(campaignRecord([episode(1, "won", []), episode(2, "won", [])]).streak).toBe(2);
    expect(campaignRecord([episode(1, "won", []), episode(2, "lost", [])]).streak).toBe(0);
    expect(campaignRecord([]).streak).toBe(0);
  });
});
