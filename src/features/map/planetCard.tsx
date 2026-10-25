import { Crosshair, FileText, MousePointerClick, Pin, PinOff, X } from "lucide-react";
import { useEffect, type CSSProperties } from "react";
import type { Planet } from "../../api/types";
import { biomeTint, planetTint, regenPctPerHour } from "../../domain/biomes";
import { hasPawn, type PlanetEffect } from "../../domain/effects";
import { factionColor, factionLabel } from "../../domain/factions";
import { fmtInt, fmtPct } from "../../domain/format";
import { anomalyOf, type Landmark } from "../../domain/galaxy";
import { liberationPercent, trackedPercent } from "../../domain/progress";
import { useSettings, useUi } from "../../state";
import { DssIcon, SubfactionEmblem } from "../../ui/emblems";
import { Meter } from "../../ui/primitives";
import { CityList } from "../planet/Cities";
import { useStationPlanets } from "../planets/useFronts";
import { PoiGlyph, poiImage, poiKind, poiOrigin } from "./poi";

export type Hover = { x: number; y: number } & (
  { planet: Planet; landmark?: never } | { landmark: Landmark; planet?: never }
);

function resistance(regen: number): { label: string; color: string } {
  if (regen < 1) return { label: "Low", color: "var(--good)" };
  if (regen < 2.5) return { label: "Average", color: "var(--warn)" };
  return { label: "High", color: "var(--bad)" };
}

const ANOMALY_KIND: Record<string, string> = {
  wormhole: "Wormhole",
  blackhole: "Black hole",
  fractured: "Fractured planet",
  scorched: "Scorched",
};

const enemyOf = (p: Planet) => p.event?.faction ?? (p.currentOwner === "Humans" ? undefined : p.currentOwner);

/** The colours and picture a planet's card is drawn with. */
function cardLook(p: Planet, effects?: PlanetEffect[]): CSSProperties {
  const tint = planetTint(p, effects);
  // As in the game, a point of interest's (or an anomaly's) picture replaces the biome's.
  const art = poiImage(
    (effects ?? []).filter((e) => e.kind === "poi"),
    anomalyOf(p, effects),
  );
  return {
    "--sky": tint.sky,
    "--ground": tint.ground,
    "--art": art ? `url("${art}")` : tint.art,
    "--tone": factionColor(enemyOf(p) ?? "Humans"),
  } as CSSProperties;
}

/** What the map's hover card and planet panel both show: picture, name, what's there and the fight. */
function PlanetSummary({ p, active, effects }: { p: Planet; active: boolean; effects?: PlanetEffect[] }) {
  const enemy = enemyOf(p);
  const anomaly = anomalyOf(p, effects);
  const kind = p.event
    ? "Defense"
    : active
      ? "Liberation"
      : anomaly
        ? ANOMALY_KIND[anomaly]
        : p.currentOwner === "Humans"
          ? "Secured"
          : "Enemy held";
  const shown = (effects ?? []).filter(hasPawn);
  const pois = shown.filter((e) => e.kind === "poi");
  // Black holes and broken worlds list what they are too, as the game's tooltip does.
  const rows = [...(effects ?? []).filter((e) => e.kind === "look" && poiKind(e.name) === "anomaly"), ...pois];
  const chips = shown.filter((e) => e.kind !== "poi");
  const pct = enemy ? trackedPercent(p) : 100;
  const regen = regenPctPerHour(p);
  const res = resistance(regen);
  return (
    <>
      <div className={`hc-art ${poiImage(pois, anomaly) ? "poi" : ""}`} />
      <div className="hc-head">
        <span className="hc-kind caps">{kind}</span>
        <b className="hc-name">{p.name}</b>
        <span className="hc-sector caps">{p.sector} sector</span>
      </div>
      {rows.length > 0
        ? rows.map((e) => (
            <div key={e.id} className="hc-poi">
              <div className="hc-poi-head caps">
                <svg viewBox="-12 -12 24 24" width={16} height={16} aria-hidden>
                  <PoiGlyph kind={poiKind(e.name)} faction={poiOrigin(poiKind(e.name), enemy)} size={24} />
                </svg>
                {e.name}
              </div>
              <p>{e.description}</p>
            </div>
          ))
        : p.biome?.description && <p className="hc-desc">{p.biome.description}</p>}
      {chips.length > 0 && (
        <ul className="hc-effects caps">
          {chips.map((e) => (
            <li key={e.id} className={e.kind}>
              {e.kind === "subfaction" && enemy && (
                <SubfactionEmblem name={e.name} faction={enemy} size={12} color="currentColor" />
              )}
              {e.name}
            </li>
          ))}
        </ul>
      )}
      {enemy && (
        <div className="hc-progress">
          <Meter value={pct} color="var(--f-humans)" rest={factionColor(enemy)} size="m" />
          <span className="num caps">
            {fmtPct(p.event ? pct : liberationPercent(p), 4)} {p.event ? "defended" : "liberated"}
          </span>
        </div>
      )}
      {enemy && (
        <div className="hc-row caps">
          <span>Resistance</span>
          <b style={{ color: res.color }}>
            {res.label} <span className="num">({regen.toFixed(2)}%)</span>
          </b>
        </div>
      )}
    </>
  );
}

