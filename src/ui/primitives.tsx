import { ChevronLeft, ChevronRight, type LucideIcon } from "lucide-react";
import { useEffect, useState, type CSSProperties, type ReactNode } from "react";
import { factionColor, factionLabel } from "../domain/factions";
import { fmtDuration } from "../domain/format";
import { FactionEmblem } from "./emblems";

/** Panel with a hatch-striped header strip: icon box, title, and an optional right slot. */
export function Panel(props: {
  title?: ReactNode;
  icon?: LucideIcon;
  right?: ReactNode;
  /** Tints the header stripe and icon box. Defaults to the CTA yellow. */
  tone?: string;
  glass?: boolean;
  children: ReactNode;
  className?: string;
  bodyClass?: string;
}) {
  const style = props.tone ? ({ "--tone": props.tone } as CSSProperties) : undefined;
  return (
    <section className={`panel ${props.glass ? "glass" : ""} ${props.className ?? ""}`} style={style}>
      {props.title != null && (
        <header className="panel-head">
          {props.icon && (
            <span className="panel-icon">
              <props.icon size={14} strokeWidth={2} />
            </span>
          )}
          <h2 className="panel-title">{props.title}</h2>
          {props.right != null && <div className="panel-right">{props.right}</div>}
        </header>
      )}
      <div className={props.bodyClass ?? "panel-body"}>{props.children}</div>
    </section>
  );
}

/**
 * Progress meter: tick-gapped fill with a hatched remainder. `rest` colours the remainder
 * (usually the enemy faction), `marker` draws a projected end position.
 */
export function Meter(props: {
  value: number;
  color: string;
  rest?: string;
  marker?: number | null;
  size?: "s" | "m" | "l";
  /** Chevron fill used for the big liberation bars. */
  chevron?: boolean;
  label?: string;
}) {
  const v = Math.max(0, Math.min(100, props.value));
  const style = { "--fill": props.color, "--rest": props.rest ?? "var(--line-strong)" } as CSSProperties;
  return (
    <div
      className={`meter meter-${props.size ?? "m"} ${props.chevron ? "chevron" : ""}`}
      style={style}
      role="progressbar"
      aria-valuenow={Math.round(v)}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-label={props.label}
    >
      <div className="meter-fill" style={{ width: `${v}%` }} />
      <div className="meter-rest" style={{ left: `${v}%` }} />
      {props.marker != null && props.marker > 0 && props.marker < 100 && (
        <div className="meter-marker" style={{ left: `${props.marker}%` }} />
      )}
    </div>
  );
}

/** Kept for older call sites; same as a Meter. */
export function ProgressBar(props: {
  value: number;
  color: string;
  tall?: boolean;
  striped?: boolean;
  marker?: number | null;
  rest?: string;
}) {
  return (
    <Meter
      value={props.value}
      color={props.color}
      rest={props.rest}
      marker={props.marker}
      size={props.tall ? "l" : "m"}
    />
  );
}

export function Keycap({ k }: { k: string }) {
  return <kbd className="keycap">{k}</kbd>;
}

export function HudButton(props: {
  icon: LucideIcon;
  label: string;
  onClick?: () => void;
  active?: boolean;
  keyHint?: string;
  size?: number;
}) {
  return (
    <button
      className={`hud-btn icon ${props.active ? "on" : ""}`}
      onClick={props.onClick}
      aria-label={props.label}
      title={props.keyHint ? `${props.label} (${props.keyHint})` : props.label}
    >
      <props.icon size={props.size ?? 18} strokeWidth={1.75} />
    </button>
  );
}

export function CtaButton(props: { icon?: LucideIcon; children: ReactNode; onClick?: () => void }) {
  return (
    <button className="cta-btn" onClick={props.onClick}>
      {props.icon && <props.icon size={15} strokeWidth={2} />}
      {props.children}
    </button>
  );
}

