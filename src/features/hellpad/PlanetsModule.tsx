import { ArrowDownWideNarrow, Flame, Pin, Satellite, Search, Swords, Target, Users } from "lucide-react";
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
import { useSettings, useUi } from "../../state";
import { FactionEmblem } from "../../ui/emblems";
import { hazardIcon } from "../../ui/hazardIcons";
import { Empty, Loading } from "../../ui/primitives";
import { useMajorOrderPlanets, useStationPlanets } from "../planets/useFronts";
import { GalacticControl } from "./GalacticControl";

const ORDER: Faction[] = ["Humans", "Terminids", "Automaton", "Illuminate"];

function PlanetRow(props: { p: Planet; active: boolean; mo: boolean; station: boolean; pinned: boolean }) {
  const { p, active, mo, station, pinned } = props;
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
        className={`pi-row ${active ? "active" : ""} ${pinned ? "pinned" : ""}`}
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
          {pinned && (
            <span className="pi-icon pin" title="Pinned">
              <Pin size={13} />
            </span>
          )}
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
  const pinnedList = useSettings((s) => s.pinned);
  const pinned = useMemo(() => new Set(pinnedList), [pinnedList]);
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<PlanetFilter>("all");
  const [sort, setSort] = useState<SectorSort>("players");
  const listRef = useRef<HTMLDivElement>(null);
  const active = useMemo(() => new Set((campaigns ?? []).map((c) => c.planet.index)), [campaigns]);

  const groups = useMemo(() => {
    if (!planets) return [];
    const shown = planets.filter((p) => matchesSearch(p, query) && matchesFilter(p, filter, active, pinned));
    return groupBySector(shown, active, sort);
  }, [planets, query, filter, sort, active, pinned]);
  // Pinned planets also get a shelf above the sectors, in the order they were pinned.
  const shelf = useMemo(() => {
    if (!planets || filter === "pinned") return [];
    const byIdx = new Map(planets.map((p) => [p.index, p]));
    return pinnedList.flatMap((i) => {
      const p = byIdx.get(i);
      return p && matchesSearch(p, query) && matchesFilter(p, filter, active, pinned) ? [p] : [];
    });
  }, [planets, pinnedList, pinned, query, filter, active]);

  if (!planets) return <Loading what="the star chart" rows={4} />;
  const searched = planets.filter((p) => matchesSearch(p, query));
  const count = (f: PlanetFilter) => searched.filter((p) => matchesFilter(p, f, active, pinned)).length;
  const row = (p: Planet) => (
    <PlanetRow
      key={p.index}
      p={p}
      active={active.has(p.index)}
      mo={mo.has(p.index)}
      station={stations.has(p.index)}
      pinned={pinned.has(p.index)}
    />
  );

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
        <button className={`filter-chip ${filter === "pinned" ? "on" : ""}`} onClick={() => setFilter("pinned")}>
          <Pin size={14} /> Pinned <span className="count">{count("pinned")}</span>
        </button>
        {ORDER.map((f) => (
          <button key={f} className={`filter-chip ${filter === f ? "on" : ""}`} onClick={() => setFilter(f)}>
            <FactionEmblem faction={f} size={14} /> {FACTIONS[f].label} <span className="count">{count(f)}</span>
          </button>
        ))}
      </div>
      {shelf.length > 0 && (
        <section className="pi-sector pi-pinned" aria-label="Pinned planets">
          <header className="pi-sector-head" style={{ "--k": "var(--cta)" } as CSSProperties}>
            <span className="pi-sector-glyph">
              <Pin size={15} />
            </span>
            <h2>Pinned</h2>
            <span className="pi-sector-rule" />
            <span className="pi-sector-meta caps num">
              <span>
                {shelf.length} planet{shelf.length === 1 ? "" : "s"}
              </span>
            </span>
          </header>
          <ul className="pi-grid">{shelf.map(row)}</ul>
        </section>
      )}
      {groups.length === 0 ? (
        <Empty>
          {filter === "pinned" && !query
            ? "No pinned planets yet. Pick a planet and choose Pin to top."
            : `No planets match “${query}”.`}
        </Empty>
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
            <ul className="pi-grid">{g.planets.map(row)}</ul>
          </section>
        ))
      )}
    </div>
  );
}
