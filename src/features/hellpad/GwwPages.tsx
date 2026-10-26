import {
  Activity,
  Archive,
  ArrowLeft,
  ArrowRight,
  ArrowUpRight,
  Building2,
  CalendarDays,
  Check,
  CircleHelp,
  CloudFog,
  Earth,
  Flag,
  Globe2,
  GraduationCap,
  History,
  Landmark,
  LayoutGrid,
  Lightbulb,
  Network,
  Newspaper,
  Orbit,
  Quote,
  Radio,
  Rocket,
  Shield,
  TriangleAlert,
  Users,
  type LucideIcon,
} from "lucide-react";
import { useEffect, useRef, useState, type CSSProperties, type ReactNode, type RefCallback } from "react";
import { CHAPTERS, MANUALS, WIKI, WIKI_LICENCE, type Manual, type Section, type Source } from "../../data/gww";
import {
  ADS,
  CITIZENS,
  COMPANIES,
  GOV,
  HOLIDAYS,
  HOLIDAYS_SITE,
  MINISTRIES,
  type Ad,
  type Company,
  type Holiday,
  type Ministry,
  type SiteSection,
  type Tie,
} from "../../data/gwwSites";
import { factionColor, factionLabel } from "../../domain/factions";
import { fmtDate, fmtDateTime, fmtInt, sestClock, sestDate } from "../../domain/format";
import { nextYearly, plainText } from "../../domain/gwwText";
import { useSpaceStations } from "../../api/queries";
import { useStationJumps } from "../../api/stations";
import { useWarDays } from "../../api/warSamples";
import { useHistory } from "../../history/useHistory";
import { useSettings, useUi } from "../../state";
import { DssIcon, FactionEmblem, type IconLike } from "../../ui/emblems";
import { Empty, Loading, OutLink, useMedia, useNow } from "../../ui/primitives";
import { Figure } from "./GwwFigures";
import { LivePanel } from "./GwwLive";
import { adUrl, iconUrl, logoUrl } from "./gwwArt";
import { DIRS, LOG_ADDRESS, LOG_TITLE, Rich, titleOf, useGwwNav } from "./gwwNav";
import { JumpRoute } from "./StationJumps";

type HeadingRef = RefCallback<HTMLElement>;

/* ---------- Pictures ---------- */

/** Drawn icons for manuals the game has no picture for. */
const MANUAL_ICONS: Record<string, IconLike> = {
  basics: Earth,
  resistance: Activity,
  regions: Building2,
  "supply-lines": Network,
  dss: DssIcon,
  "super-destroyers": Rocket,
  campaigns: Flag,
  gloom: CloudFog,
  misconceptions: CircleHelp,
};

const APP_ICONS: Record<string, IconLike> = {
  news: Newspaper,
  overview: Shield,
  map: Orbit,
  campaigns: LayoutGrid,
  stations: DssIcon,
  past: Archive,
  planets: Globe2,
};

const DIR_ICONS: Record<string, LucideIcon> = {
  manuals: GraduationCap,
  gov: Landmark,
  corps: Building2,
};

/** Every manual's icon as a ring around a glyph, as the game draws its campaign icons (which bring their own ring). */
export function ManualIcon({ m, size }: { m: Manual; size: number }) {
  const url = m.icon && iconUrl(m.icon);
  const Icon = MANUAL_ICONS[m.id] ?? Lightbulb;
  const ringed = url && m.icon!.startsWith("campaign-");
  return (
    <span className={`gww-glyph ${ringed ? "" : "ring"}`} style={{ fontSize: size }} aria-hidden>
      {url ? <i style={{ "--glyph": `url("${url}")` } as CSSProperties} /> : <Icon strokeWidth={2.25} />}
    </span>
  );
}

export function CompanyMark({ c }: { c: Company }) {
  const url = c.logo && logoUrl(c.logo);
  return url ? (
    <img className="gww-logo" src={url} alt="" />
  ) : (
    <span className="gww-wordmark caps" aria-hidden>
      {c.name}
    </span>
  );
}

/** A small picture for any link target. */
export function TargetIcon({ target, size = 20 }: { target: string; size?: number }) {
  const [kind, id] = [target.slice(0, target.indexOf(":")), target.slice(target.indexOf(":") + 1)];
  const img = (src: string | undefined) => (src ? <img src={src} alt="" width={size} height={size} /> : null);
  if (kind === "manual") {
    const m = MANUALS.find((x) => x.id === id);
    return m ? <ManualIcon m={m} size={size} /> : null;
  }
  if (kind === "ministry") return img(iconUrl(MINISTRIES.find((x) => x.id === id)?.icon ?? ""));
  if (kind === "company") {
    const c = COMPANIES.find((x) => x.id === id);
    const src = c?.logo && logoUrl(c.logo);
    return src ? img(src) : <Building2 size={size} strokeWidth={1.5} aria-hidden />;
  }
  if (kind === "holiday" || (kind === "site" && id === "holidays")) return <CalendarDays size={size} strokeWidth={1.5} aria-hidden />;
  if (kind === "site") {
    if (id === "log") return <History size={size} strokeWidth={1.5} aria-hidden />;
    return img(iconUrl(id === "citizens" ? CITIZENS.icon : GOV.icon));
  }
  const Icon = kind === "dir" ? DIR_ICONS[id] : kind === "app" ? APP_ICONS[id] : undefined;
  return Icon ? <Icon size={size} strokeWidth={1.5} aria-hidden /> : null;
}

