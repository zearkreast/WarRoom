import { Flag } from "lucide-react";
import type { CSSProperties } from "react";
import { useEpisodes } from "../../api/queries";
import { CAMPAIGN_REWARDS } from "../../data/campaignRewards";
import { factionColor } from "../../domain/factions";
import { fmtDate } from "../../domain/format";
import { campaignTally, type EpisodeLive, type EpisodeStatus } from "../../domain/snapshot";
import { FactionEmblem } from "../../ui/emblems";
import { Empty, Panel, StatusChip } from "../../ui/primitives";
import { MarkupText } from "../news/NewsCard";
import "./campaign.css";

export const OUTCOME: Record<EpisodeStatus, { label: string; tone: string }> = {
  active: { label: "In progress", tone: "var(--info)" },
  won: { label: "Victory", tone: "var(--good)" },
  lost: { label: "Failed", tone: "var(--bad)" },
};

/** A Galactic Campaign runs three Major Orders; the game names each one only when it is issued. */
const CAMPAIGN_ORDERS = 3;

const REWARD_RULE: Record<EpisodeStatus, string> = {
  active: "Earned by winning most of the campaign's Major Orders.",
  won: "Earned: most Major Orders were won.",
  lost: "Lost: most Major Orders failed.",
};

export function OutcomeChip({ status }: { status: EpisodeStatus }) {
  return <StatusChip tone={OUTCOME[status].tone}>{OUTCOME[status].label}</StatusChip>;
}

export function campaignDates(e: EpisodeLive): string {
  return e.end == null ? `Since ${fmtDate(e.start)}` : `${fmtDate(e.start)} – ${fmtDate(e.end)}`;
}

/** Emblem, name, dates and outcome: the campaign's title row. */
export function CampaignHead({ e }: { e: EpisodeLive }) {
  return (
    <div className="cmp-head" style={{ "--tone": factionColor(e.faction) } as CSSProperties}>
      <FactionEmblem faction={e.faction} size={22} />
      <span className="cmp-name">
        <b className="cmp-title">{e.title}</b>
        <span className="cmp-dates caps num">{campaignDates(e)}</span>
      </span>
      <OutcomeChip status={e.status} />
    </div>
  );
}

/** Description, reward, tally and each Major Order with its briefing, as the game's Control Center lists them. */
export function CampaignBody({ e }: { e: EpisodeLive }) {
  const tally = campaignTally(e);
  const reward = e.reward == null ? undefined : CAMPAIGN_REWARDS[e.reward];
  const toCome = e.status === "active" ? Math.max(0, CAMPAIGN_ORDERS - e.phases.length) : 0;
  return (
    <div className="cmp-body">
      {e.description && <p className="cmp-desc">{e.description}</p>}
      <div className={`cmp-reward ${e.status}`}>
        <span className="cmp-label caps">Campaign reward</span>
        <b className="cmp-reward-name">{reward?.name ?? reward?.kind ?? "Not announced"}</b>
        {reward?.name && <span className="cmp-reward-kind caps">{reward.kind}</span>}
        <span className="cmp-rule">{REWARD_RULE[e.status]}</span>
      </div>
      <div className="cmp-tally caps num">
        Major Orders <b>{tally.won}</b> won · <b>{tally.lost}</b> lost
        {tally.running > 0 && (
          <>
            {" "}
            · <b>{tally.running}</b> running
          </>
        )}
        {toCome > 0 && (
          <>
            {" "}
            · <b>{toCome}</b> to come
          </>
        )}
      </div>
      <ol className="cmp-orders">
        {e.phases.map((p, i) => (
          <li key={i}>
            <details>
              <summary>
                <span className="cmp-mo caps num">MO {i + 1}</span>
                <b className="cmp-mo-title">{p.title}</b>
                {p.medals != null && <span className="cmp-medals caps num">{p.medals} medals</span>}
                <OutcomeChip status={p.status} />
              </summary>
              {p.issued != null && <span className="cmp-issued caps num">Issued {fmtDate(p.issued)}</span>}
              {p.briefing && <MarkupText message={p.briefing} className="cmp-text" whole />}
              {p.outro && <MarkupText message={p.outro} className="cmp-text cmp-outro" whole />}
            </details>
          </li>
        ))}
        {Array.from({ length: toCome }, (_, i) => (
          <li key={`slot-${i}`} className="cmp-slot">
            <span className="cmp-mo caps num">MO {e.phases.length + i + 1}</span>
            <span className="cmp-slot-text caps">Awaiting orders from High Command</span>
          </li>
        ))}
      </ol>
    </div>
  );
}

/** The running Galactic Campaign; between campaigns, the last one and how it ended. */
export function CampaignPanel() {
  const { data, isLoading } = useEpisodes();
  if (isLoading) return null;
  const latest = (data ?? []).reduce<EpisodeLive | undefined>((m, e) => (!m || e.start > m.start ? e : m), undefined);
  return (
    <Panel
      className="cmp-panel"
      icon={Flag}
      title={latest && latest.status !== "active" ? "Last campaign" : "Galactic campaign"}
    >
      {latest ? (
        <div className="cmp">
          <CampaignHead e={latest} />
          <CampaignBody e={latest} />
        </div>
      ) : (
        <Empty>Campaign data arrives after the collector update.</Empty>
      )}
    </Panel>
  );
}
