import { useQuery } from "@tanstack/react-query";
import { POLL_MS } from "../config";
import type { WarSampleRow } from "../domain/snapshot";
import { useLastUpdate } from "./queries";
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
