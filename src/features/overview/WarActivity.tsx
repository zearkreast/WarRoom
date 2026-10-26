import { Activity } from "lucide-react";
import { useMemo, useState, type CSSProperties } from "react";
import {
  Area,
  CartesianGrid,
  ComposedChart,
  Curve,
  Line,
  ReferenceDot,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
  type LineDrawShapeProps,
  type TooltipContentProps,
} from "recharts";
import type { NameType, ValueType } from "recharts/types/component/DefaultTooltipContent";
import { useLastUpdate, useWar } from "../../api/queries";
import type { Faction } from "../../api/types";
import { useFrontActivity, useWarSamples } from "../../api/warSamples";
import { factionLabel } from "../../domain/factions";
import {
  averageReadings,
  mergeActivity,
  plotActivity,
  type ActivityPoint,
  type ActivitySeries,
  type FrontKey,
} from "../../domain/frontActivity";
import { fmtDateTime, fmtInt, fmtTime } from "../../domain/format";
import { fmtHourTick, hourTicks } from "../../ui/chart";
import { FactionEmblem } from "../../ui/emblems";
import { Empty, Loading, Panel, useMedia } from "../../ui/primitives";
import "./warActivity.css";

/** Chart fills per faction (dark-band steps of the faction hues). */
export const CHART: Record<Faction, string> = {
  Humans: "var(--c-humans)",
  Terminids: "var(--c-terminids)",
  Automaton: "var(--c-automaton)",
  Illuminate: "var(--c-illuminate)",
};
export const ENEMIES: Faction[] = ["Terminids", "Illuminate", "Automaton"];

type Series = Faction | "online" | "impact";
/** Bottom to top of the stack. */
const STACK: Faction[] = ["Terminids", "Automaton", "Illuminate"];
const FRONT_KEY: Partial<Record<Faction, FrontKey>> = {
  Terminids: "terminids",
  Automaton: "automatons",
  Illuminate: "illuminate",
};
const PLOT_KEY: Partial<Record<Faction, "plotTerminids" | "plotAutomatons" | "plotIlluminate">> = {
  Terminids: "plotTerminids",
  Automaton: "plotAutomatons",
  Illuminate: "plotIlluminate",
};
// Series colours are the panel's own tokens (warActivity.css).
const FRONT: Partial<Record<Faction, string>> = {
  Terminids: "var(--wa-terminids)",
  Automaton: "var(--wa-automaton)",
  Illuminate: "var(--wa-illuminate)",
};
const ONLINE = "var(--wa-online)";
const IMPACT = "var(--wa-impact)";
const MUTED = "var(--wa-muted)";
const FULL_DAY_MS = 23.5 * 3_600_000;
const STEP = 25_000;
/** Readings are averaged into windows this long: about 240 points a day, 120 on a phone. */
const READING_WINDOW_MS = 6 * 60_000;
const NARROW_WINDOW_MS = 12 * 60_000;
const NARROW = "(max-width: 720px)";
// Keep in sync with --wa-morph in warActivity.css.
const MORPH_MS = 700;
// Keep in sync with --ease in theme.css, which the CSS fades use.
const MORPH_EASE = "cubic-bezier(0.2,0.85,0.25,1)";
const TICK = { fill: MUTED, fontSize: 10 } as const;
const NARROW_TICK = { fill: MUTED, fontSize: 9 } as const;

const fmtK = (v: number) => (v === 0 ? "0" : `${Math.round(v / 1000)}k`);

/** Five evenly spaced ticks around the multiplier's range, which moves in the fourth decimal. */
function impactScale(values: number[]): { domain: [number, number]; ticks: number[] } {
  const lo = Math.min(...values);
  const hi = Math.max(...values);
  const pad = Math.max((hi - lo) * 0.12, 0.0004);
  const from = Math.max(0, lo - pad);
  const to = hi + pad;
  return { domain: [from, to], ticks: Array.from({ length: 5 }, (_, i) => from + ((to - from) * i) / 4) };
}

function extreme(points: ActivityPoint[], value: (p: ActivityPoint) => number, pick: (a: number, b: number) => boolean) {
  return points.reduce<ActivityPoint | null>((m, p) => (!m || pick(value(p), value(m)) ? p : m), null);
}

