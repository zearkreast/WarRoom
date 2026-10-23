import { Activity } from "lucide-react";
import { useState, type CSSProperties } from "react";
import { Area, AreaChart, CartesianGrid, Line, LineChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
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
          <span className="tile-label">Galactic impact mod</span>
          <b className="num" style={{ color: "var(--info)" }}>
            ×{(war?.impactMultiplier ?? 0).toFixed(6)}
          </b>
        </div>
        <div>
          <span className="tile-label">Peak Helldivers</span>
          <b className="num">{peak ? fmtInt(peak.players) : "—"}</b>
          {peak && <span className="ov-when num">{fmtTime(peak.at)}</span>}
        </div>
        <div>
          <span className="tile-label">Lowest Helldivers</span>
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
          {impact.length > 1 ? (
            <ResponsiveContainer width="100%" height={120}>
              <LineChart data={impact} syncId="ov-war" margin={{ top: 6, right: 8, bottom: 0, left: 0 }}>
                <CartesianGrid {...gridProps} />
                <XAxis dataKey="at" type="number" scale="time" domain={domain} ticks={ticks} hide />
                <YAxis
                  domain={["auto", "auto"]}
                  tickFormatter={(v) => Number(v).toFixed(3)}
                  {...axisProps}
                  width={48}
                />
                <Tooltip {...tooltipProps} formatter={(v) => [`×${Number(v).toFixed(6)}`, "Impact"]} />
                <Line dataKey="impact" stroke="var(--info)" strokeWidth={2} dot={false} isAnimationActive={false} />
              </LineChart>
            </ResponsiveContainer>
          ) : (
            <p className="ov-note">
              Its history is recorded from the next collector update; the current value is above.
            </p>
          )}
          <div className="pd-chart-label caps">Helldivers by front</div>
          <ResponsiveContainer width="100%" height={180}>
            <AreaChart data={samples} syncId="ov-war" margin={{ top: 6, right: 8, bottom: 0, left: 0 }}>
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
              <YAxis tickFormatter={(v) => fmtCompact(Number(v))} {...axisProps} width={48} />
              <Tooltip {...tooltipProps} formatter={(v, n) => [fmtInt(Number(v)), n]} />
              {ENEMIES.filter((f) => !hidden.has(f)).map((f) => (
                <Area
                  key={f}
                  dataKey={key[f]}
                  name={factionLabel(f)}
                  stackId="fronts"
                  stroke="var(--surface-1)"
                  strokeWidth={2}
                  fill={CHART[f]}
                  fillOpacity={0.85}
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
