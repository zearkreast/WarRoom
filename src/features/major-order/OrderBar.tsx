import { useEffect, useRef, useState, type CSSProperties } from "react";
import { fmtPct } from "../../domain/format";
import { taskPercent, type TaskView } from "../../domain/majorOrder";

// Keep in sync with --mo-grow in majorOrder.css.
const GROW_MS = 1100;

/** For screen readers: "21.2 million", not "21.2M". */
const spoken = new Intl.NumberFormat("en-US", { notation: "compact", compactDisplay: "long", maximumFractionDigits: 1 });

const reducedMotion = () => typeof window !== "undefined" && !!window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;

/** Counts up to `to` from zero on first paint and glides to each new value. */
export function useCountUp(to: number): number {
  const [shown, setShown] = useState(() => (reducedMotion() ? to : 0));
  const at = useRef(shown);
  useEffect(() => {
    const from = at.current;
    if (reducedMotion() || from === to) {
      at.current = to;
      setShown(to);
      return;
    }
    const start = performance.now();
    let raf = requestAnimationFrame(function tick(now) {
      const k = Math.min(1, (now - start) / GROW_MS);
      at.current = k < 1 ? from + (to - from) * (1 - (1 - k) ** 3) : to;
      setShown(at.current);
      if (k < 1) raf = requestAnimationFrame(tick);
    });
    return () => cancelAnimationFrame(raf);
  }, [to]);
  return shown;
}

/** A count task as it counts up: the tally shown, its percent, and the final percent read out to screen readers. */
export function useCountTask(t: TaskView) {
  const shown = useCountUp(t.current);
  const final = taskPercent(t);
  return {
    shown,
    // The bar follows the count-up too, not a CSS transition, so it can't drift from the numbers.
    pct: taskPercent({ ...t, current: shown, done: shown >= t.goal }),
    final,
    valueText: `${spoken.format(t.current)} of ${spoken.format(t.goal)}, ${fmtPct(final, 1)}${t.done ? ", complete" : ""}`,
  };
}

/**
 * A Major Order progress bar: cells brightening toward a glowing head over a hatched remainder with
 * quarter marks. It draws `shown`, so a count-up can move it in step with the numbers beside it.
 */
export function OrderBar(props: {
  shown: number;
  /** Where the bar is headed; the value screen readers hear. */
  value: number;
  label: string;
  valueText?: string;
  /** Fill colour. */
  tone: string;
  /** Remainder colour, usually the enemy holding the planet; the fill colour when left out. */
  rest?: string;
  /** Where the bar is projected to stand at the deadline, drawn only when it falls short. */
  marker?: number | null;
  done?: boolean;
  slim?: boolean;
}) {
  const width = Math.max(0, Math.min(100, props.shown));
  const style = { "--k": props.tone, "--rest": props.rest ?? props.tone } as CSSProperties;
  return (
    <div
      className={`mo-bar${props.rest ? " versus" : ""}${props.slim ? " slim" : ""}`}
      style={style}
      role="progressbar"
      aria-label={props.label}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={Math.floor(props.value)}
      aria-valuetext={props.valueText}
    >
      <div className="mo-bar-fill" style={{ width: `${width}%` }}>
        <i className="mo-bar-sheen" />
      </div>
      {!props.done && <i className="mo-bar-head" style={{ left: `${width}%` }} />}
      {props.marker != null && props.marker > 0 && props.marker < 100 && (
        <i className="mo-bar-mark" style={{ left: `${props.marker}%` }} />
      )}
    </div>
  );
}
