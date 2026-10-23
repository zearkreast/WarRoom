import { select } from "d3-selection";
import { zoom, zoomIdentity, type ZoomBehavior, type ZoomTransform } from "d3-zoom";
import {
  ArrowUpRight,
  Compass,
  Layers,
  Maximize,
  MousePointerClick,
  Rotate3d,
  Settings,
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
import { useCampaigns, useFreshPlanets } from "../../api/queries";
import type { Faction, Planet } from "../../api/types";
import { biomeTint, regenPctPerHour } from "../../domain/biomes";
import { FACTIONS, factionColor } from "../../domain/factions";
import { fmtPct } from "../../domain/format";
import {
  anomalyOf,
  buildPolarSectors,
  GALAXY_SCALE,
  LANDMARKS,
  project,
  RING_RADII,
  sectorTerritory,
  supplyLines,
  territoryLabels,
  type Anomaly,
  type Landmark,
} from "../../domain/galaxy";
import { byIndex } from "../../domain/planets";
import { liberationPercent, trackedPercent } from "../../domain/progress";
import { useSettings, useUi, type MapOverlay, type MapPlanets, type MapQuality } from "../../state";
import { HudButton, Loading, Meter, Stepper } from "../../ui/primitives";
import { useMajorOrderPlanets } from "../planets/useFronts";
import { OVERLAY_OPTIONS, PLANET_OPTIONS, QUALITY_OPTIONS } from "./mapOptions";
import "./map.css";

const EXTENT = GALAXY_SCALE * 1.1;
const STAR_COUNT: Record<MapQuality, number> = { max: 4200, normal: 2600, min: 900 };
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

const turn = ([x, y]: Pt, a: number): Pt => [x * Math.cos(a) - y * Math.sin(a), x * Math.sin(a) + y * Math.cos(a)];
/** Map units to the zoom layer's own units: rotated, then squashed by the tilt. */
const toLayer = (p: Pt, cam: Camera): Pt => {
  const [x, y] = turn(p, cam.rot);
  return [x, y * Math.cos(cam.tilt)];
};
const fromLayer = ([x, y]: Pt, cam: Camera): Pt => turn([x, y / Math.cos(cam.tilt)], -cam.rot);

/** Deterministic background stars so the field doesn't reshuffle between renders. */
function backgroundStars(n: number) {
  let seed = 7;
  const rnd = () => (seed = (seed * 16807) % 2147483647) / 2147483647;
  return Array.from({ length: n }, () => {
    const a = rnd() * Math.PI * 2;
    // Denser towards the core, like the holotable's galaxy.
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

function drawStars(canvas: HTMLCanvasElement, stars: Star[], t: ZoomTransform, cam: Camera) {
  const dpr = window.devicePixelRatio || 1;
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

  // The game's sector rings, so planets read against the same bands in every overlay.
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

const ENEMIES: Faction[] = ["Terminids", "Automaton", "Illuminate"];

/**
 * Enemy territory on the game's ring sectors. Regions draws it as glowing fronts over a faint
 * wash; Sectors adds the sector grid and hatching, solid where the enemy holds every planet
 * of a sector and faint where Super Earth still holds some (contested).
 */
const Territory = memo(function Territory({
  planets,
  mode,
  glow,
  onHover,
}: {
  planets: Planet[];
  mode: "regions" | "sectors";
  glow: boolean;
  onHover?: (p: Planet | null) => void;
}) {
  const geo = useMemo(() => buildPolarSectors(planets), [planets]);
  const territory = useMemo(() => sectorTerritory(planets), [planets]);
  const shapes = useMemo(() => {
    let grid = "";
    const fronts = new Map<Faction, string>();
    const splits = new Map<Faction, string>();
    for (const e of geo.edges) {
      grid += e.d;
      const ta = territory.get(e.a) ?? null;
      const tb = e.b ? (territory.get(e.b) ?? null) : null;
      if (ta?.faction === tb?.faction) {
        // Same enemy on both sides: only the held/contested line between them is drawn.
        if (ta && ta.held !== tb?.held) splits.set(ta.faction, (splits.get(ta.faction) ?? "") + e.d);
        continue;
      }
      for (const t of [ta, tb]) if (t) fronts.set(t.faction, (fronts.get(t.faction) ?? "") + e.d);
    }
    const held = new Map<Faction, string>();
    const contested = new Map<Faction, string>();
    for (const [s, d] of geo.fills) {
      const t = territory.get(s);
      if (t) (t.held ? held : contested).set(t.faction, ((t.held ? held : contested).get(t.faction) ?? "") + d);
    }
    return { grid, fronts, splits, held, contested, labels: territoryLabels(geo, territory) };
  }, [geo, territory]);
  const firstOf = useMemo(() => {
    const m = new Map<string, Planet>();
    for (const p of planets) if (!m.has(p.sector)) m.set(p.sector, p);
    return m;
  }, [planets]);
  const sectors = mode === "sectors";

  return (
    <g className={`territory ${mode} ${glow ? "glow" : ""}`}>
      {sectors && (
        <defs>
          {ENEMIES.flatMap((f) =>
            (["held", "contested"] as const).map((kind) => (
              <pattern
                key={`${f}-${kind}`}
                id={`hatch-${f}-${kind}`}
                className="hatch"
                width={HATCH}
                height={HATCH}
                patternUnits="userSpaceOnUse"
                patternTransform="rotate(45)"
              >
                <rect
                  width={HATCH}
                  height={HATCH}
                  fill={FACTIONS[f].color}
                  fillOpacity={kind === "held" ? 0.2 : 0.04}
                />
                <line
                  x1={1}
                  y1={0}
                  x2={1}
                  y2={HATCH}
                  stroke={FACTIONS[f].color}
                  strokeOpacity={kind === "held" ? 0.5 : 0.28}
                  strokeWidth={kind === "held" ? 2 : 1.2}
                />
              </pattern>
            )),
          )}
        </defs>
      )}
      {sectors && <circle r={geo.rim} className="sg-disc" />}
      {[...shapes.held].map(([f, d]) => (
        <path key={`h-${f}`} d={d} className="tf-held" fill={sectors ? `url(#hatch-${f}-held)` : FACTIONS[f].color} />
      ))}
      {[...shapes.contested].map(([f, d]) => (
        <path
          key={`c-${f}`}
          d={d}
          className="tf-contested"
          fill={sectors ? `url(#hatch-${f}-contested)` : FACTIONS[f].color}
        />
      ))}
      {sectors && <path d={shapes.grid} className="sg-grid" />}
      {[...shapes.splits].map(([f, d]) => (
        <path key={`s-${f}`} d={d} className="tf-split" style={{ color: FACTIONS[f].color }} />
      ))}
      {[...shapes.fronts].map(([f, d]) => (
        <g key={`f-${f}`} style={{ color: FACTIONS[f].color }}>
          {glow && <path d={d} className="tf-glow" />}
          <path d={d} className="tf-front" />
        </g>
      ))}
      {sectors && <circle r={geo.rim} className="sg-rim" />}
      {sectors &&
        shapes.labels.map((l) => (
          <g key={l.faction}>
            <path id={`tl-${l.faction}`} d={l.d} fill="none" />
            <text className="sg-label" fill={FACTIONS[l.faction].color}>
              <textPath href={`#tl-${l.faction}`} startOffset="50%">
                {FACTIONS[l.faction].label}
              </textPath>
            </text>
          </g>
        ))}
      {onHover && (
        <g className="sg-hits" onPointerLeave={() => onHover(null)}>
          {[...geo.fills].map(([s, d]) => (
            <path key={s} d={d} onPointerEnter={() => onHover(firstOf.get(s) ?? null)} />
          ))}
        </g>
      )}
    </g>
  );
});

type LaneMode = "dim" | "bright" | "front";

const Lanes = memo(function Lanes({ planets, mode }: { planets: Planet[]; mode: LaneMode }) {
  const lookup = byIndex(planets);
  const lines = useMemo(() => supplyLines(planets), [planets]);
  return (
    <g className={`lanes ${mode}`}>
      {lines.map(([a, b]) => {
        const pa = lookup.get(a);
        const pb = lookup.get(b);
        if (!pa || !pb) return null;
        const [x1, y1] = project(pa.position);
        const [x2, y2] = project(pb.position);
        // Lanes inside one enemy's holdings take its colour, as on the in-game map.
        const held = mode === "front" && pa.currentOwner === pb.currentOwner && pa.currentOwner !== "Humans";
        return (
          <line
            key={`${a}-${b}`}
            x1={x1}
            y1={y1}
            x2={x2}
            y2={y2}
            stroke={held ? factionColor(pa.currentOwner) : undefined}
          />
        );
      })}
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
      <radialGradient id="blackhole">
        <stop offset="0%" stopColor="#000" stopOpacity={1} />
        <stop offset="46%" stopColor="#000" stopOpacity={1} />
        <stop offset="54%" stopColor="#dfe9f5" stopOpacity={0.75} />
        <stop offset="66%" stopColor="#7d93ad" stopOpacity={0.25} />
        <stop offset="100%" stopColor="#7d93ad" stopOpacity={0} />
      </radialGradient>
    </defs>
  );
});

/** Meridia's purple wormhole and Penta's black hole, drawn in place of a world. */
function AnomalyMark({ kind }: { kind: Anomaly }) {
  return kind === "wormhole" ? (
    <g className="anomaly wormhole">
      <circle r={9} fill="url(#wormhole)" />
      <circle r={5.6} className="swirl" />
    </g>
  ) : (
    <g className="anomaly blackhole">
      <circle r={8} fill="url(#blackhole)" />
      <circle r={3.6} className="horizon" />
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

const HEX = Array.from({ length: 6 }, (_, i) => {
  const a = ((i * 60 - 90) * Math.PI) / 180;
  return `${(Math.cos(a) * 3.8).toFixed(2)},${(Math.sin(a) * 3.8).toFixed(2)}`;
}).join(" ");

/** Hexagon marker over a campaign planet; yellow for Major Order targets. */
function CampaignBadge({ mo }: { mo: boolean }) {
  return (
    <g className={`camp-badge ${mo ? "mo" : ""}`} transform="translate(0,-10)">
      <polygon points={HEX} />
      <circle r={1.6} />
      <path d="M-2.7,0H-1.1M1.1,0H2.7M0,-2.7V-1.1M0,1.1V2.7" />
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
  glow: boolean;
  faction: string;
  color: string;
  fill: string;
  size: number;
  anomaly?: Anomaly;
  active?: boolean;
  mo?: boolean;
  enemy?: boolean;
  pinned?: boolean;
}) {
  return (
    <g className="pbody">
      <circle r={10} className="hit" />
      {props.anomaly ? (
        <AnomalyMark kind={props.anomaly} />
      ) : props.style === "spheres" ? (
        <>
          <circle r={4.4 * props.size} className="sphere" fill={props.fill} />
          <circle r={4.4 * props.size} fill="url(#sphere-shade)" className="shade" />
          {props.enemy && <circle r={5.6} className="owner-ring" stroke={props.color} />}
        </>
      ) : (
        <>
          {props.glow && <circle r={props.active ? 9 : 6} fill={`url(#halo-${props.faction})`} className="halo" />}
          {props.active && <circle r={7} className="pulse" stroke={props.color} />}
          {props.mo && <circle r={5} className="mo-ring" />}
          <circle r={(props.active ? 2.2 : 1.5) * props.size} className="core" fill={props.color} />
        </>
      )}
      {props.style === "spheres" && props.active && <CampaignBadge mo={Boolean(props.mo)} />}
      {props.pinned && <PinMark />}
    </g>
  );
}

export type Hover = { x: number; y: number } & (
  | { planet: Planet; landmark?: never }
  | { landmark: Landmark; planet?: never }
);

function resistance(regen: number): { label: string; color: string } {
  if (regen < 1) return { label: "Low", color: "var(--good)" };
  if (regen < 2.5) return { label: "Average", color: "var(--cta)" };
  return { label: "High", color: "var(--bad)" };
}

function HoverCard({ h, active }: { h: Hover; active: boolean }) {
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
  const tint = biomeTint(p.biome?.name);
  const enemy = p.event?.faction ?? (p.currentOwner === "Humans" ? undefined : p.currentOwner);
  const anomaly = anomalyOf(p);
  const kind = p.event
    ? "Defense"
    : active
      ? "Liberation"
      : anomaly
        ? anomaly === "wormhole"
          ? "Wormhole"
          : "Black hole"
        : p.currentOwner === "Humans"
          ? "Secured"
          : "Enemy held";
  const pct = enemy ? trackedPercent(p) : 100;
  const regen = regenPctPerHour(p);
  const res = resistance(regen);
  const style = {
    ...place,
    "--sky": tint.sky,
    "--ground": tint.ground,
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

export function GalaxyMap({ onHoverSector }: { onHoverSector?: (p: Planet | null) => void }) {
  const fresh = useFreshPlanets().data;
  const campaigns = useCampaigns();
  const mo = useMajorOrderPlanets();
  const { selectedPlanet, focusSeq, openOptions } = useUi();
  const overlay = useSettings((s) => s.mapOverlay);
  const planetStyle = useSettings((s) => s.mapPlanets);
  const quality = useSettings((s) => s.mapQuality);
  const pinned = useSettings((s) => s.pinned);
  const setSetting = useSettings((s) => s.set);
  const svgRef = useRef<SVGSVGElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const layerRef = useRef<SVGGElement>(null);
  const zoomRef = useRef<ZoomBehavior<SVGSVGElement, unknown> | null>(null);
  const transformRef = useRef<ZoomTransform>(zoomIdentity);
  const camRef = useRef<Camera>({ rot: 0, tilt: 0 });
  const freshRef = useRef(fresh);
  freshRef.current = fresh;
  const [cam, setCam] = useState<Camera>(camRef.current);
  const [hover, setHover] = useState<Hover | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const closeSettings = useCallback(() => setSettingsOpen(false), []);

  const stars = useMemo(() => backgroundStars(STAR_COUNT[quality]), [quality]);
  const active = useMemo(() => new Set((campaigns.data ?? []).map((c) => c.planet.index)), [campaigns.data]);
  const pinnedSet = useMemo(() => new Set(pinned), [pinned]);
  const loaded = Boolean(fresh);

  const redraw = useCallback(() => {
    if (canvasRef.current) drawStars(canvasRef.current, stars, transformRef.current, camRef.current);
  }, [stars]);

  const apply = useCallback(
    (t: ZoomTransform) => {
      const svg = svgRef.current;
      const layer = layerRef.current;
      if (!svg || !layer) return;
      transformRef.current = t;
      const { rot, tilt } = camRef.current;
      layer.setAttribute(
        "transform",
        `translate(${t.x},${t.y}) scale(${t.k}) scale(1,${Math.cos(tilt)}) rotate(${(rot * 180) / Math.PI})`,
      );
      // Markers counter-scale, un-rotate and un-squash through CSS variables, so camera moves never re-render React.
      layer.style.setProperty("--k", String(t.k));
      layer.style.setProperty("--unrot", `${(-rot * 180) / Math.PI}deg`);
      layer.style.setProperty("--unsquash", String(1 / Math.cos(tilt)));
      svg.classList.toggle("z-labels", t.k > 0.9);
      svg.classList.toggle("z-sectors", t.k > 1.6);
      svg.classList.toggle("z-planets", t.k > 3);
      scaleHatches(svg, t.k);
      redraw();
    },
    [redraw],
  );

  useEffect(() => {
    const svg = svgRef.current;
    if (!svg || !layerRef.current) return;
    const z = zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.5, 14])
      .on("zoom", (e) => {
        apply(e.transform as ZoomTransform);
        setHover(null);
      });
    zoomRef.current = z;
    select(svg).call(z).on("dblclick.zoom", null);
    fit();
    const ro = new ResizeObserver(() => fit());
    ro.observe(svg);
    return () => ro.disconnect();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [loaded]);

  useEffect(redraw, [redraw]);
  useEffect(() => {
    if (svgRef.current) scaleHatches(svgRef.current, transformRef.current.k);
  }, [overlay, fresh]);

  function centre(): Pt {
    const { width, height } = svgRef.current!.getBoundingClientRect();
    return [width / 2, height / 2];
  }

  function fit() {
    const svg = svgRef.current;
    if (!svg || !zoomRef.current) return;
    const { width, height } = svg.getBoundingClientRect();
    const k = Math.min(width, height) / (EXTENT * 2);
    select(svg).call(zoomRef.current.transform, zoomIdentity.translate(width / 2, height / 2).scale(k));
  }

  function zoomBy(f: number) {
    if (svgRef.current && zoomRef.current)
      select(svgRef.current).transition().duration(250).call(zoomRef.current.scaleBy, f);
  }

  /** Rotates and tilts about the middle of the screen, keeping whatever is there in place. */
  function setCamera(next: Partial<Camera>) {
    const svg = svgRef.current;
    if (!svg || !zoomRef.current) return;
    const t = transformRef.current;
    const c = centre();
    const world = fromLayer(t.invert(c) as Pt, camRef.current);
    camRef.current = {
      rot: next.rot ?? camRef.current.rot,
      tilt: Math.min(MAX_TILT, Math.max(0, next.tilt ?? camRef.current.tilt)),
    };
    setCam(camRef.current);
    const [x, y] = toLayer(world, camRef.current);
    const nt = zoomIdentity.translate(c[0] - t.k * x, c[1] - t.k * y).scale(t.k);
    select(svg).call(zoomRef.current.transform, nt);
    apply(nt);
  }

  // Fly to a planet chosen elsewhere ("Show on map") or from the map's own options ("Focus").
  useEffect(() => {
    const svg = svgRef.current;
    const p = selectedPlanet != null && freshRef.current ? byIndex(freshRef.current).get(selectedPlanet) : undefined;
    if (!svg || !zoomRef.current || !p || focusSeq === 0) return;
    const { width, height } = svg.getBoundingClientRect();
    const [x, y] = toLayer(project(p.position), camRef.current);
    const k = Math.max(3, Math.min(width, height) / 280);
    select(svg)
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
    if (!d) return;
    setCamera({ rot: d.cam.rot + (e.clientX - d.x) * DRAG_ROT, tilt: d.cam.tilt - (e.clientY - d.y) * DRAG_TILT });
  };
  const endDrag = () => (drag.current = null);

  if (!fresh) return <Loading what="galactic war data" />;

  const nextOverlay = OVERLAYS[(OVERLAYS.indexOf(overlay) + 1) % OVERLAYS.length];
  const glow = quality !== "min";
  const spheres = planetStyle === "spheres";
  const leave = () => setHover(null);

  return (
    <div className={`map-wrap q-${quality}`}>
      <canvas ref={canvasRef} className="starfield-canvas" aria-hidden />
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
          onHoverSector?.(null);
        }}
      >
        <HaloDefs />
        <g ref={layerRef}>
          {overlay !== "galaxy" && <Territory planets={fresh} mode={overlay} glow={glow} onHover={onHoverSector} />}
          <Lanes planets={fresh} mode={overlay === "sectors" ? "front" : overlay === "galaxy" ? "bright" : "dim"} />
          <AttackLines planets={fresh} active={active} />
          <g className="planets">
            {fresh.map((p) => {
              const [x, y] = project(p.position);
              const isActive = active.has(p.index);
              const isMo = mo.has(p.index);
              const isPinned = pinnedSet.has(p.index);
              const faction = p.event?.faction ?? p.currentOwner;
              const color = factionColor(faction);
              const cls = [
                "planet",
                isActive && "active",
                isMo && "mo",
                isPinned && "pinned",
                selectedPlanet === p.index && "selected",
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
                  onClick={(e) => openOptions({ planet: p.index, x: e.clientX, y: e.clientY })}
                  onPointerEnter={(e) => {
                    setHover({ planet: p, x: e.clientX, y: e.clientY });
                    onHoverSector?.(p);
                  }}
                  onPointerLeave={leave}
                >
                  <g className="upright">
                    <PlanetBody
                      style={planetStyle}
                      glow={glow}
                      faction={faction}
                      color={color}
                      fill={p.index === 0 ? "#3f8cc2" : sphereColor(biomeTint(p.biome?.name).ground)}
                      size={p.index === 0 ? 1.6 : 1}
                      anomaly={anomalyOf(p)}
                      active={isActive}
                      mo={isMo}
                      enemy={p.currentOwner !== "Humans" || Boolean(p.event)}
                      pinned={isPinned}
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
                  onPointerEnter={(e) => setHover({ landmark: l, x: e.clientX, y: e.clientY })}
                  onPointerLeave={leave}
                >
                  <g className="upright">
                    <PlanetBody
                      style={planetStyle}
                      glow={glow}
                      faction="Humans"
                      color="#c98a6a"
                      fill="#b5532f"
                      size={spheres ? 0.85 : 0.9}
                    />
                    <text className="planet-label">{l.name}</text>
                  </g>
                </g>
              );
            })}
          </g>
        </g>
      </svg>
      <div className="map-rail">
        <HudButton icon={ZoomIn} label="Zoom in" keyHint="+" onClick={() => zoomBy(1.5)} />
        <HudButton icon={ZoomOut} label="Zoom out" keyHint="-" onClick={() => zoomBy(1 / 1.5)} />
        <HudButton icon={Maximize} label="Fit galaxy" keyHint="0" onClick={fit} />
        <button
          className={`hud-btn icon map-compass ${cam.rot || cam.tilt ? "on" : ""}`}
          onClick={() => setCamera({ rot: 0, tilt: 0 })}
          aria-label="Reset rotation and tilt"
          title="Reset rotation and tilt"
        >
          <Compass size={18} strokeWidth={1.75} style={{ transform: `rotate(${cam.rot - Math.PI / 4}rad)` }} />
        </button>
        <HudButton
          icon={Rotate3d}
          label={cam.tilt ? "Flatten the map" : "Tilt the map"}
          keyHint="R / F"
          active={cam.tilt > 0}
          onClick={() => setCamera({ tilt: cam.tilt > 0 ? 0 : TILT_PRESET })}
        />
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
      {hover && <HoverCard h={hover} active={hover.planet ? active.has(hover.planet.index) : false} />}
    </div>
  );
}
