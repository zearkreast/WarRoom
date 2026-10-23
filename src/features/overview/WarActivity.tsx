import { Activity } from "lucide-react";
import { useState, type CSSProperties } from "react";
import { Area, AreaChart, CartesianGrid, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { useLastUpdate, useWar } from "../../api/queries";
import type { Faction } from "../../api/types";
import { useFrontActivity, useWarSamples } from "../../api/warSamples";
import type { FrontBucket } from "../../domain/frontActivity";
import { factionLabel } from "../../domain/factions";
import { fmtCompact, fmtInt, fmtTime } from "../../domain/format";
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

function extreme(buckets: FrontBucket[], pick: (a: number, b: number) => boolean) {
  return buckets.reduce<FrontBucket | null>((m, s) => (!m || pick(s.players, m.players) ? s : m), null);
}

const FULL_DAY_MS = 23.5 * 3_600_000;

/** The multiplier moves in the fourth decimal, so "auto" gives identical tick labels; use round steps instead. */
function impactScale(rows: { impact: number }[]): { domain: [number, number]; ticks: number[] } {
  const vals = rows.map((r) => r.impact);
  const lo = Math.min(...vals);
  const hi = Math.max(...vals);
  const raw = Math.max(hi - lo, 0.00002) / 2;
  const mag = 10 ** Math.floor(Math.log10(raw));
  const step = [1, 2, 5, 10].map((m) => m * mag).find((s) => s >= raw) ?? 10 * mag;
  const from = Math.floor(lo / step);
  const to = Math.ceil(hi / step);
  const ticks = Array.from({ length: to - from + 1 }, (_, i) => Number(((from + i) * step).toPrecision(6)));
  return { domain: [ticks[0], ticks[ticks.length - 1]], ticks };
}

/**
 * Galaxy-wide telemetry for the last 24 hours, or since the collector's latest gap: impact multiplier
 * over Helldivers per front (two charts, one time axis). Fronts show at once from the planet samples;
 * the impact line appears once the collector records it.
 */
export function WarActivity() {
  const { data, isLoading } = useFrontActivity();
  const impactRows = useWarSamples().data ?? [];
  const war = useWar().data;
  const lastUpdate = useLastUpdate().data;
  const [hidden, setHidden] = useState<Set<Faction>>(new Set());
  const samples = data ?? [];
  const peak = extreme(samples, (a, b) => a > b);
  const low = extreme(samples, (a, b) => a < b);
  const domain: [number, number] = [samples[0]?.at ?? 0, samples[samples.length - 1]?.at ?? 0];
  const ticks = hourTicks(domain[0], domain[1]);
  const impact = impactRows.filter((r) => r.at >= domain[0]);
  const impactAxis = impact.length > 1 ? impactScale(impact) : null;
  const span = domain[1] - domain[0] >= FULL_DAY_MS ? "last 24 h" : `since ${fmtTime(domain[0])}`;
  const toggle = (f: Faction) =>
    setHidden((h) => {
      const n = new Set(h);
      if (n.has(f)) n.delete(f);
      else n.add(f);
      return n;
    });
  const key: Record<Faction, keyof FrontBucket> = {
    Terminids: "terminids",
    Automaton: "automatons",
    Illuminate: "illuminate",
    Humans: "players",
  };

  return (
    <Panel
      className="ov-activity"
      title={`Galactic war activity · ${samples.length > 1 ? span : "last 24 h"}`}
      icon={Activity}
    >
      <div className="ov-heads">
        <div>
          <span className="pd-key" style={{ "--k": "var(--info)" } as CSSProperties}>
            Galactic impact mod
          </span>
          <b className="num" style={{ color: "var(--info)" }}>
            ×{(war?.impactMultiplier ?? 0).toFixed(6)}
          </b>
        </div>
        <div>
          <span className="pd-key" style={{ "--k": "var(--cta)" } as CSSProperties}>
            Peak Helldivers
          </span>
          <b className="num">{peak ? fmtInt(peak.players) : "—"}</b>
          {peak && <span className="ov-when num">{fmtTime(peak.at)}</span>}
        </div>
        <div>
          <span className="pd-key" style={{ "--k": "var(--text-dim)" } as CSSProperties}>
            Lowest Helldivers
          </span>
          <b className="num">{low ? fmtInt(low.players) : "—"}</b>
          {low && <span className="ov-when num">{fmtTime(low.at)}</span>}
        </div>
      </div>
      {isLoading || lastUpdate == null ? (
        <Loading what="war telemetry" rows={2} />
      ) : samples.length < 2 ? (
        <Empty>No front readings in the last 24 hours; the collector may be down.</Empty>
      ) : (
        <div className="ov-charts">
          <div className="pd-chart-label caps">Impact multiplier</div>
          {impactAxis ? (
            <ResponsiveContainer width="100%" height={110}>
              <AreaChart data={impact} syncId="ov-war" margin={{ top: 8, right: 8, bottom: 0, left: 0 }}>
                <defs>
                  <linearGradient id="ov-impact" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stopColor="var(--info)" stopOpacity={0.28} />
                    <stop offset="100%" stopColor="var(--info)" stopOpacity={0.02} />
                  </linearGradient>
                </defs>
                <CartesianGrid {...gridProps} />
                <XAxis dataKey="at" type="number" scale="time" domain={domain} ticks={ticks} hide />
                <YAxis
                  domain={impactAxis.domain}
                  ticks={impactAxis.ticks}
                  tickFormatter={(v) => Number(v).toFixed(5)}
                  {...axisProps}
                  width={56}
                />
                <Tooltip {...tooltipProps} formatter={(v) => [`×${Number(v).toFixed(6)}`, "Impact"]} />
                <Area
                  dataKey="impact"
                  stroke="var(--info)"
                  strokeWidth={2}
                  fill="url(#ov-impact)"
                  baseValue="dataMin"
                  isAnimationActive={false}
                />
              </AreaChart>
            </ResponsiveContainer>
          ) : (
            <p className="ov-note">
              Its history is recorded from the next collector update; the current value is above.
            </p>
          )}
          <div className="pd-chart-label caps">Helldivers by front</div>
          <ResponsiveContainer width="100%" height={170}>
            <AreaChart data={samples} syncId="ov-war" margin={{ top: 8, right: 8, bottom: 0, left: 0 }}>
              <defs>
                {ENEMIES.map((f) => (
                  <linearGradient key={f} id={`ov-front-${f}`} x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stopColor={CHART[f]} stopOpacity={0.45} />
                    <stop offset="100%" stopColor={CHART[f]} stopOpacity={0.08} />
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
              <YAxis tickFormatter={(v) => fmtCompact(Number(v))} {...axisProps} width={56} />
              <Tooltip {...tooltipProps} formatter={(v, n) => [fmtInt(Number(v)), n]} />
              {ENEMIES.filter((f) => !hidden.has(f)).map((f) => (
                <Area
                  key={f}
                  dataKey={key[f]}
                  name={factionLabel(f)}
                  stackId="fronts"
                  stroke={CHART[f]}
                  strokeWidth={2}
                  fill={`url(#ov-front-${f})`}
                  isAnimationActive={false}
                />
              ))}
            </AreaChart>
          </ResponsiveContainer>
          <div className="ov-chips" role="group" aria-label="Show fronts">
            {ENEMIES.map((f) => (
              <button
                key={f}
                className={`filter-chip ${hidden.has(f) ? "" : "on"}`}
                aria-pressed={!hidden.has(f)}
                onClick={() => toggle(f)}
                style={{ "--k": CHART[f] } as CSSProperties}
              >
                <FactionEmblem faction={f} size={13} /> Fighting {factionLabel(f)}
              </button>
            ))}
          </div>
        </div>
      )}
    </Panel>
  );
}
