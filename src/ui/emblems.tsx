import { useId, type CSSProperties } from "react";
import { factionColor } from "../domain/factions";
import invasionFleet from "../assets/emblems/invasion-fleet.png";

// The game's faction and sub-faction emblems, as traced on the Helldivers Wiki; every shape fills with `currentColor`.
const FILES = import.meta.glob<string>("../assets/emblems/*.svg", { query: "?raw", import: "default", eager: true });

interface Art {
  box: [number, number, number, number];
  /** Inner SVG markup, or a white-on-clear picture used as a mask. */
  body?: string;
  mask?: string;
}

const ART: Record<string, Art> = { "invasion-fleet": { box: [0, 0, 128, 128], mask: invasionFleet } };
for (const [path, svg] of Object.entries(FILES)) {
  const slug = path.split("/").pop()!.replace(".svg", "");
  const box = svg.match(/viewBox="([^"]+)"/)![1].split(/\s+/).map(Number) as Art["box"];
  const body = svg.replace(/^[\s\S]*?<svg[^>]*>/, "").replace(/<\/svg>\s*$/, "").replace(/<desc>[\s\S]*?<\/desc>/, "");
  ART[slug] = { box, body };
}

const FACTION_ART: Record<string, string> = {
  Humans: "super-earth",
  Terminids: "terminids",
  Automaton: "automaton",
  Illuminate: "illuminate",
};

// Effect names as the game writes them. Hive Lords and Dragonroaches are units with no emblem of their own.
const SUBFACTION_ART: [RegExp, string][] = [
  [/predator strain/i, "predator-strain"],
  [/spore burst strain/i, "spore-burst-strain"],
  [/rupture strain/i, "rupture-strain"],
  [/jet brigade/i, "jet-brigade"],
  [/incineration corps/i, "incineration-corps"],
  [/cyborg/i, "cyborgs"],
  [/appropriators/i, "appropriators"],
  [/vote snatchers/i, "vote-snatchers"],
  [/mindless masses/i, "mindless-masses"],
  [/invasion fleet|great host/i, "invasion-fleet"],
];

/** The emblem for a sub-faction effect, by its name; null for units that have none. */
export function subfactionArt(name: string): string | null {
  return SUBFACTION_ART.find(([re]) => re.test(name))?.[1] ?? null;
}

export const hasEmblem = (faction: string) => faction in FACTION_ART;

/**
 * An emblem as SVG content, centred on (0, 0) and `size` units across, for use inside another SVG.
 * `art` is a faction name or a sub-faction art key; it takes the colour of `color`, else the faction's.
 */
export function EmblemArt({
  art,
  size,
  color,
  className,
  style,
}: {
  art: string;
  size: number;
  color?: string;
  className?: string;
  style?: CSSProperties;
}) {
  const maskId = useId();
  const a = ART[FACTION_ART[art] ?? art];
  if (!a) return null;
  const [x, y, w, h] = a.box;
  const s = size / Math.max(w, h);
  const transform = `translate(${-(w * s) / 2},${-(h * s) / 2}) scale(${s}) translate(${-x},${-y})`;
  const tone = color ?? (art in FACTION_ART ? factionColor(art) : undefined);
  // --u: a 24-grid unit in the emblem's own units, for strokes drawn over it.
  const look = { ...style, ...(tone ? { color: tone } : {}), "--u": Math.max(w, h) / 24 } as CSSProperties;
  if (a.mask)
    return (
      <g className={className} style={look} transform={transform}>
        <mask id={maskId}>
          <image href={a.mask} x={x} y={y} width={w} height={h} />
        </mask>
        <rect x={x} y={y} width={w} height={h} style={{ fill: "currentColor", stroke: "none" }} mask={`url(#${maskId})`} />
      </g>
    );
  return <g className={className} style={look} transform={transform} dangerouslySetInnerHTML={{ __html: a.body! }} />;
}

export function FactionEmblem({ faction, size = 16, color }: { faction: string; size?: number; color?: string }) {
  if (!hasEmblem(faction)) return null;
  return (
    <svg className="emblem" width={size} height={size} viewBox="-12 -12 24 24" aria-hidden>
      <EmblemArt art={faction} size={24} color={color} />
    </svg>
  );
}

/** A sub-faction's emblem by its effect name, or its faction's when the unit has none. */
export function SubfactionEmblem({
  name,
  faction,
  size = 16,
  color,
}: {
  name: string;
  faction: string;
  size?: number;
  color?: string;
}) {
  return (
    <svg className="emblem" width={size} height={size} viewBox="-12 -12 24 24" aria-hidden>
      <EmblemArt art={subfactionArt(name) ?? faction} size={24} color={color ?? factionColor(faction)} />
    </svg>
  );
}

/** App mark: a shield with a star, drawn in the CTA colour. */
export function AppMark({ size = 26 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 32 32" aria-hidden>
      <path d="M16 2 29 8v8c0 7-5.5 12-13 14C8.5 28 3 23 3 16V8z" fill="none" stroke="var(--cta)" strokeWidth="2" />
      <path d="m16 9 2.2 4.6 5 .6-3.7 3.4 1 5-4.5-2.5-4.5 2.5 1-5-3.7-3.4 5-.6z" fill="var(--cta)" />
    </svg>
  );
}
