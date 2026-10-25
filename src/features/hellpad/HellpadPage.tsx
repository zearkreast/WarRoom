import {
  Archive,
  ArrowLeft,
  ChevronRight,
  Gift,
  Globe,
  Globe2,
  Lock,
  Radio,
  Rocket,
  type LucideIcon,
} from "lucide-react";
import type { CSSProperties, ReactNode } from "react";
import { useCampaigns, useEpisodes, useFreshPlanets, useSpaceStations, useWar } from "../../api/queries";
import { MANUALS } from "../../data/gww";
import { factionColor, factionLabel } from "../../domain/factions";
import { fmtInt } from "../../domain/format";
import { controlShares } from "../../domain/planetIndex";
import { useSettings, useUi, type HellpadModule } from "../../state";
import { DssIcon, FactionEmblem, type IconLike } from "../../ui/emblems";
import { useFeedStatus } from "../../ui/feed";
import { Countdown, LiveDot, StatusChip } from "../../ui/primitives";
import { GwwModule } from "./GwwModule";
import { PastCampaignsModule } from "./PastCampaignsModule";
import { PlanetsModule } from "./PlanetsModule";
import { StationsModule } from "./StationsModule";
import "./hellpad.css";

const MODULES: Record<HellpadModule, { title: string; path: string; color: string }> = {
  planets: { title: "Planets", path: "hp://planets", color: "var(--f-humans)" },
  stations: { title: "Space stations", path: "hp://stations", color: "var(--impact)" },
  gww: { title: "G.W.W.", path: "hp://gww", color: "var(--cta)" },
  campaigns: { title: "Past campaigns", path: "hp://archive", color: "var(--warn)" },
};

/** The Hellpad's mark: the four stratagem arrows a Helldiver keys in on it. */
function HellpadMark({ size = 64 }: { size?: number }) {
  return (
    <svg className="hp-mark" width={size} height={size} viewBox="0 0 64 64" aria-hidden>
      <path d="M14 4h36l10 10v36L50 60H14L4 50V14Z" fill="none" stroke="currentColor" strokeWidth="2.5" />
      <path d="M18 9h28l9 9v28l-9 9H18l-9-9V18Z" fill="currentColor" opacity="0.08" />
      <g fill="currentColor">
        <path d="M32 12l8 9h-16Z" />
        <path d="M52 32l-9 8v-16Z" />
        <path d="M32 52l-8-9h16Z" />
        <path d="M12 32l9-8v16Z" />
      </g>
      <rect x="28" y="28" width="8" height="8" fill="none" stroke="currentColor" strokeWidth="2" />
    </svg>
  );
}

const REINFORCE = ["up", "down", "right", "left", "up"] as const;

/** The Reinforce stratagem's code, keyed in over and over. */
function StratagemCode() {
  return (
    <div className="hp-code" aria-hidden>
      <span className="caps">Reinforce</span>
      <span className="hp-code-keys">
        {REINFORCE.map((d, i) => (
          <i key={i} className={`hp-key ${d}`} style={{ "--i": i } as CSSProperties} />
        ))}
      </span>
    </div>
  );
}

function Uplink() {
  const { state } = useFeedStatus(15_000);
  if (state === "offline") return <StatusChip tone="var(--bad)">Uplink lost</StatusChip>;
  if (state === "stale") return <StatusChip tone="var(--warn)">Uplink stale</StatusChip>;
  return (
    <span className="hp-uplink caps">
      <LiveDot /> Uplink live
    </span>
  );
}

function Mast() {
  const war = useWar().data;
  const fronts = useCampaigns().data?.length;
  return (
    <header className="hp-mast">
      <HellpadMark />
      <div className="hp-mast-text">
        <h1 className="hp-mast-title">Hellpad</h1>
        <p className="caps">Field terminal · Super Destroyer uplink</p>
        <div className="hp-mast-chips num">
          <Uplink />
          {fronts != null && <span>{fronts} fronts open</span>}
          {war && <span>{fmtInt(war.statistics.playerCount)} Helldivers deployed</span>}
        </div>
      </div>
      <StratagemCode />
    </header>
  );
}

