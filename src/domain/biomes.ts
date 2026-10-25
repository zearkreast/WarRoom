import type { Planet } from "../api/types";
import { anomalyOf, type Anomaly } from "./galaxy";

/** Two colours standing in for a biome's look, sky/atmosphere and ground, and its landscape picture. */
export interface BiomeTint {
  sky: string;
  ground: string;
  /** CSS url() of the biome's landscape banner, when there is one. */
  art?: string;
}

// The game's 25 biomes by their API names, coloured from their landscapes.
const BIOMES: Record<string, Omit<BiomeTint, "art">> = {
  plains: { sky: "#1c2a1a", ground: "#8fae5e" },
  "ionic crimson": { sky: "#2a0d14", ground: "#c4405a" },
  "icy glaciers": { sky: "#13283a", ground: "#a9d2ef" },
  "desert dunes": { sky: "#3a2a12", ground: "#d9a54a" },
  "rocky canyons": { sky: "#2e1e14", ground: "#b47a4a" },
  "basic swamp": { sky: "#16231a", ground: "#6f8f4f" },
  tundra: { sky: "#1d2622", ground: "#9aa88a" },
  boneyard: { sky: "#1a2230", ground: "#c7d3dd" },
  "ethereal jungle": { sky: "#1e1430", ground: "#9f7fd0" },
  "volcanic jungle": { sky: "#1b2410", ground: "#5fa04a" },
  "acidic badlands": { sky: "#1d2410", ground: "#b6d33b" },
  "haunted swamp": { sky: "#16141e", ground: "#6e6a86" },
  "scorched moor": { sky: "#331a0c", ground: "#d0743a" },
  "ionic jungle": { sky: "#0e1f2a", ground: "#4fa8c8" },
  deadlands: { sky: "#1a1a1e", ground: "#8a8a92" },
  moon: { sky: "#1a1c22", ground: "#9a9ca6" },
  "desert cliffs": { sky: "#3a2412", ground: "#c98a4a" },
  magma: { sky: "#3a1208", ground: "#e2582a" },
  "hive world": { sky: "#24180a", ground: "#c98a2c" },
  "desert oasis": { sky: "#2a2a14", ground: "#d0b860" },
  "deciduous autumn forest": { sky: "#2a1a0e", ground: "#d08a3a" },
  "deciduous forest": { sky: "#16261a", ground: "#6fb05a" },
  "super earth": { sky: "#0d1f2e", ground: "#3f8cc2" },
  supercolony: { sky: "#24100a", ground: "#b8582c" },
  "cyberstan megafactory": { sky: "#1a1416", ground: "#8a4a4a" },
};

// Names the game may add later still get a fitting colour: matched against the name, first hit wins.
const TINTS: [RegExp, BiomeTint][] = [
  [/magma|volcan|lava|scorch|hellmire/i, { sky: "#3a1208", ground: "#e2582a" }],
  [/desert|sand|dune|mesa|canyon/i, { sky: "#3a2a12", ground: "#d9a54a" }],
  [/winter|ice|snow|tundra|frost|glacier|arctic/i, { sky: "#13283a", ground: "#a9d2ef" }],
  [/swamp|bog|marsh|moor|mire/i, { sky: "#16231a", ground: "#7d9b5c" }],
  [/jungle|rainforest|forest|primordial|ethereal/i, { sky: "#0f2418", ground: "#4fb06a" }],
  [/toxic|acid|blight|haz/i, { sky: "#1d2410", ground: "#b6d33b" }],
  [/crimson|blood|red/i, { sky: "#2a0d14", ground: "#c4405a" }],
  [/highland|plateau|plains|grass/i, { sky: "#1a2418", ground: "#9cab6e" }],
  [/rock|desolate|barren|waste|crater|moon|asteroid/i, { sky: "#1a1c22", ground: "#8a8c96" }],
  [/ocean|sea|water|coast|archipelago/i, { sky: "#0d1f2e", ground: "#3f8cc2" }],
  [/colony|hive|super/i, { sky: "#24180a", ground: "#c98a2c" }],
];

const DEFAULT_TINT: BiomeTint = { sky: "#141a24", ground: "#6b7d90" };

const ART = import.meta.glob<string>("../assets/biomes/*.png", { eager: true, import: "default", query: "?url" });
const slug = (name: string) => name.toLowerCase().replace(/[^a-z]+/g, "-");

export function biomeTint(biome: string | undefined): BiomeTint {
  if (!biome) return DEFAULT_TINT;
  const known = BIOMES[biome.toLowerCase()];
  const art = ART[`../assets/biomes/${slug(biome)}.png`];
  const tint = known ?? TINTS.find(([re]) => re.test(biome))?.[1] ?? DEFAULT_TINT;
  return art ? { ...tint, art: `url("${art}")` } : tint;
}

const LOOK_ART = import.meta.glob<string>("../assets/looks/*.png", { eager: true, import: "default", query: "?url" });
const look = (file: string, sky: string, ground: string): BiomeTint => {
  const art = LOOK_ART[`../assets/looks/${file}.png`];
  return { sky, ground, art: art ? `url("${art}")` : undefined };
};

// A planet the war has destroyed no longer looks like its biome.
const LOOKS: Partial<Record<Anomaly, BiomeTint>> = {
  wormhole: look("meridian-black-hole", "#0a0a2a", "#6a5cff"),
  blackhole: look("black-hole", "#050508", "#7d93ad"),
  fractured: look("fractured", "#140a10", "#e0553a"),
};

/** A planet's colours and picture: its biome's, or the black hole or debris it has become. */
export function planetTint(p: Pick<Planet, "name" | "biome">, effects?: { id: number }[]): BiomeTint {
  const anomaly = anomalyOf(p, effects);
  return (anomaly && LOOKS[anomaly]) || biomeTint(p.biome?.name);
}

/** Enemy regeneration in % of max health per hour. */
export function regenPctPerHour(p: Pick<Planet, "regenPerSecond" | "maxHealth">): number {
  return p.maxHealth ? (p.regenPerSecond / p.maxHealth) * 360_000 : 0;
}

/** Enemy resistance as the game grades it, from its regeneration in %/h. */
export function resistanceLevel(pctPerHour: number): { label: string; color: string } {
  if (pctPerHour < 1) return { label: "Low", color: "var(--good)" };
  if (pctPerHour < 2.5) return { label: "Average", color: "var(--warn)" };
  return { label: "High", color: "var(--bad)" };
}
