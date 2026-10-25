import { ChevronRight } from "lucide-react";
import type { CSSProperties } from "react";
import { useAssignments, useFreshPlanets } from "../../api/queries";
import type { Assignment, Planet } from "../../api/types";
import { factionColor, factionLabel } from "../../domain/factions";
import { fmtCompact, fmtPct } from "../../domain/format";
import { deadlineOutlook, describeAssignment, type DeadlineOutlook, type TaskView } from "../../domain/majorOrder";
import { useSettings, useUi } from "../../state";
import { FactionEmblem } from "../../ui/emblems";
import { Countdown, Panel, useNow } from "../../ui/primitives";
import { OrderBar, useCountTask, useCountUp } from "../major-order/OrderBar";
import type { Front } from "../planets/useFronts";
import "../major-order/majorOrder.css";

type Verdict = DeadlineOutlook | "no-front";

const VERDICT: Record<Verdict, { label: string; tone: string }> = {
  done: { label: "Secured", tone: "var(--good)" },
  "on-track": { label: "On track", tone: "var(--good)" },
  behind: { label: "Behind", tone: "var(--bad)" },
  measuring: { label: "Measuring", tone: "var(--label)" },
  "no-front": { label: "No front", tone: "var(--label)" },
};

function CountRow({ t }: { t: TaskView }) {
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
      <OrderBar shown={pct} value={final} label={name} valueText={valueText} tone={tone} done={t.done} />
      <div className="ms-count-foot caps">
        <span className="num">
          <b>{fmtCompact(Math.round(shown))}</b> / {fmtCompact(t.goal)}
        </span>
        {t.done ? (
          <span className="ms-chip caps" style={{ "--tone": "var(--good)" } as CSSProperties}>
            Complete
          </span>
        ) : (
          <span className="num">{fmtCompact(Math.max(0, Math.round(t.goal - shown)))} to go</span>
        )}
      </div>
    </li>
  );
}

interface TargetRow {
  planet: Planet;
  percent: number;
  end: number | null;
  verdict: Verdict;
}

function TargetItem({ r }: { r: TargetRow }) {
  const openOptions = useUi((s) => s.openOptions);
  const shown = useCountUp(r.percent);
  const v = VERDICT[r.verdict];
  const done = r.verdict === "done";
  return (
    <li style={{ "--k": factionColor(r.planet.currentOwner) } as CSSProperties}>
      <button className="ms-name" onClick={(e) => openOptions({ planet: r.planet.index, x: e.clientX, y: e.clientY })}>
        <FactionEmblem faction={r.planet.currentOwner} size={16} />
        {r.planet.name}
      </button>
      <OrderBar
        slim
        shown={shown}
        value={r.percent}
        label={r.planet.name}
        valueText={`${fmtPct(r.percent, 1)}${r.end != null && !done ? `, ${fmtPct(r.end, 1)} by the deadline` : ""}`}
        tone="var(--f-humans)"
        rest={factionColor(r.planet.currentOwner)}
        marker={done ? null : r.end}
        done={done}
      />
      <span className="ms-path num">
        {fmtPct(shown, 1)}
        {r.end != null && !done && <> → {fmtPct(r.end, 1)}</>}
      </span>
      <span className="ms-chip caps" style={{ "--tone": v.tone } as CSSProperties}>
        {v.label}
      </span>
    </li>
  );
}

function targets(a: Assignment, planets: Planet[] | undefined, fronts: Front[], now: number): TargetRow[] {
  const deadline = Date.parse(a.expiration);
  const byPlanet = new Map(fronts.map((f) => [f.planet.index, f]));
  return describeAssignment(a, planets).flatMap((t): TargetRow[] => {
    if (t.kind !== "planet" || !t.planet) return [];
    const p = t.planet;
    const front = byPlanet.get(p.index);
    if (t.done || (!front && p.currentOwner === "Humans" && !p.event))
      return [{ planet: p, percent: 100, end: 100, verdict: "done" }];
    if (!front) return [{ planet: p, percent: 0, end: null, verdict: "no-front" }];
    const { outlook, end } = deadlineOutlook(front.percent, front.rate, now, deadline);
    return [{ planet: p, percent: front.percent, end, verdict: outlook }];
  });
}

/**
 * The Major Order answered in one glance: the deadline, and whether each target makes it at the current pace.
 * Folded by default down to its header; the choice is remembered.
 */
export function MoStrip({ fronts }: { fronts: Front[] }) {
  const orders = useAssignments().data;
  const open = useSettings((s) => s.moStripOpen);
  const setSetting = useSettings((s) => s.set);
  const planets = useFreshPlanets().data;
  const now = useNow(60_000);
  if (!orders?.length) return null;

  return (
    <>
      {orders.map((a) => {
        const rows = targets(a, planets, fronts, now);
        const counts = describeAssignment(a, planets).filter((t) => t.kind === "count");
        const behind = rows.filter((r) => r.verdict === "behind");
        const measuring = rows.some((r) => r.verdict === "measuring");
        const verdict =
          behind.length > 0
            ? { tone: "var(--bad)", text: `Needs Helldivers: ${behind.map((r) => r.planet.name).join(", ")}` }
            : rows.length > 0 && rows.every((r) => r.verdict === "done" || r.verdict === "on-track")
              ? { tone: "var(--good)", text: "Every target makes the deadline at the current pace" }
              : measuring
                ? { tone: "var(--label)", text: "Measuring progress rates…" }
                : null;
        return (
          <Panel
            key={a.id}
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
                  <TargetItem key={r.planet.index} r={r} />
                ))}
              </ul>
            )}
            {open && counts.length > 0 && (
              <ul className="ms-counts">
                {counts.map((t, i) => (
                  <CountRow key={i} t={t} />
                ))}
              </ul>
            )}
          </Panel>
        );
      })}
    </>
  );
}
