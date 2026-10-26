import type { Faction, Planet } from "../src/api/types.ts";
import { FACTION_BY_ID, FACTION_ID } from "../src/domain/factions.ts";
import { detectOwnerChanges, isActive, type OwnerChange } from "../src/domain/history.ts";
import { ratePerHour, toSample, type Sample } from "../src/domain/progress.ts";
import { unlessMissing, type Supabase } from "./supabase.ts";

/** Samples kept in memory for rates. Matches the window ratePerHour looks at. */
const RATE_WINDOW_MS = 60 * 60_000;
/** PostgREST returns at most 1000 rows per request by default. */
const PAGE = 1000;
/** A city is stored at most this often; the campaigns task reads its planet every minute. */
const REGION_EVERY_MS = 4.5 * 60_000;

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

interface RegionRow {
  planet: number;
  region: number;
  at: string;
  players: number;
  percent: number | null;
}

/** A city worth a reading: Helldivers in it, or its liberation counting down. */
function regionRow(p: Planet, r: Planet["regions"][number], at: number): RegionRow | null {
  const percent = r.isAvailable && r.health != null && r.maxHealth ? 100 - (r.health / r.maxHealth) * 100 : null;
  if (r.players <= 0 && percent == null) return null;
  return { planet: p.index, region: r.id, at: new Date(at).toISOString(), players: r.players, percent };
}

/** A region's liberation as a sample of its planet, so the planet's rate maths applies to it. */
function regionSample(p: Planet, r: Planet["regions"][number], at: number): Sample | null {
  if (r.health == null || !r.maxHealth) return null;
  return {
    planet: p.index,
    at,
    owner: p.currentOwner,
    percent: 100 - (r.health / r.maxHealth) * 100,
    players: r.players,
    defending: p.event != null,
  };
}

/**
 * Turns each API snapshot into stored samples and owner-change events, and keeps the
 * last hour of samples in memory so campaign rates survive between polls.
 * Region readings are kept in memory only, so their rates start over after a restart.
 */
export class Recorder {
  private owners = new Map<number, string>();
  private recent = new Map<number, Sample[]>();
  private regions = new Map<string, Sample[]>();
  private regionWritten = new Map<string, number>();
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

  regionRate(planet: number, region: number): number | null {
    return ratePerHour(this.regions.get(`${planet}:${region}`) ?? []);
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
    const contested = (p: Planet) => isActive(p) || changed.has(p.index);
    const samples = planets.filter(contested).map((p) => toSample(p, at));
    for (const s of samples) this.remember(s);
    const idle = planets
      .filter((p) => !contested(p) && p.statistics.playerCount > 0)
      .map((p) => ({ planet: p.index, at: new Date(at).toISOString(), players: p.statistics.playerCount }));
    const cities: RegionRow[] = [];
    for (const p of planets) {
      for (const r of p.regions) {
        const key = `${p.index}:${r.id}`;
        const s = regionSample(p, r, at);
        if (s) keep(this.regions, key, s);
        const row = regionRow(p, r, at);
        if (row && at - (this.regionWritten.get(key) ?? -Infinity) >= REGION_EVERY_MS) {
          cities.push(row);
          this.regionWritten.set(key, at);
        }
      }
    }

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
    await unlessMissing(this.db.upsert("planet_players", idle, { ignoreDuplicates: true }));
    await unlessMissing(this.db.upsert("region_samples", cities, { ignoreDuplicates: true }));
    return { samples: samples.length, changes };
  }

  private remember(s: Sample) {
    keep(this.recent, s.planet, s);
  }
}

function keep<K>(map: Map<K, Sample[]>, key: K, s: Sample) {
  const list = map.get(key) ?? [];
  list.push(s);
  const cutoff = s.at - RATE_WINDOW_MS;
  while (list.length && list[0].at < cutoff) list.shift();
  map.set(key, list);
}
