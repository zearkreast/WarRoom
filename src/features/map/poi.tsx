import type { PlanetEffect } from "../../domain/effects";
import { factionColor } from "../../domain/factions";
import type { Anomaly } from "../../domain/galaxy";
import { EMBLEM_PATHS } from "../../ui/emblems";

/** Which of the game's map logos a point of interest wears. */
export type PoiKind =
  | "tcs"
  | "factory"
  | "science"
  | "industry"
  | "military"
  | "city"
  | "enemy"
  | "anomaly"
  | "generic";

const KINDS: [RegExp, PoiKind][] = [
  [/black hole|fractured/i, "anomaly"],
  [/terminid control system/i, "tcs"],
  [/factory hub/i, "factory"],
  [/jet brigade factor|communications array/i, "enemy"],
  [/science|xenoentomology|research|laborator/i, "science"],
  [/factory|forge|mine\b|extraction|refinery|e-?711/i, "industry"],
  [/training|pandora|outpost|garrison|base\b/i, "military"],
  [/city|surveillance/i, "city"],
];

export const poiKind = (name: string): PoiKind => KINDS.find(([re]) => re.test(name))?.[1] ?? "generic";

/** Who built a point of interest: an enemy's works wear its colour, everything else is Super Earth's. */
export const poiOrigin = (kind: PoiKind, enemy: string | undefined): string =>
  kind === "enemy" ? (enemy && enemy !== "Humans" ? enemy : "Automaton") : "Humans";

/** Ready-made map marks, used instead of a drawn logo where one exists. */
const MARKS = import.meta.glob<string>("../../assets/poi-marks/*.png", { eager: true, import: "default", query: "?url" });
const markOf = (kind: PoiKind) => MARKS[`../../assets/poi-marks/${kind}.png`];

const ART = import.meta.glob<string>("../../assets/poi/*.{webp,png}", { eager: true, import: "default", query: "?url" });
const art = (file: string) => ART[`../../assets/poi/${file}`];

/** The panel art the game shows for a point of interest. */
const IMAGES: [RegExp, string][] = [
  [/terminid control system/i, "tcs_plus.webp"],
  [/factory hub/i, "factory_hub.webp"],
  [/center of science/i, "center_of_science.webp"],
  [/e-?711/i, "e711.webp"],
  [/training facilit/i, "training.webp"],
  [/civilian surveillance/i, "surveillance.webp"],
  [/deep mantle forge/i, "forge.webp"],
];

/** The hover card's art for a planet: its first point of interest with a picture, else its anomaly's. */
export function poiImage(pois: PlanetEffect[], anomaly?: Anomaly): string | undefined {
  for (const e of pois) {
    const file = IMAGES.find(([re]) => re.test(e.name))?.[1];
    if (file && art(file)) return art(file);
  }
  return anomaly && anomaly !== "scorched" ? art(`${anomaly}.webp`) : undefined;
}

/** A band of a ring between radii r0 and r1, from angle a0 to a1 (degrees, clockwise from +x). */
function band(r0: number, r1: number, a0: number, a1: number): string {
  const p = (r: number, a: number) => {
    const t = (a * Math.PI) / 180;
    return `${(12 + r * Math.cos(t)).toFixed(2)},${(12 + r * Math.sin(t)).toFixed(2)}`;
  };
  const large = a1 - a0 > 180 ? 1 : 0;
  return `M${p(r1, a0)}A${r1},${r1} 0 ${large} 1 ${p(r1, a1)}L${p(r0, a1)}A${r0},${r0} 0 ${large} 0 ${p(r0, a0)}Z`;
}
const dot = (x: number, y: number, r: number) => `M${x - r},${y}a${r},${r} 0 1 0 ${2 * r},0a${r},${r} 0 1 0 ${-2 * r},0Z`;

