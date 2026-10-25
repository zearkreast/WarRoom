import { ChevronRight } from "lucide-react";
import type { CSSProperties } from "react";
import { useAssignments, useFreshPlanets } from "../../api/queries";
import type { Assignment, Planet } from "../../api/types";
import { factionColor, factionLabel } from "../../domain/factions";
import { fmtCompact, fmtPct } from "../../domain/format";
import { describeAssignment, orderVerdict, OUTLOOK, type TaskOutlook, type TaskView } from "../../domain/majorOrder";
import { trackedPercent } from "../../domain/progress";
import { useSettings, useUi } from "../../state";
import { FactionEmblem } from "../../ui/emblems";
import { Countdown, Panel } from "../../ui/primitives";
import { OrderBar, OutlookTag, useCountTask, useCountUp } from "../major-order/OrderBar";
import { useOutlooks } from "../major-order/useOutlooks";
import "../major-order/majorOrder.css";

function CountRow({ t, o }: { t: TaskView; o: TaskOutlook }) {
  const { shown, pct, final, valueText } = useCountTask(t);
  const tone = t.faction ? factionColor(t.faction) : "var(--cta)";
  const name = `${t.label}${t.faction ? ` ${factionLabel(t.faction)}` : ""}`;
  return (
    <li className={`ms-count ${t.done ? "done" : ""}`} style={{ "--k": tone } as CSSProperties}>
      <div className="ms-count-head caps">
        {t.faction && <FactionEmblem faction={t.faction} size={14} />}
        <span className="ms-count-name">
          {t.label}
          {t.faction && <b> {factionLabel(t.faction)}</b>}
        </span>
        <span className="ms-count-pct num">{fmtPct(pct, 1)}</span>
      </div>
      <OrderBar
        shown={pct}
        value={final}
        label={name}
        valueText={valueText}
        tone={tone}
        marker={t.done ? null : o.end}
        done={t.done}
      />
      <div className="ms-count-foot caps">
        <span className="num">
          <b>{fmtCompact(Math.round(shown))}</b> / {fmtCompact(t.goal)}
        </span>
        <OutlookTag o={o} />
      </div>
    </li>
  );
}

function TargetItem({ planet, o }: { planet: Planet; o: TaskOutlook }) {
  const openOptions = useUi((s) => s.openOptions);
  const done = o.outlook === "done";
  const percent = done ? 100 : trackedPercent(planet);
  const shown = useCountUp(percent);
  const v = OUTLOOK[o.outlook];
  return (
    <li style={{ "--k": factionColor(planet.currentOwner) } as CSSProperties}>
      <button className="ms-name" onClick={(e) => openOptions({ planet: planet.index, x: e.clientX, y: e.clientY })}>
        <FactionEmblem faction={planet.currentOwner} size={16} />
        {planet.name}
      </button>
      <OrderBar
        slim
        shown={shown}
        value={percent}
        label={planet.name}
        valueText={`${fmtPct(percent, 1)}${o.end != null && !done ? `, ${fmtPct(o.end, 1)} by the deadline` : ""}`}
        tone="var(--f-humans)"
        rest={factionColor(planet.currentOwner)}
        marker={done ? null : o.end}
        done={done}
      />
      <span className="ms-path num">
        {fmtPct(shown, 1)}
        {o.end != null && !done && <> → {fmtPct(o.end, 1)}</>}
      </span>
      <span className="ms-chip caps" style={{ "--tone": v.tone } as CSSProperties}>
        {v.label}
      </span>
    </li>
  );
}

function OrderStrip({ a, planets }: { a: Assignment; planets: Planet[] | undefined }) {
  const open = useSettings((s) => s.moStripOpen);
  const setSetting = useSettings((s) => s.set);
  const tasks = describeAssignment(a, planets);
  const outlooks = useOutlooks(a, tasks);
  const verdict = orderVerdict(tasks, outlooks);
  const rows = tasks.flatMap((t, i) => (t.kind === "planet" && t.planet ? [{ planet: t.planet, o: outlooks[i] }] : []));
  const counts = tasks.flatMap((t, i) => (t.kind === "count" ? [{ t, o: outlooks[i] }] : []));
  return (
    <Panel
      className={`mo-strip ${open ? "" : "folded"}`}
      title={
        <button className="ms-toggle" aria-expanded={open} onClick={() => setSetting("moStripOpen", !open)}>
          <ChevronRight size={14} className="ms-chevron" aria-hidden />
          {a.title ?? "Major Order"}
        </button>
      }
      right={
        <span className="mo-ends">
          Ends in{" "}
          <b>
            <Countdown to={Date.parse(a.expiration)} />
          </b>
        </span>
      }
    >
      {open && verdict && (
        <p className="ms-verdict" style={{ color: verdict.tone }}>
          {verdict.text}
        </p>
      )}
      {open && rows.length > 0 && (
        <ul className="ms-rows">
          {rows.map((r) => (
            <TargetItem key={r.planet.index} planet={r.planet} o={r.o} />
          ))}
        </ul>
      )}
      {open && counts.length > 0 && (
        <ul className="ms-counts">
          {counts.map((c, i) => (
            <CountRow key={i} t={c.t} o={c.o} />
          ))}
        </ul>
      )}
    </Panel>
  );
}

/**
 * The Major Order answered in one glance: the deadline, and whether each objective makes it at the current pace.
 * Folded by default down to its header; the choice is remembered.
 */
export function MoStrip() {
  const orders = useAssignments().data;
  const planets = useFreshPlanets().data;
  if (!orders?.length) return null;
  return (
    <>
      {orders.map((a) => (
        <OrderStrip key={a.id} a={a} planets={planets} />
      ))}
    </>
  );
}
