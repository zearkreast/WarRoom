import { useMemo, useState } from "react";
import { useWar } from "../../api/queries";
import type { Faction } from "../../api/types";
import { FACTIONS } from "../../domain/factions";
import { fmtInt } from "../../domain/format";
import { useSettings } from "../../state";
import { FactionEmblem } from "../../ui/emblems";
import { Hint } from "../../ui/Hint";
import { Empty, Loading } from "../../ui/primitives";
import { useFronts, type Front } from "../planets/useFronts";
import { CampaignCard } from "./CampaignCard";

type Filter = "all" | "mo" | "defense" | "pinned" | Faction;

/** Below this many divers a campaign counts as inactive (hidden unless enabled in Settings). */
const ACTIVE_PLAYERS = 100;

export const isInactive = (f: Front) =>
  f.players < ACTIVE_PLAYERS && !f.inMajorOrder && !f.pinned && f.kind !== "defense";

export function CampaignsPage() {
  const { fronts, loading } = useFronts();
  const war = useWar().data;
  const showInactive = useSettings((s) => s.showInactive);
  const setSetting = useSettings((s) => s.set);
  const [filter, setFilter] = useState<Filter>("all");

  const visible = useMemo(() => (showInactive ? fronts : fronts.filter((f) => !isInactive(f))), [fronts, showInactive]);
  const hidden = fronts.length - visible.length;
  const shown = visible.filter((f) =>
    filter === "all"
      ? true
      : filter === "mo"
        ? f.inMajorOrder
        : filter === "defense"
          ? f.kind === "defense"
          : filter === "pinned"
            ? f.pinned
            : f.enemy === filter,
  );
  const count = (pred: (f: Front) => boolean) => visible.filter(pred).length;
  const options: { id: Filter; label: string; n: number; faction?: Faction }[] = [
    { id: "all", label: "All", n: visible.length },
    { id: "mo", label: "Major Order", n: count((f) => f.inMajorOrder) },
    { id: "defense", label: "Defense", n: count((f) => f.kind === "defense") },
    ...(["Terminids", "Automaton", "Illuminate"] as Faction[]).map((fa) => ({
      id: fa as Filter,
      label: FACTIONS[fa].label,
      n: count((f) => f.enemy === fa),
      faction: fa,
    })),
    { id: "pinned", label: "Pinned", n: count((f) => f.pinned) },
  ];
  const totalPlayers = war?.statistics.playerCount ?? 0;

  return (
    <div className="page scroll starfield">
      <div className="page-inner">
        <div className="page-head">
          <div>
            <h1 className="page-title">Active campaigns</h1>
            <p className="page-sub">
              {fronts.length} fronts · {fmtInt(fronts.reduce((n, f) => n + f.players, 0))} Helldivers deployed
            </p>
          </div>
        </div>
        <Hint id="campaigns">
          Click a card for its options: pin it to the top, show it on the map, or open the planet's full record.
        </Hint>
        <div className="filter-row" role="toolbar" aria-label="Filter campaigns">
          {options
            .filter((o) => o.n > 0 || o.id === "all" || o.id === filter)
            .map((o) => (
              <button
                key={o.id}
                className={`filter-chip ${filter === o.id ? "on" : ""}`}
                onClick={() => setFilter(o.id)}
              >
                {o.faction && <FactionEmblem faction={o.faction} size={14} />}
                {o.label} <span className="count">{o.n}</span>
              </button>
            ))}
        </div>
        {loading ? (
          <Loading what="campaign data" rows={4} />
        ) : shown.length === 0 ? (
          <Empty>No campaigns match this filter.</Empty>
        ) : (
          <div className="cc-grid">
            {shown.map((f) => (
              <CampaignCard key={f.campaign.id} f={f} totalPlayers={totalPlayers} />
            ))}
          </div>
        )}
        {hidden > 0 && (
          <p className="cc-hidden caps">
            {hidden} low-activity campaign{hidden > 1 ? "s" : ""} hidden ·{" "}
            <button className="link" onClick={() => setSetting("showInactive", true)}>
              Show all
            </button>
          </p>
        )}
      </div>
    </div>
  );
}
