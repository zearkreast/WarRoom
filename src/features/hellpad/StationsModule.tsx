import { Satellite, Timer } from "lucide-react";
import type { CSSProperties } from "react";
import { useCampaigns, useFreshPlanets, useSpaceStations } from "../../api/queries";
import type { TacticalAction } from "../../api/types";
import { regenPctPerHour } from "../../domain/biomes";
import { factionColor } from "../../domain/factions";
import { fmtPct } from "../../domain/format";
import { trackedPercent } from "../../domain/progress";
import { fmtLongDuration } from "../../domain/warHistory";
import { useUi } from "../../state";
import { FactionEmblem } from "../../ui/emblems";
import { Countdown, Empty, Loading, Meter, Panel, useNow } from "../../ui/primitives";
import { usePlanetRate } from "../planets/useFronts";
import { PlanetSphere } from "../planet/PlanetDetail";

/** Splits the API's `<span data-ah="1">…</span>` highlights into plain and emphasised runs. */
function Highlighted({ text }: { text: string }) {
  // The API sometimes doubles the closing bracket ("</span>>").
  const parts = text.replace(/(<\/span>)>+/g, "$1").split(/(<span[^>]*>[\s\S]*?<\/span>)/g);
  return (
    <>
      {parts.map((part, i) => {
        const m = /^<span[^>]*>([\s\S]*?)<\/span>$/.exec(part);
        return m ? (
          <b key={i} className="st-hl">
            {m[1]}
          </b>
        ) : (
          <span key={i}>{part}</span>
        );
      })}
    </>
  );
}

function actionState(a: TacticalAction, now: number): { label: string; tone: string } {
  const expire = Date.parse(a.statusExpire);
  if (a.status === 2) return { label: "Active", tone: "var(--good)" };
  if (a.status === 3 && expire > now) return { label: "On cooldown", tone: "var(--warn)" };
  const cost = a.costs[0];
  if (cost && cost.currentValue >= cost.targetValue) return { label: "Ready", tone: "var(--good)" };
  return { label: "Gathering", tone: "var(--f-humans)" };
}

function ActionCard({ a, now }: { a: TacticalAction; now: number }) {
  const cost = a.costs[0];
  const pct = cost?.targetValue ? (cost.currentValue / cost.targetValue) * 100 : 0;
  const remaining =
    cost && cost.deltaPerSecond > 0 && cost.currentValue < cost.targetValue
      ? (cost.targetValue - cost.currentValue) / cost.deltaPerSecond
      : null;
  const state = actionState(a, now);
  return (
    <article className="st-action">
      <header>
        <h3>{a.name}</h3>
        <span className="status-chip" style={{ "--tone": state.tone } as CSSProperties}>
          {state.label}
        </span>
      </header>
      <p className="st-strategic">
        <Highlighted text={a.strategicDescription} />
      </p>
      <p className="st-desc">{a.description}</p>
      {cost && (
        <div className="st-progress">
          <Meter value={pct} color="var(--f-humans)" size="l" label={`${a.name} readiness`} />
          <div className="st-progress-row num">
            <span>
              {remaining != null
                ? `Estimated ready in ${remaining < 3600 ? "under an hour" : fmtLongDuration(remaining)}`
                : state.label === "Active"
                  ? "Deployed"
                  : "Holding"}
            </span>
            <b>{fmtPct(pct, 3)}</b>
          </div>
        </div>
      )}
    </article>
  );
}

function StationPanel({
  planetIndex,
  electionEnd,
  actions,
}: {
  planetIndex: number;
  electionEnd: string;
  actions: TacticalAction[];
}) {
  const planets = useFreshPlanets().data;
  const campaigns = useCampaigns().data;
  const rate = usePlanetRate(planetIndex);
  const openDetail = useUi((s) => s.openDetail);
  const now = useNow(30_000);
  const p = planets?.find((x) => x.index === planetIndex);
  if (!p) return null;
  const front = campaigns?.some((c) => c.planet.index === planetIndex);
  const enemy = p.event?.faction ?? (p.currentOwner === "Humans" ? undefined : p.currentOwner);
  const pct = enemy ? trackedPercent(p) : 100;
  const jump = Date.parse(electionEnd);
  return (
    <div className="st">
      <section className="st-hero brackets" style={{ "--tone": factionColor(enemy ?? "Humans") } as CSSProperties}>
        <div className="st-hero-text">
          <button className="st-planet" onClick={() => openDetail(p.index)}>
            <FactionEmblem faction={enemy ?? p.currentOwner} size={30} />
            <span>
              <b>{p.name}</b>
              <span className="caps">{p.sector} sector</span>
            </span>
          </button>
          <div className="st-stats">
            <span className="num">
              <FactionEmblem faction="Humans" size={14} /> {fmtPct(pct, 2)}{" "}
              {front ? (p.event ? "defended" : "liberated") : "held"}
            </span>
            {front && (
              <span className="num" style={{ color: rate != null && rate > 0 ? "var(--good)" : "var(--text-dim)" }}>
                {rate == null ? "Rate measuring…" : `${rate > 0 ? "+" : ""}${rate.toFixed(2)}% / h`}
              </span>
            )}
            {enemy && (
              <span className="num" style={{ color: factionColor(enemy) }}>
                <FactionEmblem faction={enemy} size={14} /> -{regenPctPerHour(p).toFixed(2)}% / h regen
              </span>
            )}
          </div>
          <div className="st-jump">
            <Timer size={16} />
            <span className="caps">Next FTL jump</span>
            <b className="num">{jump > now ? <Countdown to={jump} /> : "Vote closing"}</b>
          </div>
        </div>
        <PlanetSphere planet={p} percent={pct} enemy={factionColor(enemy ?? "Humans")} size={200} />
      </section>
      <div className="st-actions">
        {actions.map((a) => (
          <ActionCard key={a.id32} a={a} now={now} />
        ))}
      </div>
    </div>
  );
}

export function StationsModule() {
  const { data, isLoading } = useSpaceStations();
  if (isLoading) return <Loading what="station telemetry" rows={3} />;
  if (!data?.length)
    return (
      <Panel title="Democracy Space Station" icon={Satellite}>
        <Empty>No station telemetry yet. It arrives with the next collector update.</Empty>
      </Panel>
    );
  return (
    <div className="st-list">
      {data.map((s) => (
        <StationPanel
          key={s.id32}
          planetIndex={s.planetIndex}
          electionEnd={s.electionEnd}
          actions={s.tacticalActions}
        />
      ))}
    </div>
  );
}
