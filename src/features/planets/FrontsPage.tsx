import { useState } from "react";
import type { Faction } from "../../api/types";
import { FACTIONS } from "../../domain/factions";
import { Loading } from "../../ui/primitives";
import { PlanetCard } from "./PlanetCard";
import { useFronts } from "./useFronts";

type Filter = "all" | "defense" | "mo" | Faction;

export function FrontsPage() {
  const { fronts, loading } = useFronts();
  const [filter, setFilter] = useState<Filter>("all");
  const shown = fronts.filter((f) =>
    filter === "all" ? true : filter === "defense" ? f.kind === "defense" : filter === "mo" ? f.inMajorOrder : f.enemy === filter,
  );
  const options: { id: Filter; label: string }[] = [
    { id: "all", label: `All (${fronts.length})` },
    { id: "mo", label: "Major Order" },
    { id: "defense", label: "Defense" },
    ...(["Terminids", "Automaton", "Illuminate"] as Faction[]).map((f) => ({ id: f, label: FACTIONS[f].label })),
  ];
  return (
    <div className="page scroll starfield">
      <div className="page-inner">
        <h1 className="page-title">Active Fronts</h1>
        <div className="fronts-filter">
          {options.map((o) => (
            <button key={o.id} className={`hud-btn ${filter === o.id ? "on" : ""}`} onClick={() => setFilter(o.id)}>
              {o.label}
            </button>
          ))}
        </div>
        {loading ? (
          <Loading what="fronts" />
        ) : (
          <div className="fronts-grid">
            {shown.map((f) => (
              <PlanetCard key={f.campaign.id} f={f} />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
