import {
  ArrowLeft,
  BookOpen,
  CloudFog,
  Database,
  History,
  House,
  Info,
  Landmark,
  Lightbulb,
  Lock,
  Newspaper,
  Radio,
  RefreshCw,
  Satellite,
  Search,
  Shield,
  Target,
} from "lucide-react";
import { useCallback, useEffect, useRef, useState, type CSSProperties, type ReactNode } from "react";
import { APP_NAME, APP_VERSION } from "../../config";
import { ADS, COMPANIES, MANUALS, MINISTRIES, TIPS, WIKI, WIKI_LICENCE } from "../../data/gww";
import { FACTIONS, factionColor } from "../../domain/factions";
import { useHistory } from "../../history/useHistory";
import { useUi } from "../../state";
import { FactionEmblem } from "../../ui/emblems";
import { Empty, OutLink, Panel } from "../../ui/primitives";
import type { Faction } from "../../api/types";
import { iconUrl } from "./gwwArt";
import { AdBox, CompanyMark, CompanyPage, ManualIcon, ManualPage, MinistryPage, WarLog } from "./GwwPages";
import "./gww.css";

type Page =
  | { kind: "home" }
  | { kind: "log" }
  | { kind: "manual"; id: string }
  | { kind: "ministry"; id: string }
  | { kind: "company"; name: string };

const HOME = "se://gww";

function addressOf(p: Page): string {
  if (p.kind === "log") return "se://hc/war-log";
  if (p.kind === "manual") return MANUALS.find((m) => m.id === p.id)?.address ?? HOME;
  if (p.kind === "ministry") return MINISTRIES.find((m) => m.id === p.id)?.address ?? HOME;
  if (p.kind === "company") return COMPANIES.find((c) => c.name === p.name)?.address ?? HOME;
  return HOME;
}

const pickOne = <T,>(list: T[]): T => list[Math.floor(Math.random() * list.length)];

function Site(props: {
  address: string;
  title: string;
  text: string;
  icon: ReactNode;
  color?: string;
  wide?: boolean;
  onClick(): void;
}) {
  return (
    <button
      className={`gww-site ${props.wide ? "wide" : ""}`}
      style={props.color ? ({ "--site": props.color } as CSSProperties) : undefined}
      onClick={props.onClick}
    >
      <span className="gww-site-icon">{props.icon}</span>
      <span className="gww-site-body">
        <span className="gww-address num">{props.address}</span>
        <b className="gww-site-title">{props.title}</b>
        <span className="gww-site-text">{props.text}</span>
      </span>
    </button>
  );
}

function Group({ title, count, children }: { title: string; count: number; children: ReactNode }) {
  return (
    <section className="gww-group">
      <h3 className="gww-group-title caps">
        {title} <span className="num">{count}</span>
      </h3>
      <div className="gww-sites">{children}</div>
    </section>
  );
}

/** One of the game's Training Manual Tips, with a button for the next. */
function TipTicker() {
  const [at, setAt] = useState(() => Math.floor(Math.random() * TIPS.length));
  return (
    <div className="gww-ticker">
      <span className="gww-ticker-label caps">
        <Lightbulb size={14} aria-hidden /> Training manual tip
      </span>
      <p className="gww-ticker-text" key={at} aria-live="polite">
        {TIPS[at]}
      </p>
      <button className="gww-ticker-next" aria-label="Next tip" title="Next tip" onClick={() => setAt((i) => (i + 1) % TIPS.length)}>
        <RefreshCw size={14} />
      </button>
    </div>
  );
}

