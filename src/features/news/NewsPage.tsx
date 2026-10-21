import { useState } from "react";
import { useDispatches } from "../../api/queries";
import { fmtDateTime, parseMarkup } from "../../domain/format";
import { Loading, Panel } from "../../ui/primitives";
import "./news.css";

const PAGE = 25;

function Message({ text }: { text: string }) {
  return (
    <p className="dispatch-text">
      {parseMarkup(text).map((s, i) =>
        s.style == null ? <span key={i}>{s.text}</span> : <span key={i} className={`hl hl-${s.style}`}>{s.text}</span>,
      )}
    </p>
  );
}

export function NewsPage() {
  const { data, isLoading } = useDispatches();
  const [shown, setShown] = useState(PAGE);
  const sorted = [...(data ?? [])].sort((a, b) => b.id - a.id);
  return (
    <div className="page scroll starfield">
      <div className="page-inner" style={{ maxWidth: 760 }}>
        <h1 className="page-title">Dispatches</h1>
        {isLoading && <Loading what="dispatches" />}
        {sorted.slice(0, shown).map((d) => (
          <Panel key={d.id} title={<span className="num">{fmtDateTime(Date.parse(d.published))}</span>} right={<span className="num">#{d.id}</span>}>
            <div className="panel-body">
              <Message text={d.message} />
            </div>
          </Panel>
        ))}
        {shown < sorted.length && (
          <button className="hud-btn" onClick={() => setShown((n) => n + PAGE)}>
            Load older
          </button>
        )}
      </div>
    </div>
  );
}
