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
  isAvailable: boolean;
  players: number;
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