function Home({ go }: { go(p: Page): void }) {
  const setView = useUi((s) => s.setView);
  const { changes } = useHistory();
  const [query, setQuery] = useState("");
  const [ads] = useState(() => {
    const tall = pickOne(ADS.filter((a) => a.shape === "tall"));
    const wide = ADS.filter((a) => a.shape === "wide");
    const first = pickOne(wide);
    return { tall, wide: first, banner: pickOne(wide.filter((a) => a !== first)) };
  });
  const q = query.trim().toLowerCase();
  const hit = (...text: string[]) => !q || text.some((t) => t.toLowerCase().includes(q));
  const logText =
    changes.length === 1
      ? "1 planet has changed hands since tracking began."
      : changes.length
        ? `${changes.length} planets have changed hands since tracking began.`
        : "Every planet that changed hands.";
  const official = [
    hit("Galactic war log", logText, "se://hc/war-log") && (
      <Site
        key="log"
        address="se://hc/war-log"
        title="Galactic war log"
        text={logText}
        icon={<History size={26} strokeWidth={1.5} />}
        color="var(--f-humans)"
        onClick={() => go({ kind: "log" })}
      />
    ),
    hit("Strohmann News", "Dispatches and campaign news", "se://strohmann.news") && (
      <Site
        key="news"
        address="se://strohmann.news"
        title="Strohmann News"
        text="Dispatches and campaign news from Super Earth."
        icon={<Newspaper size={26} strokeWidth={1.5} />}
        color="#f074d8"
        onClick={() => setView("news")}
      />
    ),
    ...MINISTRIES.filter((m) => hit(m.name, m.text, m.motto, m.address)).map((m) => (
      <Site
        key={m.id}
        address={m.address}
        title={m.name}
        text={m.text}
        icon={<img src={iconUrl(m.icon)} alt="" />}
        color={m.color}
        onClick={() => go({ kind: "ministry", id: m.id })}
      />
    )),
  ].filter(Boolean);
  const manuals = MANUALS.filter((m) =>
    hit(m.title, m.summary, m.address, ...m.sections.map((s) => s.heading + " " + (s.text ?? ""))),
  );
  const companies = COMPANIES.filter((c) => hit(c.name, c.trade, c.text, c.slogan ?? "", c.address));
  const none = official.length + manuals.length + companies.length === 0;
  return (
    <div className="gww-home">
      <header className="gww-mast">
        <img className="gww-mast-mark" src={iconUrl("super-earth")} alt="" />
        <div className="gww-mast-text">
          <h2 className="gww-mast-title" tabIndex={-1}>
            Galactic Wide Web
          </h2>
          <p className="caps">The Federation's net · verified by the Ministry of Truth</p>
        </div>
        <label className="gww-search">
          <Search size={14} aria-hidden />
          <input
            type="search"
            placeholder="Search the G.W.W."
            aria-label="Search the Galactic Wide Web"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
        </label>
      </header>
      <TipTicker />
      <div className="gww-home-grid">
        <div className="gww-dir">
          {none && <Empty>No site on the G.W.W. matches “{query}”.</Empty>}
          {official.length > 0 && (
            <Group title="Super Earth" count={official.length}>
              {official}
            </Group>
          )}
          {manuals.length > 0 && (
            <Group title="Training manuals" count={manuals.length}>
              {manuals.map((m) => (
                <Site
                  key={m.id}
                  address={m.address}
                  title={m.title}
                  text={m.summary}
                  icon={<ManualIcon m={m} size={30} />}
                  color="var(--cta)"
                  onClick={() => go({ kind: "manual", id: m.id })}
                />
              ))}
            </Group>
          )}
          {!q && ads.banner && <AdBox ad={ads.banner} />}
          {companies.length > 0 && (
            <Group title="Corporations" count={companies.length}>
              {companies.map((c) => (
                <button
                  key={c.name}
                  className="gww-site corp"
                  style={{ "--site": c.color } as CSSProperties}
                  onClick={() => go({ kind: "company", name: c.name })}
                >
                  <span className="gww-corp-logo">
                    <CompanyMark c={c} />
                  </span>
                  <span className="gww-site-body">
                    <span className="gww-address num">{c.address}</span>
                    <b className="gww-site-title">{c.name}</b>
                    <span className="gww-site-text">{c.slogan ?? c.trade}</span>
                  </span>
                </button>
              ))}
            </Group>
          )}
        </div>
        <aside className="gww-rail" aria-label="Sponsored">
          <AdBox ad={ads.tall} />
          <AdBox ad={ads.wide} />
        </aside>
      </div>
    </div>
  );
}

