import { useDispatches } from "../../api/queries";
import { dispatchTitle } from "../../domain/format";
import { useUi } from "../../state";

/** Scrolling headline strip for the most recent dispatches. */
export function NewsTicker() {
  const { data } = useDispatches();
  const setView = useUi((s) => s.setView);
  if (!data?.length) return null;
  const items = [...data].sort((a, b) => b.id - a.id).slice(0, 8).map((d) => dispatchTitle(d.message));
  const strip = items.join("   ◆   ");
  return (
    <button className="ticker" onClick={() => setView("news")} title="Open news">
      <span className="ticker-track caps">
        <span>{strip}&nbsp;&nbsp;&nbsp;◆&nbsp;&nbsp;&nbsp;</span>
        <span>{strip}&nbsp;&nbsp;&nbsp;◆&nbsp;&nbsp;&nbsp;</span>
      </span>
    </button>
  );
}
