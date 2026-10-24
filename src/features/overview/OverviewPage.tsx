import { PieChart as PieIcon, Users } from "lucide-react";
import { useMemo, useState, type CSSProperties } from "react";
import { useCampaigns, useFreshPlanets, useWar } from "../../api/queries";
import type { Faction } from "../../api/types";
import { LIST_SIZE } from "../../domain/factionPlanets";
import { factionLabel } from "../../domain/factions";
import { fmtCompact, fmtInt, fmtPct, sestClock, sestDate, warDay } from "../../domain/format";
import { playersByEnemy } from "../../domain/snapshot";
import { fmtLongDuration } from "../../domain/warHistory";
import { FactionEmblem } from "../../ui/emblems";
import { NeonRing } from "../../ui/NeonRing";
import { PointerCard, pointerAt, type CardRow, type PointerAt } from "../../ui/PointerCard";
import { Panel, Tile, useNow } from "../../ui/primitives";
import { CampaignPanel } from "../galactic-campaigns/Campaign";
import { MajorOrderList } from "../major-order/MajorOrder";
import { TheaterOfWar } from "./TheaterOfWar";
import { CHART, ENEMIES, WarActivity } from "./WarActivity";
import "./overview.css";

const TOP_PLANETS = 5;
const OTHERS = "Others";
const ELSEWHERE = "Elsewhere";
type RingKey = Faction | typeof ELSEWHERE;

