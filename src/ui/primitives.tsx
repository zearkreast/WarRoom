import { useEffect, useState, type ReactNode } from "react";
import { factionColor, factionLabel } from "../domain/factions";
import { fmtDuration } from "../domain/format";

export function Panel(props: { title?: ReactNode; right?: ReactNode; children: ReactNode; className?: string }) {
  return (
    <section className={`panel ${props.className ?? ""}`}>
      {props.title != null && (
        <header className="panel-head">
          <span className="title">{props.title}</span>
          {props.right}
        </header>
      )}
      {props.children}
    </section>
  );
}

export function ProgressBar(props: {
  value: number;
  color: string;
  tall?: boolean;
  striped?: boolean;
  /** Optional marker position (0-100), e.g. the projected end result. */
  marker?: number | null;
  /** Background color for the remaining part, e.g. the enemy's faction color. */
  rest?: string;
}) {
  const v = Math.max(0, Math.min(100, props.value));
  return (
    <div
      className={`bar ${props.tall ? "tall" : ""} ${props.striped ? "striped" : ""}`}
      style={props.rest ? { background: `color-mix(in srgb, ${props.rest} 55%, transparent)` } : undefined}
    >
      <div className="fill" style={{ width: `${v}%`, background: props.color }} />
      {props.marker != null && props.marker > 0 && props.marker < 100 && (
        <div className="marker" style={{ left: `${props.marker}%` }} />
      )}
    </div>
  );
}

export function FactionTag({ faction }: { faction: string }) {
  return (
    <span className="chip" style={{ color: factionColor(faction) }}>
      <span className="dot" style={{ background: factionColor(faction) }} />
      {factionLabel(faction)}
    </span>
  );
}

/** Re-renders the caller every `ms` milliseconds; returns the current time. */
export function useNow(ms = 1000): number {
  const [now, setNow] = useState(Date.now);
  useEffect(() => {
    const id = setInterval(() => setNow(Date.now()), ms);
    return () => clearInterval(id);
  }, [ms]);
  return now;
}

export function Countdown({ to }: { to: number }) {
  const now = useNow(30_000);
  return <span className="num">{fmtDuration(to - now)}</span>;
}

export function Stat(props: { label: string; value: ReactNode; color?: string }) {
  return (
    <div>
      <div className="stat-label">{props.label}</div>
      <div className="stat-value" style={{ color: props.color }}>
        {props.value}
      </div>
    </div>
  );
}

export function Loading({ what }: { what: string }) {
  return <div className="empty caps">Syncing {what}…</div>;
}
