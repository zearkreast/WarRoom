import { select } from "d3-selection";
import { zoom, zoomIdentity, type ZoomBehavior, type ZoomTransform } from "d3-zoom";
import {
  ArrowUpRight,
  Compass,
  Landmark as PoiIcon,
  Layers,
  MousePointerClick,
  Rotate3d,
  Settings,
  Shield,
  TriangleAlert,
  ZoomIn,
  ZoomOut,
} from "lucide-react";
import {
  memo,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type CSSProperties,
  type PointerEvent as ReactPointerEvent,
} from "react";
import { useCampaigns, useEffects, useFreshPlanets, useSectorGeometry, useSpaceStations } from "../../api/queries";
import type { Planet } from "../../api/types";
import { biomeTint, planetTint, regenPctPerHour } from "../../domain/biomes";
import { hasPawn, type PlanetEffect } from "../../domain/effects";
import { FACTIONS, factionColor } from "../../domain/factions";
import { fmtPct } from "../../domain/format";
import {
  anomalyOf,
  GALAXY_SCALE,
  LANDMARKS,
  project,
  RING_RADII,
  sectorAt,
  supplyLines,
  type Anomaly,
  type Landmark,
  type PolarSectors,
} from "../../domain/galaxy";
import { factionMarks } from "../../domain/factionMarks";
import { byIndex, ownershipKey } from "../../domain/planets";
import { buildRegions } from "../../domain/regions";
import { liberationPercent, trackedPercent } from "../../domain/progress";
import { useSettings, useUi, type MapOverlay, type MapPlanets, type MapQuality } from "../../state";
import { EMBLEM_PATHS } from "../../ui/emblems";
import { HudButton, Loading, Meter, Stepper } from "../../ui/primitives";
import { useMajorOrderPlanets } from "../planets/useFronts";
import { GloomLayer, SectorLayer } from "./layers";
import { OVERLAY_OPTIONS, PLANET_OPTIONS, QUALITY_OPTIONS } from "./mapOptions";
import { dssArt, dssState, PAWN_ART, pawnArt, type DssState } from "./pawnArt";
import "./map.css";

const EXTENT = GALAXY_SCALE * 1.1;
const STAR_COUNT: Record<MapQuality, number> = { max: 4200, normal: 2600, min: 1600 };
/** Highest device pixel ratio the starfield canvas is drawn at; phones report up to 3 or 4. */
const STAR_DPR: Record<MapQuality, number> = { max: 4, normal: 2, min: 1.5 };
const OVERLAYS: MapOverlay[] = ["regions", "sectors", "galaxy"];

type Pt = [number, number];

/** Map rotation (radians, clockwise on screen) and tilt (radians, 0 = straight down). */
interface Camera {
  rot: number;
  tilt: number;
}

const MAX_TILT = (55 * Math.PI) / 180;
const TILT_PRESET = (45 * Math.PI) / 180;
const ROT_STEP = Math.PI / 12;
const TILT_STEP = Math.PI / 18;
const DRAG_ROT = 0.006;
const DRAG_TILT = 0.005;
const TOUCH_TILT = 0.008;
/** Finger travel (px) before a touch counts as a drag rather than a tap. */
const TOUCH_SLOP = 6;
/** Two fingers must twist this far (radians) before the map turns, so a pinch doesn't wobble it. */
const TWIST_START = 0.2;
const SCALE_EXTENT: [number, number] = [0.5, 14];

const turn = ([x, y]: Pt, a: number): Pt => [x * Math.cos(a) - y * Math.sin(a), x * Math.sin(a) + y * Math.cos(a)];
/** Map units to the zoom layer's own units: rotated, then squashed by the tilt. */
const toLayer = (p: Pt, cam: Camera): Pt => {
  const [x, y] = turn(p, cam.rot);
  return [x, y * Math.cos(cam.tilt)];
};
const fromLayer = ([x, y]: Pt, cam: Camera): Pt => turn([x, y / Math.cos(cam.tilt)], -cam.rot);

type Affine = [number, number, number, number, number, number];

/** Map units to screen pixels, in CSS `matrix()` order. */
function screenMatrix(t: ZoomTransform, cam: Camera): Affine {
  const [cr, sr, ct] = [Math.cos(cam.rot), Math.sin(cam.rot), Math.cos(cam.tilt)];
  return [t.k * cr, t.k * ct * sr, -t.k * sr, t.k * ct * cr, t.x, t.y];
}

function invert([a, b, c, d, e, f]: Affine): Affine {
  const det = a * d - b * c;
  return [d / det, -b / det, -c / det, a / det, (c * f - d * e) / det, (b * e - a * f) / det];
}

function multiply([a1, b1, c1, d1, e1, f1]: Affine, [a2, b2, c2, d2, e2, f2]: Affine): Affine {
  return [
    a1 * a2 + c1 * b2,
    b1 * a2 + d1 * b2,
    a1 * c2 + c1 * d2,
    b1 * c2 + d1 * d2,
    a1 * e2 + c1 * f2 + e1,
    b1 * e2 + d1 * f2 + f1,
  ];
}

const angleOf = (a: Pt, b: Pt) => Math.atan2(b[1] - a[1], b[0] - a[0]);
const distance = (a: Pt, b: Pt) => Math.hypot(b[0] - a[0], b[1] - a[1]);
const midpoint = (a: Pt, b: Pt): Pt => [(a[0] + b[0]) / 2, (a[1] + b[1]) / 2];

/** Deterministic background stars so the field doesn't reshuffle between renders. */
function backgroundStars(n: number) {
  let seed = 7;
  const rnd = () => (seed = (seed * 16807) % 2147483647) / 2147483647;
  return Array.from({ length: n }, () => {
    const a = rnd() * Math.PI * 2;
    // Denser towards the core.
    const r = Math.pow(rnd(), 0.8) * EXTENT * 1.35;
    const hue = rnd();
    return {
      x: Math.cos(a) * r,
      y: Math.sin(a) * r,
      s: rnd() < 0.06 ? 1.6 : rnd() * 0.9 + 0.4,
      o: rnd() * 0.55 + 0.15,
      c: hue < 0.12 ? "#9fd4ff" : hue < 0.18 ? "#ffd9a0" : "#ffffff",
    };
  });
}

type Star = ReturnType<typeof backgroundStars>[number];

