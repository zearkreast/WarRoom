import { useEffect, useState } from "react";
import { Area, AreaChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import type { OwnerChange } from "../../domain/history";
import { factionColor, factionLabel } from "../../domain/factions";
import { fmtDateTime, fmtPct } from "../../domain/format";
import type { Sample } from "../../domain/progress";
import { useHistory } from "../../history/useHistory";
import { useNow } from "../../ui/primitives";
import "./history.css";

const CHART_WINDOW_MS = 7 * 24 * 3_600_000;
const CHART_REFRESH_MS = 5 * 60_000;

export function ChangeRow({ c, showPlanet, onClick }: { c: OwnerChange; showPlanet?: boolean; onClick?: () => void }) {
  const won = c.to === "Humans";
  return (
    <li className="change-row" onClick={onClick} style={onClick ? { cursor: "pointer" } : undefined}>
      <span className="dot" style={{ background: factionColor(c.to), boxShadow: `0 0 6px ${factionColor(c.to)}` }} />
      <div>
        <div className="caps change-title">
          {showPlanet && <b>{c.planetName} </b>}
          <span style={{ color: won ? "var(--good)" : "var(--bad)" }}>{won ? "liberated" : "captured"}</span>
          {" by "}
          <span style={{ color: factionColor(c.to) }}>{factionLabel(c.to)}</span>
        </div>
        <div className="muted change-sub">
          from {factionLabel(c.from)} · {fmtDateTime(c.at)}
        </div>
      </div>
    </li>
  );
}

/** Ownership timeline and progress chart for one planet, from the shared history. */
export function PlanetHistory({ planet }: { planet: number }) {
  const store = useHistory((s) => s.store);
  const allChanges = useHistory((s) => s.changes);
  // A 7-day chart barely moves in a minute; re-reading every 5 keeps download traffic low.
  const tick = useNow(CHART_REFRESH_MS);
  const [samples, setSamples] = useState<Sample[]>([]);
  const [changes, setChanges] = useState<OwnerChange[]>([]);

  useEffect(() => {
    let live = true;
    void Promise.all([
      store.samples(planet, Date.now() - CHART_WINDOW_MS),
      store.changes({ planet, limit: 50 }),
    ])
      .then(([s, c]) => {
        if (!live) return;
        setSamples(s);
        setChanges(c);
      })
      .catch(() => undefined);
    return () => {
      live = false;
    };
  }, [store, planet, tick, allChanges]);

  return (
    <div className="planet-history">
      <div className="stat-label">Progress, last 7 days</div>
      {samples.length >= 2 ? (
        <div className="history-chart">
          <ResponsiveContainer width="100%" height={140}>
            <AreaChart data={samples} margin={{ top: 6, right: 4, bottom: 0, left: -18 }}>
              <defs>
                <linearGradient id="hist-fill" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stopColor="var(--f-humans)" stopOpacity={0.45} />
                  <stop offset="100%" stopColor="var(--f-humans)" stopOpacity={0} />
                </linearGradient>
              </defs>
              <XAxis
                dataKey="at"
                type="number"
                domain={["dataMin", "dataMax"]}
                scale="time"
                tickFormatter={(t) => new Date(t).toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" })}
                stroke="var(--dim)"
                fontSize={10}
              />
              <YAxis domain={[0, 100]} stroke="var(--dim)" fontSize={10} tickFormatter={(v) => `${v}%`} />
              <Tooltip
                contentStyle={{ background: "var(--panel-solid)", border: "1px solid var(--line-2)", fontSize: 12 }}
                labelFormatter={(t) => fmtDateTime(Number(t))}
                formatter={(v) => [fmtPct(Number(v), 3), "Progress"]}
              />
              <Area dataKey="percent" stroke="var(--f-humans)" fill="url(#hist-fill)" strokeWidth={1.5} isAnimationActive={false} />
            </AreaChart>
          </ResponsiveContainer>
        </div>
      ) : (
        <div className="muted history-empty">Not enough readings yet. The chart fills in as the collector records this planet.</div>
      )}
      <div className="stat-label">Ownership changes</div>
      {changes.length ? (
        <ul className="change-list">
          {changes.map((c, i) => (
            <ChangeRow key={i} c={c} />
          ))}
        </ul>
      ) : (
        <div className="muted history-empty">No ownership changes recorded since tracking began.</div>
      )}
    </div>
  );
}
