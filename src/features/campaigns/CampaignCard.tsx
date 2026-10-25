import { Crosshair, Pin, ShieldAlert, Target, TrendingDown, TrendingUp, Users } from "lucide-react";
import type { CSSProperties } from "react";
import { planetTint, regenPctPerHour } from "../../domain/biomes";
import { factionColor, factionLabel } from "../../domain/factions";
import { fmtInt, fmtPct } from "../../domain/format";
import { useUi } from "../../state";
import { DssIcon, FactionEmblem } from "../../ui/emblems";
import { hazardIcon } from "../../ui/hazardIcons";
import { Countdown, Meter } from "../../ui/primitives";
import type { Front } from "../planets/useFronts";
import "./campaigns.css";

/** "Victory in …" / "Falls in …" above the card, if the rate says anything. */
function Eta({ f, deadline }: { f: Front; deadline?: number }) {
  if (f.rate == null) return null;
  if (f.kind === "defense") {
    const wins = f.victoryAt != null && f.failAt != null && f.victoryAt < f.failAt;
    return (
      <span className={`cc-eta ${wins ? "good" : "bad"}`}>
        {wins ? "Defended in" : "Falls in"}{" "}
        <b>
          <Countdown to={(wins ? f.victoryAt : f.failAt) ?? 0} />
        </b>
      </span>
    );
  }
  if (f.victoryAt)
    return (
      <span className="cc-eta good">
        Victory in{" "}
        <b>
          <Countdown to={f.victoryAt} />
        </b>
        {deadline != null && f.victoryAt > deadline && <span className="cc-late caps">After MO deadline</span>}
      </span>
    );
  if (f.failAt)
    return (
      <span className="cc-eta bad">
        Withdrawn in{" "}
        <b>
          <Countdown to={f.failAt} />
        </b>
      </span>
    );
  return null;
}

function Outlook({ f }: { f: Front }) {
  const holding = f.rate == null && f.kind === "liberation" && f.percent === 0;
  if (holding)
    return (
      <div className="cc-outlook holding caps">
        <Users size={14} /> Holding for reinforcement
      </div>
    );
  const Trend = f.rate != null && f.rate < 0 ? TrendingDown : TrendingUp;
  return (
    <div className="cc-outlook">
      <span className="num cc-pct">
        {fmtPct(f.percent, 4)} <span className="caps">{f.kind === "defense" ? "defended" : "liberated"}</span>
      </span>
      {f.rate == null ? (
        <span className="muted caps">Measuring rate…</span>
      ) : (
        <span className={`num cc-rate ${f.rate > 0 ? "good" : "bad"}`}>
          <Trend size={14} /> {f.rate > 0 ? "+" : ""}
          {f.rate.toFixed(3)}% / h
        </span>
      )}
    </div>
  );
}

export function CampaignCard({ f, totalPlayers, deadline }: { f: Front; totalPlayers: number; deadline?: number }) {
  const openOptions = useUi((s) => s.openOptions);
  const p = f.planet;
  const enemy = factionColor(f.enemy);
  const tint = planetTint(p);
  const regen = regenPctPerHour(p);
  const share = totalPlayers ? (f.players / totalPlayers) * 100 : 0;
  const style = { "--enemy": enemy, "--sky": tint.sky, "--ground": tint.ground, "--art": tint.art } as CSSProperties;
  const TypeIcon = f.kind === "defense" ? ShieldAlert : Crosshair;

  return (
    <article className="cc" style={style}>
      <div className="cc-above">
        <span className="cc-type caps">
          <TypeIcon size={13} /> {f.kind}
        </span>
        <Eta f={f} deadline={deadline} />
      </div>
      <button
        className={`cc-card ${f.inMajorOrder ? "mo" : ""}`}
        onClick={(e) => openOptions({ planet: p.index, x: e.clientX, y: e.clientY })}
        aria-label={`${p.name}, ${f.kind} against ${factionLabel(f.enemy)}. Open options`}
      >
        <header className="cc-head">
          <FactionEmblem faction={f.enemy} size={24} />
          <div className="cc-title">
            <h3 className="cc-name">{p.name}</h3>
            <div className="cc-sector caps">{p.sector} sector</div>
          </div>
          {f.pinned && <Pin size={14} className="cc-pin" aria-label="Pinned" />}
        </header>
        <div className="cc-art">
          <div className="cc-badges">
            {f.inMajorOrder && (
              <span className="cc-badge mo" title="Major Order target">
                <Target size={15} />
              </span>
            )}
            {f.station && (
              <span className="cc-badge dss" title="Democracy Space Station in orbit">
                <DssIcon size={15} />
              </span>
            )}
          </div>
          <span className="cc-biome caps">{p.biome?.name}</span>
          <div className="cc-hazards">
            {p.hazards.slice(0, 4).map((h) => {
              const Icon = hazardIcon(h.name);
              return (
                <span key={h.name} className="cc-hazard" title={`${h.name}: ${h.description}`}>
                  <Icon size={13} />
                </span>
              );
            })}
          </div>
        </div>
        <div className="cc-meter">
          <Meter
            value={f.percent}
            color="var(--f-humans)"
            rest={enemy}
            marker={f.projected}
            size="l"
            chevron
            label={`${p.name} progress`}
          />
        </div>
        <Outlook f={f} />
        <footer className="cc-foot num">
          <span className="cc-share" title="Share of all Helldivers online">
            <Meter value={share} color="var(--info)" size="s" />
            {share.toFixed(0)}%
          </span>
          <span title="Helldivers deployed">
            <Users size={12} /> {fmtInt(f.players)}
          </span>
          <span title="Super Earth progress per hour, before enemy regeneration" style={{ color: "var(--f-humans)" }}>
            <FactionEmblem faction="Humans" size={12} /> {f.rate == null ? "—" : `${(f.rate + regen).toFixed(2)}%`}
          </span>
          <span title="Enemy regeneration per hour" style={{ color: enemy }}>
            <FactionEmblem faction={f.enemy} size={12} /> {regen.toFixed(2)}%
          </span>
        </footer>
      </button>
    </article>
  );
}
