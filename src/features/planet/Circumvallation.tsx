import { Fragment, useMemo } from "react";
import { useCampaigns, useFreshPlanets } from "../../api/queries";
import type { Planet } from "../../api/types";
import { factionColor } from "../../domain/factions";
import { circumvallation, neighbours, type Circumvallation } from "../../domain/sieges";
import "./siege.css";

/** The siege a front would open, if liberating it would leave enemy planets surrounded. */
export function useCircumvallation(p: Planet): Circumvallation | null {
  const planets = useFreshPlanets().data;
  const campaigns = useCampaigns().data;
  const links = useMemo(() => (planets ? neighbours(planets) : null), [planets]);
  return useMemo(() => {
    if (!planets || !campaigns || !links) return null;
    const fronts = new Set(campaigns.map((c) => c.planet.index));
    if (!fronts.has(p.index)) return null;
    return circumvallation(planets.find((x) => x.index === p.index) ?? p, planets, links, fronts);
  }, [p, planets, campaigns, links]);
}

/** A siege marker drawn for the app: arrows closing a ring on one enemy world, or on two. */
export function SiegeMark({ double, size = 30 }: { double?: boolean; size?: number }) {
  return (
    <svg className="siege-mark" width={size} height={size} viewBox="-12 -12 24 24" aria-hidden>
      <circle r="7.4" className="siege-ring" />
      {[0, 90, 180, 270].map((a) => (
        <path key={a} d="M-2.6 -11.3L0 -8.7L2.6 -11.3" transform={`rotate(${a})`} className="siege-arrow" />
      ))}
      {double ? (
        <>
          <circle cx="-2.7" r="2.2" className="siege-core" />
          <circle cx="2.7" r="2.2" className="siege-core" />
        </>
      ) : (
        <circle r="3.1" className="siege-core" />
      )}
    </svg>
  );
}

const Name = ({ p }: { p: Planet }) => <b style={{ color: factionColor(p.currentOwner) }}>{p.name}</b>;

function Names({ list }: { list: Planet[] }) {
  return (
    <>
      {list.map((p, i) => (
        <Fragment key={p.index}>
          {i > 0 && (i === list.length - 1 ? " and " : ", ")}
          <Name p={p} />
        </Fragment>
      ))}
    </>
  );
}

/** Blue when the ring is worth closing, bright for two sieges, grey when liberating the targets directly is quicker. */
export function CircumvallationNote({ front, c, className }: { front: Planet; c: Circumvallation; className?: string }) {
  const double = c.targets.length > 1;
  const title = double ? "Double circumvallation" : "Circumvallation";
  return (
    <section className={`siege ${double ? "double" : ""} ${c.worth ? "" : "direct"} ${className ?? ""}`} aria-label={title}>
      <SiegeMark double={double} />
      <div className="siege-body">
        <span className="siege-kind caps">Siege objective</span>
        <b className="siege-title caps">{title}</b>
        {!c.worth && (
          <p className="siege-notice">
            <b className="caps">Notice:</b> liberating <Names list={c.direct} /> directly is quicker than closing the ring.
          </p>
        )}
        <p>
          Liberating <Name p={front} /> leaves {double && "both "}
          <Names list={c.targets} /> with only Super Earth planets around {double ? "them" : "it"}. Held, the ring lets
          the SEAF lay {double ? "two sieges" : "siege"}, turning {double ? "their" : "its"} resistance into slow
          liberation.
        </p>
      </div>
    </section>
  );
}
