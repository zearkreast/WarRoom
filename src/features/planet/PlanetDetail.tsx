import { Compass, MapPin, Mountain, Pin, PinOff, Satellite, Target, Users } from "lucide-react";
import type { CSSProperties } from "react";
import { useCampaigns, usePlanet } from "../../api/queries";
import type { Planet } from "../../api/types";
import { biomeTint } from "../../domain/biomes";
import { factionColor, factionLabel } from "../../domain/factions";
import { fmtInt, sestClock, sestDate } from "../../domain/format";
import { trackedPercent } from "../../domain/progress";
import { useSettings, useUi } from "../../state";
import { FactionEmblem } from "../../ui/emblems";
import { FeedTag } from "../../ui/feed";
import { CtaButton, Loading, Overlay, useNow } from "../../ui/primitives";
import { buildFront, useMajorOrderPlanets, usePlanetRate, useStationPlanets, type Front } from "../planets/useFronts";
import {
  DispatchSection,
  EnvironmentSection,
  FrontLineSection,
  LiberationSection,
  ServiceRecordSection,
  SupplyLinesSection,
  WarHistorySection,
} from "./sections";
import { usePlanetRecord } from "./usePlanetRecord";
import "./planet.css";

/** Planet sphere ringed by the liberation split: Super Earth arc vs the enemy's. */
export function PlanetSphere({
  planet,
  percent,
  enemy,
  size = 220,
}: {
  planet: Planet;
  percent: number;
  enemy: string;
  size?: number;
}) {
  const tint = biomeTint(planet.biome?.name);
  const r = 100;
  const c = 2 * Math.PI * r;
  const se = (Math.max(0, Math.min(100, percent)) / 100) * c;
  const id = `sph-${planet.index}`;
  return (
    <svg className="sphere" width={size} height={size} viewBox="-120 -120 240 240" aria-hidden>
      <defs>
        <radialGradient id={id} cx="35%" cy="30%" r="80%">
          <stop offset="0%" stopColor={tint.ground} stopOpacity={0.95} />
          <stop offset="55%" stopColor={tint.ground} stopOpacity={0.55} />
          <stop offset="100%" stopColor={tint.sky} />
        </radialGradient>
        <clipPath id={`${id}-clip`}>
          <circle r={84} />
        </clipPath>
      </defs>
      <circle r={114} className="sphere-dots" />
      <circle r={r} fill="none" stroke={enemy} strokeWidth={9} opacity={0.9} />
      <circle
        r={r}
        fill="none"
        stroke="var(--f-humans)"
        strokeWidth={9}
        strokeDasharray={`${se} ${c}`}
        transform="rotate(-90)"
        className="sphere-arc"
      />
      <circle r={88} fill="var(--void)" />
      <g clipPath={`url(#${id}-clip)`}>
        <circle r={84} fill={`url(#${id})`} />
        {[-60, -30, 0, 30, 60].map((y) => (
          <ellipse key={y} cx={0} cy={y} rx={Math.sqrt(84 * 84 - y * y)} ry={6} className="sphere-grid" />
        ))}
        {[-50, -20, 20, 50].map((x) => (
          <ellipse key={x} cx={0} cy={0} rx={Math.abs(x)} ry={84} className="sphere-grid" />
        ))}
        <circle r={84} fill="url(#sphere-shade)" />
      </g>
      <defs>
        <radialGradient id="sphere-shade" cx="70%" cy="75%" r="75%">
          <stop offset="40%" stopColor="#000" stopOpacity={0} />
          <stop offset="100%" stopColor="#000" stopOpacity={0.75} />
        </radialGradient>
      </defs>
    </svg>
  );
}

function statusLine(p: Planet, front: Front | null): string {
  const owner = factionLabel(p.currentOwner);
  if (front?.kind === "defense") return `Defense campaign · ${factionLabel(front.enemy)} attacking`;
  if (front) return `Liberation campaign · ${owner} control`;
  if (p.currentOwner === "Humans") return `Secured · Super Earth control`;
  return `${owner} control`;
}

