import { useMemo } from "react";
import { useWarSamples } from "../../api/warSamples";
import type { Section } from "../../data/gww";
import { fmtCompact } from "../../domain/format";
import { averageReadings } from "../../domain/frontActivity";

type Node = { id: string; x: number; y: number; side: "se" | "enemy" };

function Planet({ p, ring }: { p: Node; ring?: boolean }) {
  return (
    <g transform={`translate(${p.x} ${p.y})`} className={`gf-planet ${p.side}`}>
      {ring && <circle className="gf-ring" r="17" />}
      <circle r="11" />
      <text y="4">{p.id}</text>
    </g>
  );
}

const link = (a: Node, b: Node, key: string, className = "gf-link") => (
  <line key={key} className={className} x1={a.x} y1={a.y} x2={b.x} y2={b.y} />
);

/** A line from `a` towards `b` stopping short of both planets, for attack arrows. */
function Attack({ a, b, side }: { a: Node; b: Node; side: Node["side"] }) {
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const d = Math.hypot(dx, dy);
  const [ux, uy] = [dx / d, dy / d];
  return (
    <line
      className={`gf-attack ${side}`}
      x1={a.x + ux * 16}
      y1={a.y + uy * 16}
      x2={b.x - ux * 21}
      y2={b.y - uy * 21}
      markerEnd={`url(#gf-head-${side})`}
    />
  );
}

function Heads() {
  return (
    <defs>
      {(["se", "enemy"] as const).map((s) => (
        <marker key={s} id={`gf-head-${s}`} viewBox="0 0 10 10" refX="6" refY="5" markerWidth="7" markerHeight="7" orient="auto">
          <path d="M0 0L10 5L0 10z" className={`gf-head ${s}`} />
        </marker>
      ))}
    </defs>
  );
}

/** A defense on C launched from D, and Super Earth's liberation of D from B: the gambit. */
function SupplyLines() {
  const P: Record<string, Node> = {
    A: { id: "A", x: 70, y: 110, side: "se" },
    B: { id: "B", x: 180, y: 50, side: "se" },
    C: { id: "C", x: 180, y: 170, side: "se" },
    D: { id: "D", x: 310, y: 110, side: "enemy" },
    E: { id: "E", x: 430, y: 50, side: "enemy" },
    F: { id: "F", x: 430, y: 170, side: "enemy" },
  };
  const links = ["AB", "AC", "BC", "BD", "CD", "DE", "DF", "EF"];
  return (
    <figure className="gww-fig">
      <svg viewBox="0 0 500 236" role="img" aria-label="Supply line diagram: enemy planet D attacks Super Earth planet C while Super Earth liberates D from B.">
        <Heads />
        {links.map((k) => link(P[k[0]], P[k[1]], k))}
        <Attack a={P.D} b={P.C} side="enemy" />
        <Attack a={P.B} b={P.D} side="se" />
        {Object.values(P).map((p) => (
          <Planet key={p.id} p={p} ring={p.id === "C" || p.id === "D"} />
        ))}
        <text className="gf-label" x="180" y="210" textAnchor="middle">
          Defense of C
        </text>
        <text className="gf-label" x="310" y="150" textAnchor="middle">
          Gambit: take D
        </text>
        <g className="gf-legend" transform="translate(16 226)">
          <circle className="gf-dot se" r="5" />
          <text x="10" y="4">Super Earth</text>
          <circle className="gf-dot enemy" r="5" cx="104" />
          <text x="114" y="4">Enemy</text>
          <line className="gf-attack enemy" x1="178" x2="206" y1="0" y2="0" />
          <text x="214" y="4">Attack along a supply line</text>
        </g>
      </svg>
      <figcaption>
        D attacks C along their supply line. Holding C needs the defense won in time, or D liberated first. Cut A, B and C,
        and D can't be reached at all.
      </figcaption>
    </figure>
  );
}

/** One planet alone among Super Earth's falls under siege; a pair cut off together doesn't. */
function Siege() {
  const ring = (cx: number, cy: number, r: number, count: number, from = 0): Node[] =>
    Array.from({ length: count }, (_, i) => {
      const a = from + (i / count) * Math.PI * 2;
      return { id: "", x: cx + Math.cos(a) * r, y: cy + Math.sin(a) * r, side: "se" as const };
    });
  const alone: Node = { id: "", x: 120, y: 100, side: "enemy" };
  const around = ring(120, 100, 62, 6, -Math.PI / 2);
  const pair: Node[] = [
    { id: "", x: 350, y: 100, side: "enemy" },
    { id: "", x: 410, y: 100, side: "enemy" },
  ];
  const fence = ring(380, 100, 70, 8, -Math.PI / 2);
  return (
    <figure className="gww-fig">
      <svg viewBox="0 0 500 214" role="img" aria-label="Siege diagram: a lone enemy planet surrounded by Super Earth planets liberates itself; two enemy planets cut off together do not.">
        {around.map((p, i) => link(p, around[(i + 1) % around.length], `r${i}`))}
        {around.map((p, i) => link(p, alone, `s${i}`, "gf-link strong"))}
        {fence.map((p, i) => link(p, fence[(i + 1) % fence.length], `f${i}`))}
        {fence.map((p, i) => link(p, pair[p.x < 380 ? 0 : 1], `g${i}`))}
        {link(pair[0], pair[1], "pair")}
        {[...around, ...fence].map((p, i) => (
          <g key={i} transform={`translate(${p.x} ${p.y})`} className="gf-planet se small">
            <circle r="7" />
          </g>
        ))}
        <g transform={`translate(${alone.x} ${alone.y})`} className="gf-planet enemy siege">
          <circle className="gf-ring" r="17" />
          <circle r="11" />
        </g>
        {pair.map((p, i) => (
          <g key={i} transform={`translate(${p.x} ${p.y})`} className="gf-planet enemy">
            <circle r="11" />
          </g>
        ))}
        <text className="gf-label good" x="120" y="196" textAnchor="middle">
          Alone: SEAF siege, about −0.5% an hour
        </text>
        <text className="gf-label" x="380" y="196" textAnchor="middle">
          Cut off as a pair: no siege
        </text>
      </svg>
      <figcaption>Only a planet with no neighbour of its own side left falls under a SEAF siege.</figcaption>
    </figure>
  );
}

