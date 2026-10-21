import { useHistory } from "../../history/useHistory";
import { useUi } from "../../state";
import { Panel } from "../../ui/primitives";
import { ChangeRow } from "./PlanetHistory";

export function HistoryPage() {
  const { changes, ready } = useHistory();
  const select = useUi((s) => s.select);
  const setView = useUi((s) => s.setView);
  return (
    <div className="page scroll starfield">
      <div className="page-inner" style={{ maxWidth: 760 }}>
        <h1 className="page-title">War History</h1>
        <p className="muted" style={{ margin: 0, fontSize: 13 }}>
          Every planet that changes hands is recorded by the WarRoom collector and shared with every user. Pick a
          planet on the map and open its History tab to see its progress chart.
        </p>
        <Panel title="Recent captures" right={<span className="num">{changes.length}</span>}>
          <div className="panel-body">
            {!ready ? (
              <div className="muted">Opening history…</div>
            ) : changes.length === 0 ? (
              <div className="muted">No planets have changed hands since tracking began.</div>
            ) : (
              <ul className="change-list">
                {changes.map((c, i) => (
                  <ChangeRow
                    key={i}
                    c={c}
                    showPlanet
                    onClick={() => {
                      select(c.planet);
                      setView("map");
                    }}
                  />
                ))}
              </ul>
            )}
          </div>
        </Panel>
      </div>
    </div>
  );
}
