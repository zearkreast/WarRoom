// Snapshot shapes stored in Supabase: written by the collector, read by the app.
// Only type imports here, so the collector can run this file directly under Node.
import type {
  Campaign,
  EpisodeRaw,
  Dispatch,
  Faction,
  Planet,
  PlanetEvent,
  Region,
  SpaceStation,
  Statistics,
  TacticalAction,
  War,
} from "../api/types";

export type SnapshotKind =
  | "war"
  | "assignments"
  | "campaigns"
  | "planets"
  | "planets_static"
  | "dispatches"
  | "space_stations"
  | "episodes";

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

/** Counters the planet detail shows. timePlayed duplicates missionTime; accuracy is derivable. */
const LIVE_STATS = [
  "playerCount",
  "missionsWon",
  "missionsLost",
  "missionSuccessRate",
  "missionTime",
  "terminidKills",
  "automatonKills",
  "illuminateKills",
  "bulletsFired",
  "bulletsHit",
  "deaths",
  "friendlies",
] as const;

/** Planet statistics with zero counters left out (most planets have never been fought on). */
export type LiveStats = Partial<Pick<Statistics, (typeof LIVE_STATS)[number]>>;

function liveStats(s: Statistics): LiveStats {
  const out: LiveStats = {};
  for (const k of LIVE_STATS) if (s[k]) out[k] = s[k];
  return out;
}

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
    regions: p.regions.map((r) => ({
      id: r.id,
      name: r.name,
      description: r.description,
      size: r.size,
      maxHealth: r.maxHealth,
    })),
  };
}

export function toLive(p: Planet): PlanetLive {
  return {
    index: p.index,
    health: p.health,
    currentOwner: p.currentOwner,
    regenPerSecond: p.regenPerSecond,
    disabled: p.disabled,
    event: p.event,
    attacking: p.attacking,
    statistics: liveStats(p.statistics),
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
    // The API lists "None" as a hazard on planets without any.
    hazards: s.hazards.filter((h) => h.name.toLowerCase() !== "none"),
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

/** Space station as stored: the planet is reduced to its index (the app has the planet list). */
export interface StationLive {
  id32: number;
  planetIndex: number;
  electionEnd: string;
  flags: number;
  tacticalActions: TacticalAction[];
}

export function toStationLive(s: SpaceStation): StationLive {
  return {
    id32: s.id32,
    planetIndex: s.planet.index,
    electionEnd: s.electionEnd,
    flags: s.flags,
    tacticalActions: s.tacticalActions,
  };
}

/** One row per war poll: galaxy-wide trends for the Overview and Planets charts. */
export interface WarSampleRow {
  at: string;
  impact: number;
  players: number;
  terminids: number;
  automatons: number;
  illuminate: number;
}

/** Helldivers on each enemy front, from the active campaigns (defenses count for the attacker). */
export function playersByEnemy(
  campaigns: Pick<Campaign, "planet">[],
): Record<"Terminids" | "Automaton" | "Illuminate", number> {
  const out = { Terminids: 0, Automaton: 0, Illuminate: 0 };
  for (const c of campaigns) {
    const enemy = c.planet.event?.faction ?? c.planet.currentOwner;
    if (enemy in out) out[enemy as keyof typeof out] += c.planet.statistics.playerCount;
  }
  return out;
}

export function toWarSample(
  war: Pick<War, "impactMultiplier" | "statistics">,
  campaigns: Pick<Campaign, "planet">[],
  at: number,
): WarSampleRow {
  const fronts = playersByEnemy(campaigns);
  return {
    at: new Date(at).toISOString(),
    impact: war.impactMultiplier,
    players: war.statistics.playerCount,
    terminids: fronts.Terminids,
    automatons: fronts.Automaton,
    illuminate: fronts.Illuminate,
  };
}

export type EpisodeStatus = "active" | "won" | "lost";

/** A story campaign with its phases; times are epoch ms. */
export interface EpisodeLive {
  id: number;
  title: string;
  description: string;
  status: EpisodeStatus;
  start: number;
  end: number | null;
  phases: { title: string; briefing: string; status: EpisodeStatus; medals: number | null }[];
}

// The game's episode and phase status codes: 2 won, 3 failed; anything else is still running.
const EPISODE_STATUS: Record<number, EpisodeStatus> = { 2: "won", 3: "lost" };
const MEDALS_MIX_ID = 897894480;

/** Converts the game's war-second timestamps using the war clock read at `fetchedAt`. */
export function toEpisodes(list: EpisodeRaw[], warTime: number, fetchedAt: number): EpisodeLive[] {
  const at = (t: number) => fetchedAt - (warTime - t) * 1000;
  return list.map((e) => ({
    id: e.id32,
    title: e.title,
    description: e.description,
    status: EPISODE_STATUS[e.status] ?? "active",
    start: at(e.startWarTime),
    end: e.endWarTime ? at(e.endWarTime) : null,
    phases: e.phases.map((p) => ({
      title: p.introTitle,
      briefing: p.introMessage,
      status: EPISODE_STATUS[p.status] ?? "active",
      medals: p.rewards.find((r) => r.mixId === MEDALS_MIX_ID)?.amount ?? null,
    })),
  }));
}
