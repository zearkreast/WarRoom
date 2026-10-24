import { memo, useMemo } from "react";
import type { Faction, Planet } from "../../api/types";
import { FACTIONS } from "../../domain/factions";
import { project, sectorTerritory, type PolarSectors } from "../../domain/galaxy";
import { ownershipKey } from "../../domain/planets";
import "./layers.css";

const ENEMIES: Faction[] = ["Terminids", "Automaton", "Illuminate"];

/**
 * Enemy territory as the game's war table draws it: only enemy sectors are painted, each a hatched
 * tile set apart by a dark seam, faint where Super Earth still holds some of its planets (contested).
 * The galaxy map and the Theater of War share it.
 */
export const SectorLayer = memo(function SectorLayer({
  geo,
  planets,
  id,
  hatch,
}: {
  geo: PolarSectors;
  planets: Planet[];
  /** Prefix for pattern ids, unique per SVG. */
  id: string;
  /** Hatch spacing in map units. */
  hatch: number;
}) {
  const owners = ownershipKey(planets);
  // eslint-disable-next-line react-hooks/exhaustive-deps
  const territory = useMemo(() => sectorTerritory(planets), [owners]);
  const shapes = useMemo(() => {
    let grid = "";
    let seams = "";
    const fronts = new Map<Faction, string>();
    const cells = new Map<Faction, string>();
    const splits = new Map<Faction, string>();
    const add = (m: Map<Faction, string>, f: Faction, d: string) => m.set(f, (m.get(f) ?? "") + d);
    for (const e of geo.edges) {
      const ta = territory.get(e.a) ?? null;
      const tb = e.b ? (territory.get(e.b) ?? null) : null;
      if (!ta && !tb) {
        grid += e.d;
        continue;
      }
      if (ta && tb && ta.faction === tb.faction) {
        seams += e.d;
        add(cells, ta.faction, e.d);
        if (ta.held !== tb.held) add(splits, ta.faction, e.d);
        continue;
      }
      for (const t of [ta, tb]) if (t) add(fronts, t.faction, e.d);
    }
    const held = new Map<Faction, string>();
    const contested = new Map<Faction, string>();
    for (const [s, d] of geo.fills) {
      const t = territory.get(s);
      if (t) add(t.held ? held : contested, t.faction, d);
    }
    return { grid, seams, fronts, cells, splits, held, contested };
  }, [geo, territory]);

  return (
    <g className="territory">
      <defs>
        {ENEMIES.flatMap((f) =>
          (["held", "contested"] as const).map((kind) => (
            <pattern
              key={`${f}-${kind}`}
              id={`${id}-hatch-${f}-${kind}`}
              className="hatch"
              width={hatch}
              height={hatch}
              patternUnits="userSpaceOnUse"
              patternTransform="rotate(45)"
            >
              <rect width={hatch} height={hatch} fill={FACTIONS[f].color} fillOpacity={kind === "held" ? 0.3 : 0.08} />
              <line
                x1={hatch * 0.15}
                y1={0}
                x2={hatch * 0.15}
                y2={hatch}
                stroke={FACTIONS[f].color}
                strokeOpacity={kind === "held" ? 0.75 : 0.32}
                strokeWidth={hatch * 0.28}
              />
            </pattern>
          )),
        )}
      </defs>
      <path d={shapes.grid} className="sg-grid" />
      {[...shapes.held].map(([f, d]) => (
        <path key={`h-${f}`} d={d} className="tf-fill" fill={`url(#${id}-hatch-${f}-held)`} />
      ))}
      {[...shapes.contested].map(([f, d]) => (
        <path key={`c-${f}`} d={d} className="tf-fill" fill={`url(#${id}-hatch-${f}-contested)`} />
      ))}
      <path d={shapes.seams} className="tf-seam" />
      {[...shapes.cells].map(([f, d]) => (
        <path key={`e-${f}`} d={d} className="tf-cell" style={{ color: FACTIONS[f].color }} />
      ))}
      {[...shapes.splits].map(([f, d]) => (
        <path key={`s-${f}`} d={d} className="tf-split" style={{ color: FACTIONS[f].color }} />
      ))}
      {[...shapes.fronts].map(([f, d]) => (
        <path key={`f-${f}`} d={d} className="tf-front" style={{ color: FACTIONS[f].color }} />
      ))}
      <circle r={geo.rim} className="sg-rim" />
    </g>
  );
});

const NOISE_SIZE = 256;

