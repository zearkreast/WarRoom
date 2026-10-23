import { useQuery } from "@tanstack/react-query";
import { POLL_MS } from "../config";
import {
  toPlanet,
  type CampaignLive,
  type PlanetLive,
  type PlanetStatic,
  type SnapshotKind,
  type StationLive,
} from "../domain/snapshot";
import { getSource } from "./source";
import type { Assignment, Campaign, Dispatch, Planet, War } from "./types";

/** A campaign with its planet rebuilt in full, plus the collector's progress rate. */
export interface LiveCampaign extends Campaign {
  rate: number | null;
}

export interface Snapshots {
  data: Partial<Record<SnapshotKind, unknown>>;
  /** Newest `fetched_at` seen, so the next poll only downloads what changed. */
  since: string;
}

let latest: Snapshots = { data: {}, since: "" };

async function pollSnapshots(): Promise<Snapshots> {
  const rows = await (await getSource()).snapshots(latest.since);
  if (!rows.length && latest.since) return latest;
  const data = { ...latest.data };
  let since = latest.since;
  for (const r of rows) {
    data[r.kind] = r.data;
    if (r.fetched_at > since) since = r.fetched_at;
  }
  latest = { data, since };
  return latest;
}

function useSnapshot<T>(select: (s: Snapshots) => T | undefined) {
  return useQuery({
    queryKey: ["snapshots"],
    queryFn: pollSnapshots,
    refetchInterval: POLL_MS,
    staleTime: POLL_MS / 2,
    retry: 2,
    select,
  });
}

// Memoized per snapshot object, so every component sees the same arrays between polls.
const memo = <T>(fn: (s: Snapshots) => T | undefined) => {
  const cache = new WeakMap<Snapshots, T | undefined>();
  return (s: Snapshots) => {
    if (!cache.has(s)) cache.set(s, fn(s));
    return cache.get(s);
  };
};

const staticByIndex = memo((s) => {
  const list = s.data.planets_static as PlanetStatic[] | undefined;
  return list && new Map(list.map((p) => [p.index, p]));
});

const selectPlanets = memo((s): Planet[] | undefined => {
  const statics = staticByIndex(s);
  const live = s.data.planets as PlanetLive[] | undefined;
  if (!statics || !live) return undefined;
  return live.flatMap((l) => {
    const st = statics.get(l.index);
    return st ? [toPlanet(st, l)] : [];
  });
});

const selectCampaigns = memo((s): LiveCampaign[] | undefined => {
  const statics = staticByIndex(s);
  const list = s.data.campaigns as CampaignLive[] | undefined;
  if (!statics || !list) return undefined;
  return list.flatMap((c) => {
    const st = statics.get(c.planet.index);
    return st
      ? [{ id: c.id, type: c.type, count: c.count, faction: c.faction, rate: c.rate, planet: toPlanet(st, c.planet) }]
      : [];
  });
});

/** All planets, with campaign planets swapped for their fresher copy (campaigns poll every minute). */
const selectFreshPlanets = memo((s): Planet[] | undefined => {
  const planets = selectPlanets(s);
  const campaigns = selectCampaigns(s);
  if (!planets) return campaigns?.map((c) => c.planet);
  const fresh = new Map((campaigns ?? []).map((c) => [c.planet.index, c.planet]));
  return planets.map((p) => fresh.get(p.index) ?? p);
});

const selectStations = (s: Snapshots) => s.data.space_stations as StationLive[] | undefined;

const selectWar = (s: Snapshots) => s.data.war as War | undefined;
const selectAssignments = (s: Snapshots) => s.data.assignments as Assignment[] | undefined;
const selectDispatches = (s: Snapshots) => s.data.dispatches as Dispatch[] | undefined;

export const usePlanets = () => useSnapshot(selectPlanets);
export const useCampaigns = () => useSnapshot(selectCampaigns);
export const useWar = () => useSnapshot(selectWar);
export const useAssignments = () => useSnapshot(selectAssignments);
export const useDispatches = () => useSnapshot(selectDispatches);
/** Newest collector write seen, epoch ms; used to warn when the collector has stopped. */
const selectLastUpdate = (s: Snapshots) => (s.since ? Date.parse(s.since) : undefined);
export const useLastUpdate = () => useSnapshot(selectLastUpdate);
export const useFreshPlanets = () => useSnapshot(selectFreshPlanets);
export const useSpaceStations = () => useSnapshot(selectStations);

/** One planet by index, fresh copy preferred; undefined while loading or unknown. */
export function usePlanet(index: number | null | undefined): Planet | undefined {
  const { data } = useFreshPlanets();
  return index == null ? undefined : data?.find((p) => p.index === index);
}
