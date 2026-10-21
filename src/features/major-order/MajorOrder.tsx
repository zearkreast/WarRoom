import { useMemo } from "react";
import { useAssignments, useCampaigns, usePlanets } from "../../api/queries";
import type { Assignment, Planet } from "../../api/types";
import { factionColor, factionLabel } from "../../domain/factions";
import { fmtCompact, fmtPct, fmtRate } from "../../domain/format";
import { describeAssignment, rewardLabel, type TaskView } from "../../domain/majorOrder";
import { liberationPercent, projectedAt, trackedPercent } from "../../domain/progress";
import { usePlanetRate } from "../planets/useFronts";
import { useUi } from "../../state";
import { Countdown, Loading, Panel, ProgressBar } from "../../ui/primitives";
import "./majorOrder.css";

/** Campaign planets refresh every minute, the full list every three, so prefer the fresher copy. */
function useFreshPlanets(): Planet[] | undefined {
  const planets = usePlanets().data;
  const campaigns = useCampaigns().data;
  return useMemo(() => {
    if (!planets) return campaigns?.map((c) => c.planet);
    const fresh = new Map((campaigns ?? []).map((c) => [c.planet.index, c.planet]));
    return planets.map((p) => fresh.get(p.index) ?? p);
  }, [planets, campaigns]);
}

function PlanetTask({ t, expires }: { t: TaskView; expires: number }) {
  const p = t.planet!;
  const rate = usePlanetRate(p.index);
  const select = useUi((s) => s.select);
  const setView = useUi((s) => s.setView);
  const owned = p.currentOwner === "Humans" && !p.event;
  const pct = owned ? 100 : trackedPercent(p);
  const end = owned ? null : projectedAt(liberationPercent(p), rate, Date.now(), expires);
  return (
    <li className={`mo-task ${t.done || owned ? "done" : ""}`}>
      <div className="mo-task-head">
        <span className="mo-check">{t.done || owned ? "■" : "□"}</span>
        <span>
          {t.label}{" "}
          <button className="link caps" style={{ color: factionColor(p.currentOwner) }} onClick={() => { select(p.index); setView("map"); }}>
            {p.name}
          </button>
        </span>
        <span className="num mo-pct">{fmtPct(pct, 1)}</span>
      </div>
      <ProgressBar value={pct} color="var(--f-humans)" rest={factionColor(p.currentOwner)} marker={end} />
      {!owned && (
        <div className="mo-task-foot num muted">
          <span>{fmtRate(rate)}</span>
          {end != null && (
            <span style={{ color: end >= 100 ? "var(--good)" : "var(--bad)" }}>End result: {fmtPct(end, 1)}</span>
          )}
        </div>
      )}
    </li>
  );
}

function CountTask({ t }: { t: TaskView }) {
  const pct = t.goal ? (t.current / t.goal) * 100 : 0;
  return (
    <li className={`mo-task ${t.done ? "done" : ""}`}>
      <div className="mo-task-head">
        <span className="mo-check">{t.done ? "■" : "□"}</span>
        <span>
          {t.label}
          {t.faction && <span style={{ color: factionColor(t.faction) }}> {factionLabel(t.faction)}</span>}
        </span>
        <span className="num mo-pct">
          {fmtCompact(t.current)} / {fmtCompact(t.goal)}
        </span>
      </div>
      <ProgressBar value={pct} color="var(--accent)" />
    </li>
  );
}

function OrderCard({ a, planets, full }: { a: Assignment; planets: Planet[] | undefined; full?: boolean }) {
  const tasks = describeAssignment(a, planets);
  const expires = Date.parse(a.expiration);
  const reward = a.rewards[0] ?? a.reward;
  return (
    <Panel
      className="mo"
      title={a.title ?? "Major Order"}
      right={
        <span className="mo-ends">
          Ends in <b><Countdown to={expires} /></b>
        </span>
      }
    >
      <div className="panel-body mo-body">
        {a.briefing && <p className="mo-briefing">{a.briefing}</p>}
        {full && a.description && <p className="muted">{a.description}</p>}
        <ul className="mo-tasks">
          {tasks.map((t, i) => (t.kind === "planet" ? <PlanetTask key={i} t={t} expires={expires} /> : <CountTask key={i} t={t} />))}
        </ul>
        {reward && (
          <div className="mo-reward">
            <span className="stat-label">Reward</span>
            <span className="caps">{rewardLabel(reward.type)}</span>
            <b className="num">{reward.amount}</b>
          </div>
        )}
      </div>
    </Panel>
  );
}

export function MajorOrderList({ full }: { full?: boolean }) {
  const { data, isLoading } = useAssignments();
  const planets = useFreshPlanets();
  if (isLoading) return <Loading what="orders" />;
  if (!data?.length) return <div className="empty caps">No active Major Order. Await further instructions.</div>;
  return (
    <>
      {data.map((a) => (
        <OrderCard key={a.id} a={a} planets={planets} full={full} />
      ))}
    </>
  );
}

export function OrdersPage() {
  return (
    <div className="page scroll starfield">
      <div className="page-inner" style={{ maxWidth: 760 }}>
        <h1 className="page-title">High Command Orders</h1>
        <MajorOrderList full />
      </div>
    </div>
  );
}
