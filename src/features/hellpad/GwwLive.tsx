import { ArrowRight, ArrowUpRight } from "lucide-react";
import { useMemo, type MouseEvent, type ReactNode } from "react";
import { useAssignments, useEffects, useEpisodes, useFreshPlanets, useSpaceStations, useWar } from "../../api/queries";
import type { Planet } from "../../api/types";
import { regenPctPerHour } from "../../domain/biomes";
import { factionColor, factionLabel } from "../../domain/factions";
import { fmtCompact, fmtInt, fmtPct, fmtRate } from "../../domain/format";
import { orderName } from "../../domain/majorOrder";
import { liberationPercent } from "../../domain/progress";
import { useUi } from "../../state";
import { FactionEmblem } from "../../ui/emblems";
import { Countdown, LiveDot, useNow } from "../../ui/primitives";
import { useFronts, type Front } from "../planets/useFronts";
import { titleOf, useGwwNav } from "./gwwNav";

/** `actions` are link targets. */
function Live(props: { title: string; actions?: string[]; children: ReactNode }) {
  const { follow } = useGwwNav();
  return (
    <section className="gww-live" aria-label={`Live: ${props.title}`}>
      <header className="gww-live-head caps">
        <LiveDot /> Live from the front <span>· {props.title}</span>
      </header>
      {props.children}
      {props.actions && (
        <div className="gww-live-actions">
          {props.actions.map((t) => (
            <button key={t} className="gww-chip" onClick={(e) => follow(t, e)}>
              {titleOf(t)} <ArrowUpRight size={12} aria-hidden />
            </button>
          ))}
        </div>
      )}
    </section>
  );
}

function Waiting({ what }: { what: string }) {
  return <p className="gww-live-empty">{what}</p>;
}

function PlanetName({ p }: { p: Planet }) {
  const openOptions = useUi((s) => s.openOptions);
  return (
    <button
      className="gww-live-planet"
      style={{ color: factionColor(p.currentOwner) }}
      onClick={(e: MouseEvent) => openOptions({ planet: p.index, x: e.clientX, y: e.clientY })}
    >
      <FactionEmblem faction={p.currentOwner} size={14} />
      {p.name}
    </button>
  );
}

function Stat({ label, value, tone }: { label: string; value: ReactNode; tone?: string }) {
  return (
    <div className="gww-live-stat">
      <span className="caps">{label}</span>
      <b className="num" style={tone ? { color: tone } : undefined}>
        {value}
      </b>
    </div>
  );
}

function Rows({ children }: { children: ReactNode }) {
  return <ul className="gww-live-rows">{children}</ul>;
}

const count = (n: number, one: string, many = `${one}s`) => `${n} ${n === 1 ? one : many}`;

const rateTone = (rate: number | null) => (rate == null ? undefined : rate > 0 ? "var(--good)" : rate < 0 ? "var(--bad)" : undefined);

function FrontRow({ f, extra }: { f: Front; extra?: ReactNode }) {
  return (
    <li>
      <PlanetName p={f.planet} />
      <span className="num">{fmtPct(f.percent, 1)}</span>
      <span className="num" style={{ color: rateTone(f.rate) }}>
        {f.rate == null ? "measuring" : fmtRate(f.rate)}
      </span>
      {extra}
    </li>
  );
}

function LiveBasics() {
  const war = useWar().data;
  const { fronts, loading } = useFronts();
  const orders = useAssignments().data;
  if (!war || loading) return <Live title="The war today"><Waiting what="Waiting for the collector…" /></Live>;
  const defenses = fronts.filter((f) => f.kind === "defense").length;
  const order = orders?.[0];
  return (
    <Live title="The war today" actions={["app:campaigns", "app:overview"]}>
      <div className="gww-live-stats">
        <Stat label="Helldivers online" value={fmtInt(war.statistics.playerCount)} />
        <Stat label="Fronts" value={`${count(fronts.length - defenses, "liberation")} · ${count(defenses, "defense")}`} />
        <Stat label="Impact modifier" value={war.impactMultiplier.toFixed(4)} tone="var(--impact)" />
        {order && <Stat label="Major Order ends in" value={<Countdown to={Date.parse(order.expiration)} />} tone="var(--cta)" />}
      </div>
    </Live>
  );
}

