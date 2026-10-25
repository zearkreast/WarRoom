import type { CSSProperties, ReactNode } from "react";
import { rewardLabel } from "../../data/campaignRewards";
import type { Dispatch } from "../../api/types";
import type { CampaignThread, ThreadEntry } from "../../domain/campaignNews";
import { dispatchKind } from "../../domain/dispatch";
import { factionColor } from "../../domain/factions";
import { dispatchTitle, fmtAgo, sestClock, sestDate } from "../../domain/format";
import { bodyEchoesTitle } from "../../domain/newsFeed";
import { campaignTally, phaseName, type EpisodeLive } from "../../domain/snapshot";
import { KIND_LABEL, MarkupText } from "./NewsCard";

const RESULT: Record<EpisodeLive["status"], string> = { active: "In progress", won: "Won", lost: "Failed" };

const clock = (at: number) => sestClock(at).slice(0, 5);

/** The frame every feed item shares: time on the rail, then a tag line, the headline and the story. */
function Item(props: {
  className: string;
  at: number;
  now: number;
  tag: string;
  unread: boolean;
  featured: boolean;
  style?: CSSProperties;
  title: string;
  children: ReactNode;
}) {
  return (
    <article
      className={`ni ${props.className} ${props.unread ? "unread" : ""} ${props.featured ? "featured" : ""}`}
      style={props.style}
    >
      <time className="ni-time num" dateTime={new Date(props.at).toISOString()}>
        {clock(props.at)}
      </time>
      <div className="ni-main">
        <div className="ni-meta caps">
          {props.featured && <span className="ni-latest">Latest</span>}
          <span className="ni-tag">{props.tag}</span>
          {props.unread && <span className="ni-new">New</span>}
          <span className="ni-age">{fmtAgo(props.at, props.now)}</span>
        </div>
        <h3 className="ni-title">{props.title}</h3>
        {props.children}
      </div>
    </article>
  );
}

export function DispatchItem({
  d,
  now,
  unread,
  featured,
}: {
  d: Dispatch;
  now: number;
  unread: boolean;
  featured: boolean;
}) {
  const kind = dispatchKind(d.message);
  return (
    <Item
      className={`ni-${kind}`}
      at={Date.parse(d.published)}
      now={now}
      tag={KIND_LABEL[kind]}
      unread={unread}
      featured={featured}
      title={dispatchTitle(d.message)}
    >
      {!bodyEchoesTitle(d.message) && <MarkupText message={d.message} className="ni-body" />}
    </Item>
  );
}

function entryLabel(x: ThreadEntry): string {
  if (!x.event) return dispatchTitle(x.dispatch.message);
  const { type, campaign: e, phase } = x.event;
  if (type === "launched") return "Campaign launched";
  if (type === "ended") return `Campaign ${e.status === "won" ? "won" : "failed"}`;
  const name = phaseName(e.phases[phase]);
  return name ? `Major Order ${phase + 1}: ${name}` : `Major Order ${phase + 1}`;
}

function EntryBody({ x }: { x: ThreadEntry }) {
  if (x.dispatch)
    return bodyEchoesTitle(x.dispatch.message) ? null : <MarkupText message={x.dispatch.message} className="ni-body" />;
  if (!x.event) return null;
  const { type, campaign: e, phase } = x.event;
  if (type === "launched")
    return (
      <>
        {e.description && <p className="ni-body">{e.description}</p>}
        {e.phases[0]?.briefing && <MarkupText message={e.phases[0].briefing} className="ni-body" whole />}
      </>
    );
  if (type === "ended") {
    const tally = campaignTally(e);
    const outro = e.phases[e.phases.length - 1]?.outro;
    return (
      <>
        <p className="ni-body">
          {tally.won} of {e.phases.length} Major Orders won. Campaign reward, {rewardLabel(e.reward)}:{" "}
          {e.status === "won" ? "earned" : "lost"}.
        </p>
        {outro && <MarkupText message={outro} className="ni-body" whole />}
      </>
    );
  }
  const briefing = e.phases[phase]?.briefing;
  return briefing ? <MarkupText message={briefing} className="ni-body" whole /> : null;
}

/** A Galactic Campaign as one story: its newest entry in full, earlier ones folded underneath. */
export function CampaignItem({
  thread,
  now,
  unread,
  featured,
}: {
  thread: CampaignThread;
  now: number;
  unread: boolean;
  featured: boolean;
}) {
  const e = thread.campaign;
  const latest = thread.entries[thread.entries.length - 1];
  const earlier = thread.entries.slice(0, -1).reverse();
  const tally = campaignTally(e);
  return (
    <Item
      className="ni-campaign"
      style={{ "--tone": factionColor(e.faction) } as CSSProperties}
      at={thread.at}
      now={now}
      tag="Campaign"
      unread={unread}
      featured={featured}
      title={e.title}
    >
      <p className="ni-sub caps num">
        <span className={`ni-result ${e.status}`}>{RESULT[e.status]}</span>
        <span>
          {tally.won} won · {tally.lost} lost{tally.running ? ` · ${tally.running} running` : ""} · since{" "}
          {sestDate(e.start)}
        </span>
      </p>
      <h4 className="ni-entry caps">{entryLabel(latest)}</h4>
      <EntryBody x={latest} />
      {earlier.length > 0 && (
        <details className="ni-thread">
          <summary className="caps">
            {earlier.length} earlier {earlier.length === 1 ? "entry" : "entries"}
          </summary>
          <ol>
            {earlier.map((x) => (
              <li key={x.key}>
                <details>
                  <summary>
                    <span className="num">
                      {clock(x.at)} · {sestDate(x.at)}
                    </span>
                    <span>{entryLabel(x)}</span>
                  </summary>
                  <EntryBody x={x} />
                </details>
              </li>
            ))}
          </ol>
        </details>
      )}
    </Item>
  );
}
