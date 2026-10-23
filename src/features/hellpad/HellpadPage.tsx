import {
  Archive,
  BookOpen,
  ChevronRight,
  Globe2,
  History,
  Lock,
  Rocket,
  Satellite,
  type LucideIcon,
} from "lucide-react";
import { useEpisodes, useSpaceStations } from "../../api/queries";
import { useHistory } from "../../history/useHistory";
import { useUi, type HellpadModule } from "../../state";
import { Hint } from "../../ui/Hint";
import { GwwModule, LegendModule } from "./GwwModule";
import { PastCampaignsModule } from "./PastCampaignsModule";
import { PlanetsModule } from "./PlanetsModule";
import { StationsModule } from "./StationsModule";
import "./hellpad.css";

const TITLES: Record<HellpadModule, string> = {
  planets: "Planets",
  stations: "Space stations",
  gww: "G.W.W.",
  campaigns: "Past campaigns",
  legend: "Icons legend",
};

function Tile(props: {
  icon: LucideIcon;
  title: string;
  text: string;
  meta?: string;
  onClick?: () => void;
  locked?: boolean;
}) {
  return (
    <button className={`hp-tile ${props.locked ? "locked" : ""}`} onClick={props.onClick} disabled={props.locked}>
      <span className="hp-tile-icon">
        <props.icon size={26} strokeWidth={1.5} />
      </span>
      <span className="hp-tile-text">
        <b>{props.title}</b>
        <span>{props.text}</span>
      </span>
      {props.locked ? (
        <span className="hp-tile-lock caps">
          <Lock size={12} /> Module building
          <span className="hp-segs">
            <i className="on" />
            <i className="on" />
            <i />
            <i />
          </span>
        </span>
      ) : (
        <span className="hp-tile-meta caps">
          {props.meta} <ChevronRight size={14} />
        </span>
      )}
    </button>
  );
}

function Hub() {
  const openModule = useUi((s) => s.openModule);
  const stations = useSpaceStations().data;
  const changes = useHistory((s) => s.changes);
  const ended = useEpisodes().data?.filter((e) => e.status !== "active").length;
  return (
    <>
      <div className="page-head">
        <div>
          <h1 className="page-title">Hellpad</h1>
          <p className="page-sub">
            Field modules for planning: the planet index, the Democracy Space Station and the war archives.
          </p>
        </div>
      </div>
      <Hint id="hellpad">Warbonds and Boosters are planned modules; they unlock in a later update.</Hint>
      <div className="hp-grid">
        <Tile
          icon={Globe2}
          title="Planets"
          text="Every planet by sector, with search, filters and live fronts."
          meta="Open index"
          onClick={() => openModule("planets")}
        />
        <Tile
          icon={Satellite}
          title="Space stations"
          text="Where the DSS is, its next FTL jump and tactical actions."
          meta={stations?.length ? `${stations.length} in orbit` : "Open"}
          onClick={() => openModule("stations")}
        />
        <Tile
          icon={History}
          title="G.W.W."
          text="Galactic war log: every planet that changed hands."
          meta={changes.length ? `${changes.length} records` : "Open"}
          onClick={() => openModule("gww")}
        />
        <Tile
          icon={Archive}
          title="Past campaigns"
          text="Every Galactic Campaign that has ended: its Major Orders, outcome and reward."
          meta={ended ? `${ended} campaigns` : "Open"}
          onClick={() => openModule("campaigns")}
        />
        <Tile
          icon={BookOpen}
          title="Icons legend"
          text="What every mark on the map and cards means."
          meta="Open"
          onClick={() => openModule("legend")}
        />
        <Tile icon={Rocket} title="Warbonds" text="Warbond pages, items and medal costs." locked />
        <Tile icon={Rocket} title="Boosters" text="Booster effects and where to find them." locked />
      </div>
    </>
  );
}

export function HellpadPage() {
  const { module, openModule } = useUi();
  return (
    <div className="page scroll starfield">
      <div className="page-inner">
        {module && (
          <nav className="hp-crumbs caps" aria-label="Breadcrumb">
            <button className="link" onClick={() => openModule(null)}>
              Hellpad
            </button>
            <ChevronRight size={14} />
            <span>{TITLES[module]}</span>
          </nav>
        )}
        {module === null && <Hub />}
        {module === "planets" && <PlanetsModule />}
        {module === "stations" && <StationsModule />}
        {module === "gww" && <GwwModule />}
        {module === "campaigns" && <PastCampaignsModule />}
        {module === "legend" && <LegendModule />}
      </div>
    </div>
  );
}