function useAttacks(): { from: Planet; to: Planet }[] | undefined {
  const planets = useFreshPlanets().data;
  return useMemo(() => {
    if (!planets) return undefined;
    const byIndex = new Map(planets.map((p) => [p.index, p]));
    return planets.flatMap((from) =>
      from.attacking.flatMap((i) => {
        const to = byIndex.get(i);
        return to ? [{ from, to }] : [];
      }),
    );
  }, [planets]);
}

function LiveSupplyLines() {
  const attacks = useAttacks();
  return (
    <Live title="Attacks along the lines" actions={["app:map"]}>
      {!attacks ? (
        <Waiting what="Waiting for the collector…" />
      ) : attacks.length === 0 ? (
        <Waiting what="No planet is attacking another right now." />
      ) : (
        <Rows>
          {attacks.slice(0, 8).map(({ from, to }) => (
            <li key={`${from.index}-${to.index}`}>
              <PlanetName p={from} />
              <ArrowRight size={14} aria-label="attacks" />
              <PlanetName p={to} />
            </li>
          ))}
        </Rows>
      )}
    </Live>
  );
}

function LiveCampaignTypes() {
  const { fronts, loading } = useFronts();
  if (loading) return <Live title="Campaigns now"><Waiting what="Waiting for the collector…" /></Live>;
  const defenses = fronts.filter((f) => f.kind === "defense");
  return (
    <Live title="Campaigns now" actions={["app:campaigns"]}>
      <div className="gww-live-stats">
        <Stat label="Liberations" value={fronts.length - defenses.length} tone="var(--f-humans)" />
        <Stat label="Defenses" value={defenses.length} tone={defenses.length ? "var(--bad)" : undefined} />
        <Stat label="Major Order targets" value={fronts.filter((f) => f.inMajorOrder).length} tone="var(--cta)" />
      </div>
    </Live>
  );
}

function LiveLiberation() {
  const { fronts, loading } = useFronts();
  const top = fronts
    .filter((f) => f.kind === "liberation")
    .sort((a, b) => b.players - a.players)
    .slice(0, 5);
  return (
    <Live title="Busiest liberations" actions={["app:campaigns"]}>
      {loading ? (
        <Waiting what="Waiting for the collector…" />
      ) : top.length === 0 ? (
        <Waiting what="No liberation campaign is open right now." />
      ) : (
        <Rows>
          {top.map((f) => (
            <FrontRow
              key={f.planet.index}
              f={f}
              extra={
                <span className="muted num">
                  {f.victoryAt ? (
                    <>
                      falls in <Countdown to={f.victoryAt} />
                    </>
                  ) : (
                    `${fmtCompact(f.players)} Helldivers`
                  )}
                </span>
              }
            />
          ))}
        </Rows>
      )}
    </Live>
  );
}

function LiveResistance() {
  const { fronts, loading } = useFronts();
  const liberations = fronts.filter((f) => f.kind === "liberation");
  const losing = liberations.filter((f) => f.rate != null && f.rate < 0);
  const stiffest = [...liberations].sort((a, b) => regenPctPerHour(b.planet) - regenPctPerHour(a.planet)).slice(0, 4);
  return (
    <Live title="Resistance now" actions={["app:campaigns"]}>
      {loading ? (
        <Waiting what="Waiting for the collector…" />
      ) : liberations.length === 0 ? (
        <Waiting what="No liberation campaign is open right now." />
      ) : (
        <>
          <div className="gww-live-stats">
            <Stat label="Fronts losing ground" value={losing.length} tone={losing.length ? "var(--bad)" : "var(--good)"} />
            <Stat label="Liberation fronts" value={liberations.length} />
          </div>
          <p className="gww-live-sub caps">Stiffest resistance</p>
          <Rows>
            {stiffest.map((f) => (
              <FrontRow
                key={f.planet.index}
                f={f}
                extra={<span className="num" style={{ color: factionColor(f.enemy) }}>−{regenPctPerHour(f.planet).toFixed(2)}%/h</span>}
              />
            ))}
          </Rows>
        </>
      )}
    </Live>
  );
}

