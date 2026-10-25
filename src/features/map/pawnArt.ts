import type { TacticalAction } from "../../api/types";
import type { PlanetEffect } from "../../domain/effects";

const ART = import.meta.glob<string>("../../assets/pawns/*.png", { eager: true, import: "default", query: "?url" });
const art = (file: string): string | undefined => ART[`../../assets/pawns/${file}.png`];

/** Every hologram by file name; the map masks its scanlines to each one's outline. */
export const PAWN_ART: [string, string][] = Object.entries(ART).map(([path, href]) => [
  path.slice(path.lastIndexOf("/") + 1, -".png".length),
  href,
]);

/** Sub-faction names, as the effect list spells them, to their hologram. */
const UNITS: [RegExp, string][] = [
  [/jet brigade/i, "bot_jetbrigade"],
  [/incineration corps/i, "bot_incincorp"],
  [/cyborgs/i, "bot_cyborgs"],
  [/predator strain/i, "bug_predator"],
  [/spore burst strain/i, "bug_sporeburst"],
  [/rupture strain/i, "bug_rupture"],
  [/dragonroach/i, "bug_dragonroach"],
  [/hive lord/i, "bug_hivelord"],
  [/great host/i, "ill_greathost"],
  [/mindless masses/i, "ill_mindlessmasses"],
  [/appropriators/i, "ill_appropriators"],
  [/vote snatchers/i, "ill_votesnatchers"],
  [/spearhead/i, "ill_spearhead"],
];

/** The hologram drawn over the map for an effect, if it has one; threats and points of interest don't. */
export function pawnArt(e: PlanetEffect): { key: string; href: string } | undefined {
  const key = e.kind === "seaf" ? "seaf" : e.kind === "subfaction" ? UNITS.find(([re]) => re.test(e.name))?.[1] : undefined;
  const href = key && art(key);
  return key && href ? { key, href } : undefined;
}

export type DssState = "active" | "cooldown" | "build" | "default";

/** The station's hologram: lit while an action is deployed, red while one cools down, building while one gathers. */
export function dssState(actions: TacticalAction[], now: number): DssState {
  const live = (a: TacticalAction) => Date.parse(a.statusExpire) > now;
  if (actions.some((a) => a.status === 2 && live(a))) return "active";
  if (actions.some((a) => a.status === 3 && live(a))) return "cooldown";
  if (actions.some((a) => a.costs.some((c) => c.deltaPerSecond > 0 && c.currentValue < c.targetValue))) return "build";
  return "default";
}

export function dssArt(state: DssState): { key: string; href: string } | undefined {
  const href = art(`dss_${state}`);
  return href ? { key: `dss_${state}`, href } : undefined;
}
