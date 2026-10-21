import { select } from "d3-selection";
import { zoom, zoomIdentity, type ZoomBehavior } from "d3-zoom";
import { memo, useEffect, useMemo, useRef } from "react";
import { useCampaigns, usePlanets } from "../../api/queries";
import type { Planet } from "../../api/types";
import { factionColor } from "../../domain/factions";
import { buildSectors, GALAXY_SCALE, project, supplyLines } from "../../domain/galaxy";
import { byIndex } from "../../domain/planets";
import { useUi } from "../../state";
import { Loading } from "../../ui/primitives";
import { useMajorOrderPlanets } from "../planets/useFronts";
import "./map.css";

const EXTENT = GALAXY_SCALE * 1.1;

/** Deterministic background stars so the field doesn't reshuffle on every render. */
function backgroundStars(n: number) {
  let seed = 7;
  const rnd = () => ((seed = (seed * 16807) % 2147483647) / 2147483647);
  return Array.from({ length: n }, () => {
    const a = rnd() * Math.PI * 2;
    const r = Math.sqrt(rnd()) * EXTENT * 1.3;
    return { x: Math.cos(a) * r, y: Math.sin(a) * r, s: rnd() * 1.1 + 0.2, o: rnd() * 0.5 + 0.15 };
  });
}

const STARS = backgroundStars(900);

const Backdrop = memo(function Backdrop() {
  return (
    <g className="backdrop">
      {STARS.map((s, i) => (
        <circle key={i} cx={s.x} cy={s.y} r={s.s} fill="#fff" opacity={s.o} />
      ))}
      {[100, 200, 300, 400, 500].map((r) => (
        <circle key={r} r={r} className="grid-ring" />
      ))}
      {Array.from({ length: 12 }, (_, i) => {
        const a = (i / 12) * Math.PI * 2;
        return <line key={i} x1={0} y1={0} x2={Math.cos(a) * 520} y2={Math.sin(a) * 520} className="grid-ring" />;
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
            <path d={s.path} fill={c} fillOpacity={0.05} stroke={c} strokeWidth={4} clipPath={`url(#sec-${i})`} className="sector-outline" />
            <text x={s.center[0]} y={s.center[1]} className="sector-label">
              {s.name}
            </text>
          </g>
        );
      })}
    </g>
  );
});

const Lanes = memo(function Lanes({ planets }: { planets: Planet[] }) {
  const lookup = byIndex(planets);
  const lines = useMemo(() => supplyLines(planets), [planets]);
  return (
    <g className="lanes">
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
            return <line key={`${p.index}>${t}`} x1={x1} y1={y1} x2={x2} y2={y2} stroke={factionColor(p.currentOwner)} />;
          }),
      )}
    </g>
  );
}

