import { useMemo } from "react";
import { useFreshPlanets } from "../../api/queries";
import type { Dispatch } from "../../api/types";
import { dispatchBody, dispatchKind, type DispatchKind } from "../../domain/dispatch";
import { dispatchTitle, fmtAgo, sestClock, sestDate } from "../../domain/format";
import { useUi } from "../../state";
import "./news.css";

export const KIND_LABEL: Record<DispatchKind, string> = {
  order: "Major order",
  victory: "Victory",
  defeat: "Defeat",
  alert: "Alert",
  advisory: "Advisory",
  news: "Dispatch",
};

/** Dispatch text with planet names turned into links to their options. */
function Body({ message }: { message: string }) {
  const planets = useFreshPlanets().data;
  const openOptions = useUi((s) => s.openOptions);
  const byName = useMemo(() => new Map((planets ?? []).map((p) => [p.name.toUpperCase(), p.index])), [planets]);
  return (
    <p className="nc-body">
      {dispatchBody(message).map((s, i) => {
        if (s.style == null) return <span key={i}>{s.text}</span>;
        const planet = byName.get(s.text.trim().toUpperCase());
        if (planet != null)
          return (
            <button
              key={i}
              className="link hl hl-planet"
              onClick={(e) => openOptions({ planet, x: e.clientX, y: e.clientY })}
            >
              {s.text}
            </button>
          );
        return (
          <span key={i} className={`hl hl-${s.style}`}>
            {s.text}
          </span>
        );
      })}
    </p>
  );
}

export function NewsCard({ d, now }: { d: Dispatch; now: number }) {
  const kind = dispatchKind(d.message);
  const at = Date.parse(d.published);
  return (
    <article className={`nc nc-${kind}`}>
      <div className="nc-banner" aria-hidden>
        <span className="nc-word">{KIND_LABEL[kind]}</span>
      </div>
      <div className="nc-tagrow">
        <span className="nc-age caps">{fmtAgo(at, now)}</span>
      </div>
      <h3 className="nc-title">{dispatchTitle(d.message)}</h3>
      <Body message={d.message} />
      <footer className="nc-foot num caps">
        <span>ID {d.id}</span>
        <span>
          SEST {sestClock(at).slice(0, 5)} · {sestDate(at)}
        </span>
      </footer>
    </article>
  );
}
