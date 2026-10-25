import { Archive, ChevronRight, Flag, Flame, Gift, Trophy } from "lucide-react";
import { useRef, type CSSProperties } from "react";
import { useEpisodes } from "../../api/queries";
import { rewardLabel } from "../../data/campaignRewards";
import { factionColor } from "../../domain/factions";
import { fmtDate } from "../../domain/format";
import { campaignRecord, campaignTally, type EpisodeLive } from "../../domain/snapshot";
import { FactionEmblem } from "../../ui/emblems";
import { Empty, Loading, Panel, Tile } from "../../ui/primitives";
import { CampaignBody, CampaignHead, OUTCOME } from "../galactic-campaigns/Campaign";

const DAY_MS = 86_400_000;

const days = (e: EpisodeLive) => (e.end == null ? null : Math.max(1, Math.round((e.end - e.start) / DAY_MS)));

const reducedMotion = () => typeof window !== "undefined" && !!window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;

/** Every campaign oldest to newest, one block each in its enemy's colour; a block opens its campaign below. */
function RecordStrip({ past, onPick }: { past: EpisodeLive[]; onPick(id: number): void }) {
  const oldestFirst = [...past].reverse();
  const last = past[0];
  return (
    <div className="pc-record">
      <div className="pc-record-head caps">
        <span>Campaign record</span>
        <span className="pc-legend" aria-hidden>
          <i className="won" /> Victory <i className="lost" /> Failed
        </span>
      </div>
      <ol className="pc-strip">
        {oldestFirst.map((e, i) => (
          <li key={e.id} style={{ "--tone": factionColor(e.faction), "--i": i } as CSSProperties}>
            <button
              className={`pc-seg ${e.status}`}
              title={e.title}
              aria-label={`${e.title}: ${OUTCOME[e.status].label}`}
              onClick={() => onPick(e.id)}
            >
              <FactionEmblem faction={e.faction} size={14} />
            </button>
          </li>
        ))}
      </ol>
      <div className="pc-record-ends caps num">
        <span>{fmtDate(oldestFirst[0].start)}</span>
        <span>{fmtDate(last.end ?? last.start)}</span>
      </div>
    </div>
  );
}

/** Every Galactic Campaign that has ended, newest first; one moves here as soon as the game closes it. */
export function PastCampaignsModule() {
  const { data, isLoading } = useEpisodes();
  const cards = useRef(new Map<number, HTMLDetailsElement>());
  const past = (data ?? []).filter((e) => e.status !== "active").sort((a, b) => b.start - a.start);
  const record = campaignRecord(past);
  const atWar = past.reduce((n, e) => n + (days(e) ?? 0), 0);

  const pick = (id: number) => {
    const card = cards.current.get(id);
    if (!card) return;
    card.open = true;
    card.querySelector("summary")?.focus({ preventScroll: true });
    card.scrollIntoView({ behavior: reducedMotion() ? "auto" : "smooth", block: "start" });
    // Restarts the highlight when the same block is picked twice.
    delete card.dataset.flash;
    void card.offsetWidth;
    card.dataset.flash = "";
  };

  return (
    <Panel
      title="Past campaigns"
      icon={Archive}
      right={
        past.length > 0 && (
          <span className="num">
            <span style={{ color: "var(--good)" }}>{record.won} won</span> ·{" "}
            <span style={{ color: "var(--bad)" }}>{past.length - record.won} failed</span>
          </span>
        )
      }
    >
      {isLoading ? (
        <Loading what="the campaign archive" rows={3} />
      ) : past.length === 0 ? (
        <Empty>{data ? "No campaign has ended yet." : "Campaign data arrives after the collector update."}</Empty>
      ) : (
        <div className="pc-archive">
          <div className="pc-stats">
            <Tile icon={Archive} label="Campaigns" value={past.length} sub={`${atWar} campaign days`} rule="var(--cta)" />
            <Tile
              icon={Trophy}
              label="Victories"
              value={record.won}
              sub={`${Math.round((record.won / past.length) * 100)}% win rate`}
              color="var(--good)"
              rule="var(--good)"
            />
            <Tile
              icon={Flag}
              label="Major Orders won"
              value={`${record.ordersWon} / ${record.orders}`}
              sub={record.orders ? `${Math.round((record.ordersWon / record.orders) * 100)}% of orders` : undefined}
              rule="var(--info)"
            />
            <Tile
              icon={Flame}
              label="Win streak"
              value={record.streak}
              sub={record.streak === 1 ? "Latest campaign" : "Campaigns in a row"}
              color={record.streak > 0 ? "var(--cta)" : undefined}
              rule="var(--cta-deep)"
            />
          </div>
          <RecordStrip past={past} onPick={pick} />
          <ol className="pc-list">
            {past.map((e, i) => {
              const t = campaignTally(e);
              const d = days(e);
              return (
                <li key={e.id} style={{ "--i": i } as CSSProperties}>
                  <details
                    className={`pc ${e.status}`}
                    style={{ "--tone": factionColor(e.faction) } as CSSProperties}
                    ref={(el) => {
                      if (el) cards.current.set(e.id, el);
                      else cards.current.delete(e.id);
                    }}
                    onAnimationEnd={(ev) => {
                      if (ev.target === ev.currentTarget) delete ev.currentTarget.dataset.flash;
                    }}
                  >
                    <summary>
                      <span className="pc-mark" aria-hidden>
                        <FactionEmblem faction={e.faction} size={132} />
                      </span>
                      <CampaignHead e={e} />
                      <span className="pc-meta caps num">
                        <span className="pc-orders">
                          <span className="pc-pips" aria-hidden>
                            {e.phases.map((p, j) => (
                              <i key={j} className={p.status} />
                            ))}
                          </span>
                          {t.won} of {e.phases.length} Major Orders won
                        </span>
                        {d != null && <span>{d === 1 ? "1 day" : `${d} days`}</span>}
                        <span className="pc-reward">
                          <Gift size={12} aria-hidden />
                          <span className="pc-reward-name">{rewardLabel(e.reward)}</span>
                          {e.status === "lost" && <span className="sr-only"> (not earned)</span>}
                        </span>
                      </span>
                      <ChevronRight className="pc-chevron" size={16} aria-hidden />
                    </summary>
                    <CampaignBody e={e} />
                  </details>
                </li>
              );
            })}
          </ol>
        </div>
      )}
    </Panel>
  );
}
