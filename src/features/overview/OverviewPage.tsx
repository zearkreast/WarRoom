import { Map as MapIcon, PieChart as PieIcon, Users } from "lucide-react";
import { useMemo, useState, type CSSProperties } from "react";
import { useCampaigns, useDispatches, useFreshPlanets, useWar } from "../../api/queries";
import type { Faction } from "../../api/types";
import { factionLabel } from "../../domain/factions";
import { fmtCompact, fmtInt, fmtPct, sestClock, sestDate, warDay } from "../../domain/format";
import { buildSectors } from "../../domain/galaxy";
import { playersByEnemy } from "../../domain/snapshot";
import { fmtLongDuration } from "../../domain/warHistory";
import { useUi } from "../../state";
import { FactionEmblem } from "../../ui/emblems";
import { NeonRing } from "../../ui/NeonRing";
import { Panel, Tile, useNow } from "../../ui/primitives";
import { MajorOrderList } from "../major-order/MajorOrder";
import { NewsCard } from "../news/NewsCard";
import { CHART, ENEMIES, WarActivity } from "./WarActivity";
import "./overview.css";

const TOP_PLANETS = 5;

function Distribution() {
  const war = useWar().data;
  const campaigns = useCampaigns().data;
  const planets = useFreshPlanets().data;
  const now = useNow(1000);
  const total = war?.statistics.playerCount ?? 0;

  const { slices, onFronts } = useMemo(() => {
    const list = (campaigns ?? [])
      .map((c) => ({
        name: c.planet.name,
        players: c.planet.statistics.playerCount,
        enemy: (c.planet.event?.faction ?? c.planet.currentOwner) as Faction,
      }))
      .sort((a, b) => b.players - a.players);
    const onFronts = list.reduce((n, p) => n + p.players, 0);
    const top = list.slice(0, TOP_PLANETS).map((p) => ({ ...p, color: CHART[p.enemy] }));
    const rest = onFronts - top.reduce((n, p) => n + p.players, 0);
    const slices = rest > 0 ? [...top, { name: "Others", players: rest, enemy: null, color: "var(--faint)" }] : top;
    return { slices, onFronts };
  }, [campaigns]);
  const [hovered, setHovered] = useState<string | null>(null);
  const hoveredSlice = slices.find((p) => p.name === hovered);
  const fronts = playersByEnemy(campaigns ?? []);
  const elsewhere = Math.max(0, total - fronts.Terminids - fronts.Automaton - fronts.Illuminate);

  const s = war?.statistics;
  const missions = s ? s.missionsWon + s.missionsLost : 0;
  const kills = s ? s.terminidKills + s.automatonKills + s.illuminateKills : 0;
  const rows: [string, string][] = s
    ? [
        ["Missions completed", `${fmtInt(s.missionsWon)} (${fmtPct((s.missionsWon / (missions || 1)) * 100, 1)})`],
        ["Missions failed", `${fmtInt(s.missionsLost)} (${fmtPct((s.missionsLost / (missions || 1)) * 100, 1)})`],
        ["Mission dive time", fmtLongDuration(s.missionTime / 1000)],
        ["Shots fired", fmtInt(s.bulletsFired)],
        ["Projectile hits", fmtInt(s.bulletsHit)],
        ["Hits to shot ratio", s.bulletsFired ? `${(s.bulletsHit / s.bulletsFired).toFixed(4)} : 1` : "—"],
        ["Kill to death ratio", s.deaths ? `${(kills / s.deaths).toFixed(2)} : 1` : "—"],
        ["Dead Terminids", fmtInt(s.terminidKills)],
        ["Dead Automatons", fmtInt(s.automatonKills)],
        ["Dead Illuminate", fmtInt(s.illuminateKills)],
        ["Helldivers KIA", fmtInt(s.deaths)],
        ["Accidentals", `${fmtInt(s.friendlies)} (${fmtPct((s.friendlies / (s.deaths || 1)) * 100, 1)})`],
      ]
    : [];

  return (
    <Panel title="Distribution of Democracy" icon={PieIcon} className="ov-dist">
      <div className="ov-dist-top">
        <span className="num muted">
          SEST {sestClock(now)} · {sestDate(now)}
        </span>
        {war && <span className="strip-day caps num">Day {warDay(war.started, now)}</span>}
      </div>
      <div className="ov-total">
        <Users size={20} />
        <b className="num">{fmtInt(total)}</b>
        <span className="caps">Helldivers online</span>
      </div>
      <div className="ov-summary">
        <Tile label="Planets charted" value={planets?.length ?? "—"} rule="var(--cta)" />
        <Tile label="Sectors" value={planets ? new Set(planets.map((p) => p.sector)).size : "—"} rule="var(--cta)" />
        <Tile label="Active fronts" value={campaigns?.length ?? "—"} rule="var(--cta)" />
        <Tile
          label="Impact mod"
          value={war ? war.impactMultiplier.toFixed(6) : "—"}
          color="var(--info)"
          rule="var(--info)"
        />
      </div>
      {slices.length > 0 && (
        <div className="ov-donut">
          <NeonRing
            size={150}
            label={`Helldivers by planet: ${slices.map((p) => `${p.name} ${fmtCompact(p.players)}`).join(", ")}`}
            segments={slices.map((p) => ({ key: p.name, label: p.name, value: p.players, color: p.color }))}
            active={hovered}
            onActive={setHovered}
            center={
              hoveredSlice ? (
                <div key={hoveredSlice.name} className="nr-readout ov-readout" style={{ color: hoveredSlice.color }}>
                  <span className="caps">{hoveredSlice.name}</span>
                  <b className="num">{fmtPct((hoveredSlice.players / (onFronts || 1)) * 100, 0)}</b>
                  <span className="pl num">
                    <Users size={11} /> {fmtInt(hoveredSlice.players)}
                  </span>
                </div>
              ) : (
                <div key="total" className="ov-donut-center">
                  <b className="num">{fmtCompact(onFronts)}</b>
                  <span className="caps">on fronts</span>
                </div>
              )
            }
          />
          <ul className="ov-legend" onPointerLeave={() => setHovered(null)}>
            {slices.map((p) => (
              <li
                key={p.name}
                className={hovered === p.name ? "on" : ""}
                style={{ "--k": p.color } as CSSProperties}
                onPointerEnter={() => setHovered(p.name)}
              >
                <span className="ov-legend-name caps">{p.name}</span>
                <span className="num">{fmtCompact(p.players)}</span>
                <span className="num muted">{fmtPct((p.players / (onFronts || 1)) * 100, 0)}</span>
              </li>
            ))}
          </ul>
        </div>
      )}
      <div className="ov-fronts">
        {ENEMIES.map((f) => (
          <div
            key={f}
            className="ov-front"
            style={{ "--k": CHART[f], "--t": `var(--f-${f.toLowerCase()})` } as CSSProperties}
          >
            <FactionEmblem faction={f} size={18} />
            <span className="caps">{factionLabel(f)}</span>
            <b className="num">{fmtInt(fronts[f as keyof typeof fronts])}</b>
            <span className="num muted">{fmtPct((fronts[f as keyof typeof fronts] / (total || 1)) * 100, 1)}</span>
            <div className="ov-front-bar">
              <span style={{ width: `${(fronts[f as keyof typeof fronts] / (total || 1)) * 100}%` }} />
            </div>
          </div>
        ))}
        <div className="ov-front" style={{ "--k": "var(--faint)" } as CSSProperties}>
          <FactionEmblem faction="Humans" size={18} color="var(--text-dim)" />
          <span className="caps">Elsewhere</span>
          <b className="num">{fmtInt(elsewhere)}</b>
          <span className="num muted">{fmtPct((elsewhere / (total || 1)) * 100, 1)}</span>
          <div className="ov-front-bar">
            <span style={{ width: `${(elsewhere / (total || 1)) * 100}%` }} />
          </div>
        </div>
      </div>
      {rows.length > 0 && (
        <dl className="ov-stats">
          {rows.map(([k, v]) => (
            <div key={k}>
              <dt>{k}</dt>
              <dd className="num">{v}</dd>
            </div>
          ))}
        </dl>
      )}
    </Panel>
  );
}

