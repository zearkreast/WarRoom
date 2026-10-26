import { useEffect, useRef, type CSSProperties, type MouseEvent, type ReactNode } from "react";
import "./neonRing.css";

export interface RingSegment {
  key: string;
  label: string;
  value: number;
  color: string;
}

const R = 92;
const GAP = 0.035; // radians between arcs

function arcPath(a0: number, a1: number): string {
  // A full circle can't be one arc command; split it.
  if (a1 - a0 >= Math.PI * 2 - 1e-6) return `${arcPath(a0, a0 + Math.PI)} ${arcPath(a0 + Math.PI, a1)}`;
  const p = (a: number) => `${(Math.cos(a) * R).toFixed(2)} ${(Math.sin(a) * R).toFixed(2)}`;
  return `M ${p(a0)} A ${R} ${R} 0 ${a1 - a0 > Math.PI ? 1 : 0} 1 ${p(a1)}`;
}

/**
 * Segmented neon ring. Each arc is its own hover target; the parent owns which one is active
 * and what the centre shows. A touch has no hover, so a tap shows an arc until the next tap, and
 * a second tap on it selects it (or lets go, without `onSelect`).
 */
export function NeonRing(props: {
  segments: RingSegment[];
  active: string | null;
  onActive(key: string | null): void;
  onPointerMove?(key: string, e: MouseEvent<SVGPathElement>): void;
  onSelect?(key: string): void;
  center: ReactNode;
  label: string;
  size?: number;
}) {
  const total = props.segments.reduce((n, s) => n + s.value, 0) || 1;
  const gap = props.segments.filter((s) => s.value > 0).length > 1 ? GAP : 0;
  let angle = -Math.PI / 2;
  const arcs = props.segments.map((s) => {
    const sweep = (s.value / total) * Math.PI * 2;
    const a0 = angle + gap / 2;
    const a1 = angle + sweep - gap / 2;
    angle += sweep;
    return { s, d: a1 > a0 ? arcPath(a0, a1) : null };
  });
  const size = props.size ?? 240;
  const root = useRef<HTMLDivElement>(null);
  const touch = useRef(false);
  const { active, onActive } = props;
  // A tap elsewhere lets go of an arc a tap picked.
  useEffect(() => {
    if (!active || !touch.current) return;
    const away = (e: Event) => {
      if (!root.current?.contains(e.target as Node)) onActive(null);
    };
    document.addEventListener("pointerdown", away);
    return () => document.removeEventListener("pointerdown", away);
  }, [active, onActive]);
  const tap = (key: string, e: MouseEvent<SVGPathElement>) => {
    if (!touch.current) return props.onSelect?.(key);
    if (props.active !== key) {
      props.onActive(key);
      props.onPointerMove?.(key, e);
    } else if (props.onSelect) props.onSelect(key);
    else props.onActive(null);
  };
  return (
    <div
      ref={root}
      className={`nr ${props.active ? "has-active" : ""}`}
      style={{ width: size, height: size }}
      role="img"
      aria-label={props.label}
      onPointerDown={(e) => {
        touch.current = e.pointerType !== "mouse";
      }}
      onPointerLeave={(e) => {
        if (e.pointerType === "mouse") props.onActive(null);
      }}
    >
      <svg viewBox="-120 -120 240 240" aria-hidden>
        <circle r={110} className="nr-dots" />
        <circle r={R} className="nr-track" />
        {arcs.map(({ s, d }) =>
          d ? (
            <path
              key={s.key}
              d={d}
              className={`nr-arc ${props.active === s.key ? "on" : ""}`}
              style={{ "--c": s.color } as CSSProperties}
              onPointerEnter={(e) => {
                if (e.pointerType === "mouse") props.onActive(s.key);
              }}
              onPointerMove={(e) => {
                if (e.pointerType === "mouse") props.onPointerMove?.(s.key, e);
              }}
              onClick={(e) => tap(s.key, e)}
            />
          ) : null,
        )}
      </svg>
      <div className="nr-center">{props.center}</div>
    </div>
  );
}
