import { useMemo } from "react";
import { useFreshPlanets } from "../../api/queries";
import { dispatchBody, type DispatchKind } from "../../domain/dispatch";
import { parseMarkup } from "../../domain/format";
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

/**
 * Game text with its markup styled and planet names turned into links to their options. Dispatches
 * lose their leading headline (cards show it as the title); `whole` keeps it, for briefings.
 */
export function MarkupText({
  message,
  className = "ni-body",
  whole = false,
}: {
  message: string;
  className?: string;
  whole?: boolean;
}) {
  const planets = useFreshPlanets().data;
  const openOptions = useUi((s) => s.openOptions);
  const byName = useMemo(() => new Map((planets ?? []).map((p) => [p.name.toUpperCase(), p.index])), [planets]);
  return (
    <p className={className}>
      {(whole ? parseMarkup(message.trim()) : dispatchBody(message)).map((s, i) => {
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
