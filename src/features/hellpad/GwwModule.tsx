import {
  ArrowLeft,
  ArrowRight,
  BookOpen,
  CalendarDays,
  CloudFog,
  Database,
  GraduationCap,
  History,
  House,
  Info,
  Landmark,
  Lightbulb,
  Lock,
  Newspaper,
  Radio,
  RefreshCw,
  Search,
  Shield,
  Target,
} from "lucide-react";
import { useCallback, useEffect, useId, useRef, useState, type FormEvent, type ReactNode } from "react";
import { APP_NAME, APP_VERSION } from "../../config";
import { CHAPTERS, MANUALS, TIPS, WIKI, WIKI_LICENCE } from "../../data/gww";
import { ADS, COMPANIES, GOV, GWW_SOURCES, HOLIDAYS, MINISTRIES } from "../../data/gwwSites";
import { FACTIONS, factionColor } from "../../domain/factions";
import { nextYearly, plainText } from "../../domain/gwwText";
import { useHistory } from "../../history/useHistory";
import { useSettings, useUi, type GwwPage } from "../../state";
import { DssIcon, FactionEmblem } from "../../ui/emblems";
import { Empty, OutLink, Panel, useMedia, useNow } from "../../ui/primitives";
import type { Faction } from "../../api/types";
import { iconUrl } from "./gwwArt";
import { addressOf, GwwNavProvider, LOG_ADDRESS, LOG_TITLE, normalize, SUGGESTED_ADDRESSES, targetAt, useGwwNav } from "./gwwNav";
import {
  AdBox,
  CompanyPage,
  CompanyTile,
  DirPage,
  GovPage,
  HolidayPage,
  HolidaysPage,
  LogsPage,
  ManualPage,
  ManualTile,
  MinistryPage,
  MinistryTile,
  MissingPage,
  pickOne,
  RAIL_QUERY,
  SiteTile,
} from "./GwwPages";
import "./gww.css";

const keyOf = (p: GwwPage) => ("id" in p ? `${p.kind}:${p.id}` : p.kind === "missing" ? `missing:${p.address}` : p.kind);

/** The address bar: shows where the reader is, and takes any address typed into it. */
function AddressBar({ page }: { page: GwwPage }) {
  const { follow, go } = useGwwNav();
  const [draft, setDraft] = useState<string | null>(null);
  const input = useRef<HTMLInputElement>(null);
  const listId = useId();
  const open = (e: FormEvent) => {
    e.preventDefault();
    const typed = draft ?? "";
    const target = targetAt(typed);
    if (target) follow(target);
    else go({ kind: "missing", address: normalize(typed) });
    input.current?.blur();
  };
  return (
    <form className="gww-url num" onSubmit={open}>
      <Lock size={12} aria-hidden />
      <input
        ref={input}
        aria-label="Address"
        value={draft ?? addressOf(page)}
        list={listId}
        spellCheck={false}
        autoComplete="off"
        autoCapitalize="off"
        enterKeyHint="go"
        onFocus={(e) => {
          setDraft(addressOf(page));
          e.currentTarget.select();
        }}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={() => setDraft(null)}
        onKeyDown={(e) => {
          if (e.key !== "Escape") return;
          e.stopPropagation();
          e.currentTarget.blur();
        }}
      />
      <datalist id={listId}>
        {SUGGESTED_ADDRESSES.map((a) => (
          <option key={a} value={a} />
        ))}
      </datalist>
    </form>
  );
}

function logLine(count: number): string {
  const won = count === 1 ? "1 planet has" : `${count} planets have`;
  return count ? `The war log and the FTL jumps. ${won} changed hands since records began.` : "The war log and the FTL jumps.";
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
    <button className={`gww-holiday caps ${next.in <= 1 ? "soon" : ""}`} onClick={(e) => follow("site:holidays", e)}>
      <CalendarDays size={13} aria-hidden /> {next.name} {when}
    </button>
  );
}

