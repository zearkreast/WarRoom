import { useEffect, useMemo, useState } from "react";
import { useDispatches, useFreshPlanets } from "../../api/queries";
import type { Faction, Planet } from "../../api/types";
import { FACTIONS, factionColor, factionLabel } from "../../domain/factions";
import { dispatchTitle, fmtDateTime } from "../../domain/format";
import { useUi } from "../../state";
import { FactionEmblem } from "../../ui/emblems";
import { KeyButton } from "../../ui/primitives";
import { MajorOrderList } from "../major-order/MajorOrder";
import { GalaxyMap } from "./GalaxyMap";

type Drawer = "dispatches" | "orders" | null;

function TitleStack({ hovered }: { hovered: Planet | null }) {
  const planets = useFreshPlanets().data;
  const sector = useMemo(() => {
    if (!hovered || !planets) return null;
    const inSector = planets.filter((p) => p.sector === hovered.sector);
    const counts = new Map<string, number>();
    for (const p of inSector) counts.set(p.currentOwner, (counts.get(p.currentOwner) ?? 0) + 1);
    const owner = [...counts].sort((a, b) => b[1] - a[1])[0]?.[0] ?? hovered.currentOwner;
    return { name: hovered.sector, owner };
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

function ControlBar() {
  const planets = useFreshPlanets().data ?? [];
  const counts = new Map<Faction, number>();
  for (const p of planets) counts.set(p.currentOwner, (counts.get(p.currentOwner) ?? 0) + 1);
  const total = planets.length || 1;
  return (
    <div className="control-bar panel glass" aria-label="Galactic control">
      <div className="control-split">
        {(Object.keys(FACTIONS) as Faction[]).map((f) => (
          <span key={f} style={{ width: `${((counts.get(f) ?? 0) / total) * 100}%`, background: FACTIONS[f].color }} />
        ))}
      </div>
      <div className="control-legend">
        {(Object.keys(FACTIONS) as Faction[]).map((f) => (
          <span key={f} className="caps">
            <FactionEmblem faction={f} size={13} />
            <b className="num" style={{ color: FACTIONS[f].color }}>
              {Math.round(((counts.get(f) ?? 0) / total) * 100)}%
            </b>
            <span className="cl-name">{FACTIONS[f].label}</span>
          </span>
        ))}
      </div>
    </div>
  );
}

function DispatchDrawer() {
  const { data } = useDispatches();
  const setView = useUi((s) => s.setView);
  const latest = [...(data ?? [])].sort((a, b) => b.id - a.id).slice(0, 4);
  return (
    <div className="map-drawer panel glass">
      <div className="md-head caps">Dispatches</div>
      {latest.length === 0 ? (
        <p className="md-empty caps">No dispatches received.</p>
      ) : (
        <ul className="md-list">
          {latest.map((d) => (
            <li key={d.id}>
              <b>{dispatchTitle(d.message)}</b>
              <span className="num">{fmtDateTime(Date.parse(d.published))}</span>
            </li>
          ))}
        </ul>
      )}
      <button className="link md-more caps" onClick={() => setView("news")}>
        Open Strohmann News
      </button>
    </div>
  );
}

export function MapPage() {
  const [drawer, setDrawer] = useState<Drawer>(null);
  const [controls, setControls] = useState(true);
  const [hovered, setHovered] = useState<Planet | null>(null);
  const dispatches = useDispatches().data?.length ?? 0;

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
      <div className="map-controls-hint">
        <KeyButton k="H" active={controls} onClick={() => setControls((c) => !c)}>
          Controls
        </KeyButton>
        {controls && (
          <p className="caps">
            Drag to pan · scroll to zoom · click a planet · <span className="num">+ − 0</span> zoom
          </p>
        )}
      </div>
      <ControlBar />
      <div className="map-toggles">
        {drawer === "dispatches" && <DispatchDrawer />}
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
