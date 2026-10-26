import { TrendingUp, Users } from "lucide-react";
import type { Faction, Planet } from "../../api/types";
import { resistanceLevel } from "../../domain/biomes";
import { cityHoursLeft, citiesOf, sizeLabel, type City, type CityState } from "../../domain/cities";
import { factionColor } from "../../domain/factions";
import { fmtDuration, fmtInt, fmtPct } from "../../domain/format";
import { useUi } from "../../state";
import { OrderBar, useCountUp } from "../major-order/OrderBar";
import "../major-order/majorOrder.css";
import "./cities.css";

export const STATE_LABEL: Record<CityState, string> = {
  liberating: "Liberating",
  locked: "Locked",
  held: "Enemy held",
  secured: "Secured",
  destroyed: "Destroyed",
  rebuilt: "Rebuilt",
  intact: "Intact",
};

function star(x: number, y: number, r: number) {
  return Array.from({ length: 10 }, (_, i) => {
    const a = (Math.PI / 5) * i - Math.PI / 2;
    const d = i % 2 ? r * 0.45 : r;
    return `${(x + Math.cos(a) * d).toFixed(2)},${(y + Math.sin(a) * d).toFixed(2)}`;
  }).join(" ");
}

/** A city's icon as the game draws it: towers under one star per size class. */
export function CityMark(props: { stars: number; state: CityState; size?: number; holder?: Faction }) {
  const { stars, state, size = 32, holder } = props;
  return (
    <svg
      className={`city-mark ${state}`}
      width={size}
      height={size}
      viewBox="0 0 24 24"
      style={holder ? { color: factionColor(holder) } : undefined}
      aria-hidden
    >
      <g className="city-stars">
        {Array.from({ length: stars }, (_, i) => (
          <polygon key={i} points={star(12 + (i - (stars - 1) / 2) * 5.6, 3.2, 2.7)} />
        ))}
      </g>
      <path className="city-towers" d="M4.5 23V12.5l3.5-2V23ZM10 23V9l2-3 2 3v14ZM16 23V10.5l3.5 2V23Z" />
      {state === "destroyed" && <path className="city-razed" d="M3 9l18 14M21 9L3 23" />}
    </svg>
  );
}

const signed = (n: number) => `${n >= 0 ? "+" : "−"}${fmtPct(Math.abs(n), 2)}/h`;

function Eta({ c }: { c: City }) {
  const hours = cityHoursLeft(c);
  if (hours != null) return <>{fmtDuration(hours * 3_600_000)}</>;
  if (c.rate != null && c.rate < 0) return <span className="bad">Losing ground</span>;
  return <span className="muted">Measuring…</span>;
}

export function CityRow(props: { c: City; enemy: Faction | undefined; full?: boolean; onOpen?(): void }) {
  const { c, enemy, full, onOpen } = props;
  const fought = !!enemy && c.percent != null && (c.state === "liberating" || c.percent > 0);
  const contested = !!enemy && (c.state === "liberating" || c.state === "locked" || c.state === "held");
  const shown = useCountUp(fought ? c.percent! : 0);
  const res = c.resistance != null ? resistanceLevel(c.resistance) : null;
  return (
    <li
      className={`city ${c.state} ${onOpen ? "openable" : ""}`}
      role={onOpen ? "button" : undefined}
      tabIndex={onOpen ? 0 : undefined}
      aria-label={onOpen ? `${c.name}: open the city` : undefined}
      onClick={onOpen}
      onKeyDown={(e) => {
        if (onOpen && (e.key === "Enter" || e.key === " ")) {
          e.preventDefault();
          onOpen();
        }
      }}
    >
      <CityMark stars={c.stars} state={c.state} holder={c.holder} />
      <div className="city-main">
        <div className="city-head">
          <b className="caps">{c.name}</b>
          <span className="city-state caps">
            {c.waiting ? `Opens at ${fmtPct(c.opensAt!, 1)}` : STATE_LABEL[c.state]}
          </span>
        </div>
        <span className="city-size caps">
          {sizeLabel(c.size)}
          {c.formerly && <> · formerly {c.formerly}</>}
          {c.players > 0 && (
            <>
              {" · "}
              <Users size={11} aria-hidden /> <span className="num">{fmtInt(c.players)}</span>
            </>
          )}
        </span>
        {fought && (
          <OrderBar
            slim={!full}
            shown={shown}
            value={c.percent!}
            label={`${c.name} liberated`}
            valueText={fmtPct(c.percent!, 1)}
            tone="var(--f-humans)"
            rest={factionColor(enemy)}
          />
        )}
        {fought && full && (
          <dl className="city-stats">
            <div>
              <dt className="caps">Liberated</dt>
              <dd className="num">{fmtPct(c.percent!, 2)}</dd>
            </div>
            <div>
              <dt className="caps">Resistance</dt>
              <dd className="num">
                {res ? (
                  <>
                    {fmtPct(c.resistance!, 2)}/h <span style={{ color: res.color }}>{res.label}</span>
                  </>
                ) : (
                  <span className="muted">—</span>
                )}
              </dd>
            </div>
            <div>
              <dt className="caps">Progress</dt>
              <dd className={`num ${c.rate == null ? "muted" : c.rate >= 0 ? "good" : "bad"}`}>
                {c.rate == null ? "Measuring…" : signed(c.rate)}
              </dd>
            </div>
            <div>
              <dt className="caps">Liberated in</dt>
              <dd className="num">
                <Eta c={c} />
              </dd>
            </div>
          </dl>
        )}
        {fought && !full && (
          <span className="city-line num caps">
            {fmtPct(c.percent!, 1)}
            {c.rate != null && <> · {signed(c.rate)}</>}
            {cityHoursLeft(c) != null && (
              <>
                {" · "}
                <Eta c={c} />
              </>
            )}
          </span>
        )}
        {contested && c.boost != null && (
          <span className="city-boost caps">
            <TrendingUp size={12} aria-hidden />
            <b className="num">+{fmtPct(c.boost, 1)}</b> {full ? "planet liberation when freed" : "boost"}
          </span>
        )}
        {full && c.note && <p className="city-note">{c.note}</p>}
        {full && c.description && <p>{c.description}</p>}
      </div>
    </li>
  );
}

export const enemyOfCities = (p: Planet) => p.event?.faction ?? (p.currentOwner === "Humans" ? undefined : p.currentOwner);

/** A planet's cities: size class, standing, and while one is fought over its progress, pace and payoff. Each opens its own page. */
export function CityList({ p, full }: { p: Planet; full?: boolean }) {
  const openCity = useUi((s) => s.openCity);
  const enemy = enemyOfCities(p);
  return (
    <ul className={`cities ${full ? "full" : ""}`}>
      {citiesOf(p).map((c) => (
        <CityRow key={c.id} c={c} enemy={enemy} full={full} onOpen={() => openCity({ planet: p.index, region: c.id })} />
      ))}
    </ul>
  );
}
