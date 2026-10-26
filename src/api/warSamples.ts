import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import { POLL_MS } from "../config";
import { frontBuckets, latestRun, type FrontBucket, type FrontRow } from "../domain/frontActivity";
import { sectorTerritory } from "../domain/galaxy";
import { byIndex } from "../domain/planets";
import type { WarSampleRow } from "../domain/snapshot";
import { useFreshPlanets, useLastUpdate } from "./queries";
import { getSource, SourceError } from "./source";

export interface WarSample {
  at: number;
  impact: number;
  players: number;
  terminids: number;
  automatons: number;
  illuminate: number;
}

const DAY_MS = 86_400_000;

/**
 * The last 24 hours of galaxy-wide samples, ending at the newest data rather than the clock.
 * Resolves to an empty list while the `war_samples` table doesn't exist yet (older backends).
 */
export function useWarSamples() {
  const lastUpdate = useLastUpdate().data;
  const end = lastUpdate ?? 0;
  return useQuery({
    queryKey: ["war_samples", Math.floor(end / POLL_MS)],
    enabled: lastUpdate != null,
    staleTime: POLL_MS,
    queryFn: async (): Promise<WarSample[]> => {
      const since = encodeURIComponent(new Date(end - DAY_MS).toISOString());
      try {
        const rows = await (
          await getSource()
        ).select<WarSampleRow>("war_samples", `select=*&at=gte.${since}&order=at.asc&limit=2000`);
        return rows.map((r) => ({ ...r, at: Date.parse(r.at) }));
      } catch (e) {
        if (e instanceof SourceError && e.status === 404) return [];
        throw e;
      }
    },
  });
}

/**
 * Helldivers per front over the last 24 hours, or since the latest gap in the data. Summed in the
 * database from planet samples the collector has always kept, so there is no warm-up. A defense counts
 * for its attacker, or the enemy holding most of its sector.
 */
export function useFrontActivity() {
  const lastUpdate = useLastUpdate().data;
  const planets = useFreshPlanets().data;
  const query = useQuery({
    queryKey: ["front_activity", Math.floor((lastUpdate ?? 0) / POLL_MS)],
    enabled: lastUpdate != null,
    staleTime: POLL_MS,
    queryFn: async (): Promise<FrontRow[]> => {
      try {
        return await (await getSource()).rpc<FrontRow>("front_activity", { hours: 24 });
      } catch (e) {
        if (e instanceof SourceError && e.status === 404) return [];
        throw e;
      }
    },
  });
  const data = useMemo((): FrontBucket[] | undefined => {
    if (!query.data || !planets) return undefined;
    const lookup = byIndex(planets);
    const territory = sectorTerritory(planets);
    return latestRun(
      frontBuckets(query.data, (i) => {
        const p = lookup.get(i);
        return p?.event?.faction ?? (p && territory.get(p.sector)?.faction);
      }),
    );
  }, [query.data, planets]);
  return { data, isLoading: query.isLoading };
}

/** A SEST day's busiest and quietest moment galaxy-wide. */
export interface WarDay {
  /** The day's start, SEST (UTC) midnight. */
  day: number;
  peak: number;
  peakAt: number;
  low: number;
  lowAt: number;
}

interface WarDayRow {
  day: string;
  peak: number;
  peak_at: string;
  low: number;
  low_at: string;
}

/** The last year of days, newest first. Empty while the `war_days` table doesn't exist yet (older backends). */
export function useWarDays() {
  const lastUpdate = useLastUpdate().data;
  return useQuery({
    queryKey: ["war_days", Math.floor((lastUpdate ?? 0) / (POLL_MS * 5))],
    enabled: lastUpdate != null,
    staleTime: POLL_MS * 5,
    queryFn: async (): Promise<WarDay[]> => {
      try {
        const rows = await (await getSource()).select<WarDayRow>("war_days", "select=*&order=day.desc&limit=366");
        return rows.map((r) => ({
          day: Date.parse(`${r.day}T00:00:00Z`),
          peak: r.peak,
          peakAt: Date.parse(r.peak_at),
          low: r.low,
          lowAt: Date.parse(r.low_at),
        }));
      } catch (e) {
        if (e instanceof SourceError && e.status === 404) return [];
        throw e;
      }
    },
  });
}
