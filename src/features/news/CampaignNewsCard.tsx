import type { CSSProperties } from "react";
import { rewardLabel } from "../../data/campaignRewards";
import type { CampaignThread, ThreadEntry } from "../../domain/campaignNews";
import { factionColor } from "../../domain/factions";
import { dispatchTitle, fmtAgo, sestClock, sestDate } from "../../domain/format";
import { campaignTally, phaseName, type EpisodeLive } from "../../domain/snapshot";
import { MarkupText } from "./NewsCard";

const RESULT: Record<EpisodeLive["status"], string> = { active: "In progress", won: "Won", lost: "Failed" };

function entryLabel(x: ThreadEntry): string {
  if (!x.event) return dispatchTitle(x.dispatch.message);
  const { type, campaign: e, phase } = x.event;
  if (type === "launched") return "Campaign launched";
  if (type === "ended") return `Campaign ${e.status === "won" ? "won" : "failed"}`;
  const name = phaseName(e.phases[phase]);
  return name ? `Major Order ${phase + 1}: ${name}` : `Major Order ${phase + 1}`;
}

function EntryBody({ x }: { x: ThreadEntry }) {
  if (x.dispatch) return <MarkupText message={x.dispatch.message} />;
  if (!x.event) return null;
  const { type, campaign: e, phase } = x.event;
  if (type === "launched")
    return (
      <>
        {e.description && <p className="nc-body">{e.description}</p>}
        {e.phases[0]?.briefing && <MarkupText message={e.phases[0].briefing} whole />}
      </>
    );
  if (type === "ended") {
    const tally = campaignTally(e);
    const outro = e.phases[e.phases.length - 1]?.outro;
    return (
      <>
        <p className="nc-body">
          {tally.won} of {e.phases.length} Major Orders won. Campaign reward, {rewardLabel(e.reward)}:{" "}
          {e.status === "won" ? "earned" : "lost"}.
        </p>
        {outro && <MarkupText message={outro} whole />}
      </>
    );
  }
  const briefing = e.phases[phase]?.briefing;
  return briefing ? <MarkupText message={briefing} whole /> : null;
}

const stamp = (at: number) => `${sestClock(at).slice(0, 5)} · ${sestDate(at)}`;

/** A Galactic Campaign as one story: its launch, orders, dispatches and end on one card. */
export function CampaignNewsCard({ thread, now }: { thread: CampaignThread; now: number }) {
  const e = thread.campaign;
  const latest = thread.entries[thread.entries.length - 1];
  const earlier = thread.entries.slice(0, -1).reverse();
  const tally = campaignTally(e);
  return (
    <article className="nc nc-campaign" style={{ "--tone": factionColor(e.faction) } as CSSProperties}>
      <div className="nc-banner" aria-hidden>
        <span className="nc-word">Campaign</span>
      </div>
      <div className="nc-tagrow">
        <span className="nc-age caps">{fmtAgo(thread.at, now)}</span>
      </div>
      <h3 className="nc-title">{e.title}</h3>
      <p className="nc-sub caps num">
        <span className={`nc-result ${e.status}`}>{RESULT[e.status]}</span>
        <span>
          {tally.won} won · {tally.lost} lost{tally.running ? ` · ${tally.running} running` : ""}
        </span>
      </p>
      <h4 className="nc-latest caps">{entryLabel(latest)}</h4>
      <EntryBody x={latest} />
      {earlier.length > 0 && (
        <ol className="nc-thread" aria-label="Earlier in this campaign">
          {earlier.map((x) => (
            <li key={x.key}>
              <details>
                <summary>
                  <span className="num">{stamp(x.at)}</span>
                  <span>{entryLabel(x)}</span>
                </summary>
                <EntryBody x={x} />
              </details>
            </li>
          ))}
        </ol>
      )}
      <footer className="nc-foot num caps">
        <span>Since {sestDate(e.start)}</span>
        <span>SEST {stamp(thread.at)}</span>
      </footer>
    </article>
  );
}
