import { Users } from "lucide-react";
import type { CSSProperties, PointerEvent, ReactNode } from "react";
import { fmtInt } from "../domain/format";

/** Pointer position plus the centre of the chart it hovers, so the card opens away from it. */
export interface PointerAt {
  x: number;
  y: number;
  cx: number;
  cy: number;
}

export interface CardRow {
  key: string;
  name: string;
  /** Helldivers; null for rows without live data (show `note` instead). */
  value: number | null;
  note?: string;
  /** Bar colour; defaults to the card's tone. */
  color?: string;
}

const WIDTH = 260;
const HEIGHT = 300;
const OFFSET = 18;

export function pointerAt(e: PointerEvent, anchor: Element): PointerAt {
  const r = anchor.getBoundingClientRect();
  return { x: e.clientX, y: e.clientY, cx: r.left + r.width / 2, cy: r.top + r.height / 2 };
}

function place(at: PointerAt): { left: number; top: number } {
  const clampX = (v: number) => Math.max(8, Math.min(v, window.innerWidth - WIDTH - 8));
  const clampY = (v: number) => Math.max(8, Math.min(v, window.innerHeight - HEIGHT - 8));
  const sideX = at.x < at.cx ? at.x - WIDTH - OFFSET : at.x + OFFSET;
  if (sideX >= 8 && sideX + WIDTH <= window.innerWidth - 8) return { left: sideX, top: clampY(at.y - HEIGHT / 2) };
  // No room beside the chart: open above or below the pointer instead.
  return { left: clampX(at.x - WIDTH / 2), top: clampY(at.y < at.cy ? at.y - HEIGHT - OFFSET : at.y + OFFSET) };
}

/** Hover card that follows the pointer: a titled list of rows with relative bars. */
export function PointerCard(props: {
  at: PointerAt;
  tone: string;
  icon?: ReactNode;
  title: string;
  badge?: ReactNode;
  rows: CardRow[];
  footer?: ReactNode;
}) {
  const max = Math.max(1, ...props.rows.map((r) => r.value ?? 0));
  const { left, top } = place(props.at);
  return (
    <div className="fmap" style={{ left, top, "--k": props.tone } as CSSProperties} aria-hidden>
      <div className="fmap-head">
        {props.icon}
        <b>{props.title}</b>
        {props.badge != null && <span className="num">{props.badge}</span>}
      </div>
      <ol>
        {props.rows.map((r) => (
          <li key={r.key}>
            <span className="fmap-name">{r.name}</span>
            <span className="fmap-val">
              {r.value == null ? (
                r.note
              ) : (
                <>
                  <Users size={10} /> {fmtInt(r.value)}
                </>
              )}
            </span>
            <span className="fmap-bar">
              <span
                style={{ width: `${((r.value ?? 0) / max) * 100}%`, "--k": r.color ?? props.tone } as CSSProperties}
              />
            </span>
          </li>
        ))}
      </ol>
      {props.footer != null && <div className="fmap-foot">{props.footer}</div>}
    </div>
  );
}