function drawStars(
  canvas: HTMLCanvasElement,
  stars: Star[],
  t: ZoomTransform,
  cam: Camera,
  rings: boolean,
  maxDpr: number,
) {
  const dpr = Math.min(window.devicePixelRatio || 1, maxDpr);
  const { width, height } = canvas.getBoundingClientRect();
  if (canvas.width !== Math.round(width * dpr)) canvas.width = Math.round(width * dpr);
  if (canvas.height !== Math.round(height * dpr)) canvas.height = Math.round(height * dpr);
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, width, height);
  const squash = Math.cos(cam.tilt);

  // Core glow around Super Earth, flattened with the disc.
  const [cx, cy] = [t.x, t.y];
  ctx.save();
  ctx.translate(cx, cy);
  ctx.scale(1, squash);
  const glow = ctx.createRadialGradient(0, 0, 0, 0, 0, 140 * t.k);
  glow.addColorStop(0, "rgba(120, 200, 255, 0.28)");
  glow.addColorStop(0.35, "rgba(70, 164, 255, 0.08)");
  glow.addColorStop(1, "rgba(70, 164, 255, 0)");
  ctx.fillStyle = glow;
  ctx.fillRect(-width * 2, (-height * 2) / squash, width * 4, (height * 4) / squash);
  ctx.restore();

  // Stars drift a little slower than the map, so zooming has depth.
  const k = Math.pow(t.k, 0.85);
  const [cos, sin] = [Math.cos(cam.rot), Math.sin(cam.rot)];
  for (const s of stars) {
    const x = (s.x * cos - s.y * sin) * k + t.x;
    const y = (s.x * sin + s.y * cos) * k + t.y;
    if (x < -2 || y < -2 || x > width + 2 || y > height + 2) continue;
    ctx.globalAlpha = s.o;
    ctx.fillStyle = s.c;
    ctx.fillRect(x, y, s.s, s.s);
  }
  ctx.globalAlpha = 1;

  // The game's sector rings, so planets read against the same bands; the Sectors overlay draws its own borders.
  if (!rings) return;
  ctx.strokeStyle = "rgba(120, 170, 230, 0.09)";
  ctx.lineWidth = 1;
  for (const r of RING_RADII) {
    const R = r * GALAXY_SCALE * t.k;
    ctx.beginPath();
    ctx.ellipse(cx, cy, R, R * squash, 0, 0, Math.PI * 2);
    ctx.stroke();
  }
}

/** Hatch spacing in screen pixels; the pattern is counter-scaled on zoom to keep it. */
const HATCH = 7;

function scaleHatches(svg: SVGSVGElement, k: number) {
  for (const p of svg.querySelectorAll("pattern.hatch"))
    p.setAttribute("patternTransform", `rotate(45) scale(${(1 / k).toFixed(4)})`);
}

const Regions = memo(function Regions({ planets }: { planets: Planet[] }) {
  const owners = ownershipKey(planets);
  // eslint-disable-next-line react-hooks/exhaustive-deps
  const regions = useMemo(() => buildRegions(planets), [owners]);
  return (
    <g className="regions">
      {regions.map((r) => {
        const c = factionColor(r.owner);
        return (
          <g key={r.name}>
            <path d={r.path} fill={c} fillOpacity={0.05} stroke="none" />
            <path d={r.path} className="region-glow" stroke={c} />
            <path d={r.path} className="region-line" stroke={c} />
          </g>
        );
      })}
    </g>
  );
});

/** Each enemy's see-through emblem on its largest connected holding, as the overlay draws it; it fades out once zoomed in. */
const FactionMarks = memo(function FactionMarks(props: {
  planets: Planet[];
  overlay: MapOverlay;
  geo: PolarSectors | undefined;
}) {
  const owners = ownershipKey(props.planets);
  // eslint-disable-next-line react-hooks/exhaustive-deps
  const marks = useMemo(
    () => factionMarks(props.planets, props.overlay, props.geo),
    [owners, props.overlay, props.geo],
  );
  return (
    <g className="faction-marks" aria-hidden>
      {marks.map((m) => (
        <g key={m.faction} transform={`translate(${m.x.toFixed(1)},${m.y.toFixed(1)})`}>
          <g className="upright">
            <path
              d={EMBLEM_PATHS[m.faction]}
              transform="translate(-36,-36) scale(3)"
              fill={factionColor(m.faction)}
              color={factionColor(m.faction)}
              fillRule="evenodd"
            />
          </g>
        </g>
      ))}
    </g>
  );
});

type LaneMode = "dim" | "bright" | "front";

/** All lanes as one path: hundreds of separate lines cost far more to restyle and paint. */
const Lanes = memo(function Lanes({ planets, mode }: { planets: Planet[]; mode: LaneMode }) {
  const d = useMemo(() => {
    const lookup = byIndex(planets);
    let d = "";
    for (const [a, b] of supplyLines(planets)) {
      const pa = lookup.get(a);
      const pb = lookup.get(b);
      if (!pa || !pb) continue;
      const [x1, y1] = project(pa.position);
      const [x2, y2] = project(pb.position);
      d += `M${x1},${y1}L${x2},${y2}`;
    }
    return d;
  }, [planets]);
  return (
    <g className={`lanes ${mode}`}>
      <path d={d} />
    </g>
  );
});

function AttackLines({ planets, active }: { planets: Planet[]; active: Set<number> }) {
  const lookup = byIndex(planets);
  return (
    <g className="attacks">
      {planets.flatMap((p) =>
        p.attacking
          .filter((t) => active.has(t))
          .map((t) => {
            const target = lookup.get(t);
            if (!target) return null;
            const [x1, y1] = project(p.position);
            const [x2, y2] = project(target.position);
            return (
              <line key={`${p.index}>${t}`} x1={x1} y1={y1} x2={x2} y2={y2} stroke={factionColor(p.currentOwner)} />
            );
          }),
      )}
    </g>
  );
}

