import { ArrowUpRight, Mouse, MousePointer2, MousePointerClick, Move } from "lucide-react";
import { useEffect, useMemo, useState, type ReactNode } from "react";
import { useDispatches, useFreshPlanets } from "../../api/queries";
import { dispatchBody, latestDispatches } from "../../domain/dispatch";
import { factionColor, factionLabel } from "../../domain/factions";
import { dispatchTitle, fmtAgo } from "../../domain/format";
import { useSettings, useUi } from "../../state";
import { FactionEmblem } from "../../ui/emblems";
import { Hint } from "../../ui/Hint";
import { Keycap, KeyButton, useNow } from "../../ui/primitives";
import { MajorOrderList } from "../major-order/MajorOrder";
import { GalaxyMap } from "./GalaxyMap";
import { OVERLAY_OPTIONS } from "./mapOptions";

type Drawer = "dispatches" | "orders" | null;

function TitleStack({ hovered }: { hovered: string | null }) {
  const planets = useFreshPlanets().data;
  const sector = useMemo(() => {
    if (!hovered || !planets) return null;
    const counts = new Map<string, number>();
    for (const p of planets)
      if (p.sector === hovered) counts.set(p.currentOwner, (counts.get(p.currentOwner) ?? 0) + 1);
    const owner = [...counts].sort((a, b) => b[1] - a[1])[0]?.[0] ?? "Humans";
    return { name: hovered, owner };
  }, [hovered, planets]);
  return (
    <div className="map-title">
      <FactionEmblem faction={sector?.owner ?? "Humans"} size={34} color={sector ? undefined : "var(--text-hi)"} />
      <div>
        <h1 className={sector ? "sector" : ""}>{sector ? `${sector.name} sector` : "Galactic war"}</h1>
        <div className="map-subtitle caps" style={sector ? { color: factionColor(sector.owner) } : undefined}>
          {sector ? `${factionLabel(sector.owner)} control` : "Galaxy overview"}
        </div>
      </div>
    </div>
  );
}

const CONTROLS: [ReactNode, string][] = [
  [<Move size={15} />, "Drag to pan"],
  [<Mouse size={15} />, "Scroll to zoom"],
  [<MousePointerClick size={15} />, "Planet options"],
  [
    <>
      <Keycap k="+" />
      <Keycap k="−" />
    </>,
    "Zoom",
  ],
  [<Keycap k="0" />, "Fit galaxy"],
  [<MousePointer2 size={15} />, "Right-drag: turn & tilt"],
  [
    <>
      <Keycap k="Q" />
      <Keycap k="E" />
    </>,
    "Rotate",
  ],
  [
    <>
      <Keycap k="R" />
      <Keycap k="F" />
    </>,
    "Tilt",
  ],
  [<Keycap k="Z" />, "Dispatches"],
  [<Keycap k="C" />, "Orders"],
  [<Keycap k="H" />, "Hide controls"],
];

/** The three map overlays side by side, one click each. */
function OverlaySwitch() {
  const overlay = useSettings((s) => s.mapOverlay);
  const set = useSettings((s) => s.set);
  return (
    <div className="map-overlays" role="radiogroup" aria-label="Map overlay">
      {OVERLAY_OPTIONS.map((o) => (
        <button
          key={o.value}
          role="radio"
          aria-checked={overlay === o.value}
          className={overlay === o.value ? "on" : ""}
          onClick={() => set("mapOverlay", o.value)}
        >
          {o.label}
        </button>
      ))}
    </div>
  );
}

function ControlsPanel() {
  return (
    <dl className="map-keys panel glass" aria-label="Map controls">
      {CONTROLS.map(([glyph, label]) => (
        <div key={label}>
          <dt>{glyph}</dt>
          <dd>{label}</dd>
        </div>
      ))}
    </dl>
  );
}

function DispatchDrawer({ now }: { now: number }) {
  const { data } = useDispatches();
  const setView = useUi((s) => s.setView);
  const count = useSettings((s) => s.mapDispatches);
  const recent = latestDispatches(data ?? [], count, now);
  return (
    <div className="map-drawer panel glass">
      <div className="md-head caps">
        Dispatches <span>{count === "72h" ? "last 72 hours" : `latest ${count}`}</span>
      </div>
      {recent.length === 0 ? (
        <p className="md-empty caps">No dispatches in the last 72 hours.</p>
      ) : (
        <ul className="md-list">
          {recent.map((d) => (
            <li key={d.id}>
              <b>{dispatchTitle(d.message)}</b>
              <p>
                {dispatchBody(d.message)
                  .map((s) => s.text)
                  .join("")}
              </p>
              <span className="num">{fmtAgo(Date.parse(d.published), now)}</span>
            </li>
          ))}
        </ul>
      )}
      <button className="md-more caps" onClick={() => setView("news")}>
        See all dispatches <ArrowUpRight size={14} />
      </button>
    </div>
  );
}

export function MapPage() {
  const [drawer, setDrawer] = useState<Drawer>(null);
  const [controls, setControls] = useState(false);
  const [hovered, setHovered] = useState<string | null>(null);
  const now = useNow(60_000);
  const count = useSettings((s) => s.mapDispatches);
  const dispatches = latestDispatches(useDispatches().data ?? [], count, now).length;

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.ctrlKey || e.metaKey || (e.target as HTMLElement)?.closest("input, textarea")) return;
      const k = e.key.toLowerCase();
      if (k === "h") setControls((c) => !c);
      if (k === "z") setDrawer((d) => (d === "dispatches" ? null : "dispatches"));
      if (k === "c") setDrawer((d) => (d === "orders" ? null : "orders"));
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  return (
    <div className="map-page">
      <GalaxyMap onHoverSector={setHovered} />
      <TitleStack hovered={hovered} />
      <div className="map-touch-hint">
        <Hint id="map-touch">
          One finger pans. Two fingers pinch to zoom, twist to turn, or slide up and down together to tilt.
        </Hint>
      </div>
      <div className="map-corner">
        <OverlaySwitch />
        <div className="map-controls-hint">
          <KeyButton k="H" active={controls} onClick={() => setControls((c) => !c)}>
            Controls
          </KeyButton>
          {controls && <ControlsPanel />}
        </div>
      </div>
      <div className="map-toggles">
        {drawer === "dispatches" && <DispatchDrawer now={now} />}
        {drawer === "orders" && (
          <div className="map-drawer orders">
            <MajorOrderList compact />
          </div>
        )}
        <div className="map-toggle-row">
          <KeyButton
            k="Z"
            active={drawer === "dispatches"}
            badge={dispatches || undefined}
            onClick={() => setDrawer((d) => (d === "dispatches" ? null : "dispatches"))}
          >
            Dispatches
          </KeyButton>
          <KeyButton
            k="C"
            active={drawer === "orders"}
            onClick={() => setDrawer((d) => (d === "orders" ? null : "orders"))}
          >
            Orders
          </KeyButton>
        </div>
      </div>
    </div>
  );
}
