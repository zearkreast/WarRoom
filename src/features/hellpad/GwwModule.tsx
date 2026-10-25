import {
  ArrowLeft,
  ArrowRight,
  BookOpen,
  CalendarDays,
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
import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";
import { APP_NAME, APP_VERSION } from "../../config";
import { CHAPTERS, MANUALS, TIPS, WIKI, WIKI_LICENCE } from "../../data/gww";
import { ADS, BRANDS, COMPANIES, GOV, HOLIDAYS, MINISTRIES } from "../../data/gwwSites";
import { FACTIONS, factionColor } from "../../domain/factions";
import { nextYearly, plainText } from "../../domain/gwwText";
import { useHistory } from "../../history/useHistory";
import { useUi, type GwwPage } from "../../state";
import { FactionEmblem } from "../../ui/emblems";
import { Empty, OutLink, Panel, useNow } from "../../ui/primitives";
import type { Faction } from "../../api/types";
import { iconUrl } from "./gwwArt";
import { GwwNavProvider, useGwwNav } from "./gwwNav";
import {
  AdBox,
  BrandPage,
  BrandTile,
  CompanyPage,
  CompanyTile,
  CourseProgress,
  DirPage,
  GovPage,
  ManualPage,
  ManualTile,
  MinistryPage,
  MinistryTile,
  SiteTile,
  WarLogPage,
} from "./GwwPages";
import "./gww.css";

const HOME = "se://gww";

function addressOf(p: GwwPage): string {
  if (p.kind === "manual") return MANUALS.find((m) => m.id === p.id)?.address ?? HOME;
  if (p.kind === "ministry") return MINISTRIES.find((m) => m.id === p.id)?.address ?? HOME;
  if (p.kind === "company") return COMPANIES.find((c) => c.id === p.id)?.address ?? HOME;
  if (p.kind === "brand") return `se://market/${p.id}`;
  if (p.kind === "site") return p.id === "gov" ? GOV.address : "se://hc/war-log";
  if (p.kind === "dir") return { manuals: "se://hc/manuals", gov: "se://gov/directory", corps: "se://corp", market: "se://market" }[p.id];
  return HOME;
}

const keyOf = (p: GwwPage) => ("id" in p ? `${p.kind}:${p.id}` : p.kind);

const pickOne = <T,>(list: T[]): T => list[Math.floor(Math.random() * list.length)];

function logLine(count: number): string {
  if (count === 1) return "1 planet has changed hands since tracking began.";
  return count ? `${count} planets have changed hands since tracking began.` : "Every planet that changed hands.";
}

function Group(props: { title: string; count: number; all?: string; children: ReactNode }) {
  const { follow } = useGwwNav();
  return (
    <section className="gww-group">
      <h3 className="gww-group-title caps">
        {props.title} <span className="num">{props.count}</span>
        {props.all && (
          <button className="gww-group-all" onClick={(e) => follow(props.all!, e)}>
            View all <ArrowRight size={12} aria-hidden />
          </button>
        )}
      </h3>
      <div className="gww-sites">{props.children}</div>
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

function HolidayChip() {
  const { follow } = useGwwNav();
  const now = useNow(60 * 60_000);
  const next = nextYearly(HOLIDAYS, new Date(now));
  if (!next) return null;
  const when = next.in === 0 ? "today" : next.in === 1 ? "tomorrow" : `in ${next.in} days`;
  return (
    <button className={`gww-holiday caps ${next.in <= 1 ? "soon" : ""}`} onClick={(e) => follow("site:gov", e)}>
      <CalendarDays size={13} aria-hidden /> {next.name} {when}
    </button>
  );
}

function Home({ logText }: { logText: string }) {
  const [query, setQuery] = useState("");
  const [ads] = useState(() => {
    const wide = ADS.filter((a) => a.shape === "wide");
    const first = pickOne(wide);
    return { tall: pickOne(ADS.filter((a) => a.shape === "tall")), wide: first, banner: pickOne(wide.filter((a) => a !== first)) };
  });
  const q = query.trim().toLowerCase();
  const hit = (...text: string[]) => !q || text.some((t) => plainText(t).toLowerCase().includes(q));
  const official = [
    hit(GOV.title, GOV.summary, GOV.address, ...GOV.sections.map((s) => `${s.heading} ${s.text ?? ""}`)) && (
      <SiteTile
        key="gov"
        target="site:gov"
        address={GOV.address}
        title={GOV.title}
        text={GOV.summary}
        icon={<img src={iconUrl(GOV.icon)} alt="" />}
        color="var(--f-humans)"
      />
    ),
    hit("Galactic war log", logText, "se://hc/war-log") && (
      <SiteTile
        key="log"
        target="site:log"
        address="se://hc/war-log"
        title="Galactic war log"
        text={logText}
        icon={<History size={26} strokeWidth={1.5} />}
        color="var(--f-humans)"
      />
    ),
    hit("Strohmann News", "Dispatches and campaign news", "se://strohmann.news") && (
      <SiteTile
        key="news"
        target="app:news"
        address="se://strohmann.news"
        title="Strohmann News"
        text="Dispatches and campaign news, in the tracker's news page."
        icon={<Newspaper size={26} strokeWidth={1.5} />}
        color="#f074d8"
        out
      />
    ),
    ...MINISTRIES.filter((m) => hit(m.name, m.summary, m.motto, m.address, ...m.sections.map((s) => s.text ?? ""))).map((m) => (
      <MinistryTile key={m.id} m={m} />
    )),
  ].filter(Boolean);
  const manuals = MANUALS.filter((m) =>
    hit(
      m.title,
      m.summary,
      m.address,
      ...m.sections.flatMap((s) => [s.heading, s.text ?? "", ...(s.points ?? []), ...(s.faq ?? []).map((f) => `${f.q} ${f.a}`)]),
    ),
  );
  const companies = COMPANIES.filter((c) =>
    hit(c.name, c.trade, c.summary, c.slogan ?? "", c.address, ...c.sections.map((s) => `${s.text ?? ""} ${(s.products ?? []).join(" ")}`)),
  );
  const brands = BRANDS.filter((b) => hit(b.name, b.trade, b.text, b.slogan ?? ""));
  const none = official.length + manuals.length + companies.length + brands.length === 0;
  return (
    <div className="gww-home">
      <header className="gww-mast">
        <img className="gww-mast-mark" src={iconUrl("super-earth")} alt="" />
        <div className="gww-mast-text">
          <h2 className="gww-mast-title" tabIndex={-1}>
            Galactic Wide Web
          </h2>
          <p className="caps">The Federation's net · verified by the Ministry of Truth</p>
          <HolidayChip />
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
      {!q && <CourseProgress compact />}
      <div className="gww-home-grid">
        <div className="gww-dir">
          {none && <Empty>No site on the G.W.W. matches “{query}”.</Empty>}
          {official.length > 0 && (
            <Group title="Super Earth" count={official.length} all="dir:gov">
              {official}
            </Group>
          )}
          {manuals.length > 0 && (
            <Group title="Training manuals" count={manuals.length} all="dir:manuals">
              {q
                ? manuals.map((m) => <ManualTile key={m.id} m={m} />)
                : CHAPTERS.map((ch, i) => [
                    <p key={ch.id} className="gww-chapter-label caps">
                      <span className="num">Chapter {i + 1}</span> {ch.title}
                    </p>,
                    ...MANUALS.filter((m) => m.chapter === ch.id).map((m) => <ManualTile key={m.id} m={m} />),
                  ])}
            </Group>
          )}
          {!q && ads.banner && <AdBox ad={ads.banner} />}
          {companies.length > 0 && (
            <Group title="Corporations" count={companies.length} all="dir:corps">
              {companies.map((c) => (
                <CompanyTile key={c.id} c={c} />
              ))}
            </Group>
          )}
          {brands.length > 0 && (
            <section className="gww-group">
              <h3 className="gww-group-title caps">
                Marketplace <span className="num">{brands.length}</span>
              </h3>
              <div className="gww-brands">
                {brands.map((b) => (
                  <BrandTile key={b.id} b={b} />
                ))}
              </div>
            </section>
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

function View({ page, logText, headingRef }: { page: GwwPage; logText: string; headingRef: (el: HTMLElement | null) => void }) {
  if (page.kind === "manual") {
    const m = MANUALS.find((x) => x.id === page.id);
    if (m) return <ManualPage m={m} headingRef={headingRef} />;
  }
  if (page.kind === "ministry") {
    const m = MINISTRIES.find((x) => x.id === page.id);
    if (m) return <MinistryPage m={m} headingRef={headingRef} />;
  }
  if (page.kind === "company") {
    const c = COMPANIES.find((x) => x.id === page.id);
    if (c) return <CompanyPage c={c} headingRef={headingRef} />;
  }
  if (page.kind === "brand") {
    const b = BRANDS.find((x) => x.id === page.id);
    if (b) return <BrandPage b={b} headingRef={headingRef} />;
  }
  if (page.kind === "site") return page.id === "gov" ? <GovPage headingRef={headingRef} /> : <WarLogPage headingRef={headingRef} />;
  if (page.kind === "dir") return <DirPage id={page.id} headingRef={headingRef} logText={logText} />;
  return <Home logText={logText} />;
}

export function GwwModule() {
  const stack = useUi((s) => s.gww);
  const setStack = useUi((s) => s.setGww);
  const { changes } = useHistory();
  const page = stack[stack.length - 1] ?? { kind: "home" };
  const root = useRef<HTMLDivElement>(null);
  const heading = useRef<HTMLElement | null>(null);
  const shown = useRef(stack);
  // Scroll position of each history entry, restored on going back.
  const scrolls = useRef<number[]>([]);
  const scroller = () => root.current?.closest<HTMLElement>(".scroll") ?? null;
  const setHeading = useCallback((el: HTMLElement | null) => {
    heading.current = el;
  }, []);
  const go = useCallback(
    (p: GwwPage) => {
      const now = useUi.getState().gww;
      scrolls.current[now.length - 1] = scroller()?.scrollTop ?? 0;
      setStack([...now, p].slice(-50));
    },
    [setStack],
  );
  const back = () => stack.length > 1 && setStack(stack.slice(0, -1));
  const home = () => page.kind !== "home" && go({ kind: "home" });
  // Not on opening: only navigation scrolls and moves focus.
  useEffect(() => {
    const before = shown.current;
    if (before === stack) return;
    shown.current = stack;
    const box = scroller();
    if (stack.length < before.length && box) box.scrollTop = scrolls.current[stack.length - 1] ?? 0;
    else root.current?.scrollIntoView({ block: "start" });
    (heading.current ?? root.current?.querySelector<HTMLElement>(".gww-mast-title"))?.focus({ preventScroll: true });
  }, [stack]);
  const logText = logLine(changes.length);
  return (
    <GwwNavProvider go={go}>
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
        <div className="gww-view" key={`${stack.length}-${keyOf(page)}`}>
          <View page={page} logText={logText} headingRef={setHeading} />
        </div>
      </div>
    </GwwNavProvider>
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
          A fan project, not affiliated with Arrowhead Game Studios or Sony Interactive Entertainment. Map artwork is
          drawn for the app. The faction and sub-faction emblems (as traced by Helldivers Wiki editors), the biome
          landscapes and the black hole and fractured planet pictures are the game's, taken from the Helldivers Wiki
          (helldivers.wiki.gg), and belong to their owners. Effect names come from the Helldivers 2 community's
          data set (github.com/helldivers-2/json, MIT licence).
        </p>
        <h3>Credits</h3>
        <p>
          The G.W.W.'s training manuals are adapted from the <OutLink href={WIKI.home.url}>{WIKI.home.title}</OutLink>,
          chiefly <OutLink href={WIKI.mechanics.url}>{WIKI.mechanics.title}</OutLink>. Its government, ministry,
          corporation and marketplace sites draw on the wiki's{" "}
          <OutLink href={WIKI.government.url}>{WIKI.government.title}</OutLink>, ministry,{" "}
          <OutLink href={WIKI.corporations.url}>{WIKI.corporations.title}</OutLink> and{" "}
          <OutLink href={WIKI.minorCorporations.url}>{WIKI.minorCorporations.title}</OutLink> pages; each page names its
          own sources at its foot. The wiki's text is shared under{" "}
          <OutLink href={WIKI_LICENCE.url}>{WIKI_LICENCE.title}</OutLink>, and so are the pages adapted from it. Training
          manual tips, slogans, poster and broadcast lines are quoted from the game, as collected on the wiki, its{" "}
          <OutLink href={WIKI.tips.url}>{WIKI.tips.title}</OutLink> page among them.
        </p>
        <p>
          The ministry, campaign and difficulty icons (vector tracings by the wiki's contributors), corporation logos,
          product pictures and city adverts are the game's, taken from the Helldivers Wiki, and belong to Arrowhead Game
          Studios and Sony Interactive Entertainment.
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