function LiveRegions() {
  const { fronts, loading } = useFronts();
  const regions = fronts
    .flatMap((f) =>
      f.planet.regions
        .filter((r) => r.isAvailable && r.health != null && r.maxHealth > 0)
        .map((r) => ({ f, r, freed: 100 - ((r.health ?? 0) / r.maxHealth) * 100 })),
    )
    .sort((a, b) => b.r.players - a.r.players)
    .slice(0, 6);
  return (
    <Live title="Cities in the fight" actions={["app:campaigns"]}>
      {loading ? (
        <Waiting what="Waiting for the collector…" />
      ) : regions.length === 0 ? (
        <Waiting what="No city is being fought over right now." />
      ) : (
        <Rows>
          {regions.map(({ f, r, freed }) => (
            <li key={`${f.planet.index}-${r.id}`}>
              <b className="caps">{r.name}</b>
              <span className="muted caps">{r.size.replace(/([a-z])([A-Z])/g, "$1 $2")}</span>
              <PlanetName p={f.planet} />
              <span className="num">{fmtPct(freed, 1)} freed</span>
              <span className="muted num">{fmtCompact(r.players)} Helldivers</span>
            </li>
          ))}
        </Rows>
      )}
    </Live>
  );
}

function LiveDefense() {
  const { fronts, loading } = useFronts();
  const planets = useFreshPlanets().data;
  const defenses = fronts.filter((f) => f.kind === "defense");
  const frontOf = new Map(fronts.map((f) => [f.planet.index, f]));
  return (
    <Live title="Defenses and their gambits" actions={["app:campaigns", "app:map"]}>
      {loading ? (
        <Waiting what="Waiting for the collector…" />
      ) : defenses.length === 0 ? (
        <Waiting what="No Super Earth planet is under attack right now." />
      ) : (
        <Rows>
          {defenses.map((f) => {
            const attackers = (planets ?? []).filter((p) => p.currentOwner !== "Humans" && p.attacking.includes(f.planet.index));
            return (
              <li key={f.planet.index} className="gww-live-defense">
                <PlanetName p={f.planet} />
                <span className="num">{fmtPct(f.percent, 1)} defended</span>
                {f.failAt && (
                  <span className="num" style={{ color: "var(--bad)" }}>
                    deadline in <Countdown to={f.failAt} />
                  </span>
                )}
                {f.projected != null && (
                  <span className="num" style={{ color: f.projected >= 100 ? "var(--good)" : "var(--bad)" }}>
                    {f.projected >= 100 ? "on pace to hold" : `on pace for ${fmtPct(Math.max(0, f.projected), 0)}`}
                  </span>
                )}
                {attackers.length > 0 && (
                  <span className="gww-live-gambit">
                    <span className="caps">Gambit</span>
                    {attackers.map((a) => (
                      <span key={a.index} className="gww-live-gambit-planet">
                        <PlanetName p={a} />
                        <span className="muted num">
                          {fmtPct(frontOf.get(a.index)?.percent ?? liberationPercent(a), 1)} liberated
                        </span>
                      </span>
                    ))}
                  </span>
                )}
              </li>
            );
          })}
        </Rows>
      )}
    </Live>
  );
}