const KIND_LABEL: Record<string, string> = {
  manual: "Training manual",
  ministry: "Ministry",
  company: "Corporation",
  holiday: "Holiday",
  site: "Super Earth",
  dir: "Directory",
  app: "In the tracker",
};

/** Wide enough for a rail of adverts beside the page; keep in sync with the 1100px breakpoint in gww.css. */
export const RAIL_QUERY = "(min-width: 1101px)";

export const pickOne = <T,>(list: T[]): T | undefined => list[Math.floor(Math.random() * list.length)];

const linkOf = (ad: Ad): string | undefined => ("by" in ad ? ad.by : undefined);

/**
 * A rail advert keeps its shape; a billboard fills the width at a fixed height. `here` is the link target of the
 * page it's shown on; an advert for that page doesn't link to it.
 */
export function AdBox({ ad, here, slot = "billboard" }: { ad: Ad; here?: string; slot?: "rail" | "billboard" }) {
  const { follow } = useGwwNav();
  const url = adUrl(ad.file);
  if (!url) return null;
  const art = { "--ad": `url("${url}")` } as CSSProperties;
  const to = linkOf(ad);
  return (
    <figure className={`gww-ad ${ad.shape} ${slot}`}>
      {!to || to === here ? (
        <div className="gww-ad-open" style={art}>
          <img src={url} alt={ad.alt} loading="lazy" />
        </div>
      ) : (
        <button
          className="gww-ad-open"
          style={art}
          onClick={(e) => follow(to, e)}
          aria-label={`${ad.alt} Open ${titleOf(to)}.`}
        >
          <img src={url} alt="" loading="lazy" />
        </button>
      )}
      <figcaption className="caps">
        <span>Sponsored</span>
        <span className="gww-ad-by">{"brand" in ad ? ad.brand : titleOf(ad.by)}</span>
      </figcaption>
    </figure>
  );
}

/** On wide screens a tall advert takes the rail, another advertiser's when the site has none of its own. */
function useSiteAds(files: string[] | undefined, here: string): { rail?: Ad; inline: Ad[] } {
  const wide = useMedia(RAIL_QUERY);
  const own = ADS.filter((a) => files?.includes(a.file));
  const [filler] = useState(() => pickOne(ADS.filter((a) => a.shape === "tall" && linkOf(a) !== here)));
  if (!wide) return { inline: own };
  const rail = own.find((a) => a.shape === "tall") ?? filler;
  return { rail, inline: own.filter((a) => a !== rail) };
}

function WithRail({ rail, here, children }: { rail?: Ad; here: string; children: ReactNode }) {
  if (!rail) return <>{children}</>;
  return (
    <div className="gww-railed">
      {children}
      <aside className="gww-rail" aria-label="Sponsored">
        <AdBox ad={rail} here={here} slot="rail" />
      </aside>
    </div>
  );
}

/* ---------- Text ---------- */

/** The in-game Training Manual Tip box, for the game's own words. */
export function TrainingTip({ text }: { text: string }) {
  return (
    <aside className="gww-quote">
      <Quote size={16} aria-hidden />
      <p>
        <b className="caps">Training manual tip</b>
        <span>{text}</span>
      </p>
    </aside>
  );
}

function Broadcast({ text }: { text: string }) {
  return (
    <aside className="gww-quote broadcast">
      <Radio size={16} aria-hidden />
      <p>
        <b className="caps">Ship broadcast</b>
        <span>“{text}”</span>
      </p>
    </aside>
  );
}

function SourcedQuote({ from, text }: { from: string; text: string }) {
  return (
    <aside className="gww-quote">
      <Quote size={16} aria-hidden />
      <p>
        <b className="caps">{from}</b>
        <span>“{text}”</span>
      </p>
    </aside>
  );
}

function FieldTip({ text }: { text: string }) {
  return (
    <aside className="gww-tip">
      <Lightbulb size={16} aria-hidden />
      <p>
        <b className="caps">Field tip</b> <Rich text={text} />
      </p>
    </aside>
  );
}

/** The term ends at the first ": ", so a link label before it mustn't contain one. */
function Point({ text }: { text: string }) {
  const at = text.indexOf(": ");
  if (at < 0 || at > 80) return <Rich text={text} />;
  return (
    <>
      <b>
        <Rich text={text.slice(0, at)} />
      </b>
      : <Rich text={text.slice(at + 2)} />
    </>
  );
}

