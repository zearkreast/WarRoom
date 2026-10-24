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
  WarStatusRaw,
} from "../api/types";

export type SnapshotKind =
  | "war"
  | "assignments"
  | "campaigns"
  | "planets"
  | "planets_static"
  | "dispatches"
  | "space_stations"
  | "episodes"
  | "planet_effects";

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

/** One Major Order of a campaign. */
export interface EpisodePhase {
  title: string;
  briefing: string;
  /** What the game says once the order has ended; empty while it runs. */
  outro: string;
  status: EpisodeStatus;
  medals: number | null;
  /**
   * When the order was issued, epoch ms. The game doesn't publish it: the first order starts with its
   * campaign, later ones are stamped by the collector when they appear, and older ones stay null.
   */
  issued: number | null;
}

/** A Galactic Campaign: Major Orders under one name, won when most of them are won; times are epoch ms. */
export interface EpisodeLive {
  id: number;
  title: string;
  description: string;
  /** The enemy it is fought against; Humans when it spans several fronts. */
  faction: Faction;
  status: EpisodeStatus;
  start: number;
  end: number | null;
  /** Item id of the campaign reward, if one is offered. */
  reward: number | null;
  phases: EpisodePhase[];
}

// The game's episode and phase status codes: 2 won, 3 failed; anything else is still running.
const EPISODE_STATUS: Record<number, EpisodeStatus> = { 2: "won", 3: "lost" };
// Same codes as FACTION_BY_ID; repeated because this file may only import types.
const RACE: Record<number, Faction> = { 1: "Humans", 2: "Terminids", 3: "Automaton", 4: "Illuminate" };
const MEDALS_MIX_ID = 897894480;

/** Converts the game's war-second timestamps using the war clock read at `fetchedAt`. */
export function toEpisodes(list: EpisodeRaw[], warTime: number, fetchedAt: number): EpisodeLive[] {
  const at = (t: number) => fetchedAt - (warTime - t) * 1000;
  return list.map((e) => {
    const start = at(e.startWarTime);
    return {
      id: e.id32,
      title: e.title,
      description: e.description,
      faction: RACE[e.race] ?? "Humans",
      status: EPISODE_STATUS[e.status] ?? "active",
      start,
      end: e.endWarTime ? at(e.endWarTime) : null,
      reward: e.rewards[0]?.mixId ?? null,
      phases: e.phases.map((p, i) => ({
        title: p.introTitle,
        briefing: p.introMessage,
        outro: p.outroMessage ?? "",
        status: EPISODE_STATUS[p.status] ?? "active",
        medals: p.rewards.find((r) => r.mixId === MEDALS_MIX_ID)?.amount ?? null,
        issued: i === 0 ? start : null,
      })),
    };
  });
}

/**
 * An order's issue date, or null when it can't be right: an older collector stamped orders of
 * campaigns that had already ended with the time it first saw them.
 */
export function issuedAt(e: Pick<EpisodeLive, "start" | "end">, issued: number | null | undefined): number | null {
  if (issued == null || issued < e.start) return null;
  return e.end != null && issued > e.end ? null : issued;
}

/**
 * Folds a fresh read into the stored list. Campaigns the game stops listing are kept, stored dates
 * win over re-derived ones (the war clock drifts a little between reads), and a Major Order that
 * appears on an already stored, still running campaign is stamped as issued at `now`.
 */
export function mergeEpisodes(stored: EpisodeLive[], fresh: EpisodeLive[], now: number): EpisodeLive[] {
  const before = new Map(stored.map((e) => [e.id, e]));
  const seen = new Set(fresh.map((e) => e.id));
  const merged = fresh.map((e) => {
    const was = before.get(e.id);
    const dates = { start: was?.start ?? e.start, end: was?.end ?? e.end };
    return {
      ...e,
      ...dates,
      phases: e.phases.map((p, i) => {
        if (i === 0) return { ...p, issued: dates.start };
        const known = was?.phases[i] ? issuedAt(dates, was.phases[i].issued) : undefined;
        return { ...p, issued: known !== undefined ? known : was && e.status === "active" ? now : null };
      }),
    };
  });
  return [...stored.filter((e) => !seen.has(e.id)), ...merged].sort((a, b) => a.start - b.start);
}

/** Major Orders won, lost and still running. The game doesn't say how many a campaign will have. */
export function campaignTally(e: EpisodeLive): { won: number; lost: number; running: number } {
  const won = e.phases.filter((p) => p.status === "won").length;
  const lost = e.phases.filter((p) => p.status === "lost").length;
  return { won, lost, running: e.phases.length - won - lost };
}

/** Galactic effect ids active on each planet and space station, from the official war status. */
export interface EffectsLive {
  planets: { index: number; ids: number[] }[];
  stations: { id: number; ids: number[] }[];
}

export function toEffectsLive(status: Pick<WarStatusRaw, "planetActiveEffects" | "spaceStations">): EffectsLive {
  const byPlanet = new Map<number, number[]>();
  for (const e of status.planetActiveEffects ?? []) {
    const ids = byPlanet.get(e.index) ?? [];
    if (!ids.includes(e.galacticEffectId)) ids.push(e.galacticEffectId);
    byPlanet.set(e.index, ids);
  }
  return {
    planets: [...byPlanet].sort(([a], [b]) => a - b).map(([index, ids]) => ({ index, ids: ids.sort((a, b) => a - b) })),
    stations: (status.spaceStations ?? []).map((s) => ({ id: s.id32, ids: [...s.activeEffectIds].sort((a, b) => a - b) })),
  };
}
