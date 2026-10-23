import { Flag } from "lucide-react";
import type { CSSProperties } from "react";
import { useEpisodes } from "../../api/queries";
import { fmtDate } from "../../domain/format";
import type { EpisodeStatus } from "../../domain/snapshot";
import { Empty, Panel } from "../../ui/primitives";
import { MarkupText } from "../news/NewsCard";

const STATUS: Record<EpisodeStatus, { label: string; tone: string }> = {
  active: { label: "In progress", tone: "var(--info)" },
  won: { label: "Won", tone: "var(--good)" },
  lost: { label: "Failed", tone: "var(--bad)" },
};

const PAST_SHOWN = 5;

function Chip({ status }: { status: EpisodeStatus }) {
  const s = STATUS[status];
  return (
    <span className="ac-chip caps" style={{ "--tone": s.tone } as CSSProperties}>
      {s.label}
    </span>
  );
}

/** The game's running story campaign: its phases, the current briefing, and how earlier campaigns ended. */
export function ActiveCampaign() {
  const episodes = useEpisodes().data;
  if (!episodes) return null;
  const current = [...episodes].reverse().find((e) => e.status === "active");
  const past = episodes.filter((e) => e !== current).reverse();
  const phase = current?.phases[current.phases.length - 1];

  return (
    <Panel className="ac" icon={Flag} title="Active campaign">
      {current ? (
        <>
          <div className="ac-head">
            <b className="ac-title">{current.title}</b>
            <span className="ac-since caps num">Since {fmtDate(current.start)}</span>
          </div>
          <p className="ac-desc">{current.description}</p>
          <ol className="ac-phases">
            {current.phases.map((p, i) => (
              <li key={i} className={p.status}>
                <span className="caps num">Phase {i + 1}</span>
                <b>{p.title}</b>
                <Chip status={p.status} />
              </li>
            ))}
          </ol>
          {phase?.briefing && (
            <details className="ac-briefing">
              <summary className="caps">Phase briefing</summary>
              <MarkupText message={phase.briefing} className="ac-briefing-text" />
            </details>
          )}
          {phase?.medals != null && (
            <p className="ac-reward caps">
              Phase reward <b className="num">{phase.medals}</b> Warbond Medals
            </p>
          )}
        </>
      ) : (
        <Empty>No story campaign is running. Await further instructions.</Empty>
      )}
      {past.length > 0 && (
        <div className="ac-past">
          <span className="ac-sub caps">Earlier campaigns</span>
          <ul>
            {past.slice(0, PAST_SHOWN).map((e) => (
              <li key={e.id}>
                <span className="ac-past-name">{e.title}</span>
                <span className="num caps">
                  {fmtDate(e.start)}
                  {e.end != null && ` – ${fmtDate(e.end)}`}
                </span>
                <Chip status={e.status} />
              </li>
            ))}
          </ul>
        </div>
      )}
    </Panel>
  );
}
