import type { CSSProperties } from "react";
import { rewardLabel } from "../../data/campaignRewards";
import { campaignHeadline, type CampaignEvent } from "../../domain/campaignNews";
import { factionColor } from "../../domain/factions";
import { fmtAgo, sestClock, sestDate } from "../../domain/format";
import { campaignTally } from "../../domain/snapshot";
import { MarkupText } from "./NewsCard";

/** A Galactic Campaign's launch, new Major Order or end, in the same card as a dispatch. */
export function CampaignNewsCard({ ev, now }: { ev: CampaignEvent; now: number }) {
  const e = ev.campaign;
  const order = e.phases[ev.phase];
  const tally = campaignTally(e);
  return (
    <article className="nc nc-campaign" style={{ "--tone": factionColor(e.faction) } as CSSProperties}>
      <div className="nc-banner" aria-hidden>
        <span className="nc-word">Campaign</span>
      </div>
      <div className="nc-tagrow">
        <span className="nc-age caps">{fmtAgo(ev.at, now)}</span>
      </div>
      <h3 className="nc-title">{campaignHeadline(ev)}</h3>
      {ev.type === "launched" && e.description && <p className="nc-body">{e.description}</p>}
      {ev.type === "ended" ? (
        <>
          <p className="nc-body">
            {tally.won} of {e.phases.length} Major Orders won. Campaign reward, {rewardLabel(e.reward)}:{" "}
            {e.status === "won" ? "earned" : "lost"}.
          </p>
          {order?.outro && <MarkupText message={order.outro} whole />}
        </>
      ) : (
        order?.briefing && <MarkupText message={order.briefing} whole />
      )}
      <footer className="nc-foot num caps">
        <span>{e.title}</span>
        <span>
          SEST {sestClock(ev.at).slice(0, 5)} · {sestDate(ev.at)}
        </span>
      </footer>
    </article>
  );
}