function Hero({ p, front }: { p: Planet; front: Front | null }) {
  const { showOnMap, openModule } = useUi();
  const stations = useStationPlanets();
  const mo = useMajorOrderPlanets();
  const now = useNow(1000);
  const enemyFaction = front?.enemy ?? (p.currentOwner === "Humans" ? undefined : p.currentOwner);
  const enemy = factionColor(enemyFaction);
  const tone = factionColor(front?.enemy ?? p.currentOwner);
  const percent = front ? front.percent : trackedPercent(p);
  const tint = biomeTint(p.biome?.name);
  const style = { "--tone": tone, "--sky": tint.sky, "--ground": tint.ground } as CSSProperties;
  return (
    <section className="pd-hero brackets" style={style}>
      <div className="pd-hero-main">
        <nav className="pd-crumbs caps" aria-label="Breadcrumb">
          <button className="link" onClick={() => openModule("planets")}>
            Planets
          </button>
          <span>/</span>
          <span>{p.sector} sector</span>
        </nav>
        <div className="pd-name-row">
          <FactionEmblem faction={front?.enemy ?? p.currentOwner} size={46} />
          <h1 className="pd-name">{p.name}</h1>
        </div>
        <div className="pd-status caps">{statusLine(p, front)}</div>
        <div className="pd-chips">
          <span className="chip">
            <Compass size={13} /> {p.sector} sector
          </span>
          {p.biome?.name && (
            <span className="chip">
              <Mountain size={13} /> {p.biome.name}
            </span>
          )}
          <span className="chip num">
            <Users size={13} /> {fmtInt(p.statistics.playerCount)} <span className="muted">deployed</span>
          </span>
          {stations.has(p.index) && (
            <button className="chip chip-dss" onClick={() => openModule("stations")}>
              <Satellite size={13} /> Democracy Space Station
            </button>
          )}
          {mo.has(p.index) && (
            <span className="chip chip-mo">
              <Target size={13} /> Major Order target
            </span>
          )}
        </div>
        <CtaButton icon={MapPin} onClick={() => showOnMap(p.index)}>
          Show on map
        </CtaButton>
      </div>
      <div className="pd-hero-sphere">
        <PlanetSphere planet={p} percent={enemyFaction ? percent : 100} enemy={enemy} />
      </div>
      <footer className="pd-hero-foot caps">
        <span>
          Satcom / {p.sector} / {p.name}
        </span>
        <span className="pd-live">
          <FeedTag />
          <span className="num">
            SEST {sestDate(now)} {sestClock(now)}
          </span>
        </span>
      </footer>
    </section>
  );
}

function Body({ index }: { index: number }) {
  const p = usePlanet(index);
  const campaigns = useCampaigns().data;
  const rate = usePlanetRate(index);
  const mo = useMajorOrderPlanets();
  const stations = useStationPlanets();
  const record = usePlanetRecord(index);
  if (!p) return <Loading what="planet record" />;
  const campaign = campaigns?.find((c) => c.planet.index === index);
  const front = campaign ? buildFront(campaign, rate, mo, Date.now(), { stations }) : null;
  return (
    <div className="pd">
      <Hero p={p} front={front} />
      {front && <LiberationSection p={p} front={front} />}
      {front && <FrontLineSection p={p} front={front} record={record} />}
      <ServiceRecordSection p={p} />
      <WarHistorySection p={p} record={record} />
      <div className="pd-cols">
        <DispatchSection p={p} front={front} inMajorOrder={mo.has(p.index)} station={stations.has(p.index)} />
        <EnvironmentSection p={p} />
      </div>
      <SupplyLinesSection p={p} />
    </div>
  );
}

export function PlanetDetail() {
  const detail = useUi((s) => s.detail);
  const openDetail = useUi((s) => s.openDetail);
  const pinned = useSettings((s) => (detail == null ? false : s.pinned.includes(detail)));
  const togglePin = useSettings((s) => s.togglePin);
  if (detail == null) return null;
  return (
    <Overlay
      onClose={() => openDetail(null)}
      label="Planet details"
      footer={
        <button className={`hud-btn pd-pin ${pinned ? "on" : ""}`} onClick={() => togglePin(detail)}>
          {pinned ? <PinOff size={15} /> : <Pin size={15} />} {pinned ? "Unpin" : "Pin to top"}
        </button>
      }
    >
      <Body key={detail} index={detail} />
    </Overlay>
  );
}
