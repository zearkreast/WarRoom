import { useState, type CSSProperties } from "react";
import { useFreshPlanets } from "../../api/queries";
import type { StationJump } from "../../api/stations";
import { factionColor } from "../../domain/factions";
import { fmtAgo, fmtDateTime } from "../../domain/format";
import { fmtLongDuration } from "../../domain/warHistory";
import { useUi } from "../../state";
import { FactionEmblem } from "../../ui/emblems";

/** A station's FTL jumps as a route, newest first; past `limit` the rest wait behind a button. */
export function JumpRoute({ jumps, now, limit = 8 }: { jumps: StationJump[]; now: number; limit?: number }) {
  const [all, setAll] = useState(false);
  const planets = useFreshPlanets().data;
  const openDetail = useUi((s) => s.openDetail);
  const shown = all ? jumps : jumps.slice(0, limit);
  const stop = (index: number | null) => {
    const p = index == null ? undefined : planets?.find((x) => x.index === index);
    if (!p) return <span className="st-hop-planet dim caps">{index == null ? "Unknown" : `Planet ${index}`}</span>;
    const owner = p.event?.faction ?? p.currentOwner;
    return (
      <button className="st-hop-planet caps" style={{ "--tone": factionColor(owner) } as CSSProperties} onClick={() => openDetail(p.index)}>
        <FactionEmblem faction={owner} size={16} />
        <b>{p.name}</b>
        <span>{p.sector}</span>
      </button>
    );
  };
  return (
    <>
      <ol className="st-hops">
        {shown.map((j, i) => {
          // Newest first, so the stay at a destination ends with the jump before it in the list.
          const left = i === 0 ? now : jumps[i - 1].at;
          return (
            <li key={j.at} className={i === 0 ? "now" : ""}>
              <div className="st-hop-when">
                <span className="num">{fmtDateTime(j.at)}</span>
                <span className="caps">{fmtAgo(j.at, now)}</span>
              </div>
              <i className="st-hop-node" aria-hidden />
              <div className="st-hop-route">
                {stop(j.from)}
                <span className="st-hop-arrow" aria-label="to" />
                {stop(j.to)}
              </div>
              <span className="st-hop-stay caps num">
                {i === 0 ? "In orbit · " : "Stayed "}
                {fmtLongDuration(Math.max(0, (left - j.at) / 1000))}
              </span>
            </li>
          );
        })}
      </ol>
      {jumps.length > limit && (
        <button className="gww-chip gww-more" onClick={() => setAll((a) => !a)}>
          {all ? "Show fewer" : `Show all ${jumps.length} jumps`}
        </button>
      )}
    </>
  );
}