/** One readout for every series at a moment. */
function ActivityTip({ active, payload, label, hidden }: TooltipContentProps<ValueType, NameType> & { hidden: Set<Series> }) {
  const row = payload?.[0]?.payload as ActivityPoint | undefined;
  if (!active || !row) return null;
  const lines: [string, string, string][] = [];
  if (!hidden.has("online") && row.online != null) lines.push(["Helldivers online", fmtInt(row.online), ONLINE]);
  for (const f of [...STACK].reverse())
    if (!hidden.has(f)) lines.push([`Fighting ${factionLabel(f)}`, fmtInt(row[FRONT_KEY[f]!]), FRONT[f]!]);
  if (!hidden.has("impact") && row.impact != null) lines.push(["Galactic impact mod", row.impact.toFixed(6), IMPACT]);
  return (
    <div className="wa-tip">
      <p className="wa-tip-when num">{fmtDateTime(Number(label))}</p>
      {lines.map(([name, value, color]) => (
        <p key={name}>
          <i style={{ background: color }} />
          <span>{name}</span>
          <b className="num">{value}</b>
        </p>
      ))}
    </div>
  );
}

/** Recharts' default line shape wipes a line in from the left the first time it animates; this one only morphs. */
function MorphLine({ animationElapsedTime, isAnimating, isEntrance, visibleLength, ...curve }: LineDrawShapeProps) {
  return <Curve {...curve} />;
}

/** The newest online reading: a ring that pings, as a live feed does. */
function LiveRing({ cx, cy }: { cx?: number; cy?: number }) {
  if (cx == null || cy == null) return <g />;
  return (
    <g aria-hidden className="wa-live">
      <circle cx={cx} cy={cy} r={5} fill="none" stroke={ONLINE} strokeWidth={1.5} className="wa-ping" />
      <circle cx={cx} cy={cy} r={5} fill="none" stroke={ONLINE} strokeWidth={1.5} />
    </g>
  );
}

function Stat({ label, value, when, tone, toned }: { label: string; value: string; when?: string; tone: string; toned?: boolean }) {
  return (
    <div>
      <span className="pd-key" style={{ "--k": tone } as CSSProperties}>
        {label}
      </span>
      <b className="num" style={toned ? { color: tone } : undefined}>
        {value}
      </b>
      {when && <span className="ov-when num">{when}</span>}
    </div>
  );
}

/**
 * Galaxy-wide telemetry for the last 24 hours, or since the collector's latest gap: Helldivers per
 * front stacked under the online total, with the impact multiplier on its own right-hand axis.
 */