/**
 * The Galactic Wide Web: the Federation's net in a browser frame. Super Earth's own sites and its
 * ministries, training manuals on how the war works, the corporations, and adverts between them.
 */
export function GwwModule() {
  const [stack, setStack] = useState<Page[]>([{ kind: "home" }]);
  const page = stack[stack.length - 1];
  const root = useRef<HTMLDivElement>(null);
  const heading = useRef<HTMLElement | null>(null);
  const shown = useRef(stack);
  const setHeading = useCallback((el: HTMLElement | null) => {
    heading.current = el;
  }, []);
  const go = (p: Page) => setStack((s) => [...s, p]);
  const back = () => setStack((s) => (s.length > 1 ? s.slice(0, -1) : s));
  const home = () => setStack((s) => (s.length > 1 ? [...s, { kind: "home" }] : s));
  // On each navigation, not on opening: bring the page into view and move focus to its heading.
  useEffect(() => {
    if (shown.current === stack) return;
    shown.current = stack;
    root.current?.scrollIntoView({ block: "start" });
    (heading.current ?? root.current?.querySelector<HTMLElement>(".gww-mast-title"))?.focus({ preventScroll: true });
  }, [stack]);
  const manual = page.kind === "manual" ? MANUALS.find((m) => m.id === page.id) : undefined;
  const ministry = page.kind === "ministry" ? MINISTRIES.find((m) => m.id === page.id) : undefined;
  const company = page.kind === "company" ? COMPANIES.find((c) => c.name === page.name) : undefined;
  return (
    <div className="gww" ref={root}>
      <div className="gww-bar">
        <button className="gww-nav" aria-label="Back" title="Back" disabled={stack.length < 2} onClick={back}>
          <ArrowLeft size={16} />
        </button>
        <button className="gww-nav" aria-label="G.W.W. home" title="Home" disabled={page.kind === "home"} onClick={home}>
          <House size={16} />
        </button>
        <div className="gww-url num" aria-label="Address">
          <Lock size={12} aria-hidden />
          <span>{addressOf(page)}</span>
        </div>
        <span className="gww-brand-tag caps" aria-hidden>
          G.W.W.
        </span>
      </div>
      <div className="gww-load" key={stack.length} aria-hidden />
      <div className="gww-view">
        {page.kind === "home" && <Home go={go} />}
        {page.kind === "log" && <WarLog headingRef={setHeading} />}
        {manual && <ManualPage m={manual} headingRef={setHeading} open={(id) => go({ kind: "manual", id })} />}
        {ministry && (
          <MinistryPage m={ministry} headingRef={setHeading} openCompany={(name) => go({ kind: "company", name })} />
        )}
        {company && <CompanyPage c={company} headingRef={setHeading} />}
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
        <h3>Credits</h3>
        <p>
          The G.W.W.'s training manuals are adapted from the <OutLink href={WIKI.home.url}>{WIKI.home.title}</OutLink>,
          chiefly <OutLink href={WIKI.mechanics.url}>{WIKI.mechanics.title}</OutLink>; its ministry and corporation pages
          draw on the wiki's <OutLink href={WIKI.ministries.url}>{WIKI.ministries.title}</OutLink> and{" "}
          <OutLink href={WIKI.corporations.url}>{WIKI.corporations.title}</OutLink> pages. The wiki's text is shared under{" "}
          <OutLink href={WIKI_LICENCE.url}>{WIKI_LICENCE.title}</OutLink>, and so are the manuals adapted from it. Training
          manual tips and slogans are quoted from the game, as collected on the wiki's{" "}
          <OutLink href={WIKI.tips.url}>{WIKI.tips.title}</OutLink> page.
        </p>
        <p>
          The ministry, campaign and difficulty icons (vector tracings by the wiki's contributors), corporation logos and
          city adverts are the game's, taken from the Helldivers Wiki, and belong to Arrowhead Game Studios and Sony
          Interactive Entertainment.
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