export function GalaxyMap() {
  const planets = usePlanets();
  const campaigns = useCampaigns();
  const mo = useMajorOrderPlanets();
  const { selectedPlanet, select: selectPlanet } = useUi();
  const svgRef = useRef<SVGSVGElement>(null);
  const layerRef = useRef<SVGGElement>(null);
  const zoomRef = useRef<ZoomBehavior<SVGSVGElement, unknown> | null>(null);

  // Campaign planets carry fresher state; overlay them onto the full list.
  const merged = useMemo(() => {
    if (!planets.data) return undefined;
    const fresh = new Map((campaigns.data ?? []).map((c) => [c.planet.index, c.planet]));
    return planets.data.map((p) => fresh.get(p.index) ?? p);
  }, [planets.data, campaigns.data]);
  const active = useMemo(() => new Set((campaigns.data ?? []).map((c) => c.planet.index)), [campaigns.data]);

  useEffect(() => {
    const svg = svgRef.current;
    const layer = layerRef.current;
    if (!svg || !layer) return;
    const z = zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.6, 12])
      .on("zoom", (e) => {
        const { x, y, k } = e.transform;
        layer.setAttribute("transform", `translate(${x},${y}) scale(${k})`);
        // Labels counter-scale through a CSS variable, so zooming doesn't re-render React.
        layer.style.setProperty("--k", String(k));
        svg.classList.toggle("z-labels", k > 0.55);
        svg.classList.toggle("z-sectors", k > 1.4);
        svg.classList.toggle("z-planets", k > 3);
      });
    zoomRef.current = z;
    select(svg).call(z).on("dblclick.zoom", null);
    fit();
    const ro = new ResizeObserver(() => fit());
    ro.observe(svg);
    return () => ro.disconnect();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [Boolean(merged)]);

  /** Width covered by the desktop sidebar, so the galaxy centers in the visible area. */
  function leftInset(width: number) {
    return width > 900 ? 352 : 0;
  }

  function fit() {
    const svg = svgRef.current;
    if (!svg || !zoomRef.current) return;
    const { width, height } = svg.getBoundingClientRect();
    const inset = leftInset(width);
    const k = Math.min(width - inset, height) / (EXTENT * 2);
    select(svg).call(zoomRef.current.transform, zoomIdentity.translate(inset + (width - inset) / 2, height / 2).scale(k));
  }

  function zoomBy(f: number) {
    if (svgRef.current && zoomRef.current) select(svgRef.current).transition().duration(250).call(zoomRef.current.scaleBy, f);
  }

  // Fly to a planet selected from elsewhere in the app.
  useEffect(() => {
    const svg = svgRef.current;
    const p = selectedPlanet != null && merged ? byIndex(merged).get(selectedPlanet) : undefined;
    if (!svg || !zoomRef.current || !p) return;
    const { width, height } = svg.getBoundingClientRect();
    const [x, y] = project(p.position);
    const k = Math.max(3, Math.min(width, height) / 300);
    // Center between the sidebar and the drawer on desktop; above the bottom sheet on mobile.
    const cx = width > 900 ? leftInset(width) + (width - leftInset(width) - 424) / 2 : width / 2;
    const cy = width > 900 ? height / 2 : height * 0.28;
    select(svg)
      .transition()
      .duration(600)
      .call(zoomRef.current.transform, zoomIdentity.translate(cx - x * k, cy - y * k).scale(k));
  }, [selectedPlanet, merged]);

  if (!merged) return <Loading what="galactic war data" />;

  return (
    <div className="map-wrap">
      <svg ref={svgRef} className="galaxy" onClick={(e) => e.target === e.currentTarget && selectPlanet(null)}>
        <defs>
          <filter id="glow" x="-200%" y="-200%" width="500%" height="500%">
            <feGaussianBlur stdDeviation="2.5" result="b" />
            <feMerge>
              <feMergeNode in="b" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
        </defs>
        <g ref={layerRef}>
          <Backdrop />
          <Sectors planets={planets.data!} />
          <Lanes planets={merged} />
          <AttackLines planets={merged} active={active} />
          <g className="planets">
            {merged.map((p) => {
              const [x, y] = project(p.position);
              const isActive = active.has(p.index);
              const color = factionColor(p.currentOwner);
              const cls = ["planet", isActive && "active", mo.has(p.index) && "mo", selectedPlanet === p.index && "selected"]
                .filter(Boolean)
                .join(" ");
              return (
                <g key={p.index} className={cls} transform={`translate(${x},${y})`} onClick={() => selectPlanet(p.index)}>
                  <circle r={9} className="hit" />
                  {isActive && <circle r={7} className="pulse" stroke={p.event ? "var(--f-terminids)" : color} />}
                  {mo.has(p.index) && <circle r={5.5} className="mo-ring" />}
                  <circle r={isActive ? 3.2 : 2} fill={color} filter={isActive ? "url(#glow)" : undefined} className="core" />
                  <text y={-8} className="planet-label">
                    {p.name}
                  </text>
                </g>
              );
            })}
          </g>
        </g>
      </svg>
      <div className="map-controls">
        <button className="hud-btn" onClick={() => zoomBy(1.5)} aria-label="Zoom in">+</button>
        <button className="hud-btn" onClick={() => zoomBy(1 / 1.5)} aria-label="Zoom out">−</button>
        <button className="hud-btn" onClick={fit} aria-label="Fit galaxy">⤢</button>
      </div>
      <div className="map-hint caps muted">Drag to pan · scroll to zoom · click a planet</div>
    </div>
  );
}