/** One line on where the Democracy Space Station is and when it jumps next. */
function StationReport() {
  const openModule = useUi((s) => s.openModule);
  const station = useSpaceStations().data?.[0];
  const planet = useFreshPlanets().data?.find((p) => p.index === station?.planetIndex);
  if (!station || !planet) return null;
  const jump = Date.parse(station.electionEnd);
  return (
    <button className="hp-report" onClick={() => openModule("stations")}>
      <span className="hp-report-label caps">
        <Radio size={14} aria-hidden /> Station report
      </span>
      <span className="hp-report-text">
        The DSS holds orbit over <b>{planet.name}</b>, {planet.sector} sector.{" "}
        {jump > Date.now() ? (
          <>
            Next FTL jump in <b className="num"><Countdown to={jump} /></b>.
          </>
        ) : (
          "The destination vote is closing."
        )}
      </span>
      <ChevronRight size={16} aria-hidden />
    </button>
  );
}

function ModuleTile(props: {
  id: HellpadModule;
  icon: IconLike;
  text: string;
  meta: string;
  children?: ReactNode;
}) {
  const openModule = useUi((s) => s.openModule);
  const m = MODULES[props.id];
  return (
    <button className="hp-mod" style={{ "--mod": m.color } as CSSProperties} onClick={() => openModule(props.id)}>
      <span className="hp-mod-icon">
        <props.icon size={26} strokeWidth={1.5} />
      </span>
      <span className="hp-mod-body">
        <span className="hp-mod-path num">{m.path}</span>
        <b className="hp-mod-title">{m.title}</b>
        <span className="hp-mod-text">{props.text}</span>
        {props.children}
        <span className="hp-mod-go caps">
          {props.meta} <ChevronRight size={14} aria-hidden />
        </span>
      </span>
    </button>
  );
}

function LockedTile({ icon: Icon, title, text }: { icon: LucideIcon; title: string; text: string }) {
  return (
    <div className="hp-mod locked">
      <span className="hp-mod-icon">
        <Icon size={26} strokeWidth={1.5} />
      </span>
      <span className="hp-mod-body">
        <span className="hp-mod-path num">hp://{title.toLowerCase()}</span>
        <b className="hp-mod-title">{title}</b>
        <span className="hp-mod-text">{text}</span>
        <span className="hp-mod-go caps">
          <Lock size={12} aria-hidden /> Module building
          <span className="hp-segs" aria-hidden>
            <i className="on" />
            <i className="on" />
            <i />
            <i />
          </span>
        </span>
      </span>
    </div>
  );
}

function PlanetsTile() {
  const planets = useFreshPlanets().data;
  const shares = planets ? controlShares(planets) : [];
  return (
    <ModuleTile
      id="planets"
      icon={Globe2}
      text="Every planet by sector, with search, filters and live fronts."
      meta={planets ? `${planets.length} planets indexed` : "Open index"}
    >
      {planets && (
        <span className="hp-control">
          <span
            className="hp-control-bar"
            role="img"
            aria-label={`Planet control: ${shares.map((s) => `${factionLabel(s.faction)} ${Math.round(s.pct)}%`).join(", ")}`}
          >
            {shares.map((s) => (
              <i key={s.faction} style={{ flexGrow: s.planets, background: factionColor(s.faction) }} />
            ))}
          </span>
          <span className="hp-control-keys num" aria-hidden>
            {shares.map((s) => (
              <span key={s.faction} style={{ color: factionColor(s.faction) }}>
                <FactionEmblem faction={s.faction} size={12} /> {Math.round(s.pct)}%
              </span>
            ))}
          </span>
        </span>
      )}
    </ModuleTile>
  );
}

