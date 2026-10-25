import {
  Activity,
  ArrowLeft,
  ArrowRight,
  Building2,
  CircleHelp,
  CloudFog,
  Flag,
  History,
  Lightbulb,
  Network,
  Quote,
  Satellite,
  type LucideIcon,
} from "lucide-react";
import { type CSSProperties, type RefCallback } from "react";
import { ADS, COMPANIES, MANUALS, WIKI, WIKI_LICENCE, type Ad, type Company, type Manual, type Ministry } from "../../data/gww";
import { factionColor, factionLabel } from "../../domain/factions";
import { fmtDateTime } from "../../domain/format";
import { useHistory } from "../../history/useHistory";
import { useUi } from "../../state";
import { FactionEmblem } from "../../ui/emblems";
import { Empty, Loading, OutLink, Panel } from "../../ui/primitives";
import { Figure } from "./GwwFigures";
import { adUrl, iconUrl, logoUrl } from "./gwwArt";

/** Drawn icons for manuals the game has no picture for. */
const MANUAL_ICONS: Record<string, LucideIcon> = {
  resistance: Activity,
  regions: Building2,
  "supply-lines": Network,
  dss: Satellite,
  campaigns: Flag,
  gloom: CloudFog,
  misconceptions: CircleHelp,
};

export function ManualIcon({ m, size }: { m: Manual; size: number }) {
  const url = m.icon && iconUrl(m.icon);
  if (url) return <img src={url} alt="" width={size} height={size} />;
  const Icon = MANUAL_ICONS[m.id] ?? Lightbulb;
  return <Icon size={size} strokeWidth={1.5} aria-hidden />;
}

export function AdBox({ ad }: { ad: Ad }) {
  const url = adUrl(ad.file);
  if (!url) return null;
  return (
    <figure className={`gww-ad ${ad.shape}`}>
      <img src={url} alt={ad.alt} loading="lazy" />
      <figcaption className="caps">Sponsored</figcaption>
    </figure>
  );
}

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

function FieldTip({ text }: { text: string }) {
  return (
    <aside className="gww-tip">
      <Lightbulb size={16} aria-hidden />
      <p>
        <b className="caps">Field tip</b> {text}
      </p>
    </aside>
  );
}

