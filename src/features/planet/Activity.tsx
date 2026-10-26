import { Users } from "lucide-react";
import { useId, type CSSProperties, type ReactNode } from "react";
import { Area, CartesianGrid, ComposedChart, Line, ReferenceLine, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { fmtCompact, fmtInt, fmtPct, fmtTime } from "../../domain/format";
import type { PlayerReading } from "../../history/store";
import { axisProps, fmtHourTick, gridProps, hourTicks, tooltipProps } from "../../ui/chart";
import { Empty, Panel } from "../../ui/primitives";

/** Helldivers over time, however few the readings: a lone one still shows as a point. */
export function ActivityChart({ points, height = 140 }: { points: PlayerReading[]; height?: number }) {
  const gradient = useId();
  const last = points[points.length - 1];
  const domain: [number, number] = [points[0]?.at ?? 0, last?.at ?? 0];
  const few = points.length < 4;
  return (
    <ResponsiveContainer width="100%" height={height}>
      <ComposedChart data={points} margin={{ top: 8, right: 8, bottom: 0, left: 0 }}>
        <defs>
          <linearGradient id={gradient} x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor="var(--cta)" stopOpacity={0.28} />
            <stop offset="100%" stopColor="var(--cta)" stopOpacity={0.02} />
          </linearGradient>
        </defs>
        <CartesianGrid {...gridProps} />
        <XAxis
          dataKey="at"
          type="number"
          scale="time"
          domain={few ? ["dataMin - 1800000", "dataMax + 1800000"] : domain}
          ticks={few ? undefined : hourTicks(domain[0], domain[1])}
          tickFormatter={fmtHourTick}
          {...axisProps}
        />
        <YAxis tickFormatter={(v) => fmtCompact(Number(v))} {...axisProps} width={44} allowDecimals={false} />
        <Tooltip {...tooltipProps} formatter={(v) => [fmtInt(Number(v)), "Helldivers"]} />
        <Area
          dataKey="players"
          stroke="var(--cta)"
          strokeWidth={2}
          fill={`url(#${gradient})`}
          dot={few ? { r: 3, fill: "var(--cta)", strokeWidth: 0 } : false}
          isAnimationActive={false}
        />
        {last && <ReferenceLine x={last.at} stroke="var(--text-dim)" />}
      </ComposedChart>
    </ResponsiveContainer>
  );
}

/** Liberation over time, 0 to 100%. */
export function ProgressChart({ points, height = 140 }: { points: { at: number; percent: number }[]; height?: number }) {
  const domain: [number, number] = [points[0]?.at ?? 0, points[points.length - 1]?.at ?? 0];
  const few = points.length < 4;
  return (
    <ResponsiveContainer width="100%" height={height}>
      <ComposedChart data={points} margin={{ top: 8, right: 8, bottom: 0, left: 0 }}>
        <CartesianGrid {...gridProps} />
        <XAxis
          dataKey="at"
          type="number"
          scale="time"
          domain={few ? ["dataMin - 1800000", "dataMax + 1800000"] : domain}
          ticks={few ? undefined : hourTicks(domain[0], domain[1])}
          tickFormatter={fmtHourTick}
          {...axisProps}
        />
        <YAxis domain={[0, 100]} ticks={[0, 25, 50, 75, 100]} tickFormatter={(v) => `${v}%`} {...axisProps} width={44} />
        <Tooltip {...tooltipProps} formatter={(v) => [fmtPct(Number(v), 2), "Liberated"]} />
        <Line
          dataKey="percent"
          stroke="var(--c-humans)"
          strokeWidth={2}
          dot={few ? { r: 3, fill: "var(--c-humans)", strokeWidth: 0 } : false}
          isAnimationActive={false}
        />
      </ComposedChart>
    </ResponsiveContainer>
  );
}

function Head({ label, tone, children, sub }: { label: string; tone: string; children: ReactNode; sub?: string }) {
  return (
    <div>
      <span className="pd-key" style={{ "--k": tone } as CSSProperties}>
        {label}
      </span>
      <b className="num">{children}</b>
      {sub && <span className="pd-sub caps num">{sub}</span>}
    </div>
  );
}

/** Helldivers somewhere over the last 24 hours, with the busiest and quietest readings. */
export function ActivitySection(props: {
  title: string;
  now: number;
  points: PlayerReading[];
  loading: boolean;
  empty: string;
  children?: ReactNode;
}) {
  const { points } = props;
  const peak = points.reduce<PlayerReading | null>((m, r) => (!m || r.players > m.players ? r : m), null);
  const low = points.reduce<PlayerReading | null>((m, r) => (!m || r.players < m.players ? r : m), null);
  return (
    <Panel title={props.title} icon={Users}>
      <div className="pd-stack">
        <div className="pd-heads">
          <Head label="Now" tone="var(--cta)">
            {fmtInt(props.now)}
          </Head>
          <Head label="Peak" tone="var(--cta)" sub={peak ? fmtTime(peak.at) : undefined}>
            {peak ? fmtInt(peak.players) : "—"}
          </Head>
          <Head label="Lowest" tone="var(--text-dim)" sub={low ? fmtTime(low.at) : undefined}>
            {low ? fmtInt(low.players) : "—"}
          </Head>
          <Head label="Readings" tone="var(--text-dim)">
            {points.length}
          </Head>
        </div>
        {points.length > 0 ? (
          <ActivityChart points={points} />
        ) : (
          <Empty>{props.loading ? "Loading telemetry…" : props.empty}</Empty>
        )}
        {props.children}
      </div>
    </Panel>
  );
}
