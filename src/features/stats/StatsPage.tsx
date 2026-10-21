import { useMemo } from "react";
import { Cell, Pie, PieChart, ResponsiveContainer, Tooltip } from "recharts";
import { useCampaigns, useWar } from "../../api/queries";
import type { Faction } from "../../api/types";
import { FACTIONS, factionColor } from "../../domain/factions";
import { fmtCompact, fmtInt, fmtPct } from "../../domain/format";
import { useUi } from "../../state";
import { Loading, Panel, Stat } from "../../ui/primitives";
import { AlertSettingsPanel } from "./AlertSettings";
import "./stats.css";

/** Where the divers are: players per faction front, and the busiest planets. */
function Distribution() {
  const { data } = useCampaigns();
  const select = useUi((s) => s.select);
  const setView = useUi((s) => s.setView);
  const { byFaction, top, total } = useMemo(() => {
    const byFaction = new Map<Faction, number>();
    const planets = (data ?? []).map((c) => {
      const enemy = c.planet.event?.faction ?? c.planet.currentOwner;
      byFaction.set(enemy, (byFaction.get(enemy) ?? 0) + c.planet.statistics.playerCount);
      return { index: c.planet.index, name: c.planet.name, enemy, players: c.planet.statistics.playerCount };
    });
    const total = planets.reduce((s, p) => s + p.players, 0);
    return { byFaction, top: planets.sort((a, b) => b.players - a.players).slice(0, 10), total };
  }, [data]);
  const pie = [...byFaction].map(([f, v]) => ({ name: FACTIONS[f].label, value: v, color: factionColor(f) }));

  return (
    <Panel title="Distribution of Democracy" right={<span className="num">{fmtInt(total)} on fronts</span>}>
      <div className="panel-body dist">
        <div className="dist-chart">
          <ResponsiveContainer width="100%" height={200}>
            <PieChart>
              <Pie data={pie} dataKey="value" innerRadius={55} outerRadius={85} stroke="var(--bg)" strokeWidth={2} isAnimationActive={false}>
                {pie.map((p) => (
                  <Cell key={p.name} fill={p.color} />
                ))}
              </Pie>
              <Tooltip
                contentStyle={{ background: "var(--panel-solid)", border: "1px solid var(--line-2)", fontSize: 12 }}
                itemStyle={{ color: "var(--text)" }}
                formatter={(v) => [`${fmtInt(Number(v))} (${fmtPct((Number(v) / (total || 1)) * 100, 1)})`, "Helldivers"]}
              />
            </PieChart>
          </ResponsiveContainer>
        </div>
        <ol className="dist-list">
          {top.map((p) => (
            <li key={p.index}>
              <button
                className="link caps"
                style={{ color: factionColor(p.enemy) }}
                onClick={() => {
                  select(p.index);
                  setView("map");
                }}
              >
                {p.name}
              </button>
              <span className="dist-bar">
                <span style={{ width: `${(p.players / (top[0]?.players || 1)) * 100}%`, background: factionColor(p.enemy) }} />
              </span>
              <span className="num">{fmtInt(p.players)}</span>
            </li>
          ))}
        </ol>
      </div>
    </Panel>
  );
}

export function StatsPage() {
  const { data: war, isLoading } = useWar();
  const s = war?.statistics;
  return (
    <div className="page scroll starfield">
      <div className="page-inner">
        <h1 className="page-title">Galactic War Statistics</h1>
        {isLoading && <Loading what="war statistics" />}
        {s && (
          <Panel title="Galactic totals" right={<span className="num">Impact ×{war!.impactMultiplier.toFixed(6)}</span>}>
            <div className="panel-body stats-grid">
              <Stat label="Helldivers online" value={fmtInt(s.playerCount)} color="var(--accent)" />
              <Stat label="Missions won" value={fmtCompact(s.missionsWon)} color="var(--good)" />
              <Stat label="Missions lost" value={fmtCompact(s.missionsLost)} color="var(--bad)" />
              <Stat label="Success rate" value={`${s.missionSuccessRate}%`} />
              <Stat label="Terminids killed" value={fmtCompact(s.terminidKills)} color="var(--f-terminids)" />
              <Stat label="Automatons killed" value={fmtCompact(s.automatonKills)} color="var(--f-automaton)" />
              <Stat label="Illuminate killed" value={fmtCompact(s.illuminateKills)} color="var(--f-illuminate)" />
              <Stat label="Helldivers KIA" value={fmtCompact(s.deaths)} />
              <Stat label="Accidentals" value={`${fmtCompact(s.friendlies)} (${fmtPct((s.friendlies / s.deaths) * 100, 1)})`} />
              <Stat label="Shots fired" value={fmtCompact(s.bulletsFired)} />
              <Stat
                label="Kill / death"
                value={`${((s.terminidKills + s.automatonKills + s.illuminateKills) / s.deaths).toFixed(1)} : 1`}
              />
              <Stat label="Time in combat" value={`${fmtCompact(s.missionTime / 3600 / 24 / 365)} yrs`} />
            </div>
          </Panel>
        )}
        <div className="stats-cols">
          <Distribution />
          <AlertSettingsPanel />
        </div>
      </div>
    </div>
  );
}
