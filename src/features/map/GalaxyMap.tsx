import { select } from "d3-selection";
import { zoom, zoomIdentity, type ZoomBehavior, type ZoomTransform } from "d3-zoom";
import { Layers, Maximize, MousePointerClick, Settings, ZoomIn, ZoomOut } from "lucide-react";
import { memo, useCallback, useEffect, useMemo, useRef, useState, type CSSProperties } from "react";
import { useCampaigns, useFreshPlanets, usePlanets } from "../../api/queries";
import type { Planet } from "../../api/types";
import { biomeTint, regenPctPerHour } from "../../domain/biomes";
import { FACTIONS, factionColor } from "../../domain/factions";
import { fmtPct } from "../../domain/format";
import { buildRegions, buildSectors, GALAXY_SCALE, project, supplyLines } from "../../domain/galaxy";
import { byIndex } from "../../domain/planets";
import { liberationPercent, trackedPercent } from "../../domain/progress";
import { useSettings, useUi, type MapOverlay, type MapQuality } from "../../state";
import { HudButton, Loading, Meter } from "../../ui/primitives";
import { useMajorOrderPlanets } from "../planets/useFronts";
import "./map.css";

const EXTENT = GALAXY_SCALE * 1.1;
const STAR_COUNT: Record<MapQuality, number> = { max: 4200, normal: 2600, min: 900 };
const OVERLAYS: MapOverlay[] = ["regions", "sectors", "galaxy"];

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

function drawStars(canvas: HTMLCanvasElement, stars: Star[], t: ZoomTransform) {
  const dpr = window.devicePixelRatio || 1;
  const { width, height } = canvas.getBoundingClientRect();
  if (canvas.width !== Math.round(width * dpr)) canvas.width = Math.round(width * dpr);
  if (canvas.height !== Math.round(height * dpr)) canvas.height = Math.round(height * dpr);
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, width, height);

  // Core glow around Super Earth.
  const [cx, cy] = [t.x, t.y];
  const glow = ctx.createRadialGradient(cx, cy, 0, cx, cy, 140 * t.k);
  glow.addColorStop(0, "rgba(120, 200, 255, 0.28)");
  glow.addColorStop(0.35, "rgba(70, 164, 255, 0.08)");
  glow.addColorStop(1, "rgba(70, 164, 255, 0)");
  ctx.fillStyle = glow;
  ctx.fillRect(0, 0, width, height);

  // Stars drift a little slower than the map, so zooming has depth.
  const k = Math.pow(t.k, 0.85);
  for (const s of stars) {
    const x = s.x * k + t.x;
    const y = s.y * k + t.y;
    if (x < -2 || y < -2 || x > width + 2 || y > height + 2) continue;
    ctx.globalAlpha = s.o;
    ctx.fillStyle = s.c;
    ctx.fillRect(x, y, s.s, s.s);
  }
  ctx.globalAlpha = 1;

  // Faint polar grid, as on a war table.
  ctx.strokeStyle = "rgba(120, 170, 230, 0.06)";
  ctx.lineWidth = 1;
  for (const r of [120, 240, 360, 480]) {
    ctx.beginPath();
    ctx.arc(cx, cy, r * t.k, 0, Math.PI * 2);
    ctx.stroke();
  }
}

const Regions = memo(function Regions({ planets, glow }: { planets: Planet[]; glow: boolean }) {
  const regions = useMemo(() => buildRegions(planets), [planets]);
  return (
    <g className="regions">
      {regions.map((r) => {
        const c = factionColor(r.owner);
        return (
          <g key={r.name}>
            <path d={r.path} fill={c} fillOpacity={0.05} stroke="none" />
            {glow && <path d={r.path} className="region-glow" stroke={c} />}
            <path d={r.path} className="region-line" stroke={c} />
          </g>
        );
      })}
    </g>
  );
});

