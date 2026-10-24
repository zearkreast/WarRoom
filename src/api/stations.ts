import { useQuery } from "@tanstack/react-query";
import { POLL_MS } from "../config";
import { useLastUpdate } from "./queries";
import { getSource, SourceError } from "./source";

export interface StationJump {
  station: number;
  at: number;
  from: number | null;
  to: number;
}

/**
 * A station's FTL jumps, newest first, as the collector logged them. Empty while the `station_jumps`
 * table doesn't exist yet (older backends).
 */
export function useStationJumps(station: number) {
  const lastUpdate = useLastUpdate().data;
  return useQuery({
    queryKey: ["station_jumps", station, Math.floor((lastUpdate ?? 0) / (POLL_MS * 5))],
    enabled: lastUpdate != null,
    staleTime: POLL_MS * 5,
    queryFn: async (): Promise<StationJump[]> => {
      try {
        const rows = await (
          await getSource()
        ).select<{ station: number; at: string; from_planet: number | null; to_planet: number }>(
          "station_jumps",
          `select=*&station=eq.${station}&order=at.desc&limit=50`,
        );
        return rows.map((r) => ({ station: r.station, at: Date.parse(r.at), from: r.from_planet, to: r.to_planet }));
      } catch (e) {
        if (e instanceof SourceError && e.status === 404) return [];
        throw e;
      }
    },
  });
}
