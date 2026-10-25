import { Users } from "lucide-react";
import type { Planet } from "../../api/types";
import { citiesOf, sizeLabel, type CityState } from "../../domain/cities";
import { factionColor } from "../../domain/factions";
import { fmtInt, fmtPct } from "../../domain/format";
import { Meter } from "../../ui/primitives";
import "./cities.css";

const STATE_LABEL: Record<CityState, string> = {
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
export function CityMark({ stars, state, size = 32 }: { stars: number; state: CityState; size?: number }) {
  return (
    <svg className={`city-mark ${state}`} width={size} height={size} viewBox="0 0 24 24" aria-hidden>
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

/** A planet's cities: size class, standing, and how far each is liberated while it is fought over. */
export function CityList({ p, full }: { p: Planet; full?: boolean }) {
  const cities = citiesOf(p);
  const enemy = p.event?.faction ?? (p.currentOwner === "Humans" ? undefined : p.currentOwner);
  return (
    <ul className={`cities ${full ? "full" : ""}`}>
      {cities.map((c) => {
        const fought = enemy && c.percent != null && (c.state === "liberating" || c.percent > 0);
        return (
          <li key={c.id} className={`city ${c.state}`}>
            <CityMark stars={c.stars} state={c.state} />
            <div className="city-main">
              <div className="city-head">
                <b className="caps">{c.name}</b>
                <span className="city-state caps">{STATE_LABEL[c.state]}</span>
              </div>
              <span className="city-size caps">
                {sizeLabel(c.size)}
                {c.formerly && <> · formerly {c.formerly}</>}
              </span>
              {fought && (
                <div className="city-progress">
                  <Meter
                    value={c.percent!}
                    color="var(--f-humans)"
                    rest={factionColor(enemy)}
                    label={`${c.name} liberated`}
                  />
                  <span className="num caps">
                    {fmtPct(c.percent!, 1)} liberated
                    {c.players > 0 && (
                      <>
                        {" · "}
                        <Users size={11} aria-hidden /> {fmtInt(c.players)}
                      </>
                    )}
                  </span>
                </div>
              )}
              {full && c.note && <p className="city-note">{c.note}</p>}
              {full && c.description && <p>{c.description}</p>}
            </div>
          </li>
        );
      })}
    </ul>
  );
}