/** Liberation over a day at three turnouts, against the same resistance. Illustrative, not data. */
function Decay() {
  const W = 500;
  const H = 200;
  const x = (h: number) => 44 + (h / 24) * (W - 150);
  const y = (pct: number) => 16 + (1 - pct / 100) * (H - 48);
  const line = (net: number) =>
    Array.from({ length: 25 }, (_, h) => `${h ? "L" : "M"}${x(h).toFixed(1)} ${y(Math.max(0, Math.min(100, 40 + net * h))).toFixed(1)}`).join("");
  const series = [
    { net: 2.2, label: "Above resistance", className: "good" },
    { net: 0, label: "Matching it", className: "flat" },
    { net: -1.2, label: "Below it", className: "bad" },
  ];
  return (
    <figure className="gww-fig">
      <svg viewBox={`0 0 ${W} ${H}`} role="img" aria-label="Illustration: liberation rises when turnout outpaces resistance, holds when it matches, and falls when it doesn't.">
        {[0, 50, 100].map((p) => (
          <g key={p}>
            <line className="gf-grid" x1={x(0)} x2={x(24)} y1={y(p)} y2={y(p)} />
            <text className="gf-tick" x={x(0) - 8} y={y(p) + 4} textAnchor="end">
              {p}%
            </text>
          </g>
        ))}
        {[0, 12, 24].map((h) => (
          <text key={h} className="gf-tick" x={x(h)} y={H - 10} textAnchor="middle">
            {h} h
          </text>
        ))}
        {series.map((s) => {
          const end = Math.max(0, Math.min(100, 40 + s.net * 24));
          return (
            <g key={s.label} className={`gf-series ${s.className}`}>
              <path d={line(s.net)} />
              <text x={x(24) + 8} y={y(end) + 4}>
                {s.label}
              </text>
            </g>
          );
        })}
      </svg>
      <figcaption>An illustration: the same planet from 40%, with Helldivers dealing more, as much as, or less than it wins back.</figcaption>
    </figure>
  );
}

/** The last day's impact modifier against Helldivers online, from this tracker's collector. */
function ImpactLive() {
  const readings = useWarSamples().data;
  const points = useMemo(
    () => averageReadings(readings ?? [], 10 * 60_000).filter((r) => r.players > 0 && r.impact > 0),
    [readings],
  );
  if (points.length < 6) return null;
  const W = 500;
  const H = 220;
  const px = points.map((p) => p.players);
  const iy = points.map((p) => p.impact);
  const [x0, x1] = [Math.min(...px), Math.max(...px)];
  const [y0, y1] = [Math.min(...iy), Math.max(...iy)];
  const x = (v: number) => 58 + ((v - x0) / (x1 - x0 || 1)) * (W - 78);
  const y = (v: number) => 14 + (1 - (v - y0) / (y1 - y0 || 1)) * (H - 52);
  return (
    <figure className="gww-fig">
      <svg viewBox={`0 0 ${W} ${H}`} role="img" aria-label="Scatter of the last day: the impact modifier is lower when more Helldivers are online.">
        <line className="gf-grid" x1={x(x0)} x2={x(x1)} y1={y(y0)} y2={y(y0)} />
        <line className="gf-grid" x1={x(x0)} x2={x(x0)} y1={y(y0)} y2={y(y1)} />
        {points.map((p, i) => (
          <circle
            key={p.at}
            className="gf-point"
            cx={x(p.players)}
            cy={y(p.impact)}
            r="3"
            style={{ opacity: 0.25 + 0.75 * (i / (points.length - 1)) }}
          />
        ))}
        <text className="gf-tick" x={x(x0)} y={H - 20} textAnchor="start">
          {fmtCompact(x0)}
        </text>
        <text className="gf-tick" x={x(x1)} y={H - 20} textAnchor="end">
          {fmtCompact(x1)}
        </text>
        <text className="gf-tick" x={(x(x0) + x(x1)) / 2} y={H - 6} textAnchor="middle">
          Helldivers online
        </text>
        <text className="gf-tick" x={x(x0) - 6} y={y(y1) + 4} textAnchor="end">
          {y1.toFixed(4)}
        </text>
        <text className="gf-tick" x={x(x0) - 6} y={y(y0) + 4} textAnchor="end">
          {y0.toFixed(4)}
        </text>
      </svg>
      <figcaption>
        Live from this tracker: each dot is ten minutes of the last day, brighter for more recent. More Helldivers online,
        lower modifier.
      </figcaption>
    </figure>
  );
}

export function Figure({ kind }: { kind: NonNullable<Section["figure"]> }) {
  if (kind === "supply-lines") return <SupplyLines />;
  if (kind === "siege") return <Siege />;
  if (kind === "decay") return <Decay />;
  return <ImpactLive />;
}
