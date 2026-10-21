import { useMemo } from "react";
import { useAssignments, useCampaigns } from "../../api/queries";
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
}

export function buildFront(c: Campaign, rate: number | null, moPlanets: Set<number>, now: number): Front {
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

/** The collector's progress rate (%/h) for a planet with an active campaign, else null. */
export function usePlanetRate(planet: number | null | undefined): number | null {
  const { data } = useCampaigns();
  return data?.find((c) => c.planet.index === planet)?.rate ?? null;
}

export function useFronts(): { fronts: Front[]; loading: boolean } {
  const { data, isLoading, dataUpdatedAt } = useCampaigns();
  const mo = useMajorOrderPlanets();
  const fronts = useMemo(() => {
    const now = dataUpdatedAt || Date.now();
    return (data ?? [])
      .map((c) => buildFront(c, c.rate, mo, now))
      .sort(
        (a, b) =>
          Number(b.inMajorOrder) - Number(a.inMajorOrder) ||
          Number(b.kind === "defense") - Number(a.kind === "defense") ||
          b.players - a.players,
      );
  }, [data, mo, dataUpdatedAt]);
  return { fronts, loading: isLoading };
}
