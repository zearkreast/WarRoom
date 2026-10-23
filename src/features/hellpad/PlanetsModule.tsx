import { ArrowDownWideNarrow, Flame, Satellite, Search, Swords, Target, Users } from "lucide-react";
import { useMemo, useState, type CSSProperties } from "react";
import { useCampaigns, useFreshPlanets, useWar } from "../../api/queries";
import type { Faction, Planet } from "../../api/types";
import { biomeTint } from "../../domain/biomes";
import { FACTIONS, factionColor, factionLabel } from "../../domain/factions";
import { fmtInt, fmtPct, sestClock, sestDate } from "../../domain/format";
import {
  controlShares,
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
import { Empty, LiveDot, Loading, Tile, useNow } from "../../ui/primitives";
import { useMajorOrderPlanets, useStationPlanets } from "../planets/useFronts";
import { WarActivity } from "../overview/WarActivity";

const ORDER: Faction[] = ["Humans", "Terminids", "Automaton", "Illuminate"];

/** Faction control ring: one arc per faction, sized by its share of planets. */
function ControlRing({ shares, focus }: { shares: ReturnType<typeof controlShares>; focus: Faction }) {
  const r = 92;
  const c = 2 * Math.PI * r;
  const gap = 4;
  let offset = 0;
  const f = shares.find((s) => s.faction === focus)!;
  return (
    <div className="pi-ring">
      <svg viewBox="-110 -110 220 220" aria-hidden>
        <circle r={104} className="pi-ring-dots" />
        {shares.map((s) => {
          const len = Math.max(0, (s.pct / 100) * c - gap);
          const el = (
            <circle
              key={s.faction}
              r={r}
              fill="none"
              stroke={`var(--c-${s.faction.toLowerCase()})`}
              strokeWidth={s.faction === focus ? 12 : 7}
              strokeDasharray={`${len} ${c}`}
              strokeDashoffset={-offset}
              transform="rotate(-90)"
              opacity={s.faction === focus ? 1 : 0.55}
              className="pi-arc"
            />
          );
          offset += (s.pct / 100) * c;
          return el;
        })}
        <circle r={78} fill="var(--void)" />
      </svg>
      <div className="pi-ring-center">
        <FactionEmblem faction={focus} size={30} />
        <span className="caps">{factionLabel(focus)}</span>
        <b className="num" style={{ color: factionColor(focus) }}>
          {Math.round(f.pct)}%
        </b>
        <span className="num caps">{f.planets} planets</span>
        <span className="num pi-ring-players">
          <Users size={11} /> {fmtInt(f.players)}
        </span>
      </div>
    </div>
  );
}

function Hero({ planets, active }: { planets: Planet[]; active: Set<number> }) {
  const war = useWar().data;
  const now = useNow(1000);
  const shares = useMemo(() => controlShares(planets), [planets]);
  const [focus, setFocus] = useState<Faction>("Terminids");
  const sectors = new Set(planets.map((p) => p.sector)).size;
  return (
    <section className="pi-hero brackets">
      <div className="pi-hero-main">
        <h1 className="pi-title">Planets</h1>
        <p className="page-sub">Every planet in the Galactic War, with live liberation progress and faction control.</p>
        <div className="pi-tiles">
          <Tile label="Planets charted" value={planets.length} rule="var(--cta)" />
          <Tile label="Sectors" value={sectors} rule="var(--cta)" />
          <Tile label="Active fronts" value={active.size} rule="var(--cta)" />
          <Tile label="Helldivers online" value={war ? fmtInt(war.statistics.playerCount) : "—"} rule="var(--cta)" />
          <Tile
            label="Impact mod"
            value={war ? war.impactMultiplier.toFixed(6) : "—"}
            color="var(--info)"
            rule="var(--info)"
          />
        </div>
        <div className="pi-control">
          <span className="tile-label">Galactic control</span>
          <div className="pi-control-chips" role="radiogroup" aria-label="Faction">
            {shares.map((s) => (
              <button
                key={s.faction}
                role="radio"
                aria-checked={focus === s.faction}
                className={`pi-chip ${focus === s.faction ? "on" : ""}`}
                style={{ "--k": factionColor(s.faction) } as CSSProperties}
                onClick={() => setFocus(s.faction)}
              >
                <FactionEmblem faction={s.faction} size={15} />
                {FACTIONS[s.faction].label}
                <b className="num">{Math.round(s.pct)}%</b>
              </button>
            ))}
          </div>
        </div>
      </div>
      <ControlRing shares={shares} focus={focus} />
      <footer className="pd-hero-foot caps">
        <span>Satcom / Star chart index</span>
        <span className="pd-live">
          <LiveDot /> Live feed{" "}
          <span className="num">
            SEST {sestDate(now)} {sestClock(now)}
          </span>
        </span>
      </footer>
    </section>
  );
}

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
      <Hero planets={planets} active={active} />
      <WarActivity />
      <div className="pi-controls">
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