function LiveImpact() {
  const war = useWar().data;
  return (
    <Live title="The modifier now" actions={["app:overview"]}>
      {!war ? (
        <Waiting what="Waiting for the collector…" />
      ) : (
        <div className="gww-live-stats">
          <Stat label="Impact modifier" value={war.impactMultiplier.toFixed(4)} tone="var(--impact)" />
          <Stat label="Helldivers online" value={fmtInt(war.statistics.playerCount)} />
          <Stat label="Missions won, all war" value={fmtCompact(war.statistics.missionsWon)} />
        </div>
      )}
    </Live>
  );
}

function LiveDss() {
  const stations = useSpaceStations().data;
  const planets = useFreshPlanets().data;
  const now = useNow(30_000);
  const station = stations?.[0];
  const planet = station && planets?.find((p) => p.index === station.planetIndex);
  const running = station?.tacticalActions.filter((a) => a.status === 2 && Date.parse(a.statusExpire) > now) ?? [];
  return (
    <Live title="The station now" actions={["app:stations"]}>
      {!stations ? (
        <Waiting what="Waiting for the collector…" />
      ) : !station ? (
        <Waiting what="The Democracy Space Station is out of service." />
      ) : (
        <div className="gww-live-stats">
          <Stat label="In orbit over" value={planet ? <PlanetName p={planet} /> : "Unknown planet"} />
          <Stat label="Next jump in" value={<Countdown to={Date.parse(station.electionEnd)} />} tone="var(--cta)" />
          <Stat
            label="Running"
            value={running.length ? running.map((a) => a.name).join(", ") : "No tactical action"}
            tone={running.length ? "var(--good)" : undefined}
          />
        </div>
      )}
    </Live>
  );
}

function LiveCampaigns() {
  const episodes = useEpisodes().data;
  const orders = useAssignments().data;
  const running = episodes?.find((e) => e.status === "active");
  const order = orders?.[0];
  return (
    <Live title="The story now" actions={["app:overview", "app:past"]}>
      {!episodes && !orders ? (
        <Waiting what="Waiting for the collector…" />
      ) : (
        <div className="gww-live-stats">
          <Stat label="Galactic Campaign" value={running?.title ?? "Between campaigns"} tone={running ? factionColor(running.faction) : undefined} />
          <Stat
            label="Major Order"
            value={order ? (orderName(episodes) ?? order.title ?? "Major Order") : "None issued"}
            tone={order ? "var(--cta)" : undefined}
          />
          {order && <Stat label="Ends in" value={<Countdown to={Date.parse(order.expiration)} />} />}
        </div>
      )}
    </Live>
  );
}

function LiveGloom() {
  const effects = useEffects().data;
  const { fronts } = useFronts();
  const inGloom = fronts.filter((f) => effects?.gloom.has(f.planet.index));
  return (
    <Live title="The Gloom now" actions={["app:map"]}>
      {!effects ? (
        <Waiting what="Waiting for the collector…" />
      ) : (
        <>
          <div className="gww-live-stats">
            <Stat label="Planets in or near the Gloom" value={effects.gloom.size} tone="var(--f-terminids)" />
            <Stat label="Of them, fronts" value={inGloom.length} />
          </div>
          {inGloom.length > 0 && (
            <Rows>
              {inGloom.slice(0, 6).map((f) => (
                <FrontRow key={f.planet.index} f={f} extra={<span className="muted caps">{factionLabel(f.enemy)}</span>} />
              ))}
            </Rows>
          )}
        </>
      )}
    </Live>
  );
}

const PANELS: Record<string, () => ReactNode> = {
  basics: LiveBasics,
  "supply-lines": LiveSupplyLines,
  "campaign-types": LiveCampaignTypes,
  liberation: LiveLiberation,
  resistance: LiveResistance,
  regions: LiveRegions,
  defense: LiveDefense,
  impact: LiveImpact,
  dss: LiveDss,
  campaigns: LiveCampaigns,
  gloom: LiveGloom,
};

export function LivePanel({ manual }: { manual: string }) {
  const Panel = PANELS[manual];
  return Panel ? <Panel /> : null;
}

export const hasLivePanel = (manual: string) => manual in PANELS;
