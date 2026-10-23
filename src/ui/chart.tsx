import { fmtDateTime, fmtTime } from "../domain/format";

// Shared Recharts styling: hairline solid grid and axes one step off the surface,
// text in label ink (never series colours), a crosshair tooltip on every chart.

export const HOUR = 3_600_000;

export const axisProps = {
  stroke: "var(--c-grid)",
  tick: { fill: "var(--label)", fontSize: 10 },
  tickLine: false,
} as const;

export const gridProps = { stroke: "var(--c-grid)", strokeDasharray: "", vertical: false } as const;

export const tooltipProps = {
  contentStyle: {
    background: "var(--surface-1)",
    border: "1px solid var(--line-strong)",
    borderRadius: 0,
    fontSize: 12,
    boxShadow: "var(--shadow-2)",
  },
  labelStyle: { color: "var(--text-dim)", marginBottom: 4 },
  itemStyle: { color: "var(--text-hi)", padding: 0 },
  cursor: { stroke: "var(--text-dim)", strokeWidth: 1 },
  labelFormatter: (t: unknown) => fmtDateTime(Number(t)),
} as const;

export const fmtHourTick = (t: number) => fmtTime(t);

/** Whole-hour ticks every `stepHours` across [from, to]. */
export function hourTicks(from: number, to: number, stepHours = 4): number[] {
  const step = stepHours * HOUR;
  const out: number[] = [];
  for (let t = Math.ceil(from / step) * step; t <= to; t += step) out.push(t);
  return out;
}
