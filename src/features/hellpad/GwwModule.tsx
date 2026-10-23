import { BookOpen, Database, History, Info, Radio, Satellite, Shield, Target } from "lucide-react";
import type { CSSProperties } from "react";
import { APP_NAME, APP_VERSION } from "../../config";
import { FACTIONS, factionColor, factionLabel } from "../../domain/factions";
import { fmtDateTime } from "../../domain/format";
import { useHistory } from "../../history/useHistory";
import { useUi } from "../../state";
import { FactionEmblem } from "../../ui/emblems";
import { Empty, Loading, Panel } from "../../ui/primitives";
import type { Faction } from "../../api/types";

/** Galaxy-wide war log: every capture and loss the collector has recorded. */
export function GwwModule() {
  const { changes, ready } = useHistory();
  const openOptions = useUi((s) => s.openOptions);
  const openModule = useUi((s) => s.openModule);
  const showAbout = useUi((s) => s.showAbout);
  const liberated = changes.filter((c) => c.to === "Humans").length;
  return (
    <div className="gww">
      <Panel
        title="Galactic war log"
        icon={History}
        right={
          ready && (
            <span className="num">
              <span style={{ color: "var(--f-humans)" }}>{liberated} liberated</span> ·{" "}
              <span style={{ color: "var(--bad)" }}>{changes.length - liberated} lost</span>
            </span>
          )
        }
      >
        {!ready ? (
          <Loading what="the war archive" rows={3} />
        ) : changes.length === 0 ? (
          <Empty>No planet has changed hands since tracking began.</Empty>
        ) : (
          <ol className="wl">
            {changes.map((c) => {
              const won = c.to === "Humans";
              const color = factionColor(won ? "Humans" : c.to);
              return (
                <li key={c.id} className="wl-row">
                  <span className="wl-mark" style={{ "--k": color } as CSSProperties}>
                    <FactionEmblem faction={won ? "Humans" : c.to} size={14} />
                  </span>
                  <div>
                    <div className="wl-title caps">
                      <button
                        className="link"
                        style={{ color: "var(--text-hi)" }}
                        onClick={(e) => openOptions({ planet: c.planet, x: e.clientX, y: e.clientY })}
                      >
                        {c.planetName}
                      </button>{" "}
                      <span style={{ color }}>
                        {won ? `liberated from ${factionLabel(c.from)}` : `fell to ${factionLabel(c.to)}`}
                      </span>
                    </div>
                    <div className="wl-sub">{fmtDateTime(c.at)}</div>
                  </div>
                </li>
              );
            })}
          </ol>
        )}
      </Panel>
      <div className="gww-links">
        <button className="dp-link" onClick={() => openModule("legend")}>
          <BookOpen size={16} />
          <span>
            <b>Icons legend</b>
            Every mark used on the map and cards
          </span>
        </button>
        <button className="dp-link" onClick={() => showAbout(true)}>
          <Info size={16} />
          <span>
            <b>About</b>
            Data sources, history and alerts
          </span>
        </button>
      </div>
    </div>
  );
}

export function AboutModule() {
  return (
    <Panel title={`${APP_NAME} · v${APP_VERSION}`} icon={Info}>
      <div className="about">
        <p>
          A companion for the Helldivers 2 Galactic War: what is being fought, where, how fast it moves, what the Major
          Order needs, and what has changed.
        </p>
        <h3>Where the data comes from</h3>
        <p>
          A collector polls the community Helldivers 2 API (api.helldivers2.dev) every minute and stores snapshots in a
          shared database. The app only reads that database, so every user sees the same history from the day the
          collector first ran. Planet ownership changes, campaign progress and galaxy-wide telemetry are kept; that
          record is what powers each planet's war history.
        </p>
        <h3>Freshness</h3>
        <p>
          The status strip under the top bar turns to <b style={{ color: "var(--warn)" }}>STALE</b> when no new data has
          arrived for ten minutes, and to <b style={{ color: "var(--bad)" }}>LINK LOST</b> when the database can't be
          reached. Either way the last known data stays on screen.
        </p>
        <h3>Not affiliated</h3>
        <p>
          A fan project, not affiliated with Arrowhead Game Studios or Sony Interactive Entertainment. Faction marks and
          artwork in the app are drawn for it; no game assets are used.
        </p>
      </div>
    </Panel>
  );
}

