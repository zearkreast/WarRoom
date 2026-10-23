import { useMemo, useState } from "react";
import { useDispatches } from "../../api/queries";
import { dispatchKind, type DispatchKind } from "../../domain/dispatch";
import { Hint } from "../../ui/Hint";
import { Empty, Loading, useNow } from "../../ui/primitives";
import { KIND_LABEL, NewsCard } from "./NewsCard";

const PAGE = 20;
type Filter = "all" | DispatchKind;

export function NewsPage() {
  const { data, isLoading } = useDispatches();
  const now = useNow(60_000);
  const [shown, setShown] = useState(PAGE);
  const [filter, setFilter] = useState<Filter>("all");
  const sorted = useMemo(() => [...(data ?? [])].sort((a, b) => b.id - a.id), [data]);
  const kinds = useMemo(() => {
    const m = new Map<DispatchKind, number>();
    for (const d of sorted) m.set(dispatchKind(d.message), (m.get(dispatchKind(d.message)) ?? 0) + 1);
    return m;
  }, [sorted]);
  const list = filter === "all" ? sorted : sorted.filter((d) => dispatchKind(d.message) === filter);

  return (
    <div className="page scroll starfield">
      <div className="page-inner news-page">
        <div className="page-head">
          <div>
            <h1 className="page-title">Strohmann News</h1>
            <p className="page-sub">Dispatches from Super Earth, newest first. Planet names open their options.</p>
          </div>
        </div>
        <Hint id="news">
          Planet names in a dispatch are links: click one to pin it, find it on the map or open its record.
        </Hint>
        <div className="filter-row" role="toolbar" aria-label="Filter dispatches">
          <button className={`filter-chip ${filter === "all" ? "on" : ""}`} onClick={() => setFilter("all")}>
            All <span className="count">{sorted.length}</span>
          </button>
          {[...kinds].map(([k, n]) => (
            <button key={k} className={`filter-chip ${filter === k ? "on" : ""}`} onClick={() => setFilter(k)}>
              {KIND_LABEL[k]} <span className="count">{n}</span>
            </button>
          ))}
        </div>
        {isLoading ? (
          <Loading what="dispatches" rows={4} />
        ) : list.length === 0 ? (
          <Empty>No dispatches received yet.</Empty>
        ) : (
          <div className="news-grid">
            {list.slice(0, shown).map((d) => (
              <NewsCard key={d.id} d={d} now={now} />
            ))}
          </div>
        )}
        {shown < list.length && (
          <button className="hud-btn news-more" onClick={() => setShown((n) => n + PAGE)}>
            Load older dispatches
          </button>
        )}
      </div>
    </div>
  );
}
