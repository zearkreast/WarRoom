import { Award, CheckSquare, Square, Target } from "lucide-react";
import { useAssignments, useEpisodes, useFreshPlanets } from "../../api/queries";
import type { Assignment, Planet } from "../../api/types";
import { factionColor, factionLabel } from "../../domain/factions";
import { fmtCompact, fmtPct } from "../../domain/format";
import { describeAssignment, orderName, rewardLabel, type TaskView } from "../../domain/majorOrder";
import { liberationPercent, projectedAt, projectedCompletion, trackedPercent } from "../../domain/progress";
import { useUi } from "../../state";
import { Countdown, Empty, Loading, Meter, Panel } from "../../ui/primitives";
import { usePlanetRate } from "../planets/useFronts";
import "./majorOrder.css";

function PlanetTask({ t, expires }: { t: TaskView; expires: number }) {
  const p = t.planet!;
  const rate = usePlanetRate(p.index);
  const openOptions = useUi((s) => s.openOptions);
  const owned = p.currentOwner === "Humans" && !p.event;
  const done = t.done || owned;
  const pct = owned ? 100 : trackedPercent(p);
  const end = owned ? null : projectedAt(liberationPercent(p), rate, Date.now(), expires);
  const completeAt = owned ? null : projectedCompletion(pct, rate, Date.now());
  const Check = done ? CheckSquare : Square;
  return (
    <li className={`mo-task ${done ? "done" : ""}`}>
      <div className="mo-task-head">
        <Check size={16} className="mo-check" />
        <span>
          {t.label}{" "}
          <button
            className="link"
            style={{ color: factionColor(p.currentOwner) }}
            onClick={(e) => openOptions({ planet: p.index, x: e.clientX, y: e.clientY })}
          >
            {p.name}
          </button>
        </span>
        <span className="num mo-pct">{fmtPct(pct, 1)}</span>
      </div>
      <Meter value={pct} color="var(--f-humans)" rest={factionColor(p.currentOwner)} marker={end} size="l" chevron />
      {!owned && (
        <div className="mo-task-foot num">
          {end != null ? (
            <span style={{ color: end >= 100 ? "var(--good)" : "var(--bad)" }}>End result: {fmtPct(end, 2)}</span>
          ) : (
            <span className="muted">End result: measuring…</span>
          )}
          {completeAt && completeAt < expires && (
            <span>
              Complete in <Countdown to={completeAt} />
            </span>
          )}
        </div>
      )}
    </li>
  );
}

function CountTask({ t }: { t: TaskView }) {
  const pct = t.goal ? (t.current / t.goal) * 100 : 0;
  const Check = t.done ? CheckSquare : Square;
  return (
    <li className={`mo-task ${t.done ? "done" : ""}`}>
      <div className="mo-task-head">
        <Check size={16} className="mo-check" />
        <span>
          {t.label}
          {t.faction && <span style={{ color: factionColor(t.faction) }}> {factionLabel(t.faction)}</span>}
        </span>
        <span className="num mo-pct">
          {fmtCompact(t.current)} / {fmtCompact(t.goal)}
        </span>
      </div>
      <Meter value={pct} color="var(--cta)" size="l" />
    </li>
  );
}

export function OrderCard({
  a,
  planets,
  name,
  compact,
}: {
  a: Assignment;
  planets: Planet[] | undefined;
  name?: string;
  compact?: boolean;
}) {
  const tasks = describeAssignment(a, planets);
  const expires = Date.parse(a.expiration);
  const reward = a.rewards[0] ?? a.reward;
  return (
    <Panel
      className="mo"
      icon={Target}
      title={
        <>
          {a.title ?? "Major Order"}
          {name && <span className="mo-name">{name}</span>}
        </>
      }
      right={
        <span className="mo-ends">
          Ends in{" "}
          <b>
            <Countdown to={expires} />
          </b>
        </span>
      }
    >
      {a.briefing && <p className="mo-briefing">{a.briefing}</p>}
      {!compact && a.description && <p className="mo-desc">{a.description}</p>}
      <div className="mo-sub caps">Order overview</div>
      <ul className="mo-tasks">
        {tasks.map((t, i) =>
          t.kind === "planet" ? <PlanetTask key={i} t={t} expires={expires} /> : <CountTask key={i} t={t} />,
        )}
      </ul>
      {reward && (
        <div className="mo-reward">
          <Award size={20} />
          <span className="caps">
            Reward <b>{rewardLabel(reward.type)}</b>
          </span>
          <b className="num mo-amount">{reward.amount}</b>
        </div>
      )}
    </Panel>
  );
}

export function MajorOrderList({ compact }: { compact?: boolean }) {
  const { data, isLoading } = useAssignments();
  const planets = useFreshPlanets().data;
  const episodes = useEpisodes().data;
  if (isLoading) return <Loading what="orders" rows={2} />;
  if (!data?.length) return <Empty>No active Major Order. Await further instructions.</Empty>;
  return (
    <>
      {data.map((a) => (
        <OrderCard key={a.id} a={a} planets={planets} name={data.length === 1 ? orderName(episodes) : undefined} compact={compact} />
      ))}
    </>
  );
}
