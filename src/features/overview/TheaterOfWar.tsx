import { ArrowUpRight, Map as MapIcon, Pause, Play } from "lucide-react";
import { useEffect, useMemo, useRef, useState, type CSSProperties, type KeyboardEvent, type PointerEvent } from "react";
import { useEffects, useFreshPlanets, useSectorGeometry } from "../../api/queries";
import type { Faction } from "../../api/types";
import { factionMarks } from "../../domain/factionMarks";
import { project, supplyLines } from "../../domain/galaxy";
import { byIndex, ownershipKey } from "../../domain/planets";
import { buildRegions } from "../../domain/regions";
import { useSettings, useUi } from "../../state";
import { EmblemArt } from "../../ui/emblems";
import { Panel } from "../../ui/primitives";
import { GloomLayer, SectorLayer } from "../map/layers";
import { CHART } from "./WarActivity";

const SPIN_DEG_PER_S = 4;
const DRAG_DEG_PER_PX = 0.35;
const KEY_STEP_DEG = 10;
const ORDER: Faction[] = ["Humans", "Terminids", "Illuminate", "Automaton"];
/** The stage tilts the disc by 56°; emblems are stretched back by 1/cos(56°) to keep their shape. */
const UNSQUASH = 1 / Math.cos((56 * Math.PI) / 180);
/** Hatch spacing in disc units; the disc is drawn at about a third of its size. */
const HATCH = 18;

interface OwnerGroup {
  owner: Faction;
  held: boolean;
  paths: string[];
}

/** The galaxy disc seen at an angle, drawn like the map's chosen overlay; it turns slowly and can be dragged round. */
export function TheaterOfWar() {
  const planets = useFreshPlanets().data;
  const geo = useSectorGeometry().data;
  const effects = useEffects().data;
  const overlay = useSettings((s) => s.mapOverlay);
  const spin = useSettings((s) => s.twSpin);
  const setSetting = useSettings((s) => s.set);
  const setView = useUi((s) => s.setView);
  const [sector, setSector] = useState<string | null>(null);
  const owners = planets ? ownershipKey(planets) : "";
  const byOwner = useMemo((): OwnerGroup[] => {
    if (!planets || overlay !== "regions") return [];
    return buildRegions(planets)
      .map((r) => ({ owner: r.owner, held: true, paths: [r.path] }))
      .sort((a, b) => ORDER.indexOf(a.owner) - ORDER.indexOf(b.owner));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [owners, overlay]);
  // eslint-disable-next-line react-hooks/exhaustive-deps
  const marks = useMemo(() => (planets ? factionMarks(planets, overlay, geo) : []), [owners, overlay, geo]);
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
    // Lanes follow the static waypoints.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [geo, overlay]);

  const stage = useRef<HTMLDivElement>(null);
  const disc = useRef<SVGGElement>(null);
  const angle = useRef(0);
  const drag = useRef<{ x: number; from: number } | null>(null);
  const turn = (deg: number) => {
    angle.current = (deg + 360) % 360;
    disc.current?.setAttribute("transform", `rotate(${angle.current.toFixed(2)})`);
    disc.current?.style.setProperty("--unrot", `${(-angle.current).toFixed(2)}deg`);
  };

  useEffect(() => {
    const el = stage.current;
    if (!el) return;
    const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    let onScreen = true;
    const spinning = () => useSettings.getState().twSpin;
    const io = new IntersectionObserver(([entry]) => (onScreen = entry.isIntersecting));
    io.observe(el);
    let last = performance.now();
    let raf = requestAnimationFrame(function tick(t) {
      // Frames stop while the window is hidden; don't jump by the whole gap when they resume.
      const dt = Math.min((t - last) / 1000, 0.1);
      last = t;
      if (!reduced && spinning() && onScreen && !drag.current && !document.hidden)
        turn(angle.current + SPIN_DEG_PER_S * dt);
      raf = requestAnimationFrame(tick);
    });
    return () => {
      cancelAnimationFrame(raf);
      io.disconnect();
    };
  }, [planets != null]);

  if (!planets || !geo) return null;

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
        <>
          <button
            className="hud-btn icon tw-spin"
            onClick={() => setSetting("twSpin", !spin)}
            aria-pressed={!spin}
            aria-label={spin ? "Stop turning" : "Turn the disc"}
            title={spin ? "Stop turning" : "Turn the disc"}
          >
            {spin ? <Pause size={14} /> : <Play size={14} />}
          </button>
          <button className="hud-btn tw-open" onClick={() => setView("map")}>
            Open map <ArrowUpRight size={14} />
          </button>
        </>
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
        onPointerLeave={() => setSector(null)}
        onKeyDown={onKeyDown}
      >
        <svg viewBox="-560 -560 1120 1120" aria-hidden>
          <circle r={545} className="tw-rim" />
          <g ref={disc}>
            {overlay !== "sectors" && (
              <g className="tw-owner se" style={{ "--c": CHART.Humans } as CSSProperties}>
                <circle r={geo.rim} />
              </g>
            )}
            {overlay === "sectors" && <SectorLayer geo={geo} planets={planets} id="tw" hatch={HATCH} />}
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
            {effects && <GloomLayer planets={planets} gloom={effects.gloom} />}
            <g className="tw-hits">
              {[...geo.fills].map(([name, d]) => (
                <path key={name} d={d} onPointerEnter={() => setSector(name)} />
              ))}
            </g>
            {marks.map((m) => (
              <g key={m.faction} transform={`translate(${m.x.toFixed(1)},${m.y.toFixed(1)})`}>
                <g className="tw-mark" style={{ "--unsquash": UNSQUASH } as CSSProperties}>
                  <EmblemArt art={m.faction} size={84} />
                </g>
              </g>
            ))}
            <line x1={0} y1={-545} x2={0} y2={-470} className="tw-tick" />
          </g>
          <circle r={70} className="tw-core-glow" />
          <g className="tw-core-mark">
            <g transform="scale(1,1.778)">
              <EmblemArt art="Humans" size={86} />
            </g>
          </g>
        </svg>
        <span className="tw-hint caps">{sector ? `${sector} sector` : "Drag to rotate"}</span>
      </div>
    </Panel>
  );
}