const HaloDefs = memo(function HaloDefs() {
  return (
    <defs>
      {(Object.keys(FACTIONS) as (keyof typeof FACTIONS)[]).map((f) => (
        <radialGradient key={f} id={`halo-${f}`}>
          <stop offset="0%" stopColor="#fff" stopOpacity={0.9} />
          <stop offset="25%" stopColor={FACTIONS[f].color} stopOpacity={0.55} />
          <stop offset="100%" stopColor={FACTIONS[f].color} stopOpacity={0} />
        </radialGradient>
      ))}
      <radialGradient id="sphere-shade" cx="35%" cy="30%" r="75%">
        <stop offset="0%" stopColor="#fff" stopOpacity={0.55} />
        <stop offset="35%" stopColor="#fff" stopOpacity={0} />
        <stop offset="75%" stopColor="#000" stopOpacity={0.3} />
        <stop offset="100%" stopColor="#000" stopOpacity={0.7} />
      </radialGradient>
      <radialGradient id="wormhole">
        <stop offset="0%" stopColor="#05020a" stopOpacity={1} />
        <stop offset="38%" stopColor="#1a0828" stopOpacity={1} />
        <stop offset="58%" stopColor="#b674d2" stopOpacity={0.95} />
        <stop offset="72%" stopColor="#7d3fb0" stopOpacity={0.45} />
        <stop offset="100%" stopColor="#7d3fb0" stopOpacity={0} />
      </radialGradient>
      <radialGradient id="halo-scorched">
        <stop offset="0%" stopColor="#ffd2a1" stopOpacity={0.9} />
        <stop offset="30%" stopColor="#ff7a2e" stopOpacity={0.5} />
        <stop offset="100%" stopColor="#ff7a2e" stopOpacity={0} />
      </radialGradient>
      <radialGradient id="fracture-glow">
        <stop offset="0%" stopColor="#ff7a4a" stopOpacity={0.55} />
        <stop offset="60%" stopColor="#c83a2a" stopOpacity={0.12} />
        <stop offset="100%" stopColor="#c83a2a" stopOpacity={0} />
      </radialGradient>
      <radialGradient id="blackhole">
        <stop offset="0%" stopColor="#000" stopOpacity={1} />
        <stop offset="46%" stopColor="#000" stopOpacity={1} />
        <stop offset="54%" stopColor="#dfe9f5" stopOpacity={0.75} />
        <stop offset="66%" stopColor="#7d93ad" stopOpacity={0.25} />
        <stop offset="100%" stopColor="#7d93ad" stopOpacity={0} />
      </radialGradient>
      {/* Hologram light: the column fades as it rises, the pool on the table fades from its centre. */}
      <linearGradient id="holo-fade-ramp" x1={0} y1={1} x2={0} y2={0}>
        <stop offset="0%" stopColor="#fff" stopOpacity={0.85} />
        <stop offset="100%" stopColor="#fff" stopOpacity={0.05} />
      </linearGradient>
      <mask id="holo-fade" maskContentUnits="objectBoundingBox">
        <rect width={1} height={1} fill="url(#holo-fade-ramp)" />
      </mask>
      <radialGradient id="holo-pool-ramp">
        <stop offset="0%" stopColor="#fff" stopOpacity={0.7} />
        <stop offset="45%" stopColor="#fff" stopOpacity={0.25} />
        <stop offset="100%" stopColor="#fff" stopOpacity={0} />
      </radialGradient>
      <mask id="holo-pool" maskContentUnits="objectBoundingBox">
        <rect width={1} height={1} fill="url(#holo-pool-ramp)" />
      </mask>
      <linearGradient id="holo-mirror-ramp" x1={0} y1={0} x2={0} y2={1}>
        <stop offset="0%" stopColor="#fff" stopOpacity={1} />
        <stop offset="70%" stopColor="#fff" stopOpacity={0} />
      </linearGradient>
      <mask id="holo-mirror" maskContentUnits="objectBoundingBox">
        <rect width={1} height={1} fill="url(#holo-mirror-ramp)" />
      </mask>
      <pattern id="holo-scan" width={HOLO_SIZE} height={0.9} patternUnits="userSpaceOnUse">
        <rect width={HOLO_SIZE} height={0.35} fill="#fff" fillOpacity={0.28} />
      </pattern>
      <linearGradient id="holo-sweep-band" x1={0} y1={0} x2={0} y2={1}>
        <stop offset="0%" stopColor="#fff" stopOpacity={0} />
        <stop offset="50%" stopColor="#fff" stopOpacity={0.75} />
        <stop offset="100%" stopColor="#fff" stopOpacity={0} />
      </linearGradient>
      {/* Each hologram's own outline, for its scanlines and sweep; every pawn draws its unit at the same spot. */}
      {PAWN_ART.map(([key, href]) => (
        <mask
          key={key}
          id={`holo-mask-${key}`}
          maskUnits="userSpaceOnUse"
          x={-HOLO_SIZE / 2}
          y={-HOLO_SIZE}
          width={HOLO_SIZE}
          height={HOLO_SIZE}
          style={{ maskType: "alpha" }}
        >
          <image href={href} x={-HOLO_SIZE / 2} y={-HOLO_SIZE} width={HOLO_SIZE} height={HOLO_SIZE} />
        </mask>
      ))}
    </defs>
  );
});

/** Planets the game draws as something other than a whole world: wormhole, black hole, debris, burnt rock. */
function AnomalyMark({ kind }: { kind: Anomaly }) {
  if (kind === "wormhole")
    return (
      <g className="anomaly wormhole">
        <circle r={9} fill="url(#wormhole)" />
        <circle r={5.6} className="swirl" />
      </g>
    );
  if (kind === "blackhole")
    return (
      <g className="anomaly blackhole">
        <circle r={8} fill="url(#blackhole)" />
        <circle r={3.6} className="horizon" />
      </g>
    );
  // What the Meridian singularity left: a hot core with the planet's crust drifting apart around it.
  return (
    <g className="anomaly fractured">
      <circle r={6.5} fill="url(#fracture-glow)" />
      <path className="rock" d="M-1.9,-2.6L0.4,-3.3L2.1,-1.9L1.6,0.2L-0.6,0.9L-2.4,-0.6Z" />
      <path className="rock" d="M2.6,0.9L4.6,1.3L4.9,3.3L3.1,4.2L1.8,2.7Z" />
      <path className="rock" d="M-4.6,1.1L-2.3,1.5L-2.0,3.6L-3.9,4.4L-5.2,2.9Z" />
      <path className="rock" d="M3.4,-4.6L4.9,-4.2L4.6,-2.8L3.2,-3.0Z" />
      <path className="rock small" d="M-4.6,-3.6L-3.5,-3.9L-3.3,-2.9L-4.4,-2.7Z" />
      <circle className="rock small" cx={0.2} cy={4.4} r={0.6} />
      <circle className="ember" r={0.9} cx={0.1} cy={1.6} />
    </g>
  );
}

const PAWN_ICON = { threat: TriangleAlert, seaf: Shield, poi: PoiIcon } as const;

const DSS_TONE: Record<DssState, string> = {
  active: "var(--cta)",
  cooldown: "var(--bad)",
  build: "var(--f-humans)",
  default: "var(--f-humans)",
};
/** A unit's light follows its own faction, whoever holds the planet it is on. */
const UNIT_TONE: Record<string, string> = {
  bug: factionColor("Terminids"),
  bot: factionColor("Automaton"),
  ill: factionColor("Illuminate"),
  seaf: factionColor("Humans"),
};

/**
 * Hologram size and spacing in marker units. They stand in two staggered ranks on the table, the
 * first clear of the planet's glow; the back rank sits further away, so it rises up the screen less
 * the more the map is tilted.
 */
const HOLO_SIZE = 22;
const HOLO_X = 15;
const HOLO_STEP = 15;

/**
 * A unit as a hologram on the war table: a projector ring lying on the map, which turns and
 * foreshortens with the camera, a column of light, and the unit on it. Seen from straight above the
 * unit sits over its projector; tilting the map lifts it off and shows its reflection in the table.
 */
function HoloPawn(props: { art: { key: string; href: string }; tone: string; x: number; back: boolean; delay: number }) {
  const { art } = props;
  const s = HOLO_SIZE;
  const unit = <image href={art.href} x={-s / 2} y={-s} width={s} height={s} />;
  return (
    <g transform={`translate(${props.x},3)`}>
      <g
        className={`holo ${props.back ? "back" : ""}`}
        style={{ color: props.tone, "--delay": `${-props.delay}s` } as CSSProperties}
      >
        <g className="holo-floor">
          <circle r={8} className="holo-pool" mask="url(#holo-pool)" />
          <circle r={5.4} className="holo-ring" />
          <circle r={4.2} className="holo-ticks" />
          <circle r={1.3} className="holo-emitter" />
        </g>
        <g className="holo-mirror" mask="url(#holo-mirror)">
          <g className="holo-flip">{unit}</g>
        </g>
        <path className="holo-beam" d="M-1.6,0L-6,-1H6L1.6,0Z" mask="url(#holo-fade)" />
        <g className="holo-lift">
          <g className="holo-unit">
            {unit}
            <g className="holo-light" mask={`url(#holo-mask-${art.key})`}>
              <rect x={-s / 2} y={-s} width={s} height={s} fill="url(#holo-scan)" />
              <rect className="holo-sweep" x={-s / 2} y={-4} width={s} height={4} fill="url(#holo-sweep-band)" />
            </g>
          </g>
        </g>
      </g>
    </g>
  );
}