const Sectors = memo(function Sectors({ planets }: { planets: Planet[] }) {
  const sectors = useMemo(() => buildSectors(planets), [planets]);
  return (
    <g className="sectors">
      <defs>
        {sectors.map((s, i) => (
          <clipPath key={s.name} id={`sec-${i}`}>
            <path d={s.path} />
          </clipPath>
        ))}
      </defs>
      {sectors.map((s, i) => {
        const c = factionColor(s.owner);
        return (
          <g key={s.name}>
            {/* Doubled stroke clipped to the shape draws an inset outline, leaving gaps between sectors. */}
            <path
              d={s.path}
              fill={c}
              fillOpacity={0.04}
              stroke={c}
              strokeWidth={4}
              clipPath={`url(#sec-${i})`}
              className="sector-outline"
            />
            <text x={s.center[0]} y={s.center[1]} className="sector-label">
              {s.name}
            </text>
          </g>
        );
      })}
    </g>
  );
});

const Lanes = memo(function Lanes({ planets, bright }: { planets: Planet[]; bright: boolean }) {
  const lookup = byIndex(planets);
  const lines = useMemo(() => supplyLines(planets), [planets]);
  return (
    <g className={`lanes ${bright ? "bright" : ""}`}>
      {lines.map(([a, b]) => {
        const pa = lookup.get(a);
        const pb = lookup.get(b);
        if (!pa || !pb) return null;
        const [x1, y1] = project(pa.position);
        const [x2, y2] = project(pb.position);
        return <line key={`${a}-${b}`} x1={x1} y1={y1} x2={x2} y2={y2} />;
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
    </defs>
  );
});

export interface Hover {
  planet: Planet;
  x: number;
  y: number;
}

function resistance(regen: number): { label: string; color: string } {
  if (regen < 1) return { label: "Low", color: "var(--good)" };
  if (regen < 2.5) return { label: "Average", color: "var(--cta)" };
  return { label: "High", color: "var(--bad)" };
}

function HoverCard({ h, active }: { h: Hover; active: boolean }) {
  const p = h.planet;
  const tint = biomeTint(p.biome?.name);
  const enemy = p.event?.faction ?? (p.currentOwner === "Humans" ? undefined : p.currentOwner);
  const kind = p.event ? "Defense" : active ? "Liberation" : p.currentOwner === "Humans" ? "Secured" : "Enemy held";
  const pct = enemy ? trackedPercent(p) : 100;
  const regen = regenPctPerHour(p);
  const res = resistance(regen);
  const style = {
    left: Math.min(h.x + 18, window.innerWidth - 300),
    top: Math.max(8, h.y - 40),
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

export function GalaxyMap({ onHoverSector }: { onHoverSector?: (p: Planet | null) => void }) {
  const planets = usePlanets();
  const fresh = useFreshPlanets().data;
  const campaigns = useCampaigns();
  const mo = useMajorOrderPlanets();
  const { selectedPlanet, openOptions, setView } = useUi();
  const overlay = useSettings((s) => s.mapOverlay);
  const quality = useSettings((s) => s.mapQuality);
  const setSetting = useSettings((s) => s.set);
  const svgRef = useRef<SVGSVGElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const layerRef = useRef<SVGGElement>(null);
  const zoomRef = useRef<ZoomBehavior<SVGSVGElement, unknown> | null>(null);
  const transformRef = useRef<ZoomTransform>(zoomIdentity);
  const [hover, setHover] = useState<Hover | null>(null);

  const stars = useMemo(() => backgroundStars(STAR_COUNT[quality]), [quality]);
  const active = useMemo(() => new Set((campaigns.data ?? []).map((c) => c.planet.index)), [campaigns.data]);
  const loaded = Boolean(fresh);

  const redraw = useCallback(() => {
    if (canvasRef.current) drawStars(canvasRef.current, stars, transformRef.current);
  }, [stars]);

  useEffect(() => {
    const svg = svgRef.current;
    const layer = layerRef.current;
    if (!svg || !layer) return;
    const z = zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.5, 14])
      .on("zoom", (e) => {
        const t = e.transform as ZoomTransform;
        transformRef.current = t;
        layer.setAttribute("transform", `translate(${t.x},${t.y}) scale(${t.k})`);
        // Labels and markers counter-scale through a CSS variable, so zooming doesn't re-render React.
        layer.style.setProperty("--k", String(t.k));
        svg.classList.toggle("z-labels", t.k > 0.9);
        svg.classList.toggle("z-sectors", t.k > 1.6);
        svg.classList.toggle("z-planets", t.k > 3);
        redraw();
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

  // Fly to a planet chosen elsewhere ("Show on map").
  useEffect(() => {
    const svg = svgRef.current;
    const p = selectedPlanet != null && fresh ? byIndex(fresh).get(selectedPlanet) : undefined;
    if (!svg || !zoomRef.current || !p) return;
    const { width, height } = svg.getBoundingClientRect();
    const [x, y] = project(p.position);
    const k = Math.max(3, Math.min(width, height) / 280);
    select(svg)
      .transition()
      .duration(700)
      .call(zoomRef.current.transform, zoomIdentity.translate(width / 2 - x * k, height / 2 - y * k).scale(k));
  }, [selectedPlanet, fresh]);

  // H toggles nothing here; the page owns the controls panel. Keyboard zoom: + / - / 0.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.target as HTMLElement)?.closest("input, textarea")) return;
      if (e.key === "+" || e.key === "=") zoomBy(1.5);
      if (e.key === "-") zoomBy(1 / 1.5);
      if (e.key === "0") fit();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  if (!fresh || !planets.data) return <Loading what="galactic war data" />;

  const nextOverlay = OVERLAYS[(OVERLAYS.indexOf(overlay) + 1) % OVERLAYS.length];
  const glow = quality !== "min";

  return (
    <div className={`map-wrap q-${quality}`}>
      <canvas ref={canvasRef} className="starfield-canvas" aria-hidden />
      <svg
        ref={svgRef}
        className={`galaxy ov-${overlay}`}
        role="img"
        aria-label="Galactic war map"
        onPointerLeave={() => {
          setHover(null);
          onHoverSector?.(null);
        }}
      >
        <HaloDefs />
        <g ref={layerRef}>
          {overlay === "regions" && <Regions planets={fresh} glow={glow} />}
          {overlay === "sectors" && <Sectors planets={planets.data} />}
          <Lanes planets={fresh} bright={overlay === "galaxy"} />
          <AttackLines planets={fresh} active={active} />
          <g className="planets">
            {fresh.map((p) => {
              const [x, y] = project(p.position);
              const isActive = active.has(p.index);
              const isMo = mo.has(p.index);
              const color = factionColor(p.event?.faction ?? p.currentOwner);
              const cls = [
                "planet",
                isActive && "active",
                isMo && "mo",
                selectedPlanet === p.index && "selected",
                p.currentOwner === "Humans" && "se",
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
                  onPointerLeave={() => setHover(null)}
                >
                  <g className="pbody">
                    <circle r={10} className="hit" />
                    {glow && (
                      <circle
                        r={isActive ? 9 : 6}
                        fill={`url(#halo-${p.event?.faction ?? p.currentOwner})`}
                        className="halo"
                      />
                    )}
                    {isActive && <circle r={7} className="pulse" stroke={color} />}
                    {isMo && <circle r={5} className="mo-ring" />}
                    <circle r={isActive ? 2.2 : 1.5} className="core" fill={color} />
                  </g>
                  <text y={-9} className="planet-label">
                    {p.name}
                  </text>
                  {isActive && (
                    <text y={12} className="planet-tag" fill={color}>
                      {p.event ? "Defend" : "Liberate"}
                    </text>
                  )}
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
        <HudButton
          icon={Layers}
          label={`Map overlay: ${overlay} (next: ${nextOverlay})`}
          onClick={() => setSetting("mapOverlay", nextOverlay)}
        />
        <HudButton icon={Settings} label="Map settings" onClick={() => setView("settings")} />
      </div>
      {hover && <HoverCard h={hover} active={active.has(hover.planet.index)} />}
    </div>
  );
}