function SectionBlock({ sec, id }: { sec: SiteSection; id: string }) {
  return (
    <section id={id} className="gww-section">
      <h3>{sec.heading}</h3>
      {sec.text && (
        <p>
          <Rich text={sec.text} />
        </p>
      )}
      {sec.points && (
        <ul className="gww-points">
          {sec.points.map((p) => (
            <li key={p}>
              <Point text={p} />
            </li>
          ))}
        </ul>
      )}
      {sec.products && (
        <ul className="gww-products" aria-label="Products">
          {sec.products.map((p) => (
            <li key={p} className="caps">
              {p}
            </li>
          ))}
        </ul>
      )}
      {sec.timeline && (
        <ol className="gww-timeline">
          {sec.timeline.map((d) => (
            <li key={d.when + d.text}>
              <span className="gww-timeline-when caps num">{d.when}</span>
              <p>
                <Rich text={d.text} />
              </p>
            </li>
          ))}
        </ol>
      )}
      {sec.table && (
        <div className="gww-table-wrap">
          <table className="gww-table num">
            <thead>
              <tr>
                {sec.table.head.map((h) => (
                  <th key={h} scope="col">
                    {h}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {sec.table.rows.map((r) => (
                <tr key={r.cells[0]}>
                  {r.cells.map((c, i) =>
                    i === 0 ? (
                      <th key={i} scope="row">
                        {r.icon && iconUrl(r.icon) && <img src={iconUrl(r.icon)} alt="" />}
                        <Rich text={c} />
                      </th>
                    ) : (
                      <td key={i}>
                        <Rich text={c} />
                      </td>
                    ),
                  )}
                </tr>
              ))}
            </tbody>
          </table>
          {sec.table.note && <p className="gww-note">{sec.table.note}</p>}
        </div>
      )}
      {sec.kinds && (
        <ul className="gww-kinds">
          {sec.kinds.map((k) => (
            <li key={k.name}>
              <span className="gww-kind-icon">{iconUrl(k.icon) && <img src={iconUrl(k.icon)} alt="" />}</span>
              <div>
                <b className="caps">{k.name}</b>
                <p>
                  <Rich text={k.text} />
                </p>
              </div>
            </li>
          ))}
        </ul>
      )}
      {sec.faq && (
        <div className="gww-faq">
          {sec.faq.map((f) => (
            <details key={f.q}>
              <summary>{f.q}</summary>
              <p>
                <Rich text={f.a} />
              </p>
            </details>
          ))}
        </div>
      )}
      {sec.folds && (
        <div className="gww-folds">
          {sec.folds.map((f) => (
            <details key={f.title} className="gww-fold">
              <summary>
                <b>{f.title}</b>
                <span className="caps">{f.note}</span>
              </summary>
              <ol className="gww-fold-list">
                {f.points.map((p) => (
                  <li key={p}>
                    <Point text={p} />
                  </li>
                ))}
              </ol>
            </details>
          ))}
        </div>
      )}
      {sec.figure && <Figure kind={sec.figure} />}
    </section>
  );
}

/* ---------- Page frame ---------- */

export interface Crumb {
  label: string;
  target?: string;
}

function Banner(props: {
  icon?: ReactNode;
  crumbs: Crumb[];
  title: string;
  lead?: ReactNode;
  headingRef: HeadingRef;
  corp?: boolean;
}) {
  const { follow } = useGwwNav();
  return (
    <header className={`gww-banner ${props.corp ? "corp" : ""}`}>
      {props.icon && <span className="gww-banner-icon">{props.icon}</span>}
      <div className="gww-banner-text">
        <nav className="gww-crumbs caps" aria-label="Where this page sits">
          {props.crumbs.map((c, i) => (
            <span key={c.label}>
              {i > 0 && <span aria-hidden> › </span>}
              {c.target ? (
                <button className="gww-crumb" onClick={(e) => follow(c.target!, e)}>
                  {c.label}
                </button>
              ) : (
                c.label
              )}
            </span>
          ))}
        </nav>
        <h2 className="gww-title" ref={props.headingRef} tabIndex={-1}>
          {props.title}
        </h2>
        {props.lead && <p className="gww-lead">{props.lead}</p>}
      </div>
    </header>
  );
}

const reducedMotion = () => typeof window !== "undefined" && !!window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;

function Toc({ sections, prefix }: { sections: Section[]; prefix: string }) {
  if (sections.length < 3) return null;
  return (
    <nav className="gww-toc" aria-label="On this page">
      <span className="caps">On this page</span>
      {sections.map((s, i) => (
        <button
          key={s.heading}
          className="gww-toc-link"
          onClick={() =>
            document.getElementById(`${prefix}-${i}`)?.scrollIntoView({ behavior: reducedMotion() ? "auto" : "smooth", block: "start" })
          }
        >
          {s.heading}
        </button>
      ))}
    </nav>
  );
}

export function Related({ targets, title = "Read next" }: { targets: string[]; title?: string }) {
  const { follow } = useGwwNav();
  if (targets.length === 0) return null;
  return (
    <nav className="gww-related" aria-label={title}>
      <h3 className="caps">{title}</h3>
      <div className="gww-related-list">
        {targets.map((t) => (
          <button key={t} className="gww-related-link" onClick={(e) => follow(t, e)}>
            <span className="gww-related-icon">
              <TargetIcon target={t} />
            </span>
            <span>
              <span className="caps">{KIND_LABEL[t.slice(0, t.indexOf(":"))]}</span>
              <b>{titleOf(t)}</b>
            </span>
            {t.startsWith("app:") ? <ArrowUpRight size={14} aria-hidden /> : <ArrowRight size={14} aria-hidden />}
          </button>
        ))}
      </div>
    </nav>
  );
}

function Ties({ ties, title }: { ties: Tie[]; title: string }) {
  const { follow } = useGwwNav();
  return (
    <section className="gww-section">
      <h3>{title}</h3>
      <div className="gww-ties">
        {ties.map((t) => (
          <button key={t.to} className="gww-tie" onClick={(e) => follow(t.to, e)}>
            <span className="gww-related-icon">
              <TargetIcon target={t.to} size={28} />
            </span>
            <span>
              <b>{titleOf(t.to)}</b>
              <span>{t.how}</span>
            </span>
          </button>
        ))}
      </div>
    </section>
  );
}

function SourceLine({ sources, quoted }: { sources?: Source[]; quoted?: string }) {
  return (
    <footer className="gww-source">
      {sources?.length ? (
        <>
          Adapted from the {WIKI.home.title}:{" "}
          {sources.map((s, i) => (
            <span key={s.url}>
              {i > 0 && ", "}
              <OutLink href={s.url}>{s.title}</OutLink>
            </span>
          ))}
          , under <OutLink href={WIKI_LICENCE.url}>{WIKI_LICENCE.title}</OutLink>, which this page shares.
        </>
      ) : (
        <>Written for this tracker.</>
      )}
      {quoted && <> {quoted}</>}
    </footer>
  );
}

function Pager({ list, at, noun }: { list: string[]; at: number; noun: string }) {
  const { follow } = useGwwNav();
  const prev = list[at - 1];
  const next = list[at + 1];
  return (
    <nav className="gww-pager" aria-label={`Other ${noun}s`}>
      {prev ? (
        <button className="gww-pager-link" onClick={(e) => follow(prev, e)}>
          <ArrowLeft size={14} aria-hidden />
          <span>
            <span className="caps">Previous {noun}</span> {titleOf(prev)}
          </span>
        </button>
      ) : (
        <span />
      )}
      {next && (
        <button className="gww-pager-link next" onClick={(e) => follow(next, e)}>
          <span>
            <span className="caps">Next {noun}</span> {titleOf(next)}
          </span>
          <ArrowRight size={14} aria-hidden />
        </button>
      )}
    </nav>
  );
}

function Ads({ ads, here }: { ads: Ad[]; here: string }) {
  if (ads.length === 0) return null;
  return (
    <div className="gww-ads-inline">
      {ads.map((a) => (
        <AdBox key={a.file} ad={a} here={here} />
      ))}
    </div>
  );
}

/* ---------- Tiles ---------- */

export function SiteTile(props: {
  target: string;
  address: string;
  title: string;
  text: string;
  icon: ReactNode;
  color?: string;
  done?: boolean;
  /** Marks a tile that leaves the G.W.W. for another page of the tracker. */
  out?: boolean;
}) {
  const { follow } = useGwwNav();
  return (
    <button
      className={`gww-site ${props.done ? "done" : ""}`}
      style={props.color ? ({ "--site": props.color } as CSSProperties) : undefined}
      onClick={(e) => follow(props.target, e)}
    >
      <span className="gww-site-icon">{props.icon}</span>
      <span className="gww-site-body">
        <span className="gww-address num">
          {props.address}
          {props.out && (
            <>
              <ArrowUpRight size={11} aria-hidden />
              <span className="sr-only"> (leaves the G.W.W.)</span>
            </>
          )}
        </span>
        <b className="gww-site-title">{props.title}</b>
        <span className="gww-site-text">{props.text}</span>
      </span>
      {props.done && (
        <span className="gww-site-done" title="Read">
          <Check size={12} aria-label="Read" />
        </span>
      )}
    </button>
  );
}

export function CompanyTile({ c }: { c: Company }) {
  const { follow } = useGwwNav();
  return (
    <button className="gww-site corp" style={{ "--site": c.color } as CSSProperties} onClick={(e) => follow(`company:${c.id}`, e)}>
      <span className="gww-corp-logo">
        <CompanyMark c={c} />
      </span>
      <span className="gww-site-body">
        <span className="gww-address num">{c.address}</span>
        <b className="gww-site-title">{c.name}</b>
        <span className="gww-site-text">{c.slogan ?? c.summary}</span>
      </span>
    </button>
  );
}

export const manualTarget = (m: Manual) => `manual:${m.id}`;

export function ManualTile({ m }: { m: Manual }) {
  const read = useSettings((s) => s.gwwRead.includes(m.id));
  return (
    <SiteTile
      target={manualTarget(m)}
      address={m.address}
      title={m.title}
      text={m.summary}
      icon={<ManualIcon m={m} size={30} />}
      color="var(--cta)"
      done={read}
    />
  );
}

export function MinistryTile({ m }: { m: Ministry }) {
  return (
    <SiteTile
      target={`ministry:${m.id}`}
      address={m.address}
      title={m.name}
      text={m.summary}
      icon={<img src={iconUrl(m.icon)} alt="" />}
      color={m.color}
    />
  );
}

const HUMANITY = MINISTRIES.find((m) => m.id === "humanity")!.color;

export function OfficialTiles({ logText }: { logText: string }) {
  return (
    <>
      <SiteTile
        target="site:gov"
        address={GOV.address}
        title={GOV.title}
        text={GOV.summary}
        icon={<img src={iconUrl(GOV.icon)} alt="" />}
        color="var(--f-humans)"
      />
      <SiteTile
        target="site:log"
        address={LOG_ADDRESS}
        title={LOG_TITLE}
        text={logText}
        icon={<History size={26} strokeWidth={1.5} />}
        color="var(--f-humans)"
      />
      <SiteTile
        target="app:news"
        address="se://strohmann.news"
        title="Strohmann News"
        text="Dispatches and campaign news, in the tracker's news page."
        icon={<Newspaper size={26} strokeWidth={1.5} />}
        color="#f074d8"
        out
      />
    </>
  );
}

export function CourseProgress({ compact }: { compact?: boolean }) {
  const read = useSettings((s) => s.gwwRead);
  const clearRead = useSettings((s) => s.clearRead);
  const { follow } = useGwwNav();
  const done = MANUALS.filter((m) => read.includes(m.id)).length;
  const next = MANUALS.find((m) => !read.includes(m.id));
  return (
    <section className={`gww-course ${compact ? "compact" : ""}`} aria-label="Recruit's course">
      <div className="gww-course-head">
        <GraduationCap size={20} aria-hidden />
        <div>
          <b className="caps">Recruit's course</b>
          <span className="num">
            {done} of {MANUALS.length} training manuals read
          </span>
        </div>
        {next ? (
          <button className="gww-course-next" onClick={(e) => follow(manualTarget(next), e)}>
            <span className="caps">{done === 0 ? "Start with" : "Continue with"}</span>
            <b>{next.title}</b>
            <ArrowRight size={14} aria-hidden />
          </button>
        ) : (
          <span className="gww-course-done caps">Course complete. Democracy thanks you.</span>
        )}
      </div>
      <ol className="gww-course-bar" aria-label="Manuals, in reading order">
        {MANUALS.map((m) => (
          <li key={m.id} className={read.includes(m.id) ? "on" : ""}>
            <button
              title={m.title}
              aria-label={`${m.title}${read.includes(m.id) ? ", read" : ""}`}
              onClick={(e) => follow(manualTarget(m), e)}
            />
          </li>
        ))}
      </ol>
      {!compact && done > 0 && (
        <button className="link gww-course-reset caps" onClick={clearRead}>
          Start the course over
        </button>
      )}
    </section>
  );
}

/* ---------- Pages ---------- */

export function ManualPage({ m, headingRef }: { m: Manual; headingRef: HeadingRef }) {
  const markRead = useSettings((s) => s.markRead);
  const end = useRef<HTMLElement>(null);
  const at = MANUALS.indexOf(m);
  const chapter = CHAPTERS.findIndex((c) => c.id === m.chapter);
  const prefix = `gww-${m.id}`;
  // A manual counts as read once its end has been on screen.
  useEffect(() => {
    const el = end.current;
    if (!el || typeof IntersectionObserver === "undefined") return;
    const seen = new IntersectionObserver((entries) => {
      if (entries.some((e) => e.isIntersecting)) markRead(m.id);
    });
    seen.observe(el);
    return () => seen.disconnect();
  }, [m.id, markRead]);
  return (
    <article className="gww-page">
      <Banner
        icon={<ManualIcon m={m} size={42} />}
        crumbs={[
          { label: "Training manuals", target: "dir:manuals" },
          { label: `Chapter ${chapter + 1}: ${CHAPTERS[chapter].title}` },
        ]}
        title={m.title}
        lead={m.summary}
        headingRef={headingRef}
      />
      <Toc sections={m.sections} prefix={prefix} />
      <LivePanel manual={m.id} />
      {m.sections.map((sec, i) => (
        <SectionBlock key={sec.heading} sec={sec} id={`${prefix}-${i}`} />
      ))}
      {m.quotes?.map((q) => <TrainingTip key={q} text={q} />)}
      {m.tips.map((t) => (
        <FieldTip key={t} text={t} />
      ))}
      <Related targets={m.related} />
      <span ref={end} className="gww-end" aria-hidden />
      <SourceLine sources={m.sources} quoted={m.quotes?.length ? "Training manual tips are quoted from the game." : undefined} />
      <Pager list={MANUALS.map(manualTarget)} at={at} noun="manual" />
    </article>
  );
}

export function MinistryPage({ m, headingRef }: { m: Ministry; headingRef: HeadingRef }) {
  const prefix = `gww-${m.id}`;
  const here = `ministry:${m.id}`;
  const ads = useSiteAds(m.ads, here);
  return (
    <WithRail rail={ads.rail} here={here}>
      <article className="gww-page" style={{ "--site": m.color } as CSSProperties}>
        <Banner
          icon={iconUrl(m.icon) && <img src={iconUrl(m.icon)} alt="" width={72} height={72} />}
          crumbs={[{ label: "Super Earth", target: "dir:gov" }, { label: "Ministries" }]}
          title={m.name}
          lead={m.slogan ? <span className="gww-slogan">“{m.slogan}”</span> : m.summary}
          headingRef={headingRef}
        />
        <TrainingTip text={m.motto} />
        <Toc sections={m.sections} prefix={prefix} />
        {m.sections.map((sec, i) => (
          <SectionBlock key={sec.heading} sec={sec} id={`${prefix}-${i}`} />
        ))}
        {m.ties && <Ties ties={m.ties} title="Ties" />}
        <Ads ads={ads.inline} here={here} />
        <Related targets={m.related ?? []} />
        <SourceLine sources={[m.source, WIKI.government]} quoted="Its tip and poster line are quoted from the game." />
        <Pager list={MINISTRIES.map((x) => `ministry:${x.id}`)} at={MINISTRIES.indexOf(m)} noun="ministry" />
      </article>
    </WithRail>
  );
}

export function CompanyPage({ c, headingRef }: { c: Company; headingRef: HeadingRef }) {
  const prefix = `gww-${c.id}`;
  const showcase = c.showcase && logoUrl(c.showcase.image);
  const here = `company:${c.id}`;
  const ads = useSiteAds(c.ads, here);
  return (
    <WithRail rail={ads.rail} here={here}>
      <article className="gww-page" style={{ "--site": c.color } as CSSProperties}>
        <Banner
          // The name is beside the banner's picture, so a company without a logo shows no picture there.
          icon={c.logo && logoUrl(c.logo) ? <CompanyMark c={c} /> : undefined}
          corp
          crumbs={[{ label: "Corporations", target: "dir:corps" }, { label: c.trade }]}
          title={c.name}
          lead={c.slogan ? <span className="gww-slogan">“{c.slogan}”</span> : c.summary}
          headingRef={headingRef}
        />
        {c.broadcast && <Broadcast text={c.broadcast} />}
        <Toc sections={c.sections} prefix={prefix} />
        {c.sections.map((sec, i) => (
          <SectionBlock key={sec.heading} sec={sec} id={`${prefix}-${i}`} />
        ))}
        {showcase && (
          <figure className="gww-showcase">
            <img src={showcase} alt={c.showcase!.alt} loading="lazy" />
          </figure>
        )}
        {c.quote && <TrainingTip text={c.quote} />}
        {c.ties && <Ties ties={c.ties} title="Ties" />}
        <Ads ads={ads.inline} here={here} />
        <Related targets={c.related ?? []} />
        <SourceLine sources={[c.source]} quoted="Slogans and broadcast lines are quoted from the game." />
        <Pager list={COMPANIES.map((x) => `company:${x.id}`)} at={COMPANIES.indexOf(c)} noun="corporation" />
      </article>
    </WithRail>
  );
}

type GovSite = "gov" | "citizens" | "holidays";

const GOV_TABS: [GovSite, string][] = [
  ["gov", "Managed Democracy"],
  ["citizens", "Citizen classes"],
  ["holidays", "Holidays"],
];

/** Managed Democracy's own menu: its sub-sites are reached from here, not from the G.W.W. home. */
function GovNav({ at }: { at: GovSite }) {
  const { follow } = useGwwNav();
  return (
    <nav className="gww-sitenav" aria-label="Managed Democracy">
      <span className="gww-sitenav-brand caps">
        <img src={iconUrl(GOV.icon)} alt="" width={18} height={18} />
        se://gov
      </span>
      {GOV_TABS.map(([id, label]) => (
        <button
          key={id}
          className={`gww-sitenav-link caps ${id === at ? "on" : ""}`}
          aria-current={id === at ? "page" : undefined}
          onClick={(e) => follow(`site:${id}`, e)}
        >
          {label}
        </button>
      ))}
    </nav>
  );
}

const govCrumbs = (site: GovSite): Crumb[] => [
  { label: "Super Earth", target: "dir:gov" },
  ...(site === "gov" ? [] : [{ label: GOV.title, target: "site:gov" }]),
];

/** Super Earth's own sites: Managed Democracy, and the Citizenship Classification Structure under it. */
export function GovPage({ site, headingRef }: { site: "gov" | "citizens"; headingRef: HeadingRef }) {
  const prefix = `gww-${site}`;
  const s = site === "gov" ? GOV : CITIZENS;
  return (
    <article className="gww-page" style={{ "--site": site === "gov" ? "var(--f-humans)" : HUMANITY } as CSSProperties}>
      <GovNav at={site} />
      <Banner
        icon={<img src={iconUrl(s.icon)} alt="" width={72} height={72} />}
        crumbs={govCrumbs(site)}
        title={s.title}
        lead={s.summary}
        headingRef={headingRef}
      />
      {site === "citizens" && CITIZENS.broadcasts.map((b) => <Broadcast key={b} text={b} />)}
      <Toc sections={s.sections} prefix={prefix} />
      {s.sections.map((sec, i) => (
        <SectionBlock key={sec.heading} sec={sec} id={`${prefix}-${i}`} />
      ))}
      {site === "gov" && GOV.quotes.map((q) => <TrainingTip key={q} text={q} />)}
      <Related targets={s.related} />
      <SourceLine
        sources={s.sources}
        quoted={site === "gov" ? "Training manual tips are quoted from the game." : "Ship broadcasts are quoted from the game."}
      />
    </article>
  );
}

const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

function DateBadge({ h }: { h: Holiday }) {
  return (
    <span className="gww-date" aria-hidden>
      <span className="caps">{MONTHS[h.month - 1]}</span>
      <b className="num">{h.day}</b>
    </span>
  );
}

const countdown = (days: number) => (days === 0 ? "Today" : days === 1 ? "Tomorrow" : `In ${days} days`);

export function HolidaysPage({ headingRef }: { headingRef: HeadingRef }) {
  const { follow } = useGwwNav();
  const now = useNow(60 * 60_000);
  const next = nextYearly(HOLIDAYS, new Date(now));
  const until = (h: Holiday) => nextYearly([h], new Date(now))!.in;
  return (
    <article className="gww-page" style={{ "--site": "var(--cta)" } as CSSProperties}>
      <GovNav at="holidays" />
      <Banner
        icon={<CalendarDays size={40} strokeWidth={1.5} />}
        crumbs={govCrumbs("holidays")}
        title={HOLIDAYS_SITE.title}
        lead={HOLIDAYS_SITE.summary}
        headingRef={headingRef}
      />
      <ol className="gww-holidays">
        {HOLIDAYS.map((h) => {
          const days = until(h);
          return (
            <li key={h.id}>
              <button
                className={`gww-holiday-card ${h === next ? "next" : ""} ${days === 0 ? "today" : ""}`}
                style={{ "--site": h.color } as CSSProperties}
                onClick={(e) => follow(`holiday:${h.id}`, e)}
              >
                <DateBadge h={h} />
                <span className="gww-holiday-body">
                  <span className="gww-address num">{h.address}</span>
                  <b className="gww-site-title">{h.name}</b>
                  <span className="caps gww-holiday-when">{h.when}</span>
                  <span className="gww-site-text">{h.summary}</span>
                  {h.sections[0]?.text && (
                    <span className="gww-holiday-story">{plainText(h.sections[0].text)}</span>
                  )}
                </span>
                <span className={`gww-holiday-in caps num ${days <= 1 ? "soon" : ""}`}>{countdown(days)}</span>
              </button>
            </li>
          );
        })}
      </ol>
      <Related targets={HOLIDAYS_SITE.related} />
      <SourceLine sources={HOLIDAYS.flatMap((h) => h.sources)} />
    </article>
  );
}

export function HolidayPage({ h, headingRef }: { h: Holiday; headingRef: HeadingRef }) {
  const prefix = `gww-${h.id}`;
  const now = useNow(60 * 60_000);
  const days = nextYearly([h], new Date(now))!.in;
  return (
    <article className="gww-page" style={{ "--site": h.color } as CSSProperties}>
      <GovNav at="holidays" />
      <Banner
        icon={<DateBadge h={h} />}
        crumbs={[...govCrumbs("holidays"), { label: "Holidays", target: "site:holidays" }]}
        title={h.name}
        lead={
          <>
            {h.summary}{" "}
            <span className={`gww-holiday-in caps num ${days <= 1 ? "soon" : ""}`}>
              {h.when} · {countdown(days)}
            </span>
          </>
        }
        headingRef={headingRef}
      />
      {h.quotes.slice(0, 1).map((q) => (
        <SourcedQuote key={q.text} {...q} />
      ))}
      <Toc sections={h.sections} prefix={prefix} />
      {h.sections.map((sec, i) => (
        <SectionBlock key={sec.heading} sec={sec} id={`${prefix}-${i}`} />
      ))}
      {h.quotes.slice(1).map((q) => (
        <SourcedQuote key={q.text} {...q} />
      ))}
      <Related targets={h.related} />
      <SourceLine sources={h.sources} quoted={h.quotes.length ? "Decrees, carols, tips and broadcasts are quoted from the game." : undefined} />
      <Pager list={HOLIDAYS.map((x) => `holiday:${x.id}`)} at={HOLIDAYS.indexOf(h)} noun="holiday" />
    </article>
  );
}

export function MissingPage({ address, headingRef }: { address: string; headingRef: HeadingRef }) {
  return (
    <article className="gww-page" style={{ "--site": "var(--bad)" } as CSSProperties}>
      <Banner
        icon={<TriangleAlert size={40} strokeWidth={1.5} />}
        crumbs={[{ label: "G.W.W.", target: "home" }, { label: "Error 404" }]}
        title="Site not found"
        lead={
          <>
            No site answers at <span className="num">{address}</span>. If one ever did, the Ministry of Truth has seen to it.
          </>
        }
        headingRef={headingRef}
      />
      <Related title="Try instead" targets={["dir:manuals", "dir:gov", "dir:corps"]} />
    </article>
  );
}

function LogBox(props: { title: string; note: ReactNode; icon: ReactNode; children: ReactNode }) {
  return (
    <details className="gww-fold gww-logbox" open>
      <summary>
        <span className="gww-logbox-icon" aria-hidden>
          {props.icon}
        </span>
        <b>{props.title}</b>
        <span className="caps num">{props.note}</span>
      </summary>
      <div className="gww-logbox-body">{props.children}</div>
    </details>
  );
}

const LOG_DAYS = 7;

function WarLog() {
  const { changes, ready } = useHistory();
  const openOptions = useUi((s) => s.openOptions);
  const [show, setShow] = useState<"all" | "won" | "lost">("all");
  const [allDays, setAllDays] = useState(false);
  const list = changes.filter((c) => show === "all" || (show === "won") === (c.to === "Humans"));
  const days: { day: string; items: typeof list }[] = [];
  for (const c of list) {
    const day = fmtDate(c.at);
    if (days[days.length - 1]?.day !== day) days.push({ day, items: [] });
    days[days.length - 1].items.push(c);
  }
  return (
    <>
      <div className="gww-filter" role="group" aria-label="Show">
        {(
          [
            ["all", "Everything"],
            ["won", "Liberated"],
            ["lost", "Lost"],
          ] as const
        ).map(([k, label]) => (
          <button key={k} className={`gww-chip ${show === k ? "on" : ""}`} aria-pressed={show === k} onClick={() => setShow(k)}>
            {label}
          </button>
        ))}
      </div>
      {!ready ? (
        <Loading what="the war archive" rows={3} />
      ) : list.length === 0 ? (
        <Empty>
          {show === "all"
            ? "No planet has changed hands since tracking began."
            : `No planet has been ${show === "won" ? "liberated" : "lost"} since tracking began.`}
        </Empty>
      ) : (
        (allDays ? days : days.slice(0, LOG_DAYS)).map((d) => (
          <section key={d.day} className="gww-log-day">
            <h3 className="caps num">{d.day}</h3>
            <ol className="wl">
              {d.items.map((c) => {
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
          </section>
        ))
      )}
      {ready && days.length > LOG_DAYS && (
        <button className="gww-chip gww-more" onClick={() => setAllDays((a) => !a)}>
          {allDays ? "Show fewer days" : `Show all ${days.length} days`}
        </button>
      )}
    </>
  );
}

const ACTIVITY_DAYS = 14;

function DayStat({ kind, value, at, top }: { kind: "peak" | "low"; value: number; at: string; top: number }) {
  return (
    <span className={`gww-day-stat ${kind}`}>
      <span className="gww-day-num num">
        <b>{fmtInt(value)}</b>
        <span className="caps">
          <span className="sr-only">{kind === "peak" ? "peak" : "lowest"} </span>at {at}
        </span>
      </span>
      <span className="gww-day-bar" aria-hidden>
        <i style={{ width: `${(value / top) * 100}%` }} />
      </span>
    </span>
  );
}

function ActivityDays({ now }: { now: number }) {
  const { data: days = [], isLoading } = useWarDays();
  const [all, setAll] = useState(false);
  const shown = all ? days : days.slice(0, ACTIVITY_DAYS);
  const top = Math.max(1, ...shown.map((d) => d.peak));
  const today = Math.floor(now / 86_400_000) * 86_400_000;
  const hm = (ms: number) => sestClock(ms).slice(0, 5);
  if (isLoading) return <Loading what="the activity log" rows={2} />;
  if (days.length === 0) return <Empty>No days logged yet. Each day is recorded from the next collector update on.</Empty>;
  return (
    <>
      <div className="gww-days-head caps" aria-hidden>
        <span>SEST day</span>
        <span>Peak online</span>
        <span>Lowest online</span>
      </div>
      <ol className="gww-days">
        {shown.map((d) => (
          <li key={d.day} className={d.day === today ? "today" : ""}>
            <span className="gww-day-date num">
              {sestDate(d.day)}
              {d.day === today && <span className="caps">Today, so far</span>}
            </span>
            <DayStat kind="peak" value={d.peak} at={hm(d.peakAt)} top={top} />
            <DayStat kind="low" value={d.low} at={hm(d.lowAt)} top={top} />
          </li>
        ))}
      </ol>
      <p className="gww-note">Each bar is the day's count against the busiest day listed.</p>
      {days.length > ACTIVITY_DAYS && (
        <button className="gww-chip gww-more" onClick={() => setAll((a) => !a)}>
          {all ? "Show fewer days" : `Show all ${days.length} days`}
        </button>
      )}
    </>
  );
}

function StationJumps({ station, name, now }: { station: number; name: string; now: number }) {
  const { data: jumps = [], isLoading } = useStationJumps(station);
  return (
    <LogBox
      title={`${name} FTL jumps`}
      note={jumps.length === 1 ? "1 jump" : `${jumps.length} jumps`}
      icon={<DssIcon size={18} />}
    >
      {isLoading ? (
        <Loading what="the jump log" rows={2} />
      ) : jumps.length === 0 ? (
        <Empty>No jumps logged yet. Each jump is recorded from the next collector update on.</Empty>
      ) : (
        <JumpRoute jumps={jumps} now={now} />
      )}
    </LogBox>
  );
}

/** Every planet that changed hands and every station jump, as the collector logged them. */
export function LogsPage({ headingRef }: { headingRef: HeadingRef }) {
  const { changes, ready } = useHistory();
  const stations = useSpaceStations().data ?? [];
  const now = useNow(60_000);
  const liberated = changes.filter((c) => c.to === "Humans").length;
  return (
    <article className="gww-page" style={{ "--site": "var(--f-humans)" } as CSSProperties}>
      <Banner
        icon={<History size={40} strokeWidth={1.5} />}
        crumbs={[{ label: "Super Earth", target: "dir:gov" }]}
        title={LOG_TITLE}
        lead="The war log, Helldivers online day by day and the space stations' FTL jumps, since tracking began."
        headingRef={headingRef}
      />
      <LogBox
        title="War log"
        icon={<Globe2 size={18} />}
        note={
          ready ? (
            <>
              <span style={{ color: "var(--f-humans)" }}>{liberated} liberated</span> ·{" "}
              <span style={{ color: "var(--bad)" }}>{changes.length - liberated} lost</span>
            </>
          ) : (
            "Planets that changed hands"
          )
        }
      >
        <WarLog />
      </LogBox>
      <LogBox title="Helldivers online" icon={<Users size={18} />} note="Each SEST day, midnight to midnight">
        <ActivityDays now={now} />
      </LogBox>
      {stations.map((st, i) => (
        <StationJumps key={st.id32} station={st.id32} name={i === 0 ? "DSS" : `Station ${i + 1}`} now={now} />
      ))}
      <Related targets={["manual:liberation", "manual:dss", "app:stations", "app:past"]} />
    </article>
  );
}

export function DirPage({ id, headingRef, logText }: { id: keyof typeof DIRS; headingRef: HeadingRef; logText: string }) {
  const Icon = DIR_ICONS[id];
  const lead: Record<keyof typeof DIRS, string> = {
    manuals: "How the Second Galactic War works, in reading order.",
    gov: "The Federation's own sites: its government, its ministries, its logs and its news.",
    corps: "The mega-corporations that arm, feed, heal and bank the Federation.",
  };
  return (
    <article className="gww-page wide">
      <Banner
        icon={<Icon size={40} strokeWidth={1.5} />}
        crumbs={[{ label: "G.W.W.", target: "home" }]}
        title={DIRS[id]}
        lead={lead[id]}
        headingRef={headingRef}
      />
      {id === "manuals" && (
        <>
          <CourseProgress />
          {CHAPTERS.map((ch, i) => (
            <section key={ch.id} className="gww-chapter">
              <h3>
                <span className="caps num">Chapter {i + 1}</span> {ch.title}
              </h3>
              <p className="gww-chapter-sub">{ch.summary}</p>
              <div className="gww-sites">
                {MANUALS.filter((m) => m.chapter === ch.id).map((m) => (
                  <ManualTile key={m.id} m={m} />
                ))}
              </div>
            </section>
          ))}
        </>
      )}
      {id === "gov" && (
        <div className="gww-sites">
          <OfficialTiles logText={logText} />
          {MINISTRIES.map((m) => (
            <MinistryTile key={m.id} m={m} />
          ))}
        </div>
      )}
      {id === "corps" && (
        <div className="gww-sites">
          {COMPANIES.map((c) => (
            <CompanyTile key={c.id} c={c} />
          ))}
        </div>
      )}
      <Related
        title="Elsewhere on the G.W.W."
        targets={(["dir:manuals", "dir:gov", "dir:corps"] as const).filter((t) => t !== `dir:${id}`)}
      />
    </article>
  );
}

