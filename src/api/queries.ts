import { useQuery } from "@tanstack/react-query";
import { POLL_MS } from "../config";
import { GLOOM_IDS, planetEffects, type PlanetEffect } from "../domain/effects";
import {
  toPlanet,
  type CampaignLive,
  type EffectsLive,
  type EpisodeLive,
  type OrderSample,
  type PlanetLive,
  type PlanetStatic,
  type SnapshotKind,
  type StationLive,
} from "../domain/snapshot";
import { buildPolarSectors, type PolarSectors } from "../domain/galaxy";
import { fixStatic, isOffMap } from "../domain/planets";
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

/**
 * Caches a result on the identity of its inputs: a poll replaces only the snapshots that changed, so
 * nothing built from the others (every planet, when only the war counters moved) is rebuilt.
 */
function memoOn<A extends object, B extends object, T>(fn: (a: A, b: B) => T) {
  const cache = new WeakMap<A, WeakMap<B, T>>();
  return (a: A, b: B): T => {
    let inner = cache.get(a);
    if (!inner) cache.set(a, (inner = new WeakMap()));
    if (!inner.has(b)) inner.set(b, fn(a, b));
    return inner.get(b)!;
  };
}
function memo<A extends object, T>(fn: (a: A) => T) {
  const cache = new WeakMap<A, T>();
  return (a: A): T => {
    if (!cache.has(a)) cache.set(a, fn(a));
    return cache.get(a)!;
  };
}

const fixStatics = memo((list: PlanetStatic[]) => list.map(fixStatic));
const staticOf = (s: Snapshots) => {
  const list = s.data.planets_static as PlanetStatic[] | undefined;
  return list && fixStatics(list);
};
const staticByIndex = memo((list: PlanetStatic[]) => new Map(list.map((p) => [p.index, p])));

const buildPlanets = memoOn((statics: PlanetStatic[], live: PlanetLive[]): Planet[] => {
  const byIndex = staticByIndex(statics);
  return live.flatMap((l) => {
    const st = byIndex.get(l.index);
    return st && !isOffMap(st) ? [toPlanet(st, l)] : [];
  });
});
const selectPlanets = (s: Snapshots): Planet[] | undefined => {
  const statics = staticOf(s);
  const live = s.data.planets as PlanetLive[] | undefined;
  return statics && live ? buildPlanets(statics, live) : undefined;
};

const buildCampaigns = memoOn((statics: PlanetStatic[], list: CampaignLive[]): LiveCampaign[] => {
  const byIndex = staticByIndex(statics);
  return list.flatMap((c) => {
    const st = byIndex.get(c.planet.index);
    return st
      ? [{ id: c.id, type: c.type, count: c.count, faction: c.faction, rate: c.rate, planet: toPlanet(st, c.planet) }]
      : [];
  });
});
const selectCampaigns = (s: Snapshots): LiveCampaign[] | undefined => {
  const statics = staticOf(s);
  const list = s.data.campaigns as CampaignLive[] | undefined;
  return statics && list ? buildCampaigns(statics, list) : undefined;
};

/** All planets, with campaign planets swapped for their fresher copy (campaigns poll every minute). */
const mergeFresh = memoOn((planets: Planet[], campaigns: LiveCampaign[]): Planet[] => {
  const fresh = new Map(campaigns.map((c) => [c.planet.index, c.planet]));
  return planets.map((p) => fresh.get(p.index) ?? p);
});
const selectFreshPlanets = (s: Snapshots): Planet[] | undefined => {
  const planets = selectPlanets(s);
  const campaigns = selectCampaigns(s);
  if (!planets) return campaigns?.map((c) => c.planet);
  return campaigns ? mergeFresh(planets, campaigns) : planets;
};

/** Sector shapes depend only on where planets are, so they're built once per static snapshot. */
const buildGeometry = memo((statics: PlanetStatic[]) => buildPolarSectors(statics.filter((p) => !isOffMap(p))));
const selectGeometry = (s: Snapshots): PolarSectors | undefined => {
  const statics = staticOf(s);
  return statics && buildGeometry(statics);
};

export interface WarEffects {
  /** Named effects per planet index; planets without any are absent. */
  planets: Map<number, PlanetEffect[]>;
  /** Planets in or on the edge of the Gloom. */
  gloom: Set<number>;
  stations: Map<number, PlanetEffect[]>;
}
const buildEffects = memo(
  (live: EffectsLive): WarEffects => ({
    planets: new Map(live.planets.map((p) => [p.index, planetEffects(p.ids)])),
    gloom: new Set(live.planets.filter((p) => p.ids.some((id) => GLOOM_IDS.has(id))).map((p) => p.index)),
    stations: new Map(live.stations.map((s) => [s.id, planetEffects(s.ids)])),
  }),
);
const selectEffects = (s: Snapshots) => {
  const live = s.data.planet_effects as EffectsLive | undefined;
  return live && buildEffects(live);
};

const selectStations = (s: Snapshots) => s.data.space_stations as StationLive[] | undefined;
const selectEpisodes = (s: Snapshots) => s.data.episodes as EpisodeLive[] | undefined;

const selectWar = (s: Snapshots) => s.data.war as War | undefined;
const selectAssignments = (s: Snapshots) => s.data.assignments as Assignment[] | undefined;
const selectDispatches = (s: Snapshots) => s.data.dispatches as Dispatch[] | undefined;
const selectOrderSamples = (s: Snapshots) => s.data.order_samples as OrderSample[] | undefined;

export const usePlanets = () => useSnapshot(selectPlanets);
export const useCampaigns = () => useSnapshot(selectCampaigns);
export const useWar = () => useSnapshot(selectWar);
export const useAssignments = () => useSnapshot(selectAssignments);
export const useDispatches = () => useSnapshot(selectDispatches);
/** The last day of Major Order readings; undefined until the collector has written them once. */
export const useOrderSamples = () => useSnapshot(selectOrderSamples);
/** Newest collector write seen, epoch ms; used to warn when the collector has stopped. */
const selectLastUpdate = (s: Snapshots) => (s.since ? Date.parse(s.since) : undefined);
export const useLastUpdate = () => useSnapshot(selectLastUpdate);
export const useFreshPlanets = () => useSnapshot(selectFreshPlanets);
export const useSectorGeometry = () => useSnapshot(selectGeometry);
export const useSpaceStations = () => useSnapshot(selectStations);
/** Story campaigns, oldest first; undefined until the collector has written them once. */
export const useEpisodes = () => useSnapshot(selectEpisodes);
/** Galactic effects on planets and stations; undefined until the collector has written them once. */
export const useEffects = () => useSnapshot(selectEffects);

/** One planet by index, fresh copy preferred; undefined while loading or unknown. */
export function usePlanet(index: number | null | undefined): Planet | undefined {
  const { data } = useFreshPlanets();
  return index == null ? undefined : data?.find((p) => p.index === index);
}