/**
 * The planet's sub-factions, SEAF presence and the Democracy Space Station as holograms beside it,
 * as the game's war table shows them; threats and points of interest, which have none, follow as tags.
 */
function Pawns({ effects, faction, dss, seed }: { effects: PlanetEffect[]; faction: string; dss?: DssState; seed: number }) {
  const holos: { art: { key: string; href: string }; tone: string }[] = [];
  const station = dss && dssArt(dss);
  if (dss && station) holos.push({ art: station, tone: DSS_TONE[dss] });
  const tags: PlanetEffect[] = [];
  for (const e of effects) {
    const art = pawnArt(e);
    if (art) holos.push({ art, tone: UNIT_TONE[art.key.split("_")[0]] });
    else tags.push(e);
  }
  const placed = holos.map((h, i) => ({ ...h, x: HOLO_X + i * HOLO_STEP, back: i % 2 === 1, delay: (seed * 0.73 + i * 1.3) % 6 }));
  return (
    <>
      {holos.length > 0 && (
        <g className={`holos ${station ? "dss" : ""}`}>
          {/* The back rank first, so the front one stands before it. */}
          {[...placed.filter((h) => h.back), ...placed.filter((h) => !h.back)].map((h) => (
            <HoloPawn key={h.art.key} art={h.art} tone={h.tone} x={h.x} back={h.back} delay={h.delay} />
          ))}
        </g>
      )}
      {tags.length > 0 && (
        <PawnTags effects={tags} faction={faction} x={holos.length ? HOLO_X + holos.length * HOLO_STEP + 4 : 7} />
      )}
    </>
  );
}

/** Threats and points of interest, as tags in a row. */
function PawnTags({ effects, faction, x: x0 }: { effects: PlanetEffect[]; faction: string; x: number }) {
  return (
    <g className="pawns" transform={`translate(${x0},1)`}>
      {effects.map((e, i) => {
        const x = i * 6.4;
        const tone = e.kind === "subfaction" ? factionColor(faction) : e.kind === "seaf" ? "var(--f-humans)" : "var(--cta)";
        return (
          <g key={e.id} className={`pawn ${e.kind}`} transform={`translate(${x},0)`} style={{ color: tone }}>
            <rect x={-2.8} y={-2.8} width={5.6} height={5.6} />
            {e.kind === "subfaction" ? (
              <path d={EMBLEM_PATHS[faction]} transform="translate(-2.1,-2.1) scale(0.175)" fillRule="evenodd" />
            ) : (
              (() => {
                const Icon = PAWN_ICON[e.kind as keyof typeof PAWN_ICON];
                return <Icon size={4} x={-2} y={-2} strokeWidth={2.6} />;
              })()
            )}
          </g>
        );
      })}
    </g>
  );
}

/** Biome colour pulled towards grey, so the spheres read as worlds rather than markers. */
function sphereColor(hex: string): string {
  const mix = (i: number) =>
    Math.round(parseInt(hex.slice(i, i + 2), 16) * 0.55 + parseInt("8a93a0".slice(i - 1, i + 1), 16) * 0.45)
      .toString(16)
      .padStart(2, "0");
  return `#${mix(1)}${mix(3)}${mix(5)}`;
}

/** Square tag over an active planet in the fighting faction's colour: crosshair to liberate, shield to defend. */
function ActiveTag({ defense, mo, color }: { defense: boolean; mo: boolean; color: string }) {
  return (
    <g className={`act-tag ${mo ? "mo" : ""}`} transform="translate(0,-10.5)" style={{ color }}>
      <path className="act-stem" d="M0,3.4V5.6" />
      <rect x={-3.4} y={-3.4} width={6.8} height={6.8} />
      {defense ? (
        <path d="M0,-2.2 2,-1.4V0.2C2,1.3 1.1,2 0,2.4-1.1,2-2,1.3-2,0.2V-1.4Z" />
      ) : (
        <>
          <circle r={1.3} />
          <path d="M-2.3,0H-1.3M1.3,0H2.3M0,-2.3V-1.3M0,1.3V2.3" />
        </>
      )}
    </g>
  );
}

function PinMark() {
  return (
    <g className="pin-mark" transform="translate(7,-7)">
      <path d="M0,4.5C0,4.5 -3.4,1.2 -3.4,-1.1A3.4,3.4 0 0 1 3.4,-1.1C3.4,1.2 0,4.5 0,4.5Z" />
      <circle cy={-1.1} r={1.2} />
    </g>
  );
}

function PlanetBody(props: {
  style: MapPlanets;
  faction: string;
  color: string;
  fill: string;
  size: number;
  anomaly?: Anomaly;
  active?: boolean;
  defense?: boolean;
  mo?: boolean;
  enemy?: boolean;
  pinned?: boolean;
  pawns?: PlanetEffect[];
  dss?: DssState;
  seed?: number;
}) {
  // Razed worlds keep a planet's size and shape; only their colour and glow turn to embers.
  const scorched = props.anomaly === "scorched";
  return (
    <g className="pbody">
      <circle r={10} className="hit" />
      {props.anomaly && props.anomaly !== "scorched" ? (
        <AnomalyMark kind={props.anomaly} />
      ) : props.style === "spheres" ? (
        <>
          <circle
            r={4.4 * props.size}
            className={`sphere ${scorched ? "scorched" : ""}`}
            fill={scorched ? "#8a3a16" : props.fill}
          />
          <circle r={4.4 * props.size} fill="url(#sphere-shade)" className="shade" />
          {props.enemy && <circle r={5.6} className="owner-ring" stroke={props.color} />}
        </>
      ) : (
        <>
          <circle
            r={props.active ? 9 : 6}
            fill={scorched ? "url(#halo-scorched)" : `url(#halo-${props.faction})`}
            className="halo"
          />
          {props.active && <circle r={7} className="pulse" stroke={props.color} />}
          {props.mo && <circle r={5} className="mo-ring" />}
          <circle
            r={(props.active ? 2.2 : 1.5) * props.size}
            className={`core ${scorched ? "scorched" : ""}`}
            fill={scorched ? "#ff7a2e" : props.color}
          />
        </>
      )}
      {props.style === "spheres" && props.active && (
        <ActiveTag defense={Boolean(props.defense)} mo={Boolean(props.mo)} color={props.color} />
      )}
      {props.pinned && <PinMark />}
      {(props.dss || (props.pawns && props.pawns.length > 0)) && (
        <Pawns effects={props.pawns ?? []} faction={props.faction} dss={props.dss} seed={props.seed ?? 0} />
      )}
    </g>
  );
}

export type Hover = { x: number; y: number } & (
  | { planet: Planet; landmark?: never }
  | { landmark: Landmark; planet?: never }
);

