import type { Planet } from "../api/types";

/** Two colours standing in for a biome's look: sky/atmosphere and ground. */
export interface BiomeTint {
  sky: string;
  ground: string;
}

// Matched against the biome name, first hit wins.
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

export function biomeTint(biome: string | undefined): BiomeTint {
  if (!biome) return DEFAULT_TINT;
  return TINTS.find(([re]) => re.test(biome))?.[1] ?? DEFAULT_TINT;
}

/** Enemy regeneration in % of max health per hour. */
export function regenPctPerHour(p: Pick<Planet, "regenPerSecond" | "maxHealth">): number {
  return p.maxHealth ? (p.regenPerSecond / p.maxHealth) * 360_000 : 0;
}