/** Tileable fractal value noise as a PNG data URL, coloured like the Gloom; made once per session. */
let noiseUrl: string | null = null;
function gloomTexture(): string {
  if (noiseUrl) return noiseUrl;
  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = NOISE_SIZE;
  const ctx = canvas.getContext("2d");
  if (!ctx) return "";
  let seed = 11;
  const rnd = () => (seed = (seed * 16807) % 2147483647) / 2147483647;
  const field = new Float32Array(NOISE_SIZE * NOISE_SIZE);
  let amp = 1;
  for (let cell = 64; cell >= 4; cell /= 2) {
    const n = NOISE_SIZE / cell;
    const grid = Array.from({ length: n * n }, rnd);
    const at = (gx: number, gy: number) => grid[((gy + n) % n) * n + ((gx + n) % n)];
    for (let y = 0; y < NOISE_SIZE; y++)
      for (let x = 0; x < NOISE_SIZE; x++) {
        const gx = Math.floor(x / cell);
        const gy = Math.floor(y / cell);
        const fx = (x % cell) / cell;
        const fy = (y % cell) / cell;
        const sx = fx * fx * (3 - 2 * fx);
        const sy = fy * fy * (3 - 2 * fy);
        const top = at(gx, gy) + (at(gx + 1, gy) - at(gx, gy)) * sx;
        const bottom = at(gx, gy + 1) + (at(gx + 1, gy + 1) - at(gx, gy + 1)) * sx;
        field[y * NOISE_SIZE + x] += (top + (bottom - top) * sy) * amp;
      }
    amp /= 2;
  }
  const img = ctx.createImageData(NOISE_SIZE, NOISE_SIZE);
  for (let i = 0; i < field.length; i++) {
    const v = Math.min(1, Math.max(0, (field[i] / 1.9 - 0.2) * 1.8));
    img.data[i * 4] = 232;
    img.data[i * 4 + 1] = 170 + v * 50;
    img.data[i * 4 + 2] = 60 + v * 50;
    img.data[i * 4 + 3] = Math.round(Math.pow(v, 1.3) * 240);
  }
  ctx.putImageData(img, 0, 0);
  return (noiseUrl = canvas.toDataURL());
}

/** Map units around a Gloom planet that its cloud covers. */
const GLOOM_REACH = 78;

/**
 * The Gloom: a drifting amber cloud over the planets the game marks as in or on the edge of it, on
 * every overlay. The texture is a pre-rendered image, so panning doesn't re-run a filter.
 */
export const GloomLayer = memo(function GloomLayer({
  planets,
  gloom,
  id,
}: {
  planets: Planet[];
  gloom: Set<number>;
  id: string;
}) {
  const spots = useMemo(
    () => planets.filter((p) => gloom.has(p.index)).map((p) => project(p.position)),
    [planets, gloom],
  );
  const box = useMemo(() => {
    if (!spots.length) return null;
    const xs = spots.map(([x]) => x);
    const ys = spots.map(([, y]) => y);
    const pad = GLOOM_REACH * 1.2;
    return { x: Math.min(...xs) - pad, y: Math.min(...ys) - pad, w: Math.max(...xs) - Math.min(...xs) + 2 * pad, h: Math.max(...ys) - Math.min(...ys) + 2 * pad };
  }, [spots]);
  const texture = useMemo(() => (box ? gloomTexture() : ""), [box]);
  if (!box || !texture) return null;
  return (
    <g className="gloom" aria-hidden>
      <defs>
        <radialGradient id={`${id}-gloom-spot`}>
          <stop offset="0%" stopColor="#fff" stopOpacity={1} />
          <stop offset="55%" stopColor="#fff" stopOpacity={0.75} />
          <stop offset="100%" stopColor="#fff" stopOpacity={0} />
        </radialGradient>
        <mask id={`${id}-gloom-mask`} maskUnits="userSpaceOnUse" x={box.x} y={box.y} width={box.w} height={box.h}>
          {spots.map(([x, y], i) => (
            <circle key={i} cx={x} cy={y} r={GLOOM_REACH} fill={`url(#${id}-gloom-spot)`} />
          ))}
        </mask>
        <pattern id={`${id}-gloom-tex`} width={NOISE_SIZE} height={NOISE_SIZE} patternUnits="userSpaceOnUse">
          <image href={texture} width={NOISE_SIZE} height={NOISE_SIZE} />
        </pattern>
      </defs>
      <rect
        x={box.x}
        y={box.y}
        width={box.w}
        height={box.h}
        fill={`url(#${id}-gloom-tex)`}
        mask={`url(#${id}-gloom-mask)`}
      />
    </g>
  );
});