const MARKS: { icon: React.ReactNode; name: string; text: string }[] = [
  {
    icon: (
      <svg width="28" height="28" viewBox="-14 -14 28 28" aria-hidden>
        <circle r="9" fill="none" stroke="var(--f-automaton)" strokeWidth="1.5" />
        <circle r="3" fill="var(--f-automaton)" stroke="#fff" strokeWidth="0.8" />
      </svg>
    ),
    name: "Active planet",
    text: "A pulsing ring on the map: Helldivers are fighting for this planet right now.",
  },
  {
    icon: (
      <svg width="28" height="28" viewBox="-14 -14 28 28" aria-hidden>
        <circle r="8" fill="none" stroke="var(--cta)" strokeWidth="1.5" strokeDasharray="3 2" />
        <circle r="3" fill="var(--f-humans)" stroke="#fff" strokeWidth="0.8" />
      </svg>
    ),
    name: "Major Order target",
    text: "A dashed yellow ring on the map, and a target badge on cards and planet rows.",
  },
  {
    icon: (
      <svg width="28" height="28" viewBox="0 0 28 28" aria-hidden>
        <line x1="2" y1="22" x2="26" y2="6" stroke="var(--f-terminids)" strokeWidth="1.5" strokeDasharray="4 3" />
      </svg>
    ),
    name: "Attack line",
    text: "A dashed line from an enemy world to the planet it is attacking.",
  },
  {
    icon: (
      <svg width="28" height="28" viewBox="0 0 28 28" aria-hidden>
        <path d="M3 20 C 6 6, 18 4, 25 10 S 20 26, 3 20Z" fill="none" stroke="var(--f-illuminate)" strokeWidth="2" />
      </svg>
    ),
    name: "Front contour",
    text: "Regions view: the outline of one enemy's contiguous territory. Super Earth space is left open.",
  },
  {
    icon: <Target size={20} color="var(--cta)" />,
    name: "Major Order badge",
    text: "The planet is an objective of the current Major Order.",
  },
  {
    icon: <Satellite size={20} color="var(--f-humans)" />,
    name: "Democracy Space Station",
    text: "The DSS is in orbit; see Hellpad › Space Stations for its tactical actions.",
  },
  {
    icon: <Shield size={20} color="var(--text-hi)" />,
    name: "Defense",
    text: "The planet belongs to Super Earth and is under attack until the event timer runs out.",
  },
  {
    icon: <Radio size={20} color="var(--info)" />,
    name: "Dispatch",
    text: "A message from Super Earth. Planet names inside it are links.",
  },
  {
    icon: <Database size={20} color="var(--text-dim)" />,
    name: "Stale data",
    text: "No new data from the collector for ten minutes; numbers are the last known values.",
  },
];

export function LegendModule() {
  return (
    <div className="legend-page">
      <Panel title="Factions" icon={BookOpen}>
        <ul className="legend-list factions">
          {(Object.keys(FACTIONS) as Faction[]).map((f) => (
            <li key={f}>
              <FactionEmblem faction={f} size={26} />
              <b className="caps" style={{ color: factionColor(f) }}>
                {FACTIONS[f].label}
              </b>
            </li>
          ))}
        </ul>
      </Panel>
      <Panel title="Marks and badges" icon={BookOpen}>
        <ul className="legend-list">
          {MARKS.map((m) => (
            <li key={m.name}>
              <span className="legend-icon">{m.icon}</span>
              <div>
                <b className="caps">{m.name}</b>
                <p>{m.text}</p>
              </div>
            </li>
          ))}
        </ul>
      </Panel>
    </div>
  );
}
