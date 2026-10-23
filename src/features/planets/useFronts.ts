import { useMemo } from "react";
import { useAssignments, useCampaigns, useSpaceStations } from "../../api/queries";
import { useSettings } from "../../state";
import type { Campaign, Faction, Planet } from "../../api/types";
import {
  defensePercent,
  liberationPercent,
  projectedAt,
  projectedCompletion,
  projectedLoss,
} from "../../domain/progress";

export interface Front {
  planet: Planet;
  campaign: Campaign;
  kind: "liberation" | "defense";
  enemy: Faction;
  percent: number;
  rate: number | null;
  /** Victory time at the current rate, if it's progressing. */
  victoryAt: number | null;
  /** Liberation: when progress hits 0%. Defense: the event deadline. */
  failAt: number | null;
  /** Defense only: projected % at the deadline. */
  projected: number | null;
  players: number;
  inMajorOrder: boolean;
  pinned: boolean;
  station: boolean;
}

export function buildFront(
  c: Campaign,
  rate: number | null,
  moPlanets: Set<number>,
  now: number,
  marks: { pinned?: Set<number>; stations?: Set<number> } = {},
): Front {
  const p = c.planet;
  const def = defensePercent(p);
  if (def != null && p.event) {
    const deadline = Date.parse(p.event.endTime);
    return {
      planet: p,
      campaign: c,
      kind: "defense",
      enemy: p.event.faction,
      percent: def,
      rate,
      victoryAt: projectedCompletion(def, rate, now),
      failAt: deadline,
      projected: projectedAt(def, rate, now, deadline),
      players: p.statistics.playerCount,
      inMajorOrder: moPlanets.has(p.index),
      pinned: marks.pinned?.has(p.index) ?? false,
      station: marks.stations?.has(p.index) ?? false,
    };
  }
  const lib = liberationPercent(p);
  return {
    planet: p,
    campaign: c,
    kind: "liberation",
    enemy: p.currentOwner,
    percent: lib,
    rate,
    victoryAt: projectedCompletion(lib, rate, now),
    failAt: projectedLoss(lib, rate, now),
    projected: null,
    players: p.statistics.playerCount,
    inMajorOrder: moPlanets.has(p.index),
    pinned: marks.pinned?.has(p.index) ?? false,
    station: marks.stations?.has(p.index) ?? false,
  };
}

/** Planet indices referenced by any active Major Order task. */
export function useMajorOrderPlanets(): Set<number> {
  const { data } = useAssignments();
  return useMemo(() => {
    const s = new Set<number>();
    for (const a of data ?? []) {
      for (const t of a.tasks) {
        const i = t.valueTypes.indexOf(12);
        if (i >= 0 && t.values[i] > 0) s.add(t.values[i]);
      }
    }
    return s;
  }, [data]);
}

/** Each Major Order target planet's order deadline, epoch ms (the earliest, if two orders share one). */
export function useMajorOrderDeadlines(): Map<number, number> {
  const { data } = useAssignments();
  return useMemo(() => {
    const m = new Map<number, number>();
    for (const a of data ?? []) {
      const ends = Date.parse(a.expiration);
      for (const t of a.tasks) {
        const i = t.valueTypes.indexOf(12);
        const planet = i >= 0 ? t.values[i] : 0;
        if (planet > 0 && !(ends >= (m.get(planet) ?? Infinity))) m.set(planet, ends);
      }
    }
    return m;
  }, [data]);
}

/** Planet indices with a Democracy Space Station in orbit. */
export function useStationPlanets(): Set<number> {
  const { data } = useSpaceStations();
  return useMemo(() => new Set((data ?? []).map((s) => s.planetIndex)), [data]);
}

/** The collector's progress rate (%/h) for a planet with an active campaign, else null. */
export function usePlanetRate(planet: number | null | undefined): number | null {
  const { data } = useCampaigns();
  return data?.find((c) => c.planet.index === planet)?.rate ?? null;
}

export function useFronts(): { fronts: Front[]; loading: boolean } {
  const { data, isLoading, dataUpdatedAt } = useCampaigns();
  const mo = useMajorOrderPlanets();
  const stations = useStationPlanets();
  const pinnedList = useSettings((s) => s.pinned);
  const fronts = useMemo(() => {
    const now = dataUpdatedAt || Date.now();
    const pinned = new Set(pinnedList);
    return (data ?? [])
      .map((c) => buildFront(c, c.rate, mo, now, { pinned, stations }))
      .sort(
        (a, b) =>
          Number(b.pinned) - Number(a.pinned) ||
          Number(b.inMajorOrder) - Number(a.inMajorOrder) ||
          Number(b.kind === "defense") - Number(a.kind === "defense") ||
          b.players - a.players,
      );
  }, [data, mo, stations, pinnedList, dataUpdatedAt]);
  return { fronts, loading: isLoading };
}
