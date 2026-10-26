import { Building2, Compass, Flag, Gauge, Lock, Shield, Timer, TrendingUp, Users } from "lucide-react";
import { useEffect, useState, type CSSProperties } from "react";
import { useLastUpdate, usePlanet } from "../../api/queries";
import type { Faction } from "../../api/types";
import { planetTint, resistanceLevel } from "../../domain/biomes";
import { cityHoursLeft, citiesOf, sizeLabel, type City } from "../../domain/cities";
import { factionColor, factionLabel } from "../../domain/factions";
import { fmtDuration, fmtInt, fmtPct } from "../../domain/format";
import { trackedPercent } from "../../domain/progress";
import type { CityReading } from "../../history/store";
import { useHistory } from "../../history/useHistory";
import { useUi, type CityRef } from "../../state";
import { Empty, Loading, Meter, Overlay, Panel, Tile, useNow } from "../../ui/primitives";
import { LiberationBar } from "../map/planetCard";
import { ActivitySection, ProgressChart } from "./Activity";
import { CityMark, enemyOfCities, STATE_LABEL } from "./Cities";

const DAY_MS = 86_400_000;
const REFRESH_MS = 5 * 60_000;

/** A city's last 24 hours of readings, re-read every 5 minutes. */
function useCityReadings(at: CityRef): { loading: boolean; rows: CityReading[] } {
  const store = useHistory((s) => s.store);
  const lastUpdate = useLastUpdate().data;
  const tick = useNow(REFRESH_MS);
  const [state, setState] = useState<{ loading: boolean; rows: CityReading[] }>({ loading: true, rows: [] });
  useEffect(() => {
    let live = true;
    void store
      .cityReadings(at.planet, at.region, (lastUpdate ?? Date.now()) - DAY_MS)
      .then((rows) => live && setState({ loading: false, rows }))
      .catch(() => live && setState((s) => ({ ...s, loading: false })));
    return () => {
      live = false;
    };
  }, [store, at.planet, at.region, tick, lastUpdate]);
  return state;
}

const signed = (n: number) => `${n >= 0 ? "+" : "−"}${fmtPct(Math.abs(n), 2)}/h`;

/** Where the city stands: its liberation while it's fought over, and what freeing it is worth. */
function CityStatus({ c, enemy }: { c: City; enemy: Faction | undefined }) {
  const fought = !!enemy && c.percent != null && (c.state === "liberating" || c.percent > 0);
  const res = c.resistance != null ? resistanceLevel(c.resistance) : null;
  const hours = cityHoursLeft(c);
  const tone = factionColor(c.holder);
  const state = c.waiting ? `Opens at ${fmtPct(c.opensAt!, 1)}` : STATE_LABEL[c.state];
  return (
    <Panel title="City status" icon={Building2} tone={tone} right={<span style={{ color: tone }}>{state}</span>}>
      <div className="pd-stack">
        {fought && (
          <Meter value={c.percent!} color="var(--f-humans)" rest={factionColor(enemy)} size="l" label="City liberated" />
        )}
        <div className="pd-tiles four">
          <Tile
            icon={Flag}
            label="Liberated"
            value={c.percent == null ? "—" : fmtPct(c.percent, 2)}
            color={fought ? "var(--f-humans)" : undefined}
          />
          <Tile
            icon={Gauge}
            label="Resistance"
            value={c.resistance == null ? "—" : `${fmtPct(c.resistance, 2)}/h`}
            sub={res ? <span style={{ color: res.color }}>{res.label}</span> : undefined}
          />
          <Tile
            icon={TrendingUp}
            label="Progress"
            value={!fought ? "—" : c.rate == null ? "Measuring…" : signed(c.rate)}
            color={fought && c.rate != null ? (c.rate >= 0 ? "var(--good)" : "var(--bad)") : undefined}
          />
          <Tile
            icon={Timer}
            label="Liberated in"
            value={hours != null ? fmtDuration(hours * 3_600_000) : fought && c.rate != null && c.rate < 0 ? "Losing ground" : "—"}
          />
        </div>
        <div className="pd-tiles three">
          <Tile icon={Users} label="Helldivers" value={fmtInt(c.players)} color="var(--cta)" />
          <Tile
            icon={TrendingUp}
            label="Planet boost"
            value={c.boost == null ? "—" : `+${fmtPct(c.boost, 1)}`}
            sub="planet liberation when freed"
            color={c.boost == null ? undefined : "var(--good)"}
          />
          {c.opensAt ? (
            <Tile icon={Lock} label="Opens at" value={fmtPct(c.opensAt, 1)} sub="planet liberation" />
          ) : (
            <Tile icon={Shield} label="Held by" value={factionLabel(c.holder)} color={tone} />
          )}
        </div>
        {(c.description || c.note) && (
          <div className="cd-about">
            {c.description && <p>{c.description}</p>}
            {c.note && <p className="cd-note">{c.note}</p>}
          </div>
        )}
      </div>
    </Panel>
  );
}

