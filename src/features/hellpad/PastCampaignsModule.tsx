import { Archive } from "lucide-react";
import { useEpisodes } from "../../api/queries";
import { rewardLabel } from "../../data/campaignRewards";
import { campaignTally } from "../../domain/snapshot";
import { Empty, Loading, Panel } from "../../ui/primitives";
import { CampaignBody, CampaignHead } from "../galactic-campaigns/Campaign";

/** Every Galactic Campaign that has ended, newest first; one moves here as soon as the game closes it. */
export function PastCampaignsModule() {
  const { data, isLoading } = useEpisodes();
  const past = (data ?? []).filter((e) => e.status !== "active").sort((a, b) => b.start - a.start);
  const won = past.filter((e) => e.status === "won").length;
  return (
    <Panel
      title="Past campaigns"
      icon={Archive}
      right={
        past.length > 0 && (
          <span className="num">
            <span style={{ color: "var(--good)" }}>{won} won</span> ·{" "}
            <span style={{ color: "var(--bad)" }}>{past.length - won} failed</span>
          </span>
        )
      }
    >
      {isLoading ? (
        <Loading what="the campaign archive" rows={3} />
      ) : past.length === 0 ? (
        <Empty>{data ? "No campaign has ended yet." : "Campaign data arrives after the collector update."}</Empty>
      ) : (
        <ol className="pc-list">
          {past.map((e) => {
            const t = campaignTally(e);
            return (
              <li key={e.id}>
                <details className="pc">
                  <summary>
                    <CampaignHead e={e} />
                    <span className="pc-meta caps num">
                      {t.won} of {e.phases.length} Major Orders won · {rewardLabel(e.reward)}
                    </span>
                  </summary>
                  <CampaignBody e={e} />
                </details>
              </li>
            );
          })}
        </ol>
      )}
    </Panel>
  );
}
