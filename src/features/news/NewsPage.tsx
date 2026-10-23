import { useMemo, useState } from "react";
import { useDispatches, useEpisodes } from "../../api/queries";
import type { Dispatch } from "../../api/types";
import { campaignEvents, type CampaignEvent } from "../../domain/campaignNews";
import { dispatchKind, type DispatchKind } from "../../domain/dispatch";
import { Hint } from "../../ui/Hint";
import { Empty, Loading, useNow } from "../../ui/primitives";
import { CampaignNewsCard } from "./CampaignNewsCard";
import { KIND_LABEL, NewsCard } from "./NewsCard";

const PAGE = 20;
type Kind = DispatchKind | "campaign";
type Filter = "all" | Kind;
type Item = { kind: Kind; at: number; key: string } & ({ d: Dispatch; ev?: never } | { ev: CampaignEvent; d?: never });

export function NewsPage() {
  const { data, isLoading } = useDispatches();
  const episodes = useEpisodes().data;
  const now = useNow(60_000);
  const [shown, setShown] = useState(PAGE);
  const [filter, setFilter] = useState<Filter>("all");
  const items = useMemo((): Item[] => {
    const dispatches = (data ?? []).map(
      (d): Item => ({ kind: dispatchKind(d.message), at: Date.parse(d.published), key: `d${d.id}`, d }),
    );
    const campaign = campaignEvents(episodes ?? []).map(
      (ev): Item => ({ kind: "campaign", at: ev.at, key: ev.key, ev }),
    );
    return [...dispatches, ...campaign].sort((a, b) => b.at - a.at);
  }, [data, episodes]);
  const kinds = useMemo(() => {
    const m = new Map<Kind, number>();
    for (const i of items) m.set(i.kind, (m.get(i.kind) ?? 0) + 1);
    return m;
  }, [items]);
  const list = filter === "all" ? items : items.filter((i) => i.kind === filter);

  return (
    <div className="page scroll starfield">
      <div className="page-inner news-page">
        <div className="page-head">
          <div>
            <h1 className="page-title">Strohmann News</h1>
            <p className="page-sub">
              Dispatches and Galactic Campaign news from Super Earth, newest first. Planet names open their options.
            </p>
          </div>
        </div>
        <Hint id="news">
          Planet names in a dispatch are links: click one to pin it, find it on the map or open its record.
        </Hint>
        <div className="filter-row" role="toolbar" aria-label="Filter news">
          <button className={`filter-chip ${filter === "all" ? "on" : ""}`} onClick={() => setFilter("all")}>
            All <span className="count">{items.length}</span>
          </button>
          {[...kinds].map(([k, n]) => (
            <button key={k} className={`filter-chip ${filter === k ? "on" : ""}`} onClick={() => setFilter(k)}>
              {k === "campaign" ? "Campaign" : KIND_LABEL[k]} <span className="count">{n}</span>
            </button>
          ))}
        </div>
        {isLoading ? (
          <Loading what="dispatches" rows={4} />
        ) : list.length === 0 ? (
          <Empty>No dispatches received yet.</Empty>
        ) : (
          <div className="news-grid">
            {list
              .slice(0, shown)
              .map((i) =>
                i.d ? (
                  <NewsCard key={i.key} d={i.d} now={now} />
                ) : (
                  <CampaignNewsCard key={i.key} ev={i.ev!} now={now} />
                ),
              )}
          </div>
        )}
        {shown < list.length && (
          <button className="hud-btn news-more" onClick={() => setShown((n) => n + PAGE)}>
            Load older news
          </button>
        )}
      </div>
    </div>
  );
}