function Body({ at }: { at: CityRef }) {
  const p = usePlanet(at.planet);
  const openDetail = useUi((s) => s.openDetail);
  const openModule = useUi((s) => s.openModule);
  const readings = useCityReadings(at);
  if (!p) return <Loading what="city record" />;
  const c = citiesOf(p).find((x) => x.id === at.region);
  if (!c) return <Empty>This city is no longer listed on {p.name}.</Empty>;
  const enemy = enemyOfCities(p);
  const tint = planetTint(p);
  const style = {
    "--tone": factionColor(enemy ?? "Humans"),
    "--sky": tint.sky,
    "--ground": tint.ground,
    "--art": tint.art,
  } as CSSProperties;
  const progress = readings.rows.flatMap((r) => (r.percent == null ? [] : [{ at: r.at, percent: r.percent }]));
  const planetPct = enemy ? trackedPercent(p) : 100;
  return (
    <div className="pd">
      <section className="pd-hero brackets" style={style}>
        <div className="pd-hero-main">
          <nav className="pd-crumbs caps" aria-label="Breadcrumb">
            <button className="link" onClick={() => openModule("planets")}>
              Planets
            </button>
            <span>/</span>
            <button className="link" onClick={() => openDetail(p.index)}>
              {p.name}
            </button>
            <span>/</span>
            <span>Cities</span>
          </nav>
          <div className="pd-name-row">
            <CityMark stars={c.stars} state={c.state} size={46} holder={c.holder} />
            <h1 className="pd-name">{c.name}</h1>
          </div>
          <div className="pd-status caps">
            {c.waiting
              ? `Opens at ${fmtPct(c.opensAt!, 1)} planet liberation`
              : `${sizeLabel(c.size)} · ${enemy ? `${factionLabel(enemy)} front` : `${factionLabel(p.currentOwner)} control`}`}
          </div>
          <div className="pd-chips">
            <span className="chip">
              <Building2 size={13} /> {sizeLabel(c.size)}
            </span>
            <button className="chip" onClick={() => openDetail(p.index)}>
              <Compass size={13} /> {p.name} · {p.sector} sector
            </button>
            <span className="chip num">
              <Users size={13} /> {fmtInt(c.players)} <span className="muted">deployed</span>
            </span>
          </div>
        </div>
      </section>
      <CityStatus c={c} enemy={enemy} />
      {enemy && !!c.opensAt && (
        <Panel title={`${p.name} liberation`} icon={Compass} tone={factionColor(enemy)}>
          <div className="pd-stack">
            <LiberationBar p={p} pct={planetPct} enemy={enemy} />
            <p className="muted">
              {c.name} opens once {p.name} is {fmtPct(c.opensAt, 1)} liberated; the planet stands at{" "}
              {fmtPct(planetPct, 2)}.
            </p>
          </div>
        </Panel>
      )}
      <ActivitySection
        title="City activity · last 24 h"
        now={c.players}
        points={readings.rows}
        loading={readings.loading}
        empty={`No readings of ${c.name} in the last 24 hours. One is kept every few minutes while Helldivers are there or it's being liberated.`}
      >
        {progress.length > 0 && (
          <>
            <div className="pd-chart-label caps">Liberation</div>
            <ProgressChart points={progress} />
          </>
        )}
      </ActivitySection>
    </div>
  );
}

export function CityDetail() {
  const city = useUi((s) => s.city);
  const openCity = useUi((s) => s.openCity);
  if (!city) return null;
  return (
    <Overlay onClose={() => openCity(null)} label="City details">
      <Body key={`${city.planet}:${city.region}`} at={city} />
    </Overlay>
  );
}
