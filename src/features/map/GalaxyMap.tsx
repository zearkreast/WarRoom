import { select } from "d3-selection";
import { interrupt } from "d3-transition";
import { zoom, zoomIdentity, type ZoomBehavior, type ZoomTransform } from "d3-zoom";
import {
  ArrowUpRight,
  BookOpen,
  Compass,
  Landmark as PoiIcon,
  Rotate3d,
  Settings,
  Shield,
  TriangleAlert,
  ZoomIn,
  ZoomOut,
  type LucideIcon,
} from "lucide-react";
import {
  forwardRef,
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
import { planetTint } from "../../domain/biomes";
import { hasPawn, type PlanetEffect } from "../../domain/effects";
import { FACTIONS, factionColor } from "../../domain/factions";
import {
  anomalyOf,
  GALAXY_SCALE,
  LANDMARKS,
  project,
  RING_RADII,
  sectorAt,
  supplyLines,
  type Anomaly,
  type PolarSectors,
} from "../../domain/galaxy";
import { factionMarks } from "../../domain/factionMarks";
import { byIndex, ownershipKey } from "../../domain/planets";
import { buildRegions } from "../../domain/regions";
import { useSettings, useUi, type MapOverlay, type MapPlanets, type MapQuality } from "../../state";
import { EmblemArt, subfactionArt } from "../../ui/emblems";
import { Loading, Stepper } from "../../ui/primitives";
import { useMajorOrderPlanets } from "../planets/useFronts";
import { GloomLayer, SectorLayer } from "./layers";
import { writeCamera } from "./cameraRules";
import { OVERLAY_OPTIONS, PLANET_OPTIONS, QUALITY_OPTIONS } from "./mapOptions";
import { PoiGlyph, poiKind, poiOrigin, type PoiKind } from "./poi";
import { DSS_TONE, dssArt, dssInfo, PAWN_ART, pawnArt, type DssAction, type DssInfo, type DssState } from "./pawnArt";
import { MapLegend } from "./MapLegend";
import { HoverCard, PlanetPanel, type Hover } from "./planetCard";
import "./map.css";

const EXTENT = GALAXY_SCALE * 1.1;
const STAR_COUNT: Record<MapQuality, number> = { max: 4200, normal: 2600, min: 1600 };
/** Highest device pixel ratio the starfield canvas is drawn at; phones report up to 3 or 4. */
const STAR_DPR: Record<MapQuality, number> = { max: 4, normal: 2, min: 1.5 };

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
/** How long the buttons and keys take to turn or tilt the map. */
const CAMERA_MS = 320;

/** An angle in (-π, π]. */
const wrap = (a: number) => Math.atan2(Math.sin(a), Math.cos(a));

const turn = ([x, y]: Pt, a: number): Pt => [x * Math.cos(a) - y * Math.sin(a), x * Math.sin(a) + y * Math.cos(a)];
/** Map units to the zoom layer's own units: rotated, then squashed by the tilt. */
const toLayer = (p: Pt, cam: Camera): Pt => {
  const [x, y] = turn(p, cam.rot);
  return [x, y * Math.cos(cam.tilt)];
};
const fromLayer = ([x, y]: Pt, cam: Camera): Pt => turn([x, y / Math.cos(cam.tilt)], -cam.rot);

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
            <EmblemArt art={m.faction} size={72} />
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
      {/* Penta's black hole, after the game's own. */}
      <radialGradient id="bh-glow">
        <stop offset="30%" stopColor="#ffb07a" stopOpacity={0.6} />
        <stop offset="60%" stopColor="#ff6a3a" stopOpacity={0.18} />
        <stop offset="100%" stopColor="#ff6a3a" stopOpacity={0} />
      </radialGradient>
      <linearGradient id="bh-disk">
        <stop offset="0%" stopColor="#ff5a2e" stopOpacity={0} />
        <stop offset="22%" stopColor="#ff7a45" stopOpacity={0.75} />
        <stop offset="50%" stopColor="#fff1e4" stopOpacity={1} />
        <stop offset="78%" stopColor="#ff9ac2" stopOpacity={0.75} />
        <stop offset="100%" stopColor="#ff5a8a" stopOpacity={0} />
      </linearGradient>
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
      {/* Each hologram recoloured in its light's colour from its own brightness, so off-colour renders match. */}
      {Object.entries(HOLO_TINT).map(([name, rgb]) => (
        <filter key={name} id={`holo-tint-${name}`} colorInterpolationFilters="sRGB">
          <feColorMatrix
            type="matrix"
            values={`${rgb
              .map((c) => {
                const m = c * 1.6 + (1 - c) * 0.5;
                return `${(0.3 * m).toFixed(3)} ${(0.59 * m).toFixed(3)} ${(0.11 * m).toFixed(3)} 0 0`;
              })
              .join(" ")} 0 0 0 1 0`}
          />
        </filter>
      ))}
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
  // The disk is drawn twice: its far half behind the shadow, its near half in front.
  if (kind === "blackhole")
    return (
      <g className="anomaly blackhole">
        <circle r={10} fill="url(#bh-glow)" />
        <ellipse rx={10.5} ry={3} fill="url(#bh-disk)" />
        <circle r={4.9} className="lens" />
        <circle r={3.9} className="photon" />
        <circle r={3.4} className="horizon" />
        <path d="M-10.5,0A10.5,3 0 0 0 10.5,0Z" fill="url(#bh-disk)" />
        <ellipse rx={8.5} ry={2.2} className="bh-flow" />
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

/** Each hologram's light: its colour on the map, and the tint its render is recoloured with. */
const DSS_LIGHT: Record<DssState, [string, string]> = {
  active: [DSS_TONE.active, "cta"],
  cooldown: [DSS_TONE.cooldown, "bad"],
  offline: [DSS_TONE.offline, "off"],
  default: [DSS_TONE.default, "humans"],
};
/** A unit's light follows its own faction, whoever holds the planet it is on. */
const UNIT_LIGHT: Record<string, [string, string]> = {
  bug: [factionColor("Terminids"), "bug"],
  bot: [factionColor("Automaton"), "bot"],
  ill: [factionColor("Illuminate"), "ill"],
  seaf: [factionColor("Humans"), "humans"],
};
/** The tints as linear RGB weights, close to the faction colours. */
const HOLO_TINT: Record<string, [number, number, number]> = {
  bug: [1, 0.58, 0.2],
  bot: [1, 0.44, 0.44],
  ill: [0.8, 0.54, 0.91],
  humans: [0.27, 0.64, 1],
  cta: [1, 1, 0.35],
  bad: [1, 0.35, 0.35],
  off: [0.55, 0.57, 0.6],
};
/** Renders that fill more of their frame than the rest, shrunk to match; SEAF's small Pelicans, grown. */
const UNIT_SCALE: Record<string, number> = {
  seaf: 1.25,
  bug_dragonroach: 0.8,
  ill_greathost: 0.85,
  bot_cyborgs: 0.85,
  bot_incincorp: 0.85,
  bot_jetbrigade: 0.85,
};

/**
 * Hologram size and spacing in marker units: drawn at HOLO_SIZE, shrunk by HOLO_SCALE, kept close to
 * the planet so it's clear whose they are.
 */
const HOLO_SIZE = 22;
const HOLO_SCALE = 0.78;
const HOLO_X = 16;
const HOLO_STEP = 11;

type Light = { art: { key: string; href: string }; tone: string; tint: string; badges?: DssAction[] };

/** Size and spacing of the DSS's active-action badges, in hologram units. */
const BADGE = 9;
const BADGE_STEP = 10;

/**
 * A unit as a hologram on the war table: a projector lying on the map, a column of light, and the
 * unit floating on it. The projector is always seen at an angle, like the renders; tilting the map
 * flattens it further and lifts the unit higher.
 */
function HoloPawn({ light, x, back, delay }: { light: Light; x: number; back: boolean; delay: number }) {
  const { art } = light;
  const s = HOLO_SIZE;
  const style = { color: light.tone, "--tint": `url(#holo-tint-${light.tint})`, "--delay": `${-delay}s` };
  return (
    <g transform={`translate(${x},2) scale(${HOLO_SCALE})`}>
      <g className={`holo ${back ? "back" : ""}`} style={style as CSSProperties}>
        <g className="holo-floor">
          <circle r={8} className="holo-pool" mask="url(#holo-pool)" />
          <circle r={5.4} className="holo-ring" />
          <circle r={4.2} className="holo-ticks" />
          <circle r={1.3} className="holo-emitter" />
        </g>
        <path className="holo-beam" d="M-1.6,0L-6,-1H6L1.6,0Z" mask="url(#holo-fade)" />
        <path className="holo-edge" d="M-1.5,0L-6,-1M1.5,0L6,-1" mask="url(#holo-fade)" />
        <g className="holo-lift">
          {light.badges && light.badges.length > 0 && (
            // Just over the station (its render leaves the top of the frame empty), outside .holo-unit
            // so they keep their colours and don't bob.
            <g className="holo-badges" transform={`translate(${(-(light.badges.length - 1) * BADGE_STEP) / 2},${-s * 0.85 - BADGE / 2 - 1})`}>
              {light.badges.map((b, i) => (
                <image key={b.key} href={b.href} x={i * BADGE_STEP - BADGE / 2} y={-BADGE / 2} width={BADGE} height={BADGE} />
              ))}
            </g>
          )}
          <g className="holo-unit">
            <g transform={`scale(${UNIT_SCALE[art.key] ?? 1})`}>
              <image href={art.href} x={-s / 2} y={-s} width={s} height={s} />
              <g className="holo-light" mask={`url(#holo-mask-${art.key})`}>
                <rect x={-s / 2} y={-s} width={s} height={s} fill="url(#holo-scan)" />
                <rect className="holo-sweep" x={-s / 2} y={-4} width={s} height={4} fill="url(#holo-sweep-band)" />
              </g>
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
function Pawns(props: { effects: PlanetEffect[]; faction: string; dss?: DssInfo; seed: number; enemy: boolean }) {
  const { faction, dss, seed } = props;
  // The game keeps an enemy unit's effect on a planet after it is liberated; Super Earth's planets show none.
  // Points of interest wear their own logo over the planet instead of a tag.
  const effects = props.effects.filter((e) => e.kind !== "poi" && (props.enemy || e.kind !== "subfaction"));
  const holos: Light[] = [];
  const station = dss && dssArt(dss.state);
  if (dss && station)
    holos.push({ art: station, tone: DSS_LIGHT[dss.state][0], tint: DSS_LIGHT[dss.state][1], badges: dss.active });
  const tags: PlanetEffect[] = [];
  for (const e of effects) {
    const art = pawnArt(e);
    const [tone, tint] = art ? UNIT_LIGHT[art.key.split("_")[0]] : [];
    if (art && tone && tint) holos.push({ art, tone, tint });
    else tags.push(e);
  }
  const placed = holos.map((light, i) => ({
    light,
    x: HOLO_X + i * HOLO_STEP,
    back: i % 2 === 1,
    delay: (seed * 0.73 + i * 1.3) % 6,
  }));
  // Holograms only read once zoomed in; until then every effect shows as a tag.
  return (
    <>
      {effects.length > 0 && <PawnTags effects={effects} faction={faction} x={7} className="pawns-all" />}
      {holos.length > 0 && (
        <g className="holos">
          {/* The back rank first, so the front one stands before it. */}
          {[...placed.filter((h) => h.back), ...placed.filter((h) => !h.back)].map((h) => (
            <HoloPawn key={h.light.art.key} light={h.light} x={h.x} back={h.back} delay={h.delay} />
          ))}
        </g>
      )}
      {tags.length > 0 && (
        <PawnTags
          effects={tags}
          faction={faction}
          x={holos.length ? HOLO_X + holos.length * HOLO_STEP + 2 : 7}
          className="pawns-rest"
        />
      )}
    </>
  );
}

/** Threats and points of interest, as tags in a row. */
function PawnTags(props: { effects: PlanetEffect[]; faction: string; x: number; className: string }) {
  const { effects, faction } = props;
  return (
    <g className={`pawns ${props.className}`} transform={`translate(${props.x},1)`}>
      {effects.map((e, i) => {
        const x = i * 6.4;
        const tone = e.kind === "subfaction" ? factionColor(faction) : e.kind === "seaf" ? "var(--f-humans)" : "var(--cta)";
        return (
          <g key={e.id} className={`pawn ${e.kind}`} transform={`translate(${x},0)`} style={{ color: tone }}>
            <rect x={-2.8} y={-2.8} width={5.6} height={5.6} />
            {e.kind === "subfaction" ? (
              <EmblemArt art={subfactionArt(e.name) ?? faction} size={4.4} />
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

/** Logos per planet and their size in marker units; keep in step with the name lift in cameraRules.ts. */
const POI_MAX = 2;
const POI_SIZE = 10;

/** The planet's points of interest as the game's logos, side by side over the planet. */
function PoiMarks({ kinds, faction }: { kinds: PoiKind[]; faction: string }) {
  return (
    <g className="poi-marks" transform="translate(0,-10)">
      {kinds.map((k, i) => (
        <g key={k} transform={`translate(${(i - (kinds.length - 1) / 2) * (POI_SIZE + 1.5)},0)`}>
          <PoiGlyph kind={k} faction={poiOrigin(k, faction)} size={POI_SIZE} />
        </g>
      ))}
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
  dss?: DssInfo;
  seed?: number;
  pois?: PoiKind[];
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
      {props.pois && props.pois.length > 0 && <PoiMarks kinds={props.pois} faction={props.faction} />}
      {(props.dss || (props.pawns && props.pawns.length > 0)) && (
        <Pawns
          effects={props.pawns ?? []}
          faction={props.faction}
          dss={props.dss}
          seed={props.seed ?? 0}
          enemy={Boolean(props.enemy)}
        />
      )}
    </g>
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
  dss: Map<number, DssInfo>;
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
        const pois = [
          ...new Set((props.effects?.get(p.index) ?? []).filter((e) => e.kind === "poi").map((e) => poiKind(e.name))),
        ].slice(0, POI_MAX);
        const cls = [
          "planet",
          isActive && "active",
          isMo && "mo",
          isPinned && "pinned",
          props.selected === p.index && "selected",
          p.currentOwner === "Humans" && "se",
          p.index === 0 && "home",
          pois.length > 0 && "poi",
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
                pois={pois}
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

const RailButton = forwardRef<
  HTMLButtonElement,
  {
    icon: LucideIcon;
    label: string;
    title?: string;
    keyHint?: string;
    active?: boolean;
    className?: string;
    iconStyle?: CSSProperties;
    onClick(): void;
  }
>(function RailButton(props, ref) {
  const title = props.title ?? props.label;
  return (
    <button
      ref={ref}
      className={`rail-btn ${props.active ? "on" : ""} ${props.className ?? ""}`}
      onClick={props.onClick}
      aria-label={title}
      aria-pressed={props.active}
      title={props.keyHint ? `${title} (${props.keyHint})` : title}
    >
      <props.icon size={18} strokeWidth={1.75} style={props.iconStyle} />
      <span className="caps">{props.label}</span>
    </button>
  );
});

/** How long the camera must be still before the costly glow filters come back. */
const SETTLE_MS = 180;

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
  const overlay = useSettings((s) => s.mapOverlay);
  const planetStyle = useSettings((s) => s.mapPlanets);
  const quality = useSettings((s) => s.mapQuality);
  const pinned = useSettings((s) => s.pinned);
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
  const [side, setSide] = useState<"settings" | "legend" | null>(null);
  const closeSettings = useCallback(() => setSide(null), []);

  const stars = useMemo(() => backgroundStars(STAR_COUNT[quality]), [quality]);
  const active = useMemo(() => new Set((campaigns.data ?? []).map((c) => c.planet.index)), [campaigns.data]);
  const pinnedSet = useMemo(() => new Set(pinned), [pinned]);
  const dss = useMemo(
    () => new Map((stations ?? []).map((s) => [s.planetIndex, dssInfo(s.tacticalActions, Date.now())])),
    [stations],
  );
  const [panel, setPanel] = useState<number | null>(null);
  const selectPlanet = useUi((s) => s.select);
  const onOpen = useCallback(
    (planet: number) => {
      setPanel(planet);
      selectPlanet(planet);
    },
    [selectPlanet],
  );
  const closePanel = useCallback(() => setPanel(null), []);
  const loaded = Boolean(fresh);

  const starsRef = useRef(stars);
  starsRef.current = stars;
  const overlayRef = useRef(overlay);
  overlayRef.current = overlay;
  const frame = useRef(0);
  const settle = useRef(0);
  const written = useRef("");

  /** Writes the camera to the DOM once per frame; nothing here touches React state. */
  const paint = useCallback(() => {
    frame.current = 0;
    const svg = svgRef.current;
    const layer = layerRef.current;
    if (!svg || !layer) return;
    const t = transformRef.current;
    const { rot, tilt } = camRef.current;
    const transform = `translate(${t.x},${t.y}) scale(${t.k}) scale(1,${Math.cos(tilt)}) rotate(${(rot * 180) / Math.PI})`;
    if (transform !== written.current) layer.setAttribute("transform", (written.current = transform));
    writeCamera(t.k, rot, tilt);
    svg.classList.toggle("z-labels", t.k > 0.9);
    svg.classList.toggle("z-sectors", t.k > 1.6);
    svg.classList.toggle("z-planets", t.k > 3);
    svg.classList.toggle("z-units", t.k > 2);
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
  }, []);

  /** Queues a paint and marks the map as moving, which drops the glow filters until it settles. */
  const schedule = useCallback(() => {
    const svg = svgRef.current;
    if (!svg) return;
    if (!frame.current) frame.current = requestAnimationFrame(paint);
    svg.classList.add("moving");
    clearTimeout(settle.current);
    settle.current = window.setTimeout(() => {
      svg.classList.remove("moving");
      scaleHatches(svg, transformRef.current.k);
    }, SETTLE_MS);
  }, [paint]);

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
    // A resize (a phone turned, a browser bar sliding away) keeps the middle in place instead of refitting.
    let size: Pt | null = null;
    const ro = new ResizeObserver(() => {
      const { width, height } = stage.getBoundingClientRect();
      const before = size;
      size = [width, height];
      if (!before) return fit();
      if (!width || !height) return;
      const t = transformRef.current;
      const shifted = zoomIdentity.translate(t.x + (width - before[0]) / 2, t.y + (height - before[1]) / 2).scale(t.k);
      select(stage).call(z.transform, shifted);
    });
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
      settleCamera();
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
    // A zoom step or fly-to still running would keep pulling towards a spot worked out for the old camera.
    interrupt(stage);
    const t = transformRef.current;
    const c = centre();
    const world = fromLayer(t.invert(c) as Pt, camRef.current);
    camRef.current = {
      rot: wrap(next.rot ?? camRef.current.rot),
      tilt: Math.min(MAX_TILT, Math.max(0, next.tilt ?? camRef.current.tilt)),
    };
    const [x, y] = toLayer(world, camRef.current);
    select(stage).call(zoomRef.current.transform, zoomIdentity.translate(c[0] - t.k * x, c[1] - t.k * y).scale(t.k));
  }

  const tween = useRef(0);
  /** Where the buttons and keys are taking the camera, so a key pressed mid-turn adds to the turn. */
  const goal = useRef<Camera | null>(null);

  /** Stops a camera tween and any zoom transition, for a gesture that takes over. */
  function settleCamera() {
    cancelAnimationFrame(tween.current);
    goal.current = null;
    if (stageRef.current) interrupt(stageRef.current);
  }

  /** Turns and tilts to `next` over CAMERA_MS, the short way round. */
  function animateCamera(next: Partial<Camera>) {
    cancelAnimationFrame(tween.current);
    const from = camRef.current;
    const to = {
      rot: next.rot ?? goal.current?.rot ?? from.rot,
      tilt: Math.min(MAX_TILT, Math.max(0, next.tilt ?? goal.current?.tilt ?? from.tilt)),
    };
    goal.current = to;
    const turnBy = wrap(to.rot - from.rot);
    const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const start = performance.now();
    const step = (now: number) => {
      const p = reduced ? 1 : Math.min(1, (now - start) / CAMERA_MS);
      const eased = 1 - Math.pow(1 - p, 3);
      setCamera({ rot: from.rot + turnBy * eased, tilt: from.tilt + (to.tilt - from.tilt) * eased });
      if (p < 1) tween.current = requestAnimationFrame(step);
      else goal.current = null;
    };
    tween.current = requestAnimationFrame(step);
  }
  useEffect(() => () => cancelAnimationFrame(tween.current), []);
  const aim = () => goal.current ?? camRef.current;

  // Fly to a planet chosen elsewhere ("Show on map") or from the map's own options ("Focus").
  useEffect(() => {
    const stage = stageRef.current;
    const p = selectedPlanet != null && freshRef.current ? byIndex(freshRef.current).get(selectedPlanet) : undefined;
    if (!stage || !zoomRef.current || !p || focusSeq === 0) return;
    setPanel(p.index);
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
      if (k === "q") animateCamera({ rot: aim().rot - ROT_STEP });
      if (k === "e") animateCamera({ rot: aim().rot + ROT_STEP });
      if (k === "r") animateCamera({ tilt: aim().tilt + TILT_STEP });
      if (k === "f") animateCamera({ tilt: aim().tilt - TILT_STEP });
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  // Right-drag (or Ctrl + drag) turns the map sideways and tilts it up and down; d3-zoom ignores both.
  const drag = useRef<{ x: number; y: number; cam: Camera } | null>(null);
  const onPointerDown = (e: ReactPointerEvent<SVGSVGElement>) => {
    if (e.button !== 2 && !(e.button === 0 && e.ctrlKey)) return;
    settleCamera();
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

  const panelPlanet = panel == null ? undefined : byIndex(fresh).get(panel);

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
        <RailButton icon={ZoomIn} label="Zoom in" keyHint="+" onClick={() => zoomBy(1.5)} />
        <RailButton icon={ZoomOut} label="Zoom out" keyHint="-" onClick={() => zoomBy(1 / 1.5)} />
        <RailButton
          ref={compassRef}
          className="map-compass"
          icon={Compass}
          iconStyle={{ transform: `rotate(${-Math.PI / 4}rad)` }}
          label="North"
          title="Turn north up and flatten the map"
          onClick={() => animateCamera({ rot: 0, tilt: 0 })}
        />
        <RailButton
          ref={tiltRef}
          icon={Rotate3d}
          label="Tilt"
          keyHint="R / F"
          title="Tilt or flatten the map"
          onClick={() => animateCamera({ tilt: aim().tilt > 0 ? 0 : TILT_PRESET })}
        />
        <RailButton
          icon={BookOpen}
          label="Legend"
          active={side === "legend"}
          onClick={() => setSide((o) => (o === "legend" ? null : "legend"))}
        />
        <RailButton
          icon={Settings}
          label="Settings"
          title="Map settings"
          active={side === "settings"}
          onClick={() => setSide((o) => (o === "settings" ? null : "settings"))}
        />
      </div>
      {side === "settings" && <MapSettings onClose={closeSettings} />}
      {side === "legend" && <MapLegend onClose={closeSettings} />}
      {panelPlanet && (
        <PlanetPanel
          p={panelPlanet}
          active={active.has(panelPlanet.index)}
          effects={effects?.planets.get(panelPlanet.index)}
          onClose={closePanel}
        />
      )}
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
