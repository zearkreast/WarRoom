import { Award, CheckSquare, Square, Target } from "lucide-react";
import type { CSSProperties } from "react";
import { useAssignments, useEpisodes, useFreshPlanets } from "../../api/queries";
import type { Assignment, Planet } from "../../api/types";
import { factionColor, factionLabel } from "../../domain/factions";
import { fmtCompact, fmtPct } from "../../domain/format";
import {
  describeAssignment,
  orderName,
  orderVerdict,
  rewardLabel,
  type TaskOutlook,
  type TaskView,
} from "../../domain/majorOrder";
import { trackedPercent } from "../../domain/progress";
import { useUi } from "../../state";
import { Countdown, Empty, Loading, Panel } from "../../ui/primitives";
import { OrderBar, OutlookTag, useCountTask, useCountUp } from "./OrderBar";
import { useOutlooks } from "./useOutlooks";
import "./majorOrder.css";

const signed = (n: number, text: string) => `${n >= 0 ? "+" : "−"}${text}`;

function PlanetTask({ t, o }: { t: TaskView; o: TaskOutlook }) {
  const p = t.planet!;
  const openOptions = useUi((s) => s.openOptions);
  const done = o.outlook === "done";
  const pct = done ? 100 : trackedPercent(p);
  const shown = useCountUp(pct);
  const Check = done ? CheckSquare : Square;
  return (
    <li className={`mo-task ${done ? "done" : ""}`} style={{ "--k": "var(--f-humans)" } as CSSProperties}>
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
        <span className="num mo-pct">{fmtPct(shown, 1)}</span>
      </div>
      <OrderBar
        shown={shown}
        value={pct}
        label={`${t.label} ${p.name}`}
        valueText={`${fmtPct(pct, 1)}${done ? ", complete" : ""}`}
        tone="var(--f-humans)"
        rest={factionColor(p.currentOwner)}
        marker={done ? null : o.end}
        done={done}
      />
      <div className="mo-task-foot num">
        <span>
          {o.rate != null && !done && (
            <>
              <b>{signed(o.rate, `${fmtPct(Math.abs(o.rate), 2)}/h`)}</b>
              {o.end != null && " · "}
            </>
          )}
          {o.end != null && !done && <>End result {fmtPct(o.end, 2)}</>}
        </span>
        <OutlookTag o={o} />
      </div>
    </li>
  );
}

function CountTask({ t, o }: { t: TaskView; o: TaskOutlook }) {
  const { shown, pct, final, valueText } = useCountTask(t);
  const tone = t.faction ? factionColor(t.faction) : "var(--cta)";
  const Check = t.done ? CheckSquare : Square;
  const perHour = o.rate != null ? (o.rate / 100) * t.goal : null;
  return (
    <li className={`mo-task ${t.done ? "done" : ""}`} style={{ "--k": tone } as CSSProperties}>
      <div className="mo-task-head">
        <Check size={16} className="mo-check" />
        <span>
          {t.label}
          {t.faction && <span style={{ color: tone }}> {factionLabel(t.faction)}</span>}
        </span>
        <span className="num mo-pct">{fmtPct(pct, 1)}</span>
      </div>
      <OrderBar
        shown={pct}
        value={final}
        label={`${t.label}${t.faction ? ` ${factionLabel(t.faction)}` : ""}`}
        valueText={valueText}
        tone={tone}
        marker={t.done ? null : o.end}
        done={t.done}
      />
      <div className="mo-task-foot num">
        <span>
          <b>{fmtCompact(Math.round(shown))}</b> / {fmtCompact(t.goal)}
          {perHour != null && !t.done && <> · {signed(perHour, `${fmtCompact(Math.abs(perHour))}/h`)}</>}
        </span>
        <OutlookTag o={o} />
      </div>
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
  const outlooks = useOutlooks(a, tasks);
  const verdict = orderVerdict(tasks, outlooks);
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
      {verdict && (
        <p className="mo-verdict-line" style={{ color: verdict.tone }}>
          {verdict.text}
        </p>
      )}
      <div className="mo-sub caps">Order overview</div>
      <ul className="mo-tasks">
        {tasks.map((t, i) =>
          t.kind === "planet" ? <PlanetTask key={i} t={t} o={outlooks[i]} /> : <CountTask key={i} t={t} o={outlooks[i]} />,
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
