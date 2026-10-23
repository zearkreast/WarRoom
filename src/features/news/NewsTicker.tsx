import { useDispatches } from "../../api/queries";
import { dispatchTitle } from "../../domain/format";
import { useUi } from "../../state";

/** Scrolling headline strip for the most recent dispatches. */
export function NewsTicker() {
  const { data } = useDispatches();
  const setView = useUi((s) => s.setView);
  if (!data?.length) return <div className="ticker" />;
  const items = [...data]
    .sort((a, b) => b.id - a.id)
    .slice(0, 8)
    .map((d) => dispatchTitle(d.message));
  const run = (copy: number) =>
    items.map((t, i) => (
      <span key={`${copy}-${i}`} className="ticker-item" aria-hidden={copy > 0}>
        {t}
      </span>
    ));
  return (
    <button className="ticker" onClick={() => setView("news")} title="Open news">
      <span className="ticker-track caps">
        {run(0)}
        {run(1)}
      </span>
    </button>
  );
}
