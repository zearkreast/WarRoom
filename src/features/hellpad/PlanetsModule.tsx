import { ArrowDownWideNarrow, Flame, Satellite, Search, Swords, Target, Users } from "lucide-react";
import { useMemo, useRef, useState, type CSSProperties } from "react";
import { useCampaigns, useFreshPlanets } from "../../api/queries";
import type { Faction, Planet } from "../../api/types";
import { biomeTint } from "../../domain/biomes";
import { FACTIONS, factionColor, factionLabel } from "../../domain/factions";
import { fmtInt, fmtPct } from "../../domain/format";
import {
  groupBySector,
  matchesFilter,
  matchesSearch,
  type PlanetFilter,
  type SectorSort,
} from "../../domain/planetIndex";
import { liberationPercent, trackedPercent } from "../../domain/progress";
import { useUi } from "../../state";
import { FactionEmblem } from "../../ui/emblems";
import { hazardIcon } from "../../ui/hazardIcons";
import { Empty, Loading } from "../../ui/primitives";
import { useMajorOrderPlanets, useStationPlanets } from "../planets/useFronts";
import { GalacticControl } from "./GalacticControl";

const ORDER: Faction[] = ["Humans", "Terminids", "Automaton", "Illuminate"];

function PlanetRow({ p, active, mo, station }: { p: Planet; active: boolean; mo: boolean; station: boolean }) {
  const openOptions = useUi((s) => s.openOptions);
  const enemy = p.event?.faction ?? (p.currentOwner === "Humans" ? undefined : p.currentOwner);
  const face = (enemy ?? p.currentOwner) as Faction;
  const tint = biomeTint(p.biome?.name);
  const pct = p.event ? trackedPercent(p) : liberationPercent(p);
  const status = active
    ? `${p.event ? "Defense" : "Liberation"} campaign · ${fmtPct(pct, 1)}`
    : `${p.currentOwner === "Humans" ? "Secured" : `${factionLabel(p.currentOwner)} control`} · ${p.biome?.name ?? "Unknown"}`;
  const style = { "--k": factionColor(face), "--sky": tint.sky, "--ground": tint.ground } as CSSProperties;
  return (
    <li>
      <button
        className={`pi-row ${active ? "active" : ""}`}
        style={style}
        onClick={(e) => openOptions({ planet: p.index, x: e.clientX, y: e.clientY })}
      >
        <FactionEmblem faction={face} size={26} />
        <span className="pi-row-text">
          <b className="pi-row-name">{p.name}</b>
          <span className="pi-row-status caps">
            {active && <span className="pi-dot" />}
            {status}
          </span>
        </span>
        <span className="pi-row-icons">
          {mo && (
            <span className="pi-icon mo" title="Major Order target">
              <Target size={13} />
            </span>
          )}
          {station && (
            <span className="pi-icon dss" title="Democracy Space Station">
              <Satellite size={13} />
            </span>
          )}
          {p.hazards.slice(0, 2).map((h) => {
            const Icon = hazardIcon(h.name);
            return (
              <span key={h.name} className="pi-icon hz" title={h.name}>
                <Icon size={13} />
              </span>
            );
          })}
          {p.statistics.playerCount > 0 && (
            <span className="pi-players num">
              <Users size={12} /> {fmtInt(p.statistics.playerCount)}
            </span>
          )}
        </span>
        {active && <span className="pi-row-progress" style={{ width: `${pct}%` }} />}
      </button>
    </li>
  );
}

const SORTS: { value: SectorSort; label: string }[] = [
  { value: "players", label: "Most Helldivers" },
  { value: "active", label: "Most active" },
  { value: "az", label: "A–Z" },
];

export function PlanetsModule() {
  const planets = useFreshPlanets().data;
  const campaigns = useCampaigns().data;
  const mo = useMajorOrderPlanets();
  const stations = useStationPlanets();
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<PlanetFilter>("all");
  const [sort, setSort] = useState<SectorSort>("players");
  const listRef = useRef<HTMLDivElement>(null);
  const active = useMemo(() => new Set((campaigns ?? []).map((c) => c.planet.index)), [campaigns]);

  const groups = useMemo(() => {
    if (!planets) return [];
    const shown = planets.filter((p) => matchesSearch(p, query) && matchesFilter(p, filter, active));
    return groupBySector(shown, active, sort);
  }, [planets, query, filter, sort, active]);

  if (!planets) return <Loading what="the star chart" rows={4} />;
  const searched = planets.filter((p) => matchesSearch(p, query));
  const count = (f: PlanetFilter) => searched.filter((p) => matchesFilter(p, f, active)).length;

  return (
    <div className="pi">
      <div className="page-head">
        <h1 className="page-title">
          Planets <span className="pi-count num">{planets.length}</span>
        </h1>
      </div>
      <GalacticControl
        planets={planets}
        onFilter={(f) => {
          setFilter(f);
          listRef.current?.scrollIntoView({ behavior: "smooth", block: "start" });
        }}
      />
      <div className="pi-controls" ref={listRef}>
        <label className="pi-search">
          <Search size={16} />
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search by planet, sector, biome or hazard"
            aria-label="Search planets"
          />
        </label>
        <label className="pi-sort">
          <ArrowDownWideNarrow size={16} />
          <select value={sort} onChange={(e) => setSort(e.target.value as SectorSort)} aria-label="Sort sectors">
            {SORTS.map((s) => (
              <option key={s.value} value={s.value}>
                {s.label}
              </option>
            ))}
          </select>
        </label>
      </div>
      <div className="filter-row" role="toolbar" aria-label="Filter planets">
        <button className={`filter-chip ${filter === "all" ? "on" : ""}`} onClick={() => setFilter("all")}>
          All <span className="count">{count("all")}</span>
        </button>
        <button className={`filter-chip ${filter === "active" ? "on" : ""}`} onClick={() => setFilter("active")}>
          <Swords size={14} /> Active fronts <span className="count">{count("active")}</span>
        </button>
        {ORDER.map((f) => (
          <button key={f} className={`filter-chip ${filter === f ? "on" : ""}`} onClick={() => setFilter(f)}>
            <FactionEmblem faction={f} size={14} /> {FACTIONS[f].label} <span className="count">{count(f)}</span>
          </button>
        ))}
      </div>
      {groups.length === 0 ? (
        <Empty>No planets match “{query}”.</Empty>
      ) : (
        groups.map((g) => (
          <section key={g.name} className="pi-sector" aria-label={`${g.name} sector`}>
            <header className="pi-sector-head" style={{ "--k": factionColor(g.owner) } as CSSProperties}>
              <span className="pi-sector-glyph">
                <FactionEmblem faction={g.owner} size={16} />
              </span>
              <h2>{g.name} sector</h2>
              {g.hottest && (
                <span className="pi-hot caps">
                  <Flame size={12} /> Hottest front
                </span>
              )}
              <span className="pi-sector-rule" />
              <span className="pi-sector-meta caps num">
                {g.active > 0 && <span className="on">{g.active} active</span>}
                <span>
                  {g.planets.length} planet{g.planets.length === 1 ? "" : "s"}
                </span>
                {g.players > 0 && (
                  <span className="pl">
                    <Users size={12} /> {fmtInt(g.players)}
                  </span>
                )}
              </span>
            </header>
            <ul className="pi-grid">
              {g.planets.map((p) => (
                <PlanetRow
                  key={p.index}
                  p={p}
                  active={active.has(p.index)}
                  mo={mo.has(p.index)}
                  station={stations.has(p.index)}
                />
              ))}
            </ul>
          </section>
        ))
      )}
    </div>
  );
}
