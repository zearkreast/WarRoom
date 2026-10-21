import type { Faction, Planet } from "../src/api/types.ts";
import { FACTION_BY_ID, FACTION_ID } from "../src/domain/factions.ts";
import { detectOwnerChanges, isActive, type OwnerChange } from "../src/domain/history.ts";
import { ratePerHour, toSample, type Sample } from "../src/domain/progress.ts";
import type { Supabase } from "./supabase.ts";

/** Samples kept in memory for rates. Matches the window ratePerHour looks at. */
const RATE_WINDOW_MS = 60 * 60_000;
/** PostgREST returns at most 1000 rows per request by default. */
const PAGE = 1000;

interface SampleRow {
  planet: number;
  at: string;
  owner: number;
  percent: number;
  players: number;
  defending: boolean;
}

const toRow = (s: Sample): SampleRow => ({
  planet: s.planet,
  at: new Date(s.at).toISOString(),
  owner: FACTION_ID[s.owner as Faction],
  percent: s.percent,
  players: s.players,
  defending: s.defending,
});

const fromRow = (r: SampleRow): Sample => ({
  planet: r.planet,
  at: Date.parse(r.at),
  owner: FACTION_BY_ID[r.owner],
  percent: r.percent,
  players: r.players,
  defending: r.defending,
});

export interface IngestResult {
  samples: number;
  changes: OwnerChange[];
}

/**
 * Turns each API snapshot into stored samples and owner-change events, and keeps the
 * last hour of samples in memory so campaign rates survive between polls.
 */
export class Recorder {
  private owners = new Map<number, string>();
  private recent = new Map<number, Sample[]>();
  private db: Supabase;

  constructor(db: Supabase) {
    this.db = db;
  }

  /** Restores state after a restart, so a restart neither forgets rates nor invents captures. */
  async load(now = Date.now()): Promise<void> {
    const owners = await this.db.select<{ planet: number; owner: number }>("planet_owners", "select=planet,owner");
    this.owners = new Map(owners.map((o) => [o.planet, FACTION_BY_ID[o.owner]]));
    const since = new Date(now - RATE_WINDOW_MS).toISOString();
    for (let offset = 0; ; offset += PAGE) {
      const rows = await this.db.select<SampleRow>(
        "samples",
        `select=*&at=gte.${since}&order=at.asc&limit=${PAGE}&offset=${offset}`,
      );
      for (const r of rows) this.remember(fromRow(r));
      if (rows.length < PAGE) break;
    }
  }

  get knownPlanets(): number {
    return this.owners.size;
  }

  rate(planet: number): number | null {
    return ratePerHour(this.recent.get(planet) ?? []);
  }

  async ingest(planets: Planet[], at: number): Promise<IngestResult> {
    const changes = detectOwnerChanges(this.owners, planets, at);
    const updated: { planet: number; owner: number }[] = [];
    for (const p of planets) {
      if (this.owners.get(p.index) !== p.currentOwner) {
        this.owners.set(p.index, p.currentOwner);
        updated.push({ planet: p.index, owner: FACTION_ID[p.currentOwner] });
      }
    }

    // A planet that just changed hands gets one sample even if it's no longer contested,
    // so its chart ends at the moment of capture.
    const changed = new Set(changes.map((c) => c.planet));
    const samples = planets.filter((p) => isActive(p) || changed.has(p.index)).map((p) => toSample(p, at));
    for (const s of samples) this.remember(s);

    // Duplicates (same planet and time) are ignored, so two collectors can safely overlap.
    await this.db.upsert("samples", samples.map(toRow), { ignoreDuplicates: true });
    await this.db.insert(
      "owner_changes",
      changes.map((c) => ({
        planet: c.planet,
        planet_name: c.planetName,
        from_owner: FACTION_ID[c.from as Faction],
        to_owner: FACTION_ID[c.to as Faction],
        at: new Date(c.at).toISOString(),
      })),
    );
    await this.db.upsert("planet_owners", updated);
    return { samples: samples.length, changes };
  }

  private remember(s: Sample) {
    const list = this.recent.get(s.planet) ?? [];
    list.push(s);
    const cutoff = s.at - RATE_WINDOW_MS;
    while (list.length && list[0].at < cutoff) list.shift();
    this.recent.set(s.planet, list);
  }
}