function Home({ logText }: { logText: string }) {
  const [query, setQuery] = useState("");
  const [ads] = useState(() => {
    const landscape = ADS.filter((a) => a.shape === "wide");
    const banner = pickOne(landscape);
    return {
      tall: pickOne(ADS.filter((a) => a.shape === "tall")),
      banner,
      second: pickOne(landscape.filter((a) => a !== banner)),
    };
  });
  const wide = useMedia(RAIL_QUERY);
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
    hit(LOG_TITLE, logText, LOG_ADDRESS, "war log", "FTL jumps") && (
      <SiteTile
        key="log"
        target="site:log"
        address={LOG_ADDRESS}
        title={LOG_TITLE}
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
        text="Dispatches and Galactic Campaign news, as they come in."
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
      ...m.sections.flatMap((s) => [
        s.heading,
        s.text ?? "",
        ...(s.points ?? []),
        ...(s.faq ?? []).map((f) => `${f.q} ${f.a}`),
        ...(s.folds ?? []).flatMap((f) => [f.title, ...f.points]),
      ]),
    ),
  );
  const companies = COMPANIES.filter((c) =>
    hit(c.name, c.trade, c.summary, c.slogan ?? "", c.address, ...c.sections.map((s) => `${s.text ?? ""} ${(s.products ?? []).join(" ")}`)),
  );
  const read = useSettings((s) => s.gwwRead);
  const readCount = MANUALS.filter((m) => read.includes(m.id)).length;
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
      <div className="gww-home-grid">
        <div className="gww-dir">
          {none && <Empty>No site on the G.W.W. matches “{query}”.</Empty>}
          {official.length > 0 && (
            <Group title="Super Earth" count={official.length} all="dir:gov">
              {official}
            </Group>
          )}
          {q && manuals.length > 0 && (
            <Group title="Training manuals" count={manuals.length} all="dir:manuals">
              {manuals.map((m) => (
                <ManualTile key={m.id} m={m} />
              ))}
            </Group>
          )}
          {!q && (
            <Group title="Training manuals" count={MANUALS.length}>
              <SiteTile
                target="dir:manuals"
                address="se://hc/manuals"
                title="Training manuals"
                text={`How the Second Galactic War works, in ${CHAPTERS.length} chapters and ${MANUALS.length} manuals. ${readCount} of ${MANUALS.length} read.`}
                icon={<GraduationCap size={28} strokeWidth={1.5} />}
                color="var(--cta)"
                done={readCount === MANUALS.length}
              />
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
          {!q && !wide && ads.second && <AdBox ad={ads.second} />}
        </div>
        {wide && ads.tall && (
          <aside className="gww-rail" aria-label="Sponsored">
            <AdBox ad={ads.tall} slot="rail" />
          </aside>
        )}
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
  if (page.kind === "holiday") {
    const h = HOLIDAYS.find((x) => x.id === page.id);
    if (h) return <HolidayPage h={h} headingRef={headingRef} />;
  }
  if (page.kind === "site") {
    if (page.id === "log") return <LogsPage headingRef={headingRef} />;
    if (page.id === "holidays") return <HolidaysPage headingRef={headingRef} />;
    return <GovPage site={page.id} headingRef={headingRef} />;
  }
  if (page.kind === "dir") return <DirPage id={page.id} headingRef={headingRef} logText={logText} />;
  if (page.kind === "missing") return <MissingPage address={page.address} headingRef={headingRef} />;
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
  // Where each history entry was scrolled to when left, restored on coming back to the same page.
  const scrolls = useRef<{ key: string; top: number }[]>([]);
  const scroller = () => root.current?.closest<HTMLElement>(".scroll") ?? null;
  const setHeading = useCallback((el: HTMLElement | null) => {
    heading.current = el;
  }, []);
  const go = useCallback(
    (p: GwwPage) => {
      const now = useUi.getState().gww;
      scrolls.current.length = now.length;
      setStack([...now, p].slice(-50));
    },
    [setStack],
  );
  // Read before the page changes, whether by a link here or the browser's Back and Forward.
  useEffect(
    () =>
      useUi.subscribe((s, was) => {
        const at = was.gww.length - 1;
        if (s.gww === was.gww || at < 0) return;
        scrolls.current[at] = { key: keyOf(was.gww[at]), top: scroller()?.scrollTop ?? 0 };
      }),
    [],
  );
  const back = () => stack.length > 1 && setStack(stack.slice(0, -1));
  const home = () => page.kind !== "home" && go({ kind: "home" });
  // Not on opening: only navigation scrolls and moves focus.
  useEffect(() => {
    const before = shown.current;
    if (before === stack) return;
    shown.current = stack;
    const box = scroller();
    const saved = scrolls.current[stack.length - 1];
    if (box && saved?.key === keyOf(page)) box.scrollTop = saved.top;
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
          <AddressBar page={page} />
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
        <h3>Privacy</h3>
        <p>
          The app collects no personal data: no account, analytics, ads or tracking. Settings and pins stay on this
          device, alerts are worked out here, and the only thing downloaded is war data from the project's read-only
          database. The full policy is in PRIVACY.md at github.com/zearkreast/WarRoom.
        </p>
        <h3>Not affiliated</h3>
        <p>
          A fan project, not affiliated with Arrowhead Game Studios or Sony Interactive Entertainment. Map artwork is
          drawn for the app. The faction and sub-faction emblems and the DSS glyph (as traced by Helldivers Wiki
          editors), the DSS's summary hologram, the biome landscapes and the black hole and fractured planet pictures
          are the game's, taken from the Helldivers Wiki (helldivers.wiki.gg), and belong to their owners. Effect names
          come from the Helldivers 2 community's data set (github.com/helldivers-2/json, MIT licence).
        </p>
        <h3>Credits</h3>
        <p>
          The G.W.W.'s training manuals, sites and holiday pages are adapted from the{" "}
          <OutLink href={WIKI.home.url}>{WIKI.home.title}</OutLink> and written to read as the Federation's own net, so
          they carry no source lines; the wiki pages they draw on are listed below. The wiki's text is shared under{" "}
          <OutLink href={WIKI_LICENCE.url}>{WIKI_LICENCE.title}</OutLink>, and so are the pages adapted from it. Training
          manual tips, slogans, decrees, dispatches, carols, poster and broadcast lines are quoted from the game, as
          collected on the wiki.
        </p>
        <ul className="about-sources">
          {GWW_SOURCES.map((s) => (
            <li key={s.url}>
              <OutLink href={s.url}>{s.title}</OutLink>
            </li>
          ))}
        </ul>
        <p>
          The ministry, campaign, difficulty and stratagem icons and the SEAF emblem (vector tracings by the wiki's
          contributors), corporation logos, product pictures and city adverts are the game's, taken from the Helldivers
          Wiki, and belong to Arrowhead Game Studios and Sony Interactive Entertainment. Martian Armory's logo is drawn
          for the app.
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
    icon: <DssIcon size={20} color="var(--f-humans)" />,
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