/** Button with a keycap prefix, holotable style: [ESC] CLOSE. */
export function KeyButton(props: {
  k: string;
  children: ReactNode;
  onClick?: () => void;
  badge?: ReactNode;
  active?: boolean;
}) {
  return (
    <button className={`key-btn ${props.active ? "on" : ""}`} onClick={props.onClick}>
      <Keycap k={props.k} />
      <span>{props.children}</span>
      {props.badge != null && <span className="key-badge num">{props.badge}</span>}
    </button>
  );
}

export function StatusChip({ tone = "var(--good)", children }: { tone?: string; children: ReactNode }) {
  return (
    <span className="status-chip" style={{ "--tone": tone } as CSSProperties}>
      {children}
    </span>
  );
}

export function LiveDot({ ok = true }: { ok?: boolean }) {
  return <span className={`live-dot ${ok ? "" : "off"}`} aria-hidden />;
}

/** ◀ VALUE ▶ selector with a segmented underline showing the position. */
export function Stepper<T extends string>(props: {
  options: { value: T; label: string }[];
  value: T;
  onChange(v: T): void;
  label: string;
}) {
  const i = Math.max(
    0,
    props.options.findIndex((o) => o.value === props.value),
  );
  const go = (d: number) => props.onChange(props.options[(i + d + props.options.length) % props.options.length].value);
  return (
    <div className="stepper" role="group" aria-label={props.label}>
      <button className="stepper-arrow" onClick={() => go(-1)} aria-label={`Previous ${props.label}`}>
        <ChevronLeft size={18} />
      </button>
      <div className="stepper-value">
        <span className="caps">{props.options[i]?.label}</span>
        <span className="stepper-segs">
          {props.options.map((o, k) => (
            <span key={o.value} className={k === i ? "on" : ""} />
          ))}
        </span>
      </div>
      <button className="stepper-arrow" onClick={() => go(1)} aria-label={`Next ${props.label}`}>
        <ChevronRight size={18} />
      </button>
    </div>
  );
}

/** Labelled value tile. */
export function Tile(props: {
  label: string;
  value: ReactNode;
  sub?: ReactNode;
  icon?: LucideIcon;
  color?: string;
  rule?: string;
}) {
  return (
    <div className="tile" style={props.rule ? ({ "--rule": props.rule } as CSSProperties) : undefined}>
      <div className="tile-label">
        {props.icon && <props.icon size={12} strokeWidth={2} />}
        {props.label}
      </div>
      <div className="tile-value num" style={{ color: props.color }}>
        {props.value}
      </div>
      {props.sub != null && <div className="tile-sub">{props.sub}</div>}
    </div>
  );
}

export function Stat(props: { label: string; value: ReactNode; color?: string }) {
  return <Tile label={props.label} value={props.value} color={props.color} />;
}

export function FactionTag({ faction }: { faction: string }) {
  return (
    <span className="faction-tag" style={{ color: factionColor(faction) }}>
      <FactionEmblem faction={faction} size={12} />
      {factionLabel(faction)}
    </span>
  );
}

/** Full-viewport overlay on grid paper, closed with ESC or the corner button. */
export function Overlay(props: { onClose(): void; children: ReactNode; label: string; footer?: ReactNode }) {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && props.onClose();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [props]);
  return (
    <div className="overlay" role="dialog" aria-modal="true" aria-label={props.label}>
      <div className="overlay-scroll scroll">{props.children}</div>
      <div className="overlay-foot">
        <KeyButton k="ESC" onClick={props.onClose}>
          Close
        </KeyButton>
        {props.footer}
      </div>
    </div>
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

/** Skeleton rows while a snapshot loads. */
export function Loading({ what, rows = 3 }: { what: string; rows?: number }) {
  return (
    <div className="loading" aria-busy="true" aria-label={`Loading ${what}`}>
      {Array.from({ length: rows }, (_, i) => (
        <div key={i} className="skeleton" />
      ))}
      <div className="loading-text caps">Receiving {what}…</div>
    </div>
  );
}

export function Empty({ children }: { children: ReactNode }) {
  return <div className="empty caps">{children}</div>;
}
