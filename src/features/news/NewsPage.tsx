import { MailOpen, Search, X } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { useToasts } from "../../alerts/notify";
import type { Dispatch } from "../../api/types";
import { dispatchTitle, fmtAgo } from "../../domain/format";
import { groupByDay, matchesQuery } from "../../domain/newsFeed";
import { useSettings } from "../../state";
import { SuperEarthSeal } from "../../ui/emblems";
import { Hint } from "../../ui/Hint";
import { CtaButton, Empty, Loading, useNow } from "../../ui/primitives";
import { KIND_LABEL } from "./NewsCard";
import { CampaignItem, DispatchItem } from "./NewsItems";
import { useNewsItems, type NewsItem, type NewsKind } from "./useNewsItems";

const PAGE = 20;
type Filter = "all" | NewsKind;

/** The newest dispatch, to read again as the game shows it in case its popup was missed. */
function LastMessage({ d, now }: { d: Dispatch; now: number }) {
  const show = useToasts((s) => s.show);
  const title = dispatchTitle(d.message);
  return (
    <section className="news-last" aria-label="Last message from Super Earth">
      <span className="news-last-seal">
        <SuperEarthSeal size={40} />
      </span>
      <div className="news-last-text">
        <span className="caps">Last message from Super Earth · {fmtAgo(Date.parse(d.published), now)}</span>
        <b>{title}</b>
      </div>
      <CtaButton
        icon={MailOpen}
        onClick={() => show({ kind: "dispatch", title: "Message from Super Earth", body: title, message: d.message })}
      >
        Read message
      </CtaButton>
    </section>
  );
}

function FeedItem({ i, now, unread, featured }: { i: NewsItem; now: number; unread: boolean; featured?: boolean }) {
  return i.d ? (
    <DispatchItem d={i.d} now={now} unread={unread} featured={Boolean(featured)} />
  ) : (
    <CampaignItem thread={i.t!} now={now} unread={unread} featured={Boolean(featured)} />
  );
}

export function NewsPage() {
  const { items, isLoading, dispatches } = useNewsItems();
  const now = useNow(60_000);
  const [shown, setShown] = useState(PAGE);
  const [filter, setFilter] = useState<Filter>("all");
  const [query, setQuery] = useState("");
  const markNewsSeen = useSettings((s) => s.markNewsSeen);
  // What counted as new when the page opened; it stays marked for this visit while the store moves on.
  const [seenAt] = useState(() => useSettings.getState().newsSeenAt);
  const newest = items[0]?.at;
  useEffect(() => {
    if (newest) markNewsSeen(newest);
  }, [newest, markNewsSeen]);

  const kinds = useMemo(() => {
    const m = new Map<NewsKind, number>();
    for (const i of items) m.set(i.kind, (m.get(i.kind) ?? 0) + 1);
    return m;
  }, [items]);
  const list = useMemo(
    () => items.filter((i) => (filter === "all" || i.kind === filter) && matchesQuery(i.text, query)),
    [items, filter, query],
  );
  const isUnread = (i: NewsItem) => seenAt > 0 && i.at > seenAt;
  const unread = items.filter(isUnread).length;
  const last = useMemo(
    () => (dispatches ?? []).reduce<Dispatch | null>((a, d) => (!a || d.id > a.id ? d : a), null),
    [dispatches],
  );
  const [featured, ...rest] = list.slice(0, shown);
  const days = groupByDay(rest, now);

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
        {last && <LastMessage d={last} now={now} />}
        <Hint id="news">
          Planet names in a dispatch are links: click one to pin it, find it on the map or open its record.
        </Hint>
        <div className="news-tools">
          <label className="news-search">
            <Search size={15} aria-hidden />
            <input
              type="search"
              placeholder="Search the news"
              value={query}
              onChange={(e) => {
                setQuery(e.target.value);
                setShown(PAGE);
              }}
              aria-label="Search the news"
            />
            {query && (
              <button className="news-search-x" aria-label="Clear search" onClick={() => setQuery("")}>
                <X size={15} />
              </button>
            )}
          </label>
          <div className="filter-row" role="toolbar" aria-label="Filter news">
            <button className={`filter-chip ${filter === "all" ? "on" : ""}`} onClick={() => setFilter("all")}>
              All <span className="count">{items.length}</span>
            </button>
            {[...kinds].map(([k, n]) => (
              <button
                key={k}
                className={`filter-chip ${filter === k ? "on" : ""}`}
                onClick={() => {
                  setFilter(k);
                  setShown(PAGE);
                }}
              >
                {k === "campaign" ? "Campaign" : KIND_LABEL[k]} <span className="count">{n}</span>
              </button>
            ))}
          </div>
        </div>
        {unread > 0 && <p className="news-unread caps">{unread} new since your last visit</p>}
        {isLoading ? (
          <Loading what="dispatches" rows={4} />
        ) : !featured ? (
          <Empty>{items.length ? "No news matches that search." : "No dispatches received yet."}</Empty>
        ) : (
          <>
            <FeedItem i={featured} now={now} unread={isUnread(featured)} featured />
            {days.map((g) => (
              <section key={g.key} className="news-day" aria-label={g.label}>
                <h2 className="news-day-head caps">
                  {g.label} <span className="num">{g.items.length}</span>
                </h2>
                <div className="news-rail">
                  {g.items.map((i) => (
                    <FeedItem key={i.key} i={i} now={now} unread={isUnread(i)} />
                  ))}
                </div>
              </section>
            ))}
          </>
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