/** The sector map seen at an angle. */
function TheaterOfWar() {
  const planets = useFreshPlanets().data;
  const setView = useUi((s) => s.setView);
  const sectors = useMemo(() => (planets ? buildSectors(planets) : []), [planets]);
  if (!planets) return null;
  return (
    <Panel title="Theater of war" icon={MapIcon}>
      <button className="ov-theater" onClick={() => setView("map")} aria-label="Open the galaxy map">
        <svg viewBox="-560 -560 1120 1120" aria-hidden>
          {sectors.map((s) => (
            <path
              key={s.name}
              d={s.path}
              fill={CHART[s.owner]}
              fillOpacity={s.owner === "Humans" ? 0.16 : 0.55}
              stroke="var(--surface-1)"
              strokeWidth={4}
            />
          ))}
          <circle r={36} fill="var(--c-humans)" opacity={0.5} />
          <circle r={12} fill="#eaf4fa" />
        </svg>
      </button>
    </Panel>
  );
}

export function OverviewPage() {
  const dispatches = useDispatches().data;
  const now = useNow(60_000);
  const latest = useMemo(() => [...(dispatches ?? [])].sort((a, b) => b.id - a.id)[0], [dispatches]);
  return (
    <div className="page scroll starfield">
      <div className="page-inner">
        <div className="page-head">
          <h1 className="page-title">High Command overview</h1>
        </div>
        <div className="ov-grid">
          <Distribution />
          <div className="ov-right">
            <MajorOrderList />
            {latest && <NewsCard d={latest} now={now} />}
          </div>
        </div>
        <WarActivity />
        <TheaterOfWar />
      </div>
    </div>
  );
}
