import { describe, expect, it } from "vitest";
import type { WarStatusRaw } from "../api/types";
import { latestDispatches } from "../domain/dispatch";
import { planetEffects } from "../domain/effects";
import { toEffectsLive } from "../domain/snapshot";
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