export function HoverCard({ h, active, effects }: { h: Hover; active: boolean; effects?: PlanetEffect[] }) {
  const place = { left: Math.min(h.x + 18, window.innerWidth - 300), top: Math.max(8, h.y - 40) };
  if (h.landmark) {
    const tint = biomeTint(h.landmark.biome);
    const style = {
      ...place,
      "--sky": tint.sky,
      "--ground": tint.ground,
      "--tone": "var(--f-humans)",
    } as CSSProperties;
    return (
      <div className="hover-card panel glass" style={style} aria-hidden>
        <div className="hc-art" />
        <div className="hc-head">
          <span className="hc-kind caps">{h.landmark.biome}</span>
          <b className="hc-name">{h.landmark.name}</b>
          <span className="hc-sector caps">{h.landmark.sector} sector</span>
        </div>
        <p className="hc-desc">{h.landmark.note}</p>
        {h.landmark.pois && (
          <>
            <div className="hc-pois-head caps">Points of interest</div>
            {h.landmark.pois.map((poi) => (
              <div key={poi.name} className="hc-poi">
                <div className="hc-poi-head caps">
                  <svg viewBox="-12 -12 24 24" width={16} height={16} aria-hidden>
                    <PoiGlyph kind={poiKind(poi.name)} faction="Humans" size={24} />
                  </svg>
                  {poi.name}
                </div>
                <p>{poi.description}</p>
              </div>
            ))}
          </>
        )}
        <div className="hc-foot caps">No campaign data</div>
      </div>
    );
  }
  return (
    <div className="hover-card panel glass" style={{ ...place, ...cardLook(h.planet, effects) }} aria-hidden>
      <PlanetSummary p={h.planet} active={active} effects={effects} />
      <div className="hc-foot caps">
        <MousePointerClick size={14} /> Click for the planet panel
      </div>
    </div>
  );
}

/** A clicked planet, docked beside the map (a sheet over it on phones) with its actions. */
export function PlanetPanel(props: { p: Planet; active: boolean; effects?: PlanetEffect[]; onClose(): void }) {
  const { p, onClose } = props;
  const pinned = useSettings((s) => s.pinned.includes(p.index));
  const togglePin = useSettings((s) => s.togglePin);
  const { openDetail, focus, openModule } = useUi();
  const hasStation = useStationPlanets().has(p.index);
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && onClose();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);
  return (
    <aside className="planet-panel panel glass" style={cardLook(p, props.effects)} aria-label={`${p.name} panel`}>
      <button className="pp-close" onClick={onClose} aria-label="Close planet panel">
        <X size={16} />
      </button>
      <div className="pp-scroll">
        <PlanetSummary p={p} active={props.active} effects={props.effects} />
        {p.regions.length > 0 && (
          <section className="pp-cities" aria-label="Cities">
            <h3 className="caps">Cities</h3>
            <CityList p={p} />
          </section>
        )}
        <dl className="pp-stats">
          <div>
            <dt className="caps">Helldivers</dt>
            <dd className="num">{fmtInt(p.statistics.playerCount)}</dd>
          </div>
          <div>
            <dt className="caps">Held by</dt>
            <dd style={{ color: factionColor(p.currentOwner) }}>{factionLabel(p.currentOwner)}</dd>
          </div>
        </dl>
      </div>
      <div className="pp-actions">
        <button className="pp-action" onClick={() => togglePin(p.index)}>
          {pinned ? <PinOff size={16} /> : <Pin size={16} />}
          <span>{pinned ? "Unpin" : "Pin"}</span>
        </button>
        <button className="pp-action" onClick={() => focus(p.index)}>
          <Crosshair size={16} />
          <span>Focus</span>
        </button>
        <button className="pp-action" onClick={() => openDetail(p.index)}>
          <FileText size={16} />
          <span>Details</span>
        </button>
        <button
          className="pp-action"
          onClick={() => openModule("stations")}
          disabled={!hasStation}
          title={hasStation ? "Space station" : "No space station at this planet"}
        >
          <DssIcon size={16} />
          <span>Station</span>
        </button>
      </div>
    </aside>
  );
}
