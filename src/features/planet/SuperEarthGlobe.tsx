import { geoDistance, geoGraticule10, geoOrthographic, geoPath, type GeoPermissibleObjects } from "d3-geo";
import { useEffect, useRef } from "react";
import { feature } from "topojson-client";
import type { GeometryCollection, Topology } from "topojson-specification";
import { SUPER_EARTH_CITIES } from "../../data/cities";

/** Degrees the globe leans, north pole toward the viewer. */
const TILT = -20;
/** Degrees of longitude turned per second. */
const SPEED = 5;
/** Europe and Africa face the viewer first. */
const START = -10;

const CITIES = Object.values(SUPER_EARTH_CITIES);

let land: Promise<GeoPermissibleObjects> | null = null;
const loadLand = () =>
  (land ??= import("world-atlas/land-110m.json").then((m) => {
    const world = m.default as unknown as Topology<{ land: GeometryCollection }>;
    return feature(world, world.objects.land);
  }));

/** Super Earth drawn as the Earth it is, turning, with its sector capitals lit or struck off. */
export function SuperEarthGlobe({ r }: { r: number }) {
  const landEl = useRef<SVGPathElement>(null);
  const gridEl = useRef<SVGPathElement>(null);
  const cityEls = useRef<(SVGGElement | null)[]>([]);

  useEffect(() => {
    let alive = true;
    let frame = 0;
    const projection = geoOrthographic().scale(r).translate([0, 0]).clipAngle(90);
    const path = geoPath(projection);
    const grid = geoGraticule10();
    loadLand().then((shape) => {
      if (!alive) return;
      const draw = (lon: number) => {
        projection.rotate([lon, TILT]);
        landEl.current?.setAttribute("d", path(shape) ?? "");
        gridEl.current?.setAttribute("d", path(grid) ?? "");
        CITIES.forEach((c, i) => {
          const el = cityEls.current[i];
          const at = projection(c.at);
          if (!el || !at) return;
          el.setAttribute("transform", `translate(${at[0].toFixed(1)},${at[1].toFixed(1)})`);
          el.style.visibility = geoDistance(c.at, [-lon, -TILT]) < Math.PI / 2 ? "" : "hidden";
        });
      };
      if (matchMedia("(prefers-reduced-motion: reduce)").matches) return draw(START);
      const t0 = performance.now();
      const tick = (t: number) => {
        draw(START + ((t - t0) / 1000) * SPEED);
        frame = requestAnimationFrame(tick);
      };
      frame = requestAnimationFrame(tick);
    });
    return () => {
      alive = false;
      cancelAnimationFrame(frame);
    };
  }, [r]);

  return (
    <g className="se-globe">
      <defs>
        <radialGradient id="se-ocean" cx="38%" cy="32%" r="75%">
          <stop offset="0%" stopColor="#2a6a9c" />
          <stop offset="100%" stopColor="#0a2340" />
        </radialGradient>
      </defs>
      <circle r={r} fill="url(#se-ocean)" />
      <path ref={gridEl} className="se-grid" />
      <path ref={landEl} className="se-land" />
      {CITIES.map((c, i) => (
        <g
          key={c.at.join()}
          ref={(el) => {
            cityEls.current[i] = el;
          }}
          className={`se-city ${c.state}`}
          style={{ visibility: "hidden" }}
        >
          {c.state === "destroyed" ? <path d="M-3 -3L3 3M3 -3L-3 3" /> : <circle r={2.6} />}
        </g>
      ))}
    </g>
  );
}
