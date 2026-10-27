import { useId, type ComponentType, type CSSProperties } from "react";
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
  const box = svg
    .match(/viewBox="([^"]+)"/)![1]
    .split(/\s+/)
    .map(Number) as Art["box"];
  const body = svg
    .replace(/^[\s\S]*?<svg[^>]*>/, "")
    .replace(/<\/svg>\s*$/, "")
    .replace(/<desc>[\s\S]*?<\/desc>/, "");
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
        <rect
          x={x}
          y={y}
          width={w}
          height={h}
          style={{ fill: "currentColor", stroke: "none" }}
          mask={`url(#${maskId})`}
        />
      </g>
    );
  return <g className={className} style={look} transform={transform} dangerouslySetInnerHTML={{ __html: a.body! }} />;
}

/** The game paints the Super Earth emblem cream over gold: the globe pale, the arcs beneath it gold. */
export function SealGradient({ id }: { id: string }) {
  return (
    <linearGradient id={id} x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stopColor="#f4e4c1" />
      <stop offset="58%" stopColor="#ead09a" />
      <stop offset="100%" stopColor="#d9a521" />
    </linearGradient>
  );
}

/** The Super Earth emblem in the game's cream and gold, for use inside another SVG with a SealGradient. */
export function SealArt({ size, gradient }: { size: number; gradient: string }) {
  return (
    <EmblemArt
      art="Humans"
      size={size}
      className="emblem-seal"
      style={{ "--seal": `url(#${gradient})` } as CSSProperties}
    />
  );
}

/** The Super Earth emblem as the game shows it, at least 36px so its globe and arcs stay apart. */
export function SuperEarthSeal({ size = 40 }: { size?: number }) {
  const id = useId();
  return (
    <svg className="emblem" width={size} height={size} viewBox="-12 -12 24 24" aria-hidden>
      <defs>
        <SealGradient id={id} />
      </defs>
      <SealArt size={24} gradient={id} />
    </svg>
  );
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

/** Anything drawn like a Lucide icon. */
export type IconLike = ComponentType<{ size?: number | string; strokeWidth?: number | string; className?: string }>;

/** The game's own DSS glyph, drawn like a Lucide icon in the current colour. */
export function DssIcon({ size = 24, className, color }: { size?: number | string; className?: string; color?: string }) {
  return (
    <svg className={`emblem ${className ?? ""}`} width={size} height={size} viewBox="-12 -12 24 24" style={{ color }} aria-hidden>
      <EmblemArt art="dss" size={24} />
    </svg>
  );
}

/** The game's Heavy SEAF Presence emblem, drawn like a Lucide icon in the current colour. */
export function SeafEmblem({ size = 24, className, color }: { size?: number | string; className?: string; color?: string }) {
  return (
    <svg className={`emblem ${className ?? ""}`} width={size} height={size} viewBox="-12 -12 24 24" style={{ color }} aria-hidden>
      <EmblemArt art="seaf" size={24} />
    </svg>
  );
}

/** A space station's emblem: the DSS glyph, or the Automaton cog and star the enemy station wore. */
export function StationEmblem({ station, size = 16, color }: { station: "dss" | "css"; size?: number; color?: string }) {
  return (
    <svg className="emblem" width={size} height={size} viewBox="-12 -12 24 24" aria-hidden>
      <EmblemArt art={station === "dss" ? "dss" : "automaton-station"} size={24} color={color} />
    </svg>
  );
}

/** The Hellpad's mark, the four stratagem arrows a Helldiver keys in on it; drawn like a Lucide icon. */
export function HellpadMark({ size = 24, className }: { size?: number | string; className?: string }) {
  // Heavier lines at dock size, where the mast's would thin out to a hairline.
  const small = Number(size) < 40;
  return (
    <svg className={className} width={size} height={size} viewBox="0 0 64 64" aria-hidden>
      <path d="M14 4h36l10 10v36L50 60H14L4 50V14Z" fill="none" stroke="currentColor" strokeWidth={small ? 5 : 2.5} />
      <path d="M18 9h28l9 9v28l-9 9H18l-9-9V18Z" fill="currentColor" opacity="0.08" />
      <g fill="currentColor">
        <path d="M32 12l8 9h-16Z" />
        <path d="M52 32l-9 8v-16Z" />
        <path d="M32 52l-8-9h16Z" />
        <path d="M12 32l9-8v16Z" />
      </g>
      <rect x="28" y="28" width="8" height="8" fill="none" stroke="currentColor" strokeWidth={small ? 4 : 2} />
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
