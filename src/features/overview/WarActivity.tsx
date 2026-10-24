import { Activity } from "lucide-react";
import { useMemo, useState, type CSSProperties } from "react";
import {
  Area,
  CartesianGrid,
  ComposedChart,
  Line,
  ReferenceDot,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
  type TooltipContentProps,
} from "recharts";
import type { NameType, ValueType } from "recharts/types/component/DefaultTooltipContent";
import { useLastUpdate, useWar } from "../../api/queries";
import type { Faction } from "../../api/types";
import { useFrontActivity, useWarSamples } from "../../api/warSamples";
import { factionLabel } from "../../domain/factions";
import { mergeActivity, type ActivityPoint } from "../../domain/frontActivity";
import { fmtCompact, fmtDateTime, fmtInt, fmtTime } from "../../domain/format";
import { axisProps, fmtHourTick, gridProps, hourTicks, tooltipProps } from "../../ui/chart";
import { FactionEmblem } from "../../ui/emblems";
import { Empty, Loading, Panel } from "../../ui/primitives";
import "./overview.css";

/** Chart fills per faction (dark-band steps of the faction hues). */
export const CHART: Record<Faction, string> = {
  Humans: "var(--c-humans)",
  Terminids: "var(--c-terminids)",
  Automaton: "var(--c-automaton)",
  Illuminate: "var(--c-illuminate)",
};
export const ENEMIES: Faction[] = ["Terminids", "Illuminate", "Automaton"];

type Series = Faction | "online" | "impact";
const FRONT_KEY: Partial<Record<Faction, "terminids" | "automatons" | "illuminate">> = {
  Terminids: "terminids",
  Automaton: "automatons",
  Illuminate: "illuminate",
};
const ONLINE = "var(--cta)";
const IMPACT = "var(--info)";
const FULL_DAY_MS = 23.5 * 3_600_000;
const AXIS_W = 52;

/** The multiplier moves in the fourth decimal, so "auto" gives identical tick labels; use round steps instead. */
function impactScale(values: number[]): { domain: [number, number]; ticks: number[] } {
  const lo = Math.min(...values);
  const hi = Math.max(...values);
  const raw = Math.max(hi - lo, 0.00002) / 2;
  const mag = 10 ** Math.floor(Math.log10(raw));
  const step = [1, 2, 5, 10].map((m) => m * mag).find((s) => s >= raw) ?? 10 * mag;
  const from = Math.floor(lo / step);
  const to = Math.ceil(hi / step);
  const ticks = Array.from({ length: to - from + 1 }, (_, i) => Number(((from + i) * step).toPrecision(6)));
  return { domain: [ticks[0], ticks[ticks.length - 1]], ticks };
}

function extreme(points: ActivityPoint[], value: (p: ActivityPoint) => number, pick: (a: number, b: number) => boolean) {
  return points.reduce<ActivityPoint | null>((m, p) => (!m || pick(value(p), value(m)) ? p : m), null);
}

