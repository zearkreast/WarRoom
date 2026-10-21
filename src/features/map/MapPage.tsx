import { usePlanets } from "../../api/queries";
import type { Faction } from "../../api/types";
import { FACTIONS } from "../../domain/factions";
import { MajorOrderList } from "../major-order/MajorOrder";
import { PlanetCard } from "../planets/PlanetCard";
import { useFronts } from "../planets/useFronts";
import { GalaxyMap } from "./GalaxyMap";
import { PlanetDrawer } from "./PlanetDrawer";
import "./drawer.css";

function Legend() {
  const planets = usePlanets().data ?? [];
  const counts = new Map<Faction, number>();
  for (const p of planets) counts.set(p.currentOwner, (counts.get(p.currentOwner) ?? 0) + 1);
  return (
    <div className="legend panel">
      {(Object.keys(FACTIONS) as Faction[]).map((f) => (
        <span key={f} className="caps">
          <span className="dot" style={{ background: FACTIONS[f].color }} />
          {FACTIONS[f].label} <b className="num">{counts.get(f) ?? 0}</b>
        </span>
      ))}
    </div>
  );
}

export function MapPage() {
  const { fronts } = useFronts();
  return (
    <div className="map-page">
      <GalaxyMap />
      <div className="map-sidebar">
        <MajorOrderList />
        <div className="fronts-list scroll">
          {fronts.slice(0, 12).map((f) => (
            <PlanetCard key={f.campaign.id} f={f} compact />
          ))}
        </div>
      </div>
      <Legend />
      <PlanetDrawer />
    </div>
  );
}