function resistance(regen: number): { label: string; color: string } {
  if (regen < 1) return { label: "Low", color: "var(--good)" };
  if (regen < 2.5) return { label: "Average", color: "var(--warn)" };
  return { label: "High", color: "var(--bad)" };
}

function HoverCard({ h, active, effects }: { h: Hover; active: boolean; effects?: PlanetEffect[] }) {
  const place = { left: Math.min(h.x + 18, window.innerWidth - 300), top: Math.max(8, h.y - 40) };
  if (h.landmark) {
    const tint = biomeTint(h.landmark.biome);
    const style = {
      ...place,
      "--sky": tint.sky,
      "--ground": tint.ground,
      "--tone": "var(--f-humans)",
    } as CSSProperties;
    return (
      <div className="hover-card panel glass" style={style} aria-hidden>
        <div className="hc-art" />
        <div className="hc-head">
          <span className="hc-kind caps">{h.landmark.biome}</span>
          <b className="hc-name">{h.landmark.name}</b>
          <span className="hc-sector caps">{h.landmark.sector} sector</span>
        </div>
        <p className="hc-desc">{h.landmark.note}</p>
        <div className="hc-foot caps">No campaign data</div>
      </div>
    );
  }
  const p = h.planet;
  const tint = planetTint(p, effects);
  const enemy = p.event?.faction ?? (p.currentOwner === "Humans" ? undefined : p.currentOwner);
  const anomaly = anomalyOf(p, effects);
  const ANOMALY_KIND: Record<string, string> = {
    wormhole: "Wormhole",
    blackhole: "Black hole",
    fractured: "Fractured planet",
    scorched: "Scorched",
  };
  const kind = p.event
    ? "Defense"
    : active
      ? "Liberation"
      : anomaly
        ? ANOMALY_KIND[anomaly]
        : p.currentOwner === "Humans"
          ? "Secured"
          : "Enemy held";
  const shown = (effects ?? []).filter(hasPawn);
  const pct = enemy ? trackedPercent(p) : 100;
  const regen = regenPctPerHour(p);
  const res = resistance(regen);
  const style = {
    ...place,
    "--sky": tint.sky,
    "--ground": tint.ground,
    "--art": tint.art,
    "--tone": factionColor(enemy ?? "Humans"),
  } as CSSProperties;
  return (
    <div className="hover-card panel glass" style={style} aria-hidden>
      <div className="hc-art" />
      <div className="hc-head">
        <span className="hc-kind caps">{kind}</span>
        <b className="hc-name">{p.name}</b>
        <span className="hc-sector caps">{p.sector} sector</span>
      </div>
      {p.biome?.description && <p className="hc-desc">{p.biome.description}</p>}
      {shown.length > 0 && (
        <ul className="hc-effects caps">
          {shown.map((e) => (
            <li key={e.id} className={e.kind}>
              {e.name}
            </li>
          ))}
        </ul>
      )}
      {enemy && (
        <div className="hc-progress">
          <Meter value={pct} color="var(--f-humans)" rest={factionColor(enemy)} size="m" />
          <span className="num caps">
            {fmtPct(p.event ? pct : liberationPercent(p), 4)} {p.event ? "defended" : "liberated"}
          </span>
        </div>
      )}
      {enemy && (
        <div className="hc-row caps">
          <span>Resistance</span>
          <b style={{ color: res.color }}>
            {res.label} <span className="num">({regen.toFixed(2)}%)</span>
          </b>
        </div>
      )}
      <div className="hc-foot caps">
        <MousePointerClick size={14} /> Click for options
      </div>
    </div>
  );
}

function MapSettings({ onClose }: { onClose(): void }) {
  const overlay = useSettings((s) => s.mapOverlay);
  const planets = useSettings((s) => s.mapPlanets);
  const quality = useSettings((s) => s.mapQuality);
  const set = useSettings((s) => s.set);
  const setView = useUi((s) => s.setView);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const onDown = (e: PointerEvent) => {
      const t = e.target as Element;
      // The rail's own button toggles the panel; let its click do that.
      if (!ref.current?.contains(t) && !t.closest(".map-rail")) onClose();
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("pointerdown", onDown);
    window.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("pointerdown", onDown);
      window.removeEventListener("keydown", onKey);
    };
  }, [onClose]);

  return (
    <div ref={ref} className="map-settings panel glass" role="dialog" aria-label="Map settings">
      <div className="ms-head caps">
        <Settings size={14} /> Map settings
      </div>
      <div className="ms-row">
        <span className="caps">Overlay</span>
        <Stepper<MapOverlay>
          label="Map overlay"
          options={OVERLAY_OPTIONS}
          value={overlay}
          onChange={(v) => set("mapOverlay", v)}
        />
      </div>
      <div className="ms-row">
        <span className="caps">Planets</span>
        <Stepper<MapPlanets>
          label="Planet style"
          options={PLANET_OPTIONS}
          value={planets}
          onChange={(v) => set("mapPlanets", v)}
        />
      </div>
      <div className="ms-row">
        <span className="caps">Quality</span>
        <Stepper<MapQuality>
          label="Render quality"
          options={QUALITY_OPTIONS}
          value={quality}
          onChange={(v) => set("mapQuality", v)}
        />
      </div>
      <button className="link ms-more caps" onClick={() => setView("settings")}>
        All settings <ArrowUpRight size={12} />
      </button>
    </div>
  );
}

const PlanetLayer = memo(function PlanetLayer(props: {
  planets: Planet[];
  effects: Map<number, PlanetEffect[]> | undefined;
  active: Set<number>;
  mo: Set<number>;
  pinned: Set<number>;
  dss: Map<number, DssState>;
  selected: number | null;
  style: MapPlanets;
  onOpen(planet: number, x: number, y: number): void;
  onHover(h: Hover | null): void;
}) {
  const { style, onHover } = props;
  const leave = () => onHover(null);
  return (
    <g className="planets">
      {props.planets.map((p) => {
        const [x, y] = project(p.position);
        const isActive = props.active.has(p.index);
        const isMo = props.mo.has(p.index);
        const isPinned = props.pinned.has(p.index);
        const faction = p.event?.faction ?? p.currentOwner;
        const color = factionColor(faction);
        const cls = [
          "planet",
          isActive && "active",
          isMo && "mo",
          isPinned && "pinned",
          props.selected === p.index && "selected",
          p.currentOwner === "Humans" && "se",
          p.index === 0 && "home",
        ]
          .filter(Boolean)
          .join(" ");
        return (
          <g
            key={p.index}
            className={cls}
            transform={`translate(${x},${y})`}
            onClick={(e) => props.onOpen(p.index, e.clientX, e.clientY)}
            onPointerEnter={(e) => onHover({ planet: p, x: e.clientX, y: e.clientY })}
            onPointerLeave={leave}
          >
            <g className="upright">
              <PlanetBody
                style={style}
                faction={faction}
                color={color}
                fill={p.index === 0 ? "#3f8cc2" : sphereColor(planetTint(p, props.effects?.get(p.index)).ground)}
                size={p.index === 0 ? 1.6 : 1}
                anomaly={anomalyOf(p, props.effects?.get(p.index))}
                active={isActive}
                defense={Boolean(p.event)}
                mo={isMo}
                enemy={p.currentOwner !== "Humans" || Boolean(p.event)}
                pinned={isPinned}
                pawns={props.effects?.get(p.index)?.filter(hasPawn)}
                dss={props.dss.get(p.index)}
                seed={p.index}
              />
              <text className="planet-label">{p.name}</text>
              {isActive && (
                <text className="planet-tag" fill={color}>
                  {p.event ? "Defend" : "Liberate"}
                </text>
              )}
            </g>
          </g>
        );
      })}
      {LANDMARKS.map((l) => {
        const [x, y] = project(l.position);
        return (
          <g
            key={l.name}
            className="planet landmark"
            transform={`translate(${x},${y})`}
            onPointerEnter={(e) => onHover({ landmark: l, x: e.clientX, y: e.clientY })}
            onPointerLeave={leave}
          >
            <g className="upright">
              <PlanetBody
                style={style}
                faction="Humans"
                color="#c98a6a"
                fill="#b5532f"
                size={style === "spheres" ? 0.85 : 0.9}
                anomaly="scorched"
              />
              <text className="planet-label">{l.name}</text>
            </g>
          </g>
        );
      })}
    </g>
  );
});

