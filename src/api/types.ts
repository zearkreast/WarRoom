export type Faction = "Humans" | "Terminids" | "Automaton" | "Illuminate";

export interface Statistics {
  missionsWon: number;
  missionsLost: number;
  missionTime: number;
  terminidKills: number;
  automatonKills: number;
  illuminateKills: number;
  bulletsFired: number;
  bulletsHit: number;
  timePlayed: number;
  deaths: number;
  revives: number;
  friendlies: number;
  missionSuccessRate: number;
  accuracy: number;
  playerCount: number;
}

export interface War {
  started: string;
  ended: string;
  now: string;
  clientVersion: string;
  factions: Faction[];
  impactMultiplier: number;
  statistics: Statistics;
}

export interface PlanetEvent {
  id: number;
  eventType: number;
  faction: Faction;
  health: number;
  maxHealth: number;
  startTime: string;
  endTime: string;
  campaignId: number;
  jointOperationIds: number[];
}

export interface Region {
  id: number;
  name: string;
  description: string;
  health: number | null;
  maxHealth: number;
  size: string;
  /** Enemy regeneration in health per second, while the region is fought over. */
  regenPerSecond: number | null;
  isAvailable: boolean;
  players: number;
  /** Progress trend in %/h from the collector; only on campaign planets, null while measuring. */
  rate?: number | null;
}

export interface Planet {
  index: number;
  name: string;
  sector: string;
  biome: { name: string; description: string };
  hazards: { name: string; description: string }[];
  hash: number;
  position: { x: number; y: number };
  waypoints: number[];
  maxHealth: number;
  health: number;
  disabled: boolean;
  initialOwner: Faction;
  currentOwner: Faction;
  regenPerSecond: number;
  event: PlanetEvent | null;
  statistics: Statistics;
  attacking: number[];
  regions: Region[];
}

export interface Campaign {
  id: number;
  planet: Planet;
  type: number;
  count: number;
  faction: Faction;
}

export interface AssignmentTask {
  type: number;
  values: number[];
  valueTypes: number[];
}

export interface Reward {
  type: number;
  amount: number;
}

export interface Assignment {
  id: number;
  progress: number[];
  title: string | null;
  briefing: string | null;
  description: string | null;
  tasks: AssignmentTask[];
  reward: Reward | null;
  rewards: Reward[];
  expiration: string;
  flags: number;
}

export interface Dispatch {
  id: number;
  published: string;
  type: number;
  message: string;
}

export interface TacticalActionCost {
  id: string;
  itemMixId: number;
  targetValue: number;
  currentValue: number;
  deltaPerSecond: number;
  maxDonationAmmount: number;
  maxDonationPeriodSeconds: number;
}

export interface TacticalAction {
  id32: number;
  mediaId32: number;
  name: string;
  description: string;
  strategicDescription: string;
  /** 1 preparing, 2 active, 3 on cooldown (0 or other: unavailable). */
  status: number;
  statusExpire: string;
  costs: TacticalActionCost[];
  effectIds: number[];
}

/** A Democracy Space Station, from /api/v1/space-stations. */
export interface SpaceStation {
  id32: number;
  planet: Planet;
  /** When the vote on the next destination closes, i.e. the next FTL jump. */
  electionEnd: string;
  flags: number;
  tacticalActions: TacticalAction[];
}

/** A story campaign as the official game API returns it; times are war seconds. */
export interface EpisodeRaw {
  id32: number;
  title: string;
  description: string;
  /** Faction code: 1 Super Earth (several fronts), 2 Terminids, 3 Automatons, 4 Illuminate. */
  race: number;
  status: number;
  startWarTime: number;
  endWarTime?: number;
  phases: {
    introTitle: string;
    introMessage: string;
    outroMessage?: string;
    status: number;
    rewards: { mixId: number; amount: number }[];
  }[];
  rewards: { mixId: number; amount: number }[];
}

/** The parts of the official game API's war status the collector keeps: effects on planets and stations. */
export interface WarStatusRaw {
  time: number;
  planetActiveEffects: { index: number; galacticEffectId: number }[];
  spaceStations: { id32: number; planetIndex: number; activeEffectIds: number[] }[];
}
