import {
  BookOpen,
  Building2,
  ChevronLeft,
  CloudFog,
  Database,
  GraduationCap,
  History,
  Info,
  Landmark,
  Lightbulb,
  Newspaper,
  Radio,
  Satellite,
  Shield,
  Target,
  type LucideIcon,
} from "lucide-react";
import { useState, type CSSProperties } from "react";
import { APP_NAME, APP_VERSION } from "../../config";
import { COMPANIES, MANUALS, type Company, type Manual } from "../../data/gww";
import { FACTIONS, factionColor, factionLabel } from "../../domain/factions";
import { fmtDateTime } from "../../domain/format";
import { useHistory } from "../../history/useHistory";
import { useUi } from "../../state";
import { FactionEmblem } from "../../ui/emblems";
import { Empty, Loading, Panel } from "../../ui/primitives";
import type { Faction } from "../../api/types";

type Page = { kind: "log" } | { kind: "manual"; id: string } | { kind: "company"; name: string } | null;

/** Galaxy-wide war log: every capture and loss the collector has recorded. */
function WarLog() {
  const { changes, ready } = useHistory();
  const openOptions = useUi((s) => s.openOptions);
  const liberated = changes.filter((c) => c.to === "Humans").length;
  return (
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
  );
}

function ManualPage({ m }: { m: Manual }) {
  return (
    <article className="gww-page">
      <p className="gww-address num">{m.address}</p>
      <h2 className="gww-title">{m.title}</h2>
      <p className="gww-lead">{m.summary}</p>
      {m.sections.map((sec) => (
        <section key={sec.heading}>
          <h3>{sec.heading}</h3>
          <p>{sec.text}</p>
        </section>
      ))}
      {m.tips.map((t) => (
        <aside key={t} className="gww-tip">
          <Lightbulb size={16} aria-hidden />
          <p>
            <b className="caps">Field tip</b> {t}
          </p>
        </aside>
      ))}
    </article>
  );
}

function CompanyPage({ c }: { c: Company }) {
  return (
    <article className="gww-page">
      <p className="gww-address num">{c.address}</p>
      <h2 className="gww-title">{c.name}</h2>
      <p className="gww-lead caps">{c.trade}</p>
      <p>{c.text}</p>
    </article>
  );
}

function Site(props: { address: string; title: string; text: string; icon: LucideIcon; onClick(): void }) {
  const Icon = props.icon;
  return (
    <button className="gww-site" onClick={props.onClick}>
      <span className="gww-address num">{props.address}</span>
      <span className="gww-site-title">
        <Icon size={16} aria-hidden /> <b>{props.title}</b>
      </span>
      <span className="gww-site-text">{props.text}</span>
    </button>
  );
}

/**
 * The Galactic War Web: the Federation's net, as a hub of sites. The war log, training manuals on how
 * the war works, the Federation's corporations, and a way through to Strohmann News.
 */
export function GwwModule() {
  const [page, setPage] = useState<Page>(null);
  const setView = useUi((s) => s.setView);
  const { changes } = useHistory();
  const manual = page?.kind === "manual" ? MANUALS.find((m) => m.id === page.id) : undefined;
  const company = page?.kind === "company" ? COMPANIES.find((c) => c.name === page.name) : undefined;
  if (page)
    return (
      <div className="gww">
        <button className="link gww-back caps" onClick={() => setPage(null)}>
          <ChevronLeft size={14} /> Galactic War Web
        </button>
        {page.kind === "log" && <WarLog />}
        {manual && <ManualPage m={manual} />}
        {company && <CompanyPage c={company} />}
      </div>
    );
  return (
    <div className="gww">
      <section className="gww-group">
        <h2 className="gww-group-title caps">Federation</h2>
        <div className="gww-sites">
          <Site
            address="se://hc/war-log"
            title="Galactic war log"
            text={
              changes.length === 1
                ? "1 planet has changed hands since tracking began."
                : changes.length
                  ? `${changes.length} planets have changed hands since tracking began.`
                  : "Every planet that changed hands."
            }
            icon={History}
            onClick={() => setPage({ kind: "log" })}
          />
          <Site
            address="se://strohmann.news"
            title="Strohmann News"
            text="Dispatches and campaign news from Super Earth."
            icon={Newspaper}
            onClick={() => setView("news")}
          />
        </div>
      </section>
      <section className="gww-group">
        <h2 className="gww-group-title caps">Training manuals</h2>
        <div className="gww-sites">
          {MANUALS.map((m) => (
            <Site
              key={m.id}
              address={m.address}
              title={m.title}
              text={m.summary}
              icon={GraduationCap}
              onClick={() => setPage({ kind: "manual", id: m.id })}
            />
          ))}
        </div>
      </section>
      <section className="gww-group">
        <h2 className="gww-group-title caps">Corporations</h2>
        <div className="gww-sites">
          {COMPANIES.map((c) => (
            <Site
              key={c.name}
              address={c.address}
              title={c.name}
              text={c.trade}
              icon={Building2}
              onClick={() => setPage({ kind: "company", name: c.name })}
            />
          ))}
        </div>
      </section>
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
          A collector polls the community Helldivers 2 API (api.helldivers2.dev) every one to five minutes, and the
          game's own API for Galactic Campaigns and planet effects, and stores snapshots in a shared database. The app
          only reads that database, so every user sees the same history from the day the collector first ran. Planet
          ownership changes, campaign progress, galaxy-wide telemetry and the space station's jumps are kept; that
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
          map artwork are drawn for the app. The biome landscapes and the black hole and fractured planet pictures are the game's, taken from the Helldivers Wiki
          (helldivers.wiki.gg), and belong to their owners. Effect names come from the Helldivers 2 community's
          data set (github.com/helldivers-2/json, MIT licence).
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
    icon: <Landmark size={20} color="var(--cta)" />,
    name: "Point of interest",
    text: "A pawn beside a planet once zoomed in: a city, a TCS+ tower network, a base or another site the game marks.",
  },
  {
    icon: <Shield size={20} color="var(--f-humans)" />,
    name: "SEAF and sub-faction pawns",
    text: "Heavy SEAF presence in blue; an enemy unit such as the Incineration Corps in that enemy's colour.",
  },
  {
    icon: <CloudFog size={20} color="var(--f-terminids)" />,
    name: "The Gloom",
    text: "An amber cloud over planets in or on the edge of the Gloom, in every overlay.",
  },
  {
    icon: (
      <svg width="28" height="28" viewBox="-14 -14 28 28" aria-hidden>
        <circle r="7" fill="#c8481c" />
        <path d="M-5,-2.5L-1,-0.6L1.4,-4.2M-1,-0.6L0.6,3.6L4.6,2.2" fill="none" stroke="#2a0904" strokeWidth="1" />
      </svg>
    ),
    name: "Scorched and fractured worlds",
    text: "Planets razed in the war are drawn burnt; those torn apart by the Meridian singularity, as debris.",
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