function Distribution() {
  const war = useWar().data;
  const campaigns = useCampaigns().data;
  const planets = useFreshPlanets().data;
  const now = useNow(1000);
  const total = war?.statistics.playerCount ?? 0;

  const { list, others, slices } = useMemo(() => {
    const list = (campaigns ?? [])
      .map((c) => ({
        name: c.planet.name,
        players: c.planet.statistics.playerCount,
        enemy: (c.planet.event?.faction ?? c.planet.currentOwner) as Faction,
      }))
      .sort((a, b) => b.players - a.players);
    const top = list.slice(0, TOP_PLANETS).map((p) => ({ ...p, color: CHART[p.enemy] }));
    const others = list.slice(TOP_PLANETS);
    const rest = others.reduce((n, p) => n + p.players, 0);
    const slices = rest > 0 ? [...top, { name: OTHERS, players: rest, enemy: null, color: "var(--faint)" }] : top;
    return { list, others, slices };
  }, [campaigns]);
  const fronts = playersByEnemy(campaigns ?? []);
  const elsewhere = Math.max(0, total - fronts.Terminids - fronts.Automaton - fronts.Illuminate);
  const arcs: { key: RingKey; label: string; value: number; color: string }[] = [
    ...ENEMIES.map((f) => ({
      key: f,
      label: factionLabel(f),
      value: fronts[f as keyof typeof fronts],
      color: CHART[f],
    })),
    { key: ELSEWHERE, label: "Elsewhere", value: elsewhere, color: "var(--faint)" },
  ];

  const [activeArc, setActiveArc] = useState<RingKey | null>(null);
  const [hoverRow, setHoverRow] = useState<string | null>(null);
  const [card, setCard] = useState<{ kind: RingKey | typeof OTHERS; at: PointerAt } | null>(null);
  const rowSlice = slices.find((p) => p.name === hoverRow);
  const arc = arcs.find((a) => a.key === (activeArc ?? rowSlice?.enemy));
  const pctOfOnline = (n: number) => fmtPct((n / (total || 1)) * 100, 1);

  const cardFor = (kind: RingKey | typeof OTHERS): CardRow[] => {
    if (kind === OTHERS)
      return ENEMIES.map((f) => ({
        key: f,
        name: factionLabel(f),
        value: others.filter((p) => p.enemy === f).reduce((n, p) => n + p.players, 0),
        color: CHART[f],
      })).filter((r) => r.value > 0);
    return list
      .filter((p) => p.enemy === kind)
      .slice(0, LIST_SIZE)
      .map((p) => ({ key: p.name, name: p.name, value: p.players }));
  };
  const cardTitle = card && (card.kind === OTHERS ? "Others" : arcs.find((a) => a.key === card.kind)?.label);
  const cardFronts = card && card.kind !== OTHERS && list.filter((p) => p.enemy === card.kind).length;

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
      {total > 0 && (
        <div className="ov-donut">
          <NeonRing
            size={150}
            label={`Helldivers by front: ${arcs.map((a) => `${a.label} ${fmtCompact(a.value)}`).join(", ")}`}
            segments={arcs}
            active={arc?.key ?? null}
            onActive={(key) => {
              setActiveArc(key as RingKey | null);
              if (!key) setCard(null);
            }}
            onPointerMove={(key, e) =>
              setCard(
                key === ELSEWHERE ? null : { kind: key as RingKey, at: pointerAt(e, e.currentTarget.ownerSVGElement!) },
              )
            }
            center={
              rowSlice ? (
                <div key={rowSlice.name} className="nr-readout ov-readout" style={{ color: rowSlice.color }}>
                  <span className="caps">{rowSlice.name}</span>
                  <b className="num">{pctOfOnline(rowSlice.players)}</b>
                  <span className="pl num">
                    <Users size={11} /> {fmtInt(rowSlice.players)}
                  </span>
                </div>
              ) : arc ? (
                <div
                  key={arc.key}
                  className="nr-readout ov-readout"
                  style={{ color: arc.key === ELSEWHERE ? "var(--text-dim)" : arc.color }}
                >
                  {arc.key !== ELSEWHERE && <FactionEmblem faction={arc.key} size={22} />}
                  <span className="caps">{arc.label}</span>
                  <b className="num">{pctOfOnline(arc.value)}</b>
                  <span className="pl num">
                    <Users size={11} /> {fmtInt(arc.value)}
                  </span>
                </div>
              ) : (
                <div key="total" className="ov-donut-center">
                  <b className="num">{fmtCompact(total)}</b>
                  <span className="caps">online</span>
                </div>
              )
            }
          />
          <ul
            className="ov-legend"
            onPointerLeave={() => {
              setHoverRow(null);
              setCard(null);
            }}
          >
            {slices.map((p) => (
              <li
                key={p.name}
                className={hoverRow === p.name ? "on" : ""}
                style={{ "--k": p.color } as CSSProperties}
                onPointerEnter={() => {
                  setHoverRow(p.name);
                  if (p.name !== OTHERS) setCard(null);
                }}
                onPointerMove={(e) => {
                  if (p.name !== OTHERS) return;
                  const ring = e.currentTarget.closest(".ov-donut")!.querySelector(".nr")!;
                  setCard({ kind: OTHERS, at: pointerAt(e, ring) });
                }}
              >
                <span className="ov-legend-name caps">{p.name}</span>
                <span className="num">{fmtCompact(p.players)}</span>
                <span className="num muted">{fmtPct((p.players / (total || 1)) * 100, 0)}</span>
              </li>
            ))}
          </ul>
        </div>
      )}
      {card && cardTitle && (
        <PointerCard
          at={card.at}
          tone={card.kind === OTHERS ? "var(--text-dim)" : CHART[card.kind as Faction]}
          icon={card.kind === OTHERS ? undefined : <FactionEmblem faction={card.kind} size={18} />}
          title={cardTitle}
          badge={card.kind === OTHERS ? `${others.length} fronts` : `${cardFronts} fronts`}
          rows={cardFor(card.kind)}
          footer={cardFronts && cardFronts > LIST_SIZE ? `+${cardFronts - LIST_SIZE} more · busiest first` : undefined}
        />
      )}
      <div className="ov-fronts">
        {ENEMIES.map((f) => (
          <div
            key={f}
            className={`ov-front ${arc?.key === f ? "on" : ""}`}
            style={{ "--k": CHART[f], "--t": `var(--f-${f.toLowerCase()})` } as CSSProperties}
            onPointerEnter={() => setActiveArc(f)}
            onPointerLeave={() => setActiveArc(null)}
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
        <div
          className={`ov-front ${arc?.key === ELSEWHERE ? "on" : ""}`}
          style={{ "--k": "var(--faint)" } as CSSProperties}
          onPointerEnter={() => setActiveArc(ELSEWHERE)}
          onPointerLeave={() => setActiveArc(null)}
        >
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

export function OverviewPage() {
  return (
    <div className="page ov-page starfield">
      <h1 className="sr-only">High Command overview</h1>
      <div className="ov-board">
        <div className="ov-col ov-col-left">
          <Distribution />
        </div>
        <div className="ov-col ov-col-mid">
          <TheaterOfWar />
        </div>
        <div className="ov-col ov-col-right">
          <MajorOrderList />
          <CampaignPanel />
        </div>
      </div>
      <div className="ov-wide">
        <WarActivity />
      </div>
    </div>
  );
}
