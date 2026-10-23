import { ArrowUpRight, Map as MapIcon } from "lucide-react";
import { useEffect, useMemo, useRef, type CSSProperties, type KeyboardEvent, type PointerEvent } from "react";
import { useFreshPlanets } from "../../api/queries";
import type { Faction } from "../../api/types";
import {
  buildPolarSectors,
  GALAXY_SCALE,
  project,
  RING_RADII,
  sectorTerritory,
  supplyLines,
} from "../../domain/galaxy";
import { byIndex } from "../../domain/planets";
import { buildRegions } from "../../domain/regions";
import { useSettings, useUi } from "../../state";
import { Panel } from "../../ui/primitives";
import { CHART } from "./WarActivity";

const SPIN_DEG_PER_S = 4;
const DRAG_DEG_PER_PX = 0.35;
const KEY_STEP_DEG = 10;
const ORDER: Faction[] = ["Humans", "Terminids", "Illuminate", "Automaton"];

const RIM = RING_RADII[RING_RADII.length - 1] * GALAXY_SCALE;

interface OwnerGroup {
  owner: Faction;
  held: boolean;
  paths: string[];
}

/** The galaxy disc seen at an angle, drawn like the map's chosen overlay; it turns slowly and can be dragged round. */
export function TheaterOfWar() {
  const planets = useFreshPlanets().data;
  const overlay = useSettings((s) => s.mapOverlay);
  const setView = useUi((s) => s.setView);
  const byOwner = useMemo((): OwnerGroup[] => {
    if (!planets || overlay === "galaxy") return [];
    if (overlay === "regions")
      return buildRegions(planets)
        .map((r) => ({ owner: r.owner, held: true, paths: [r.path] }))
        .sort((a, b) => ORDER.indexOf(a.owner) - ORDER.indexOf(b.owner));
    // Ring sectors: enemy sectors are solid where held outright, dimmer where contested.
    const territory = sectorTerritory(planets);
    const groups = new Map<string, { owner: Faction; held: boolean; paths: string[] }>();
    for (const [name, path] of buildPolarSectors(planets).fills) {
      const t = territory.get(name);
      const owner = t?.faction ?? "Humans";
      const key = `${owner}-${t?.held ?? true}`;
      const g = groups.get(key) ?? { owner, held: t?.held ?? true, paths: [] };
      g.paths.push(path);
      groups.set(key, g);
    }
    return [...groups.values()].sort((a, b) => ORDER.indexOf(a.owner) - ORDER.indexOf(b.owner));
  }, [planets, overlay]);
  const lanes = useMemo(() => {
    if (!planets || overlay !== "galaxy") return "";
    const lookup = byIndex(planets);
    return supplyLines(planets)
      .flatMap(([a, b]) => {
        const pa = lookup.get(a);
        const pb = lookup.get(b);
        if (!pa || !pb) return [];
        const [x1, y1] = project(pa.position);
        const [x2, y2] = project(pb.position);
        return [`M${x1.toFixed(1)},${y1.toFixed(1)}L${x2.toFixed(1)},${y2.toFixed(1)}`];
      })
      .join("");
  }, [planets, overlay]);

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
  }, [planets != null]);

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
      className="tw"
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
        aria-label={`Galaxy ${overlay} view. Drag or use the arrow keys to rotate.`}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={endDrag}
        onPointerCancel={endDrag}
        onKeyDown={onKeyDown}
      >
        <svg viewBox="-560 -560 1120 1120" aria-hidden>
          <circle r={545} className="tw-rim" />
          <g ref={disc}>
            {overlay !== "sectors" && (
              <g className="tw-owner se" style={{ "--c": CHART.Humans } as CSSProperties}>
                <circle r={RIM} />
              </g>
            )}
            {lanes && <path d={lanes} className="tw-lanes" />}
            {byOwner.map((g) => (
              <g
                key={`${g.owner}-${g.held}`}
                className={`tw-owner ${g.owner === "Humans" ? "se" : "enemy"} ${g.held ? "" : "contested"}`}
                style={{ "--c": CHART[g.owner] } as CSSProperties}
              >
                <path d={g.paths.join("")} />
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
