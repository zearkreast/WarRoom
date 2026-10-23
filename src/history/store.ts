import { getSource } from "../api/source";
import { FACTION_BY_ID } from "../domain/factions";
import type { OwnerChange } from "../domain/history";
import type { Sample } from "../domain/progress";

/** An ownership change as stored by the collector; `id` increases with every new row. */
export interface ChangeRecord extends OwnerChange {
  id: number;
}

/**
 * Read side of the shared war history the collector writes to Supabase. Everyone sees the
 * same history, from whenever the collector first ran.
 */
export interface HistoryStore {
  samples(planet: number, sinceMs: number): Promise<Sample[]>;
  changes(opts: { planet?: number; limit: number }): Promise<ChangeRecord[]>;
  /** Time of the oldest stored sample for a planet (or for any planet), or null if none. */
  earliest(planet?: number): Promise<number | null>;
  /** Times of samples taken while the planet was under attack, oldest first. */
  defendingTimes(planet: number): Promise<number[]>;
  /** Changes newer than `afterId`, newest first. */
  changesAfter(afterId: number, limit: number): Promise<ChangeRecord[]>;
}

interface SampleRow {
  planet: number;
  at: string;
  owner: number;
  percent: number;
  players: number;
  defending: boolean;
}

interface ChangeRow {
  id: number;
  planet: number;
  planet_name: string;
  from_owner: number;
  to_owner: number;
  at: string;
}

const toChange = (r: ChangeRow): ChangeRecord => ({
  id: r.id,
  planet: r.planet,
  planetName: r.planet_name,
  from: FACTION_BY_ID[r.from_owner],
  to: FACTION_BY_ID[r.to_owner],
  at: Date.parse(r.at),
});

/** PostgREST caps responses at 1000 rows; 7 days of 15-minute samples is ~700. */
const SAMPLE_LIMIT = 1000;

export const supabaseHistory: HistoryStore = {
  async samples(planet, sinceMs) {
    const since = encodeURIComponent(new Date(sinceMs).toISOString());
    const rows = await (
      await getSource()
    ).select<SampleRow>("samples", `select=*&planet=eq.${planet}&at=gte.${since}&order=at.desc&limit=${SAMPLE_LIMIT}`);
    return rows.reverse().map((r) => ({
      planet: r.planet,
      at: Date.parse(r.at),
      owner: FACTION_BY_ID[r.owner],
      percent: r.percent,
      players: r.players,
      defending: r.defending,
    }));
  },
  async earliest(planet) {
    const filter = planet == null ? "" : `&planet=eq.${planet}`;
    const rows = await (await getSource()).select<{ at: string }>("samples", `select=at${filter}&order=at.asc&limit=1`);
    return rows[0] ? Date.parse(rows[0].at) : null;
  },
  async defendingTimes(planet) {
    const rows = await (
      await getSource()
    ).select<{ at: string }>(
      "samples",
      `select=at&planet=eq.${planet}&defending=is.true&order=at.asc&limit=${SAMPLE_LIMIT}`,
    );
    return rows.map((r) => Date.parse(r.at));
  },
  async changes({ planet, limit }) {
    const filter = planet == null ? "" : `&planet=eq.${planet}`;
    const rows = await (
      await getSource()
    ).select<ChangeRow>("owner_changes", `select=*${filter}&order=id.desc&limit=${limit}`);
    return rows.map(toChange);
  },
  async changesAfter(afterId, limit) {
    const rows = await (
      await getSource()
    ).select<ChangeRow>("owner_changes", `select=*&id=gt.${afterId}&order=id.desc&limit=${limit}`);
    return rows.map(toChange);
  },
};