export function WarActivity() {
  const { data, isLoading } = useFrontActivity();
  const readings = useWarSamples().data;
  const war = useWar().data;
  const lastUpdate = useLastUpdate().data;
  const narrow = useMedia(NARROW);
  const tick = narrow ? NARROW_TICK : TICK;
  const [hidden, setHidden] = useState<Set<Series>>(new Set());
  // Chips toggled at least once, so their switch animation never plays on first paint.
  const [touched, setTouched] = useState<Set<Series>>(new Set());
  const [morphFor, setMorphFor] = useState<ActivityPoint[] | null>(null);
  const rows = useMemo(
    () =>
      data && lastUpdate != null
        ? mergeActivity(data, averageReadings(readings ?? [], narrow ? NARROW_WINDOW_MS : READING_WINDOW_MS), lastUpdate)
        : [],
    [data, readings, lastUpdate, narrow],
  );
  // Morph until the data next changes: refreshes redraw at once, and the duration never switches mid-morph.
  const morphing = morphFor === rows;
  const online = rows.filter((r) => r.online != null);
  const hasOnline = online.length > 1;
  // The online total once the collector has recorded it; until then, everyone on the fronts.
  const total = (p: ActivityPoint) => (hasOnline ? (p.online ?? 0) : p.players);
  const pool = hasOnline ? online : rows;
  const peak = extreme(pool, total, (a, b) => a > b);
  const low = extreme(pool, total, (a, b) => a < b);
  const last = pool[pool.length - 1];
  const impacts = rows.flatMap((r) => (r.impact == null ? [] : [r.impact]));
  const impactAxis = impacts.length > 1 ? impactScale(impacts) : null;
  const lastImpact = [...rows].reverse().find((r) => r.impact != null);
  const top = Math.max(STEP, Math.ceil((peak ? total(peak) : 0) / STEP) * STEP);
  const domain: [number, number] = [rows[0]?.at ?? 0, rows[rows.length - 1]?.at ?? 0];
  const ticks = hourTicks(domain[0], domain[1], narrow ? 6 : 4);
  const span = rows.length < 2 || domain[1] - domain[0] >= FULL_DAY_MS ? "last 24 h" : `since ${fmtTime(domain[0])}`;
  const flatImpact = impactAxis ? (impactAxis.domain[0] + impactAxis.domain[1]) / 2 : 0;
  const plot = useMemo(() => {
    const off = new Set<ActivitySeries>([...hidden].map((s) => FRONT_KEY[s as Faction] ?? (s as ActivitySeries)));
    return plotActivity(rows, off, hasOnline, flatImpact);
  }, [rows, hidden, hasOnline, flatImpact]);
  const motion = {
    isAnimationActive: "auto",
    animationDuration: morphing ? MORPH_MS : 1,
    animationEasing: MORPH_EASE,
  } as const;
  const off = (s: Series) => (hidden.has(s) ? " is-off" : "");
  const toggle = (s: Series) => {
    setHidden((h) => {
      const n = new Set(h);
      if (n.has(s)) n.delete(s);
      else n.add(s);
      return n;
    });
    setTouched((t) => (t.has(s) ? t : new Set(t).add(s)));
    setMorphFor(rows);
  };
  const chips: [Series, string][] = [
    ["online", "Helldivers online"],
    ...(["Terminids", "Automaton", "Illuminate"] as Faction[]).map((f): [Series, string] => [f, `Fighting ${factionLabel(f)}`]),
    ["impact", "Galactic impact mod"],
  ];
  const tip = (props: TooltipContentProps<ValueType, NameType>) => <ActivityTip {...props} hidden={hidden} />;

  return (
    <Panel className="ov-activity wa" title={`Galactic war activity · ${span}`} icon={Activity}>
      <div className="ov-heads">
        <Stat
          label="Galactic impact mod"
          value={`×${(lastImpact?.impact ?? war?.impactMultiplier ?? 0).toFixed(6)}`}
          tone={IMPACT}
          toned
        />
        <Stat label="Peak Helldivers" value={peak ? fmtInt(total(peak)) : "—"} when={peak ? fmtTime(peak.at) : undefined} tone={ONLINE} />
        <Stat label="Lowest Helldivers" value={low ? fmtInt(total(low)) : "—"} when={low ? fmtTime(low.at) : undefined} tone="var(--text-dim)" />
      </div>
        {isLoading || lastUpdate == null ? (
          <Loading what="war telemetry" rows={2} />
        ) : rows.length < 2 ? (
          <Empty>No front readings in the last 24 hours; the collector may be down.</Empty>
        ) : (
          <>
            <div className="wa-plot">
              <span className="wa-corner tl" aria-hidden />
              <span className="wa-corner tr" aria-hidden />
              <span className="wa-corner bl" aria-hidden />
              <span className="wa-corner br" aria-hidden />
              <ResponsiveContainer width="100%" height={narrow ? 220 : 280}>
                <ComposedChart data={plot} margin={{ top: 20, right: 0, bottom: 0, left: 0 }}>
                  <defs>
                    {STACK.map((f) => (
                      <linearGradient key={f} id={`wa-front-${f}`} x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stopColor={FRONT[f]} stopOpacity={0.45} />
                        <stop offset="100%" stopColor={FRONT[f]} stopOpacity={0.12} />
                      </linearGradient>
                    ))}
                  </defs>
                  <CartesianGrid vertical={false} stroke={MUTED} strokeOpacity={0.15} />
                  <XAxis
                    dataKey="at"
                    type="number"
                    scale="time"
                    domain={domain}
                    ticks={ticks}
                    tickFormatter={fmtHourTick}
                    tick={tick}
                    tickLine={false}
                    axisLine={false}
                    height={narrow ? 24 : 30}
                  />
                  <YAxis
                    yAxisId="divers"
                    domain={[0, top]}
                    ticks={Array.from({ length: top / STEP + 1 }, (_, i) => i * STEP)}
                    tickFormatter={(v) => fmtK(Number(v))}
                    tick={tick}
                    tickLine={false}
                    axisLine={false}
                    width={narrow ? 32 : 44}
                  />
                  {impactAxis && (
                    <YAxis
                      yAxisId="impact"
                      orientation="right"
                      domain={impactAxis.domain}
                      ticks={impactAxis.ticks}
                      tickFormatter={(v) => Number(v).toFixed(4)}
                      tick={{ ...tick, className: `wa-axis-tick${off("impact")}` }}
                      tickLine={false}
                      axisLine={false}
                      width={narrow ? 42 : 56}
                    />
                  )}
                  <Tooltip content={tip} cursor={{ stroke: MUTED, strokeOpacity: 0.5, strokeWidth: 1 }} />
                  {STACK.map((f) => (
                    <Area
                      key={f}
                      className={`wa-series wa-front wa-front-${f}${off(f)}`}
                      yAxisId="divers"
                      type="monotone"
                      dataKey={PLOT_KEY[f]!}
                      name={factionLabel(f)}
                      stackId="fronts"
                      stroke={FRONT[f]}
                      strokeWidth={1}
                      strokeOpacity={0.7}
                      fill={`url(#wa-front-${f})`}
                      fillOpacity={1}
                      activeDot={false}
                      {...motion}
                    />
                  ))}
                  <Line
                    className={`wa-series wa-online${off("online")}`}
                    yAxisId="divers"
                    type="monotone"
                    dataKey="plotOnline"
                    name="Helldivers online"
                    stroke={ONLINE}
                    strokeWidth={2.5}
                    dot={false}
                    activeDot={false}
                    connectNulls
                    shape={MorphLine}
                    {...motion}
                  />
                  {impactAxis && (
                    <Line
                      className={`wa-series wa-impact${off("impact")}`}
                      yAxisId="impact"
                      type="monotone"
                      dataKey="plotImpact"
                      name="Galactic impact mod"
                      stroke={IMPACT}
                      strokeWidth={2}
                      dot={false}
                      activeDot={false}
                      connectNulls
                      shape={MorphLine}
                      {...motion}
                    />
                  )}
                  {[peak, low].map(
                    (p, i) =>
                      p && (
                        <ReferenceDot
                          key={i}
                          className={`wa-dot${off("online")}`}
                          yAxisId="divers"
                          x={p.at}
                          y={total(p)}
                          r={4}
                          fill={ONLINE}
                          stroke="var(--wa-surface)"
                          strokeWidth={2}
                        />
                      ),
                  )}
                  {last && (
                    <ReferenceDot
                      className={`wa-live-dot${off("online")}`}
                      yAxisId="divers"
                      x={last.at}
                      y={total(last)}
                      shape={LiveRing}
                    />
                  )}
                </ComposedChart>
              </ResponsiveContainer>
            </div>
            {!impactAxis && (
              <p className="wa-note">The impact multiplier's history is recorded from the next collector update.</p>
            )}
            <div className="ov-chips" role="group" aria-label="Show series">
              {chips.map(([s, label]) => (
                <button
                  key={s}
                  className={`filter-chip ${hidden.has(s) ? "" : "on"}`}
                  aria-pressed={!hidden.has(s)}
                  data-flash={touched.has(s) ? (hidden.has(s) ? "off" : "on") : undefined}
                  onClick={() => toggle(s)}
                  style={{ "--k": s === "online" ? ONLINE : s === "impact" ? IMPACT : FRONT[s as Faction] } as CSSProperties}
                >
                    {s === "online" ? (
                      <FactionEmblem faction="Humans" size={14} color={ONLINE} />
                    ) : s === "impact" ? (
                      <Activity size={14} color={IMPACT} aria-hidden />
                    ) : (
                      <FactionEmblem faction={s} size={14} color={FRONT[s]} />
                    )}
                    {label}
                </button>
              ))}
            </div>
          </>
        )}
    </Panel>
  );
}
