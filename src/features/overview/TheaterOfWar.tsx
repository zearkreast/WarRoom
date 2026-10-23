import { ArrowUpRight, Map as MapIcon } from "lucide-react";
import { useEffect, useMemo, useRef, type CSSProperties, type KeyboardEvent, type PointerEvent } from "react";
import { useFreshPlanets } from "../../api/queries";
import type { Faction } from "../../api/types";
import { buildSectors, type SectorShape } from "../../domain/galaxy";
import { useUi } from "../../state";
import { Panel } from "../../ui/primitives";
import { CHART } from "./WarActivity";

const SPIN_DEG_PER_S = 4;
const DRAG_DEG_PER_PX = 0.35;
const KEY_STEP_DEG = 10;
const ORDER: Faction[] = ["Humans", "Terminids", "Illuminate", "Automaton"];

/** The sector disc seen at an angle; it turns slowly and can be dragged round. */
export function TheaterOfWar() {
  const planets = useFreshPlanets().data;
  const setView = useUi((s) => s.setView);
  const sectors = useMemo(() => (planets ? buildSectors(planets) : []), [planets]);
  const byOwner = useMemo(() => {
    const groups = new Map<Faction, SectorShape[]>();
    for (const s of sectors) groups.set(s.owner, [...(groups.get(s.owner) ?? []), s]);
    return ORDER.flatMap((f) => (groups.has(f) ? [[f, groups.get(f)!] as const] : []));
  }, [sectors]);

  const stage = useRef<HTMLDivElement>(null);
  const disc = useRef<SVGGElement>(null);
  const angle = useRef(0);
  const drag = useRef<{ x: number; from: number } | null>(null);
  const turn = (deg: number) => {
    angle.current = (deg + 360) % 360;
    disc.current?.setAttribute("transform", `rotate(${angle.current.toFixed(2)})`);
  };

  useEffect(() => {
    const el = stage.current;
    if (!el) return;
    const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    let onScreen = true;
    const io = new IntersectionObserver(([entry]) => (onScreen = entry.isIntersecting));
    io.observe(el);
    let last = performance.now();
    let raf = requestAnimationFrame(function tick(t) {
      // Frames stop while the window is hidden; don't jump by the whole gap when they resume.
      const dt = Math.min((t - last) / 1000, 0.1);
      last = t;
      if (!reduced && onScreen && !drag.current && !document.hidden) turn(angle.current + SPIN_DEG_PER_S * dt);
      raf = requestAnimationFrame(tick);
    });
    return () => {
      cancelAnimationFrame(raf);
      io.disconnect();
    };
  }, [sectors.length > 0]);

  if (!planets) return null;

  const onPointerDown = (e: PointerEvent<HTMLDivElement>) => {
    e.currentTarget.setPointerCapture(e.pointerId);
    drag.current = { x: e.clientX, from: angle.current };
    stage.current?.classList.add("dragging");
  };
  const onPointerMove = (e: PointerEvent<HTMLDivElement>) => {
    if (drag.current) turn(drag.current.from + (e.clientX - drag.current.x) * DRAG_DEG_PER_PX);
  };
  const endDrag = () => {
    drag.current = null;
    stage.current?.classList.remove("dragging");
  };
  const onKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.key === "ArrowLeft") turn(angle.current - KEY_STEP_DEG);
    if (e.key === "ArrowRight") turn(angle.current + KEY_STEP_DEG);
  };

  return (
    <Panel
      title="Theater of war"
      icon={MapIcon}
      bodyClass="panel-body tw-body"
      right={
        <button className="hud-btn tw-open" onClick={() => setView("map")}>
          Open map <ArrowUpRight size={14} />
        </button>
      }
    >
      <div
        ref={stage}
        className="tw-stage"
        tabIndex={0}
        role="img"
        aria-label="Sector map by owner. Drag or use the arrow keys to rotate."
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={endDrag}
        onPointerCancel={endDrag}
        onKeyDown={onKeyDown}
      >
        <svg viewBox="-560 -560 1120 1120" aria-hidden>
          <circle r={545} className="tw-rim" />
          <g ref={disc}>
            {byOwner.map(([owner, list]) => (
              <g
                key={owner}
                className={`tw-owner ${owner === "Humans" ? "se" : "enemy"}`}
                style={{ "--c": CHART[owner] } as CSSProperties}
              >
                {list.map((s) => (
                  <path key={s.name} d={s.path} />
                ))}
              </g>
            ))}
            <line x1={0} y1={-545} x2={0} y2={-470} className="tw-tick" />
          </g>
          <circle r={70} className="tw-core-glow" />
          <circle r={36} className="tw-core-ring" />
          <circle r={12} className="tw-core" />
        </svg>
        <span className="tw-hint caps">Drag to rotate</span>
      </div>
    </Panel>
  );
}
