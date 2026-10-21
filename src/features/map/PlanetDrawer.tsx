import { useState } from "react";
import { useCampaigns, usePlanets } from "../../api/queries";
import { factionColor } from "../../domain/factions";
import { fmtCompact, fmtInt, fmtPct, fmtRate } from "../../domain/format";
import { byIndex } from "../../domain/planets";
import { liberationPercent, trackedPercent } from "../../domain/progress";
import { useUi } from "../../state";
import { Countdown, FactionTag, ProgressBar, Stat } from "../../ui/primitives";
import { PlanetHistory } from "../history/PlanetHistory";
import { useMajorOrderPlanets, usePlanetRate } from "../planets/useFronts";

type Tab = "overview" | "history";

export function PlanetDrawer() {
  const { selectedPlanet, select } = useUi();
  const planets = usePlanets().data;
  const campaigns = useCampaigns().data;
  const mo = useMajorOrderPlanets();
  const rate = usePlanetRate(selectedPlanet);
  const [tab, setTab] = useState<Tab>("overview");

  if (selectedPlanet == null || !planets) return null;
  const campaign = campaigns?.find((c) => c.planet.index === selectedPlanet);
  const p = campaign?.planet ?? byIndex(planets).get(selectedPlanet);
  if (!p) return null;

  const enemy = p.event?.faction ?? (p.currentOwner === "Humans" ? undefined : p.currentOwner);
  const pct = trackedPercent(p);
  const s = p.statistics;

  return (
    <aside className="drawer panel">
      <header className="drawer-head">
        <div>
          <div className="drawer-name caps">{p.name}</div>
          <div className="muted caps drawer-sector">{p.sector} sector</div>
        </div>
        <button className="hud-btn" onClick={() => select(null)} aria-label="Close">
          ×
        </button>
      </header>
      <div className="drawer-tags">
        <FactionTag faction={p.currentOwner} />
        {mo.has(p.index) && <span className="chip" style={{ color: "var(--accent)" }}>Major Order</span>}
        {p.event && <span className="chip" style={{ color: "var(--f-terminids)" }}>Under attack</span>}
      </div>
      <div className="drawer-tabs">
        {(["overview", "history"] as Tab[]).map((t) => (
          <button key={t} className={tab === t ? "active" : ""} onClick={() => setTab(t)}>
            {t}
          </button>
        ))}
      </div>
      <div className="drawer-body scroll">
        {tab === "history" ? (
          <PlanetHistory planet={p.index} />
        ) : (
          <>
            {campaign && (
              <div className="drawer-progress">
                <div className="stat-label">{p.event ? "Defense" : "Liberation"}</div>
                <ProgressBar value={pct} color="var(--f-humans)" rest={factionColor(enemy)} tall />
                <div className="pc-stats">
                  <b className="num">{fmtPct(pct, 4)}</b>
                  <span className="num">{fmtRate(rate)}</span>
                  {p.event && (
                    <span className="num muted">
                      Ends <Countdown to={Date.parse(p.event.endTime)} />
                    </span>
                  )}
                </div>
              </div>
            )}
            {!campaign && p.currentOwner !== "Humans" && (
              <div className="muted">No active campaign. Liberation {fmtPct(liberationPercent(p), 1)}.</div>
            )}
            <div className="drawer-grid">
              <Stat label="Helldivers" value={fmtInt(s.playerCount)} />
              <Stat label="Success rate" value={`${s.missionSuccessRate}%`} />
              <Stat label="Missions won" value={fmtCompact(s.missionsWon)} />
              <Stat label="Missions lost" value={fmtCompact(s.missionsLost)} />
              <Stat label="Deaths" value={fmtCompact(s.deaths)} />
              <Stat label="Regen" value={`${((p.regenPerSecond / p.maxHealth) * 360000).toFixed(2)}%/h`} />
            </div>
            <div>
              <div className="stat-label">Biome</div>
              <div className="caps">{p.biome?.name ?? "Unknown"}</div>
              {p.biome?.description && <p className="muted drawer-desc">{p.biome.description}</p>}
            </div>
            <div>
              <div className="stat-label">Hazards</div>
              <ul className="hazards">
                {p.hazards.map((h) => (
                  <li key={h.name}>
                    <b className="caps">{h.name}</b>
                    <span className="muted">{h.description}</span>
                  </li>
                ))}
              </ul>
            </div>
            {p.regions.length > 0 && (
              <div>
                <div className="stat-label">Regions</div>
                <ul className="hazards">
                  {p.regions.map((r) => (
                    <li key={r.id}>
                      <b className="caps">{r.name}</b>
                      <span className="muted">
                        {r.size}
                        {r.health != null && ` · ${fmtPct(100 - (r.health / r.maxHealth) * 100, 1)}`}
                        {r.players > 0 && ` · ${fmtInt(r.players)} divers`}
                      </span>
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </>
        )}
      </div>
    </aside>
  );
}