// The game's map logos, redrawn on a 24-unit grid as filled shapes.
const GLYPHS: Record<Exclude<PoiKind, "enemy" | "factory">, string> = {
  // Two posts joined top and bottom, framing a Terminid claw.
  tcs:
    "M1.5 2.5h5v2h-1.3v15h1.3v2h-5v-2h1.3v-15H1.5ZM17.5 2.5h5v2h-1.3v15h1.3v2h-5v-2h1.3v-15h-1.3Z" +
    "M6.5 5.2h11v1.5h-11ZM6.5 17.3h11v1.5h-11Z" +
    "M8.4 15.6c2.6-.3 5-2.2 5.6-5.4.3-1.5 0-2.7-.6-3.5 2.7 1 4 3.6 3.3 6.4-.7 2.9-3.8 4-8.3 2.5Z" +
    "M9.4 12.9c1.1-.3 2-1.3 2.3-2.6-1 .2-2 1-2.3 2.6Z",
  // A ring round a core, three linked nodes and an outer arc.
  science:
    dot(12, 12, 1.9) +
    band(3.3, 4.9, 0, 359.9) +
    band(6.6, 8, 112, 248) +
    dot(3, 12, 2) +
    "M4.8 11.3h2.5v1.4h-2.5Z" +
    dot(19.2, 4.8, 2) +
    "M14.9 8.1l2.9-2.9 1 1-2.9 2.9Z" +
    dot(19.2, 19.2, 2) +
    "M15.9 14.9l2.9 2.9-1 1-2.9-2.9Z",
  // A mallet, head to the upper right.
  industry: "M2.6 19.6 12.4 9.8l1.8 1.8-9.8 9.8ZM11.2 6.4 15.4 2.2l6.4 6.4-4.2 4.2ZM16.6 3.4l2-2 2 2-2 2Z",
  // A banner over two chevrons.
  military: "M4 2.5h16v6.2l-8 3.8-8-3.8ZM4 11.2l8 3.8 8-3.8v3.3l-8 3.8-8-3.8ZM4 16.8l8 3.8 8-3.8v3.1l-8 3.8-8-3.8Z",
  // Columns rising to a dome.
  city: "M2.6 21.5 3.6 11l2.6-1.2v11.7ZM7.4 21.5V8l3-1.6v15.1ZM11.4 21.5V5.6l1.2-.4 1.2.4v15.9ZM14.8 21.5V6.4l3 1.6v13.5ZM19 21.5V9.8l2.6 1.2 1 10.5Z",
  // A ring struck through, for black holes and broken worlds.
  anomaly: band(6, 7.8, 0, 359.9) + "M2.5 11h19v2h-19Z",
  // A plain marker for points the map has no logo for.
  generic: band(5.2, 7.4, 0, 359.9) + dot(12, 12, 2.4),
};

/**
 * A point of interest's logo, centred on (0, 0) and `size` map units across, in its builder's colour
 * (`faction`). Drawn logos get a soft halo from a wide translucent stroke rather than a blur filter.
 */
export function PoiGlyph({ kind, faction, size }: { kind: PoiKind; faction: string; size: number }) {
  const mark = markOf(kind);
  if (mark) {
    // The marks carry their own outline and glow, so they're drawn a little larger.
    const s = size * 1.25;
    return <image className={`poi-mark ${kind}`} href={mark} x={-s / 2} y={-s / 2} width={s} height={s} />;
  }
  const d = kind === "enemy" ? (EMBLEM_PATHS[faction] ?? EMBLEM_PATHS.Automaton) : GLYPHS[kind === "factory" ? "industry" : kind];
  const transform = `translate(${-size / 2},${-size / 2}) scale(${size / 24})`;
  const fillRule = kind === "enemy" ? "evenodd" : "nonzero";
  return (
    <g className={`poi-glyph ${kind}`} style={{ color: factionColor(faction) }}>
      <path className="poi-halo" d={d} transform={transform} fillRule={fillRule} />
      <path className="poi-shape" d={d} transform={transform} fillRule={fillRule} />
    </g>
  );
}