/** Galaxy-wide war log: every capture and loss the collector has recorded. */
export function WarLog({ headingRef }: { headingRef: RefCallback<HTMLElement> }) {
  const { changes, ready } = useHistory();
  const openOptions = useUi((s) => s.openOptions);
  const liberated = changes.filter((c) => c.to === "Humans").length;
  return (
    <Panel
      title={
        <span ref={headingRef} tabIndex={-1}>
          Galactic war log
        </span>
      }
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

function Sources({ m }: { m: Manual }) {
  return (
    <footer className="gww-source">
      {m.sources?.length ? (
        <>
          Adapted from the {WIKI.home.title}:{" "}
          {m.sources.map((s, i) => (
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
      {m.quotes?.length ? <> Training manual tips are quoted from the game.</> : null}
    </footer>
  );
}

export function ManualPage(props: { m: Manual; headingRef: RefCallback<HTMLElement>; open(id: string): void }) {
  const { m } = props;
  const at = MANUALS.indexOf(m);
  const prev = MANUALS[at - 1];
  const next = MANUALS[at + 1];
  return (
    <article className="gww-page">
      <header className="gww-page-head">
        <span className="gww-page-icon">
          <ManualIcon m={m} size={40} />
        </span>
        <div>
          <p className="gww-kicker caps num">
            Training manual {at + 1} of {MANUALS.length}
          </p>
          <h2 className="gww-title" ref={props.headingRef} tabIndex={-1}>
            {m.title}
          </h2>
          <p className="gww-lead">{m.summary}</p>
        </div>
      </header>
      {m.sections.map((sec) => (
        <section key={sec.heading}>
          <h3>{sec.heading}</h3>
          {sec.text && <p>{sec.text}</p>}
          {sec.points && (
            <ul className="gww-points">
              {sec.points.map((p) => {
                const [term, rest] = p.includes(": ") ? [p.slice(0, p.indexOf(": ")), p.slice(p.indexOf(": ") + 2)] : [null, p];
                return (
                  <li key={p}>
                    {term && <b>{term}</b>} {rest}
                  </li>
                );
              })}
            </ul>
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
                            {c}
                          </th>
                        ) : (
                          <td key={i}>{c}</td>
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
                    <p>{k.text}</p>
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
                  <p>{f.a}</p>
                </details>
              ))}
            </div>
          )}
          {sec.figure && <Figure kind={sec.figure} />}
        </section>
      ))}
      {m.quotes?.map((q) => <TrainingTip key={q} text={q} />)}
      {m.tips.map((t) => (
        <FieldTip key={t} text={t} />
      ))}
      <Sources m={m} />
      <nav className="gww-pager" aria-label="Other manuals">
        {prev ? (
          <button className="gww-pager-link" onClick={() => props.open(prev.id)}>
            <ArrowLeft size={14} aria-hidden />
            <span>
              <span className="caps">Previous</span> {prev.title}
            </span>
          </button>
        ) : (
          <span />
        )}
        {next && (
          <button className="gww-pager-link next" onClick={() => props.open(next.id)}>
            <span>
              <span className="caps">Next</span> {next.title}
            </span>
            <ArrowRight size={14} aria-hidden />
          </button>
        )}
      </nav>
    </article>
  );
}

export function MinistryPage(props: { m: Ministry; headingRef: RefCallback<HTMLElement>; openCompany(name: string): void }) {
  const { m } = props;
  const partners = COMPANIES.filter((c) => m.partners?.includes(c.name));
  const ads = ADS.filter((a) => m.ads?.includes(a.file));
  return (
    <article className="gww-page gww-brand" style={{ "--site": m.color } as CSSProperties}>
      <header className="gww-banner">
        {iconUrl(m.icon) && <img src={iconUrl(m.icon)} alt="" width={72} height={72} />}
        <div>
          <p className="gww-kicker caps">Federation of Super Earth</p>
          <h2 className="gww-title" ref={props.headingRef} tabIndex={-1}>
            {m.name}
          </h2>
        </div>
      </header>
      <TrainingTip text={m.motto} />
      <p>{m.text}</p>
      {partners.length > 0 && (
        <section>
          <h3>Contractors</h3>
          <div className="gww-partners">
            {partners.map((c) => (
              <button key={c.name} className="gww-partner" onClick={() => props.openCompany(c.name)}>
                <CompanyMark c={c} />
                <span>{c.name}</span>
              </button>
            ))}
          </div>
        </section>
      )}
      {ads.length > 0 && (
        <div className="gww-ads-inline">
          {ads.map((a) => (
            <AdBox key={a.file} ad={a} />
          ))}
        </div>
      )}
    </article>
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

export function CompanyPage({ c, headingRef }: { c: Company; headingRef: RefCallback<HTMLElement> }) {
  const ads = ADS.filter((a) => c.ads?.includes(a.file));
  return (
    <article className="gww-page gww-brand" style={{ "--site": c.color } as CSSProperties}>
      <header className="gww-banner corp">
        <CompanyMark c={c} />
        <div>
          <p className="gww-kicker caps">{c.trade}</p>
          <h2 className="gww-title" ref={headingRef} tabIndex={-1}>
            {c.name}
          </h2>
          {c.slogan && <p className="gww-slogan">“{c.slogan}”</p>}
        </div>
      </header>
      <p>{c.text}</p>
      {c.quote && <TrainingTip text={c.quote} />}
      {ads.length > 0 && (
        <div className="gww-ads-inline">
          {ads.map((a) => (
            <AdBox key={a.file} ad={a} />
          ))}
        </div>
      )}
    </article>
  );
}