function StationsTile() {
  const stations = useSpaceStations().data;
  const planets = useFreshPlanets().data;
  const where = stations?.[0] && planets?.find((p) => p.index === stations[0].planetIndex)?.name;
  return (
    <ModuleTile
      id="stations"
      icon={DssIcon}
      text="Where the DSS is, its next FTL jump, its tactical actions and the Automaton station it destroyed."
      meta={stations?.length ? `${stations.length} in orbit` : "Open"}
    >
      {where && (
        <span className="hp-mod-live caps">
          <LiveDot /> DSS over {where}
        </span>
      )}
    </ModuleTile>
  );
}

function GwwTile() {
  const read = useSettings((s) => s.gwwRead);
  const done = MANUALS.filter((m) => read.includes(m.id)).length;
  return (
    <ModuleTile
      id="gww"
      icon={Globe}
      text="Galactic Wide Web: Super Earth's sites and ministries, training manuals tied to the live war, and the corporations."
      meta={done ? `${done} of ${MANUALS.length} manuals read` : "Open"}
    >
      <span className="hp-course" aria-hidden>
        {MANUALS.map((m) => (
          <i key={m.id} className={read.includes(m.id) ? "on" : ""} />
        ))}
      </span>
    </ModuleTile>
  );
}

function PastTile() {
  const ended = useEpisodes().data?.filter((e) => e.status !== "active");
  const last = ended?.reduce((a, b) => ((b.end ?? 0) > (a.end ?? 0) ? b : a), ended[0]);
  return (
    <ModuleTile
      id="campaigns"
      icon={Archive}
      text="Every Galactic Campaign that has ended: its Major Orders, outcome and reward."
      meta={ended?.length ? `${ended.length} campaigns` : "Open"}
    >
      {last && (
        <span className="hp-mod-live caps">
          Last: {last.title}
          <StatusChip tone={last.status === "won" ? "var(--good)" : "var(--bad)"}>{last.status}</StatusChip>
        </span>
      )}
    </ModuleTile>
  );
}

function Group({ title, count, children }: { title: string; count: number; children: ReactNode }) {
  return (
    <section className="hp-group">
      <h2 className="hp-group-title caps">
        {title} <span className="num">{count}</span>
      </h2>
      <div className="hp-grid">{children}</div>
    </section>
  );
}

function Hub() {
  return (
    <div className="hp-hub">
      <Mast />
      <StationReport />
      <Group title="Field modules" count={4}>
        <PlanetsTile />
        <StationsTile />
        <GwwTile />
        <PastTile />
      </Group>
      <Group title="In development" count={2}>
        <LockedTile icon={Rocket} title="Warbonds" text="Warbond pages, items and medal costs." />
        <LockedTile icon={Gift} title="Boosters" text="Booster effects and where to find them." />
      </Group>
    </div>
  );
}

/** The strip over an open module: back to the hub, and where the module sits. */
function ModuleBar({ module }: { module: HellpadModule }) {
  const openModule = useUi((s) => s.openModule);
  const m = MODULES[module];
  return (
    <nav className="hp-bar" aria-label="Breadcrumb" style={{ "--mod": m.color } as CSSProperties}>
      <button className="hp-bar-back" aria-label="Back to the Hellpad" title="Hellpad" onClick={() => openModule(null)}>
        <ArrowLeft size={16} />
      </button>
      <span className="hp-bar-path caps">
        <button className="link" onClick={() => openModule(null)}>
          Hellpad
        </button>
        <ChevronRight size={14} aria-hidden />
        <span aria-current="page">{m.title}</span>
      </span>
      <span className="hp-bar-addr num" aria-hidden>
        {m.path}
      </span>
    </nav>
  );
}

export function HellpadPage() {
  const { module } = useUi();
  return (
    <div className="page scroll starfield">
      <div className="page-inner">
        {module && <ModuleBar module={module} />}
        {module === null && <Hub />}
        {module === "planets" && <PlanetsModule />}
        {module === "stations" && <StationsModule />}
        {module === "gww" && <GwwModule />}
        {module === "campaigns" && <PastCampaignsModule />}
      </div>
    </div>
  );
}
