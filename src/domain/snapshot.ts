// Snapshot shapes stored in Supabase: written by the collector, read by the app.
// Only type imports here, so the collector can run this file directly under Node.
import type { Campaign, Dispatch, Faction, Planet, PlanetEvent, Region, Statistics } from "../api/types";

export type SnapshotKind = "war" | "assignments" | "campaigns" | "planets" | "planets_static" | "dispatches";

/** Planet fields that rarely change: sent once per app launch. */
export interface PlanetStatic {
  index: number;
  name: string;
  sector: string;
  biome: Planet["biome"];
  hazards: Planet["hazards"];
  hash: number;
  position: Planet["position"];
  waypoints: number[];
  maxHealth: number;
  initialOwner: Faction;
  regions: Pick<Region, "id" | "name" | "description" | "size" | "maxHealth">[];
}

/** The planet stats the app shows. The other counters stay out of the snapshot. */
export type LiveStats = Pick<Statistics, "playerCount" | "missionsWon" | "missionsLost" | "deaths" | "missionSuccessRate">;

/** Planet fields that change every poll. */
export interface PlanetLive {
  index: number;
  health: number;
  currentOwner: Faction;
  regenPerSecond: number;
  disabled: boolean;
  event: PlanetEvent | null;
  attacking: number[];
  statistics: LiveStats;
  regions: Pick<Region, "id" | "health" | "isAvailable" | "players">[];
}

export interface CampaignLive {
  id: number;
  type: number;
  count: number;
  faction: Faction;
  planet: PlanetLive;
  /** Progress trend in %/h, computed by the collector from its samples; null while measuring. */
  rate: number | null;
}

/** Newest dispatches kept in the snapshot. */
export const DISPATCH_LIMIT = 50;

export function toStatic(p: Planet): PlanetStatic {
  return {
    index: p.index,
    name: p.name,
    sector: p.sector,
    biome: p.biome,
    hazards: p.hazards,
    hash: p.hash,
    position: p.position,
    waypoints: p.waypoints,
    maxHealth: p.maxHealth,
    initialOwner: p.initialOwner,
    regions: p.regions.map((r) => ({ id: r.id, name: r.name, description: r.description, size: r.size, maxHealth: r.maxHealth })),
  };
}

export function toLive(p: Planet): PlanetLive {
  const s = p.statistics;
  return {
    index: p.index,
    health: p.health,
    currentOwner: p.currentOwner,
    regenPerSecond: p.regenPerSecond,
    disabled: p.disabled,
    event: p.event,
    attacking: p.attacking,
    statistics: {
      playerCount: s.playerCount,
      missionsWon: s.missionsWon,
      missionsLost: s.missionsLost,
      deaths: s.deaths,
      missionSuccessRate: s.missionSuccessRate,
    },
    regions: p.regions.map((r) => ({ id: r.id, health: r.health, isAvailable: r.isAvailable, players: r.players })),
  };
}

export function toCampaignLive(c: Campaign, rate: number | null): CampaignLive {
  return { id: c.id, type: c.type, count: c.count, faction: c.faction, planet: toLive(c.planet), rate };
}

export function newestDispatches(d: Dispatch[]): Dispatch[] {
  return [...d].sort((a, b) => b.id - a.id).slice(0, DISPATCH_LIMIT);
}

const ZERO_STATS: Statistics = {
  missionsWon: 0,
  missionsLost: 0,
  missionTime: 0,
  terminidKills: 0,
  automatonKills: 0,
  illuminateKills: 0,
  bulletsFired: 0,
  bulletsHit: 0,
  timePlayed: 0,
  deaths: 0,
  revives: 0,
  friendlies: 0,
  missionSuccessRate: 0,
  accuracy: 0,
  playerCount: 0,
};

/** Rebuilds the full `Planet` the UI works with from its static and live halves. */
export function toPlanet(s: PlanetStatic, l: PlanetLive): Planet {
  const live = new Map(l.regions.map((r) => [r.id, r]));
  return {
    ...s,
    health: l.health,
    currentOwner: l.currentOwner,
    regenPerSecond: l.regenPerSecond,
    disabled: l.disabled,
    event: l.event,
    attacking: l.attacking,
    statistics: { ...ZERO_STATS, ...l.statistics },
    regions: s.regions.map((r) => ({
      ...r,
      health: live.get(r.id)?.health ?? null,
      isAvailable: live.get(r.id)?.isAvailable ?? false,
      players: live.get(r.id)?.players ?? 0,
    })),
  };
}