/** One readout for both charts, so the fronts and the multiplier at a moment are read together. */
function ActivityTip({ active, payload, label, hidden }: TooltipContentProps<ValueType, NameType> & { hidden: Set<Series> }) {
  const row = payload?.[0]?.payload as ActivityPoint | undefined;
  if (!active || !row) return null;
  const lines: [string, string, string][] = [];
  if (!hidden.has("online") && row.online != null) lines.push(["Helldivers online", fmtInt(row.online), ONLINE]);
  for (const f of ENEMIES)
    if (!hidden.has(f)) lines.push([`Fighting ${factionLabel(f)}`, fmtInt(row[FRONT_KEY[f]!]), CHART[f]]);
  if (!hidden.has("impact") && row.impact != null) lines.push(["Impact", `×${row.impact.toFixed(6)}`, IMPACT]);
  return (
    <div className="ov-tip" style={tooltipProps.contentStyle}>
      <p className="ov-tip-when">{fmtDateTime(Number(label))}</p>
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

/**
 * Galaxy-wide telemetry for the last 24 hours, or since the collector's latest gap: Helldivers per
 * front stacked under the online total, with the impact multiplier on its own right-hand axis.
 */
export function WarActivity() {
  const { data, isLoading } = useFrontActivity();
  const readings = useWarSamples().data;
  const war = useWar().data;
  const lastUpdate = useLastUpdate().data;
  const [hidden, setHidden] = useState<Set<Series>>(new Set());
  const rows = useMemo(
    () => (data && lastUpdate != null ? mergeActivity(data, readings ?? [], lastUpdate) : []),
    [data, readings, lastUpdate],
  );
  const online = rows.filter((r) => r.online != null);
  // The online total once the collector has recorded it; until then, everyone on the fronts.
  const total = (p: ActivityPoint) => (online.length > 1 ? (p.online ?? 0) : p.players);
  const pool = online.length > 1 ? online : rows;
  const last = pool[pool.length - 1];
  const peak = extreme(pool, total, (a, b) => a > b);
  const low = extreme(pool, total, (a, b) => a < b);
  const impacts = rows.flatMap((r) => (r.impact == null ? [] : [r.impact]));
  const impactAxis = impacts.length > 1 ? impactScale(impacts) : null;
  const domain: [number, number] = [rows[0]?.at ?? 0, rows[rows.length - 1]?.at ?? 0];
  const ticks = hourTicks(domain[0], domain[1]);
  const span = domain[1] - domain[0] >= FULL_DAY_MS ? "last 24 h" : `since ${fmtTime(domain[0])}`;
  const toggle = (s: Series) =>
    setHidden((h) => {
      const n = new Set(h);
      if (n.has(s)) n.delete(s);
      else n.add(s);
      return n;
    });
  const chips: [Series, string, string][] = [
    ["online", "Helldivers online", ONLINE],
    ...ENEMIES.map((f): [Series, string, string] => [f, `Fighting ${factionLabel(f)}`, CHART[f]]),
    ["impact", "Galactic impact mod", IMPACT],
  ];
  const tip = (props: TooltipContentProps<ValueType, NameType>) => <ActivityTip {...props} hidden={hidden} />;

  return (
    <Panel className="ov-activity" title={`Galactic war activity · ${rows.length > 1 ? span : "last 24 h"}`} icon={Activity}>
      <div className="ov-heads">
        <div>
          <span className="pd-key" style={{ "--k": ONLINE } as CSSProperties}>
            Peak Helldivers
          </span>
          <b className="num">{peak ? fmtInt(total(peak)) : "—"}</b>
          {peak && <span className="ov-when num">{fmtTime(peak.at)}</span>}
        </div>
        <div>
          <span className="pd-key" style={{ "--k": "var(--text-dim)" } as CSSProperties}>
            Lowest Helldivers
          </span>
          <b className="num">{low ? fmtInt(total(low)) : "—"}</b>
          {low && <span className="ov-when num">{fmtTime(low.at)}</span>}
        </div>
        <div>
          <span className="pd-key" style={{ "--k": IMPACT } as CSSProperties}>
            Galactic impact mod
          </span>
          <b className="num" style={{ color: IMPACT }}>
            ×{(war?.impactMultiplier ?? 0).toFixed(6)}
          </b>
          <span className="ov-when">now</span>
        </div>
      </div>
      {isLoading || lastUpdate == null ? (
        <Loading what="war telemetry" rows={2} />
      ) : rows.length < 2 ? (
        <Empty>No front readings in the last 24 hours; the collector may be down.</Empty>
      ) : (
        <div className="ov-charts ov-frame">
          <ResponsiveContainer width="100%" height={300}>
            <ComposedChart data={rows} margin={{ top: 10, right: 4, bottom: 0, left: 0 }}>
              <defs>
                {ENEMIES.map((f) => (
                  <linearGradient key={f} id={`ov-front-${f}`} x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stopColor={CHART[f]} stopOpacity={0.55} />
                    <stop offset="100%" stopColor={CHART[f]} stopOpacity={0.12} />
                  </linearGradient>
                ))}
              </defs>
              <CartesianGrid {...gridProps} />
              <XAxis
                dataKey="at"
                type="number"
                scale="time"
                domain={domain}
                ticks={ticks}
                tickFormatter={fmtHourTick}
                {...axisProps}
              />
              <YAxis yAxisId="divers" tickFormatter={(v) => fmtCompact(Number(v))} {...axisProps} width={AXIS_W} />
              {impactAxis && (
                <YAxis
                  yAxisId="impact"
                  orientation="right"
                  domain={impactAxis.domain}
                  ticks={impactAxis.ticks}
                  tickFormatter={(v) => Number(v).toFixed(4)}
                  {...axisProps}
                  width={AXIS_W}
                />
              )}
              <Tooltip {...tooltipProps} content={tip} />
              {ENEMIES.filter((f) => !hidden.has(f)).map((f) => (
                <Area
                  key={f}
                  yAxisId="divers"
                  dataKey={FRONT_KEY[f]!}
                  name={factionLabel(f)}
                  stackId="fronts"
                  stroke={CHART[f]}
                  strokeWidth={1}
                  fill={`url(#ov-front-${f})`}
                  isAnimationActive={false}
                />
              ))}
              {impactAxis && !hidden.has("impact") && (
                <Line
                  yAxisId="impact"
                  dataKey="impact"
                  name="Impact"
                  stroke={IMPACT}
                  strokeWidth={2}
                  dot={false}
                  connectNulls
                  isAnimationActive={false}
                />
              )}
              {!hidden.has("online") && (
                <Line
                  yAxisId="divers"
                  dataKey={online.length > 1 ? "online" : "players"}
                  name="Helldivers online"
                  stroke={ONLINE}
                  strokeWidth={2.5}
                  strokeLinejoin="round"
                  dot={false}
                  connectNulls
                  isAnimationActive={false}
                />
              )}
              {!hidden.has("online") &&
                [peak, low].map(
                  (p, i) =>
                    p && (
                      <ReferenceDot
                        key={i}
                        yAxisId="divers"
                        x={p.at}
                        y={total(p)}
                        r={4}
                        fill={ONLINE}
                        stroke="var(--surface-1)"
                        strokeWidth={2}
                      />
                    ),
                )}
              {!hidden.has("online") && last && (
                <ReferenceDot
                  yAxisId="divers"
                  x={last.at}
                  y={total(last)}
                  r={6}
                  fill="none"
                  stroke={ONLINE}
                  strokeWidth={2}
                />
              )}
            </ComposedChart>
          </ResponsiveContainer>
          {!impactAxis && (
            <p className="ov-note">The impact multiplier's history is recorded from the next collector update.</p>
          )}
          <div className="ov-chips" role="group" aria-label="Show series">
            {chips.map(([s, label, color]) => (
              <button
                key={s}
                className={`filter-chip ov-chip ${hidden.has(s) ? "off" : ""}`}
                aria-pressed={!hidden.has(s)}
                onClick={() => toggle(s)}
                style={{ "--k": color } as CSSProperties}
              >
                {s === "online" || s === "impact" ? (
                  <i className={`ov-chip-key ${s}`} style={{ background: color }} aria-hidden />
                ) : (
                  <FactionEmblem faction={s} size={13} />
                )}
                {label}
              </button>
            ))}
          </div>
        </div>
      )}
    </Panel>
  );
}
