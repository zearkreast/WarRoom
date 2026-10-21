import { factionColor } from "../../domain/factions";
import { fmtInt, fmtPct, fmtRate } from "../../domain/format";
import { useUi } from "../../state";
import { Countdown, ProgressBar } from "../../ui/primitives";
import type { Front } from "./useFronts";
import "./planets.css";

function Outlook({ f }: { f: Front }) {
  // Untouched planets are never sampled, so they never get a rate.
  if (f.rate == null && f.kind === "liberation" && f.percent === 0)
    return <span className="muted">Holding for reinforcement</span>;
  if (f.rate == null) return <span className="muted">Measuring rate…</span>;
  if (f.kind === "defense") {
    if (f.victoryAt && f.failAt && f.victoryAt < f.failAt)
      return (
        <span style={{ color: "var(--good)" }}>
          Defended in <Countdown to={f.victoryAt} />
        </span>
      );
    return (
      <span style={{ color: "var(--bad)" }}>
        Falls in <Countdown to={f.failAt ?? 0} />
      </span>
    );
  }
  if (f.victoryAt)
    return (
      <span style={{ color: "var(--good)" }}>
        Victory in <Countdown to={f.victoryAt} />
      </span>
    );
  if (f.failAt)
    return (
      <span style={{ color: "var(--bad)" }}>
        Withdrawn in <Countdown to={f.failAt} />
      </span>
    );
  return <span className="muted">Holding for reinforcement</span>;
}

export function PlanetCard({ f, compact }: { f: Front; compact?: boolean }) {
  const select = useUi((s) => s.select);
  const setView = useUi((s) => s.setView);
  const enemy = factionColor(f.enemy);
  const stalled = f.rate != null && f.rate <= 0;
  return (
    <button
      className={`planet-card panel ${compact ? "compact" : ""}`}
      style={{ borderTopColor: enemy }}
      onClick={() => {
        select(f.planet.index);
        setView("map");
      }}
    >
      <div className="pc-head">
        <span className="dot" style={{ background: enemy, boxShadow: `0 0 8px ${enemy}` }} />
        <div className="pc-name">
          <div className="caps">{f.planet.name}</div>
          <div className="pc-sector caps">{f.planet.sector} sector</div>
        </div>
        <div className="pc-tags">
          {f.inMajorOrder && <span className="chip" style={{ color: "var(--accent)" }}>MO</span>}
          <span className="chip" style={{ color: f.kind === "defense" ? "var(--f-terminids)" : "var(--f-humans)" }}>
            {f.kind}
          </span>
        </div>
      </div>
      <ProgressBar
        value={f.percent}
        color="var(--f-humans)"
        rest={enemy}
        tall={!compact}
        striped={stalled}
        marker={f.projected}
      />
      <div className="pc-stats">
        <span className="num pc-pct">{fmtPct(f.percent, compact ? 1 : 3)}</span>
        <span className="num" style={{ color: f.rate == null ? undefined : f.rate > 0 ? "var(--good)" : "var(--bad)" }}>
          {fmtRate(f.rate)}
        </span>
        <span className="num muted">{fmtInt(f.players)} divers</span>
      </div>
      {!compact && (
        <div className="pc-outlook caps">
          <Outlook f={f} />
        </div>
      )}
    </button>
  );
}