/** How long the camera must be still before the costly glow filters come back. */
const SETTLE_MS = 180;
/**
 * Below Maximum, a moving map is its last drawn picture under a GPU-only CSS transform;
 * the SVG is redrawn this often, and on settle.
 */
const REDRAW_MS = 400;

/** `onHoverSector` gets the sector under the pointer, whatever the overlay, or null off the disc. */
export function GalaxyMap({ onHoverSector }: { onHoverSector?: (sector: string | null) => void }) {
  const fresh = useFreshPlanets().data;
  const geo = useSectorGeometry().data;
  const effects = useEffects().data;
  const campaigns = useCampaigns();
  const stations = useSpaceStations().data;
  const mo = useMajorOrderPlanets();
  const selectedPlanet = useUi((s) => s.selectedPlanet);
  const focusSeq = useUi((s) => s.focusSeq);
  const openOptions = useUi((s) => s.openOptions);
  const overlay = useSettings((s) => s.mapOverlay);
  const planetStyle = useSettings((s) => s.mapPlanets);
  const quality = useSettings((s) => s.mapQuality);
  const pinned = useSettings((s) => s.pinned);
  const setSetting = useSettings((s) => s.set);
  const stageRef = useRef<HTMLDivElement>(null);
  const svgRef = useRef<SVGSVGElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const layerRef = useRef<SVGGElement>(null);
  const compassRef = useRef<HTMLButtonElement>(null);
  const tiltRef = useRef<HTMLButtonElement>(null);
  const zoomRef = useRef<ZoomBehavior<HTMLDivElement, unknown> | null>(null);
  const transformRef = useRef<ZoomTransform>(zoomIdentity);
  const camRef = useRef<Camera>({ rot: 0, tilt: 0 });
  const qualityRef = useRef(quality);
  qualityRef.current = quality;
  const freshRef = useRef(fresh);
  freshRef.current = fresh;
  const [hover, setHover] = useState<Hover | null>(null);
  const hoverRef = useRef(hover);
  hoverRef.current = hover;
  const [sector, setSector] = useState<string | null>(null);
  const sectorRef = useRef(sector);
  const hoverSector = (s: string | null) => {
    if (s === sectorRef.current) return;
    sectorRef.current = s;
    setSector(s);
    onHoverSector?.(s);
  };
  const [settingsOpen, setSettingsOpen] = useState(false);
  const closeSettings = useCallback(() => setSettingsOpen(false), []);

  const stars = useMemo(() => backgroundStars(STAR_COUNT[quality]), [quality]);
  const active = useMemo(() => new Set((campaigns.data ?? []).map((c) => c.planet.index)), [campaigns.data]);
  const pinnedSet = useMemo(() => new Set(pinned), [pinned]);
  const dss = useMemo(
    () => new Map((stations ?? []).map((s) => [s.planetIndex, dssState(s.tacticalActions, Date.now())])),
    [stations],
  );
  const onOpen = useCallback((planet: number, x: number, y: number) => openOptions({ planet, x, y }), [openOptions]);
  const loaded = Boolean(fresh);

  const starsRef = useRef(stars);
  starsRef.current = stars;
  const overlayRef = useRef(overlay);
  overlayRef.current = overlay;
  const frame = useRef(0);
  const settle = useRef(0);
  const written = useRef({ transform: "", k: "", rot: "", squash: "" });
  /** The camera the SVG was last drawn at, and when. */
  const drawn = useRef<{ t: ZoomTransform; cam: Camera; at: number } | null>(null);

  /** Each style variable restyles every planet, so it's written only when it changes. */
  const draw = useCallback(() => {
    const svg = svgRef.current;
    const layer = layerRef.current;
    if (!svg || !layer) return;
    const t = transformRef.current;
    const cam = camRef.current;
    const deg = (cam.rot * 180) / Math.PI;
    const w = written.current;
    const transform = `translate(${t.x},${t.y}) scale(${t.k}) scale(1,${Math.cos(cam.tilt)}) rotate(${deg})`;
    if (transform !== w.transform) layer.setAttribute("transform", (w.transform = transform));
    const k = String(t.k);
    if (k !== w.k) layer.style.setProperty("--k", (w.k = k));
    const unrot = `${-deg}deg`;
    if (unrot !== w.rot) layer.style.setProperty("--unrot", (w.rot = unrot));
    const squash = String(1 / Math.cos(cam.tilt));
    if (squash !== w.squash) layer.style.setProperty("--unsquash", (w.squash = squash));
    svg.classList.toggle("z-labels", t.k > 0.9);
    svg.classList.toggle("z-sectors", t.k > 1.6);
    svg.classList.toggle("z-planets", t.k > 3);
    svg.style.transform = "";
    drawn.current = { t, cam, at: performance.now() };
  }, []);

  /** Writes the camera to the DOM once per frame; nothing here touches React state. */
  const paint = useCallback(() => {
    frame.current = 0;
    const svg = svgRef.current;
    if (!svg) return;
    const t = transformRef.current;
    const { rot, tilt } = camRef.current;
    const last = drawn.current;
    if (!last || qualityRef.current === "max" || performance.now() - last.at > REDRAW_MS) draw();
    else {
      const m = multiply(screenMatrix(t, camRef.current), invert(screenMatrix(last.t, last.cam)));
      svg.style.transform = `matrix(${m.join(",")})`;
    }
    compassRef.current?.classList.toggle("on", rot !== 0 || tilt !== 0);
    compassRef.current?.querySelector("svg")?.style.setProperty("transform", `rotate(${rot - Math.PI / 4}rad)`);
    tiltRef.current?.classList.toggle("on", tilt > 0);
    if (canvasRef.current)
      drawStars(
        canvasRef.current,
        starsRef.current,
        t,
        camRef.current,
        overlayRef.current !== "sectors",
        STAR_DPR[qualityRef.current],
      );
  }, [draw]);

  /** Queues a paint and marks the map as moving, which drops the glow filters until it settles. */
  const schedule = useCallback(() => {
    const svg = svgRef.current;
    if (!svg) return;
    if (!frame.current) frame.current = requestAnimationFrame(paint);
    // The picture on screen is current when a move begins, so the first redraw waits a full REDRAW_MS.
    if (drawn.current && !svg.classList.contains("moving")) drawn.current.at = performance.now();
    svg.classList.add("moving");
    clearTimeout(settle.current);
    settle.current = window.setTimeout(() => {
      svg.classList.remove("moving");
      draw();
      scaleHatches(svg, transformRef.current.k);
    }, SETTLE_MS);
  }, [paint, draw]);

  // Zoom lives on the stage, not the SVG: a CSS transform on the SVG would skew d3's pointer maths.
  useEffect(() => {
    const stage = stageRef.current;
    if (!stage || !layerRef.current) return;
    const z = zoom<HTMLDivElement, unknown>()
      .scaleExtent(SCALE_EXTENT)
      .touchable(() => false)
      .on("zoom", (e) => {
        transformRef.current = e.transform as ZoomTransform;
        schedule();
        if (hoverRef.current) setHover(null);
      });
    zoomRef.current = z;
    select(stage).call(z).on("dblclick.zoom", null);
    fit();
    const ro = new ResizeObserver(() => fit());
    ro.observe(stage);
    return () => {
      ro.disconnect();
      cancelAnimationFrame(frame.current);
      frame.current = 0;
      clearTimeout(settle.current);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [loaded]);

  // Touch, which d3 is told to leave alone: one finger pans; two fingers pinch to zoom and twist to
  // turn, or slide up and down together to tilt.
  useEffect(() => {
    const stage = stageRef.current;
    if (!stage) return;
    const fingers = new Map<number, Pt>();
    let start: { pts: Pt[]; t: ZoomTransform; cam: Camera } | null = null;
    let mode: "still" | "pan" | "pinch" | "tilt" = "still";
    let twist: number | null = null;
    let dragged = false;

    const local = (e: PointerEvent): Pt => {
      const b = stage.getBoundingClientRect();
      return [e.clientX - b.left, e.clientY - b.top];
    };
    // Each change in the number of fingers starts the gesture afresh from where the camera is.
    const restart = () => {
      const pts = [...fingers.values()].slice(0, 2);
      start = pts.length ? { pts, t: transformRef.current, cam: camRef.current } : null;
      mode = "still";
      twist = null;
    };
    const apply = (x: number, y: number, k: number) => {
      if (zoomRef.current) select(stage).call(zoomRef.current.transform, zoomIdentity.translate(x, y).scale(k));
    };

    const onDown = (e: PointerEvent) => {
      if (e.pointerType !== "touch") return;
      if (!fingers.size) dragged = false;
      fingers.set(e.pointerId, local(e));
      restart();
    };
    const onMove = (e: PointerEvent) => {
      if (!fingers.has(e.pointerId) || !start) return;
      fingers.set(e.pointerId, local(e));
      const now = [...fingers.values()].slice(0, 2);
      const s0 = start;
      if (now.length === 1) {
        const [dx, dy] = [now[0][0] - s0.pts[0][0], now[0][1] - s0.pts[0][1]];
        if (mode === "still" && Math.hypot(dx, dy) < TOUCH_SLOP) return;
        mode = "pan";
        dragged = true;
        apply(s0.t.x + dx, s0.t.y + dy, s0.t.k);
        return;
      }
      const [a0, b0] = s0.pts;
      const [a, b] = now;
      if (mode === "still") {
        const da: Pt = [a[0] - a0[0], a[1] - a0[1]];
        const db: Pt = [b[0] - b0[0], b[1] - b0[1]];
        const [ma, mb] = [Math.hypot(...da), Math.hypot(...db)];
        if (Math.max(ma, mb) < TOUCH_SLOP) return;
        // Fingers report moves one at a time; wait for the other before telling a tilt from a pinch.
        if (Math.min(ma, mb) < TOUCH_SLOP / 2 && Math.max(ma, mb) < 3 * TOUCH_SLOP) return;
        dragged = true;
        const sideBySide = Math.abs(b0[1] - a0[1]) < Math.abs(b0[0] - a0[0]);
        const vertical = (d: Pt) => Math.abs(d[1]) > 2 * Math.abs(d[0]);
        mode = sideBySide && da[1] * db[1] > 0 && vertical(da) && vertical(db) ? "tilt" : "pinch";
      }
      if (mode === "tilt") {
        const dy = (a[1] - a0[1] + b[1] - b0[1]) / 2;
        setCamera({ tilt: s0.cam.tilt - dy * TOUCH_TILT });
        return;
      }
      let turned = angleOf(a, b) - angleOf(a0, b0);
      turned = Math.atan2(Math.sin(turned), Math.cos(turned));
      if (twist == null && Math.abs(turned) > TWIST_START) twist = turned;
      camRef.current = { rot: s0.cam.rot + (twist == null ? 0 : turned - twist), tilt: s0.cam.tilt };
      // A small window fits the galaxy below the zoom-out limit; don't snap to the limit on pinch.
      const least = Math.min(SCALE_EXTENT[0], s0.t.k);
      const k = Math.min(SCALE_EXTENT[1], Math.max(least, (s0.t.k * distance(a, b)) / distance(a0, b0)));
      // What was under the fingers' midpoint stays under it.
      const world = fromLayer(s0.t.invert(midpoint(a0, b0)) as Pt, s0.cam);
      const [x, y] = toLayer(world, camRef.current);
      const m = midpoint(a, b);
      apply(m[0] - k * x, m[1] - k * y, k);
    };
    const onUp = (e: PointerEvent) => {
      if (fingers.delete(e.pointerId)) restart();
    };
    // A drag that ends over a planet shouldn't open it.
    const onClick = (e: MouseEvent) => {
      if (!dragged) return;
      dragged = false;
      e.stopPropagation();
    };
    stage.addEventListener("pointerdown", onDown);
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
    window.addEventListener("pointercancel", onUp);
    stage.addEventListener("click", onClick, true);
    return () => {
      stage.removeEventListener("pointerdown", onDown);
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
      window.removeEventListener("pointercancel", onUp);
      stage.removeEventListener("click", onClick, true);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [loaded]);

  useEffect(() => {
    if (canvasRef.current)
      drawStars(canvasRef.current, stars, transformRef.current, camRef.current, overlay !== "sectors", STAR_DPR[quality]);
  }, [stars, overlay, quality]);
  useEffect(() => {
    if (svgRef.current) scaleHatches(svgRef.current, transformRef.current.k);
  }, [overlay, fresh]);

  function centre(): Pt {
    const { width, height } = stageRef.current!.getBoundingClientRect();
    return [width / 2, height / 2];
  }

  function fit() {
    const stage = stageRef.current;
    if (!stage || !zoomRef.current) return;
    const { width, height } = stage.getBoundingClientRect();
    const k = Math.min(width, height) / (EXTENT * 2);
    select(stage).call(zoomRef.current.transform, zoomIdentity.translate(width / 2, height / 2).scale(k));
  }

  function zoomBy(f: number) {
    if (stageRef.current && zoomRef.current)
      select(stageRef.current).transition().duration(250).call(zoomRef.current.scaleBy, f);
  }

  /** Rotates and tilts about the middle of the screen, keeping whatever is there in place. */
  function setCamera(next: Partial<Camera>) {
    const stage = stageRef.current;
    if (!stage || !zoomRef.current) return;
    const t = transformRef.current;
    const c = centre();
    const world = fromLayer(t.invert(c) as Pt, camRef.current);
    camRef.current = {
      rot: next.rot ?? camRef.current.rot,
      tilt: Math.min(MAX_TILT, Math.max(0, next.tilt ?? camRef.current.tilt)),
    };
    const [x, y] = toLayer(world, camRef.current);
    select(stage).call(zoomRef.current.transform, zoomIdentity.translate(c[0] - t.k * x, c[1] - t.k * y).scale(t.k));
  }

  // Fly to a planet chosen elsewhere ("Show on map") or from the map's own options ("Focus").
  useEffect(() => {
    const stage = stageRef.current;
    const p = selectedPlanet != null && freshRef.current ? byIndex(freshRef.current).get(selectedPlanet) : undefined;
    if (!stage || !zoomRef.current || !p || focusSeq === 0) return;
    const { width, height } = stage.getBoundingClientRect();
    const [x, y] = toLayer(project(p.position), camRef.current);
    const k = Math.max(3, Math.min(width, height) / 280);
    select(stage)
      .transition()
      .duration(700)
      .call(zoomRef.current.transform, zoomIdentity.translate(width / 2 - x * k, height / 2 - y * k).scale(k));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [focusSeq, loaded]);

  // H, Z and C belong to the page.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.ctrlKey || e.metaKey || (e.target as Element | null)?.closest?.("input, textarea, select")) return;
      const k = e.key.toLowerCase();
      if (k === "+" || k === "=") zoomBy(1.5);
      if (k === "-") zoomBy(1 / 1.5);
      if (k === "0") fit();
      if (k === "q") setCamera({ rot: camRef.current.rot - ROT_STEP });
      if (k === "e") setCamera({ rot: camRef.current.rot + ROT_STEP });
      if (k === "r") setCamera({ tilt: camRef.current.tilt + TILT_STEP });
      if (k === "f") setCamera({ tilt: camRef.current.tilt - TILT_STEP });
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  // Right-drag (or Ctrl + drag) turns the map sideways and tilts it up and down; d3-zoom ignores both.
  const drag = useRef<{ x: number; y: number; cam: Camera } | null>(null);
  const onPointerDown = (e: ReactPointerEvent<SVGSVGElement>) => {
    if (e.button !== 2 && !(e.button === 0 && e.ctrlKey)) return;
    e.currentTarget.setPointerCapture(e.pointerId);
    drag.current = { x: e.clientX, y: e.clientY, cam: camRef.current };
  };
  const onPointerMove = (e: ReactPointerEvent<SVGSVGElement>) => {
    const d = drag.current;
    if (d) {
      setCamera({ rot: d.cam.rot + (e.clientX - d.x) * DRAG_ROT, tilt: d.cam.tilt - (e.clientY - d.y) * DRAG_TILT });
      return;
    }
    if (!geo || e.buttons) return;
    const box = stageRef.current!.getBoundingClientRect();
    const at = transformRef.current.invert([e.clientX - box.left, e.clientY - box.top]) as Pt;
    const [x, y] = fromLayer(at, camRef.current);
    hoverSector(sectorAt(geo, x / GALAXY_SCALE, -y / GALAXY_SCALE));
  };
  const endDrag = () => (drag.current = null);

  if (!fresh) return <Loading what="galactic war data" />;

  const nextOverlay = OVERLAYS[(OVERLAYS.indexOf(overlay) + 1) % OVERLAYS.length];

  return (
    <div className={`map-wrap q-${quality}`}>
      <canvas ref={canvasRef} className="starfield-canvas" aria-hidden />
      <div ref={stageRef} className="map-stage">
        <svg
          ref={svgRef}
          className={`galaxy ov-${overlay} pl-${planetStyle}`}
          role="img"
          aria-label="Galactic war map"
          onPointerDown={onPointerDown}
          onPointerMove={onPointerMove}
          onPointerUp={endDrag}
          onPointerCancel={endDrag}
          onContextMenu={(e) => e.preventDefault()}
          onPointerLeave={() => {
            setHover(null);
            hoverSector(null);
          }}
        >
          <HaloDefs />
          <g ref={layerRef}>
            {overlay === "regions" && <Regions planets={fresh} />}
            {overlay === "sectors" && geo && <SectorLayer geo={geo} planets={fresh} id="map" hatch={HATCH} />}
            <Lanes planets={fresh} mode={overlay === "sectors" ? "front" : overlay === "galaxy" ? "bright" : "dim"} />
            {effects && <GloomLayer planets={fresh} gloom={effects.gloom} />}
            <FactionMarks planets={fresh} overlay={overlay} geo={geo} />
            <AttackLines planets={fresh} active={active} />
            <PlanetLayer
              planets={fresh}
              effects={effects?.planets}
              active={active}
              mo={mo}
              pinned={pinnedSet}
              dss={dss}
              selected={selectedPlanet}
              style={planetStyle}
              onOpen={onOpen}
              onHover={setHover}
            />
          </g>
        </svg>
      </div>
      <div className="map-rail">
        <HudButton icon={ZoomIn} label="Zoom in" keyHint="+" onClick={() => zoomBy(1.5)} />
        <HudButton icon={ZoomOut} label="Zoom out" keyHint="-" onClick={() => zoomBy(1 / 1.5)} />
        <button
          ref={compassRef}
          className="hud-btn icon map-compass"
          onClick={() => setCamera({ rot: 0, tilt: 0 })}
          aria-label="Reset rotation and tilt"
          title="Reset rotation and tilt"
        >
          <Compass size={18} strokeWidth={1.75} style={{ transform: `rotate(${-Math.PI / 4}rad)` }} />
        </button>
        <button
          ref={tiltRef}
          className="hud-btn icon"
          onClick={() => setCamera({ tilt: camRef.current.tilt > 0 ? 0 : TILT_PRESET })}
          aria-label="Tilt or flatten the map"
          title="Tilt or flatten the map (R / F)"
        >
          <Rotate3d size={18} strokeWidth={1.75} />
        </button>
        <HudButton
          icon={Layers}
          label={`Map overlay: ${overlay} (next: ${nextOverlay})`}
          onClick={() => setSetting("mapOverlay", nextOverlay)}
        />
        <HudButton
          icon={Settings}
          label="Map settings"
          active={settingsOpen}
          onClick={() => setSettingsOpen((o) => !o)}
        />
      </div>
      {settingsOpen && <MapSettings onClose={closeSettings} />}
      {hover && (
        <HoverCard
          h={hover}
          active={hover.planet ? active.has(hover.planet.index) : false}
          effects={hover.planet ? effects?.planets.get(hover.planet.index) : undefined}
        />
      )}
    </div>
  );
}
