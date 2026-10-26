import { ArrowRight, FileText, History, Sparkle, Timer, Zap } from "lucide-react";
import { Fragment, useState, type CSSProperties, type MouseEvent } from "react";
import { useCampaigns, useEffects, useFreshPlanets, useSpaceStations } from "../../api/queries";
import { useStationJumps } from "../../api/stations";
import type { TacticalAction } from "../../api/types";
import { WIKI_LICENCE, type Source } from "../../data/gww";
import { PAST_STATIONS, STAR_OF_PEACE, type StationEvent, type StationRecord } from "../../data/stations";
import { regenPctPerHour } from "../../domain/biomes";
import { factionColor } from "../../domain/factions";
import { fmtDateTime, fmtPct } from "../../domain/format";
import { parseRich } from "../../domain/gwwText";
import { trackedPercent } from "../../domain/progress";
import type { StationLive } from "../../domain/snapshot";
import { fmtLongDuration } from "../../domain/warHistory";
import { useUi } from "../../state";
import { DssIcon, FactionEmblem, StationEmblem } from "../../ui/emblems";
import { Countdown, Empty, Loading, OutLink, Panel, StatusChip, useNow } from "../../ui/primitives";
import { actionIcon, dssState } from "../map/pawnArt";
import { usePlanetRate } from "../planets/useFronts";
import { PlanetSphere } from "../planet/PlanetDetail";
import { LOG_ADDRESS } from "./gwwNav";
import { CssHolo, DssHolo } from "./StationHolo";

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

/** Text with its `[[planet:Name|label]]` links opening the planet's options, in its owner's colour. */
function PlanetText({ text }: { text: string }) {
  const planets = useFreshPlanets().data;
  const openOptions = useUi((s) => s.openOptions);
  return (
    <>
      {parseRich(text).map((seg, i) => {
        if (!("kind" in seg)) return <Fragment key={i}>{seg.text}</Fragment>;
        const p = seg.kind === "planet" ? planets?.find((x) => x.name.toLowerCase() === seg.target.toLowerCase()) : undefined;
        if (!p) return <Fragment key={i}>{seg.label}</Fragment>;
        const open = (e: MouseEvent) => {
          // A keyboard click has no pointer position; open beside the link instead.
          const box = e.currentTarget.getBoundingClientRect();
          openOptions({ planet: p.index, x: e.clientX || box.left, y: e.clientY || box.bottom });
        };
        return (
          <button key={i} className="link st-planet-link" style={{ color: factionColor(p.currentOwner) }} onClick={open}>
            {seg.label}
          </button>
        );
      })}
    </>
  );
}

/** A bold term before the first ": ", when there is one. */
function Point({ text }: { text: string }) {
  const at = text.indexOf(": ");
  if (at < 0) return <PlanetText text={text} />;
  return (
    <>
      <b>{text.slice(0, at)}</b>: <PlanetText text={text.slice(at + 2)} />
    </>
  );
}

function Timeline({ events }: { events: StationEvent[] }) {
  return (
    <ol className="st-timeline">
      {events.map((e) => (
        <li key={e.when + e.text}>
          <span className="st-timeline-when caps num">{e.when}</span>
          <p>
            <PlanetText text={e.text} />
          </p>
        </li>
      ))}
    </ol>
  );
}

function Sources({ sources }: { sources: Source[] }) {
  return (
    <p className="st-source">
      Adapted from the Helldivers Wiki:{" "}
      {sources.map((s, i) => (
        <Fragment key={s.url}>
          {i > 0 && ", "}
          <OutLink href={s.url}>{s.title}</OutLink>
        </Fragment>
      ))}
      , under <OutLink href={WIKI_LICENCE.url}>{WIKI_LICENCE.title}</OutLink>.
    </p>
  );
}

function StarOfPeace() {
  return (
    <Panel
      title="Star of Peace"
      icon={Sparkle}
      tone="var(--warn)"
      right={<StatusChip tone="var(--warn)">{STAR_OF_PEACE.status}</StatusChip>}
    >
      <div className="st-sop">
        <p className="st-lead">
          <PlanetText text={STAR_OF_PEACE.summary} />
        </p>
        <ul className="st-points">
          {STAR_OF_PEACE.points.map((p) => (
            <li key={p}>
              <Point text={p} />
            </li>
          ))}
        </ul>
        <h3 className="st-sub caps">Operation Free Space and after</h3>
        <Timeline events={STAR_OF_PEACE.timeline} />
        <Sources sources={STAR_OF_PEACE.sources} />
      </div>
    </Panel>
  );
}

const DSS_STATE_LABEL = { active: "Action deployed", cooldown: "Cooling down", offline: "Offline", default: "Gathering" };

const isActive = (a: TacticalAction, now: number) => a.status === 2 && Date.parse(a.statusExpire) > now;
const isCooling = (a: TacticalAction, now: number) => a.status === 3 && Date.parse(a.statusExpire) > now;

type ActionMode = "active" | "cooling" | "ready" | "gathering";

function actionState(a: TacticalAction, now: number): { mode: ActionMode; label: string; tone: string } {
  if (isActive(a, now)) return { mode: "active", label: "Active", tone: "var(--good)" };
  if (isCooling(a, now)) return { mode: "cooling", label: "Cooldown", tone: "var(--warn)" };
  const cost = a.costs[0];
  if (cost && cost.currentValue >= cost.targetValue) return { mode: "ready", label: "Ready", tone: "var(--good)" };
  return { mode: "gathering", label: "Gathering", tone: "var(--f-humans)" };
}

function ActionIcon({ name, size }: { name: string; size: number }) {
  const href = actionIcon(name);
  if (!href) return <Zap size={size} aria-hidden />;
  return <img className="st-action-icon" src={href} width={size} height={size} alt="" />;
}

function ActionBar({ pct, mode, label }: { pct: number; mode: ActionMode; label: string }) {
  const v = Math.max(0, Math.min(100, pct));
  return (
    <div
      className={`st-bar ${mode}`}
      role="progressbar"
      aria-valuenow={Math.round(v)}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-label={label}
    >
      <div className="st-bar-fill" style={{ width: `${v}%` }} />
      {mode === "cooling" && <span className="st-bar-tag caps">Standby</span>}
    </div>
  );
}

function ActionCard({ a, now }: { a: TacticalAction; now: number }) {
  const cost = a.costs[0];
  const active = isActive(a, now);
  const cooling = isCooling(a, now);
  // A deployed action has spent its contributions; the game shows its bar full until it ends.
  const pct = active ? 100 : cost?.targetValue ? (cost.currentValue / cost.targetValue) * 100 : 0;
  const remaining =
    cost && cost.deltaPerSecond > 0 && cost.currentValue < cost.targetValue
      ? (cost.targetValue - cost.currentValue) / cost.deltaPerSecond
      : null;
  const state = actionState(a, now);
  return (
    <article className={`st-action ${state.mode}`} style={{ "--tone": state.tone } as CSSProperties}>
      <header>
        <ActionIcon name={a.name} size={34} />
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
          <ActionBar pct={pct} mode={state.mode} label={`${a.name} readiness`} />
          <div className="st-progress-row num">
            <span>
              {active ? (
                <>
                  Active · ends in <Countdown to={Date.parse(a.statusExpire)} />
                </>
              ) : cooling ? (
                <>
                  Ready again in <Countdown to={Date.parse(a.statusExpire)} />
                </>
              ) : remaining != null ? (
                `Estimated ready in ${remaining < 3600 ? "under an hour" : fmtLongDuration(remaining)}`
              ) : (
                "Holding"
              )}
            </span>
            <b>{fmtPct(pct, 3)}</b>
          </div>
        </div>
      )}
    </article>
  );
}

function JumpHistoryLink({ station }: { station: number }) {
  const jumps = useStationJumps(station).data ?? [];
  // One state change, so Back steps straight back to the station.
  const open = () =>
    useUi.setState({ view: "hellpad", module: "gww", gww: [{ kind: "home" }, { kind: "site", id: "log" }], options: null, detail: null });
  const last = jumps[0];
  return (
    <button className="st-jumplink" onClick={open}>
      <History size={18} aria-hidden />
      <span className="st-jumplink-text">
        <b className="caps">FTL jump history</b>
        <span className="num">
          {jumps.length === 0
            ? "No jumps logged yet"
            : `${jumps.length === 1 ? "1 jump" : `${jumps.length} jumps`} logged · last ${fmtDateTime(last.at)}`}
        </span>
      </span>
      <span className="st-jumplink-to caps num">
        {LOG_ADDRESS} <ArrowRight size={14} aria-hidden />
      </span>
    </button>
  );
}

function StationPanel({ station }: { station: StationLive }) {
  const { planetIndex, electionEnd, tacticalActions: actions } = station;
  const effects = [...(useEffects().data?.stations.get(station.id32) ?? [])].sort(
    (a, b) => Number(!!actionIcon(a.name)) - Number(!!actionIcon(b.name)),
  );
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
  const live = actions.filter((a) => isActive(a, now));
  const state = dssState(actions, now);
  return (
    <div className="st-layout">
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
            {live.length > 0 && (
              <ul className="st-live" aria-label="Active tactical actions">
                {live.map((a) => (
                  <li key={a.id32} title={a.name}>
                    <ActionIcon name={a.name} size={22} />
                    <span className="st-live-name">
                      <b className="caps">{a.name}</b>
                      <span className="caps">Over {p.name}</span>
                    </span>
                    <span className="num">
                      ends in <Countdown to={Date.parse(a.statusExpire)} />
                    </span>
                  </li>
                ))}
              </ul>
            )}
            {effects.length > 0 && (
              <ul className="st-effects" aria-label="Station effects">
                {effects.map((e) => (
                  <li key={e.id} className="caps" title={e.description}>
                    {actionIcon(e.name) && <ActionIcon name={e.name} size={14} />}
                    {e.name}
                  </li>
                ))}
              </ul>
            )}
          </div>
          <PlanetSphere planet={p} percent={pct} enemy={factionColor(enemy ?? "Humans")} size={200} />
        </section>
        <div className="st-actions">
          {actions.map((a) => (
            <ActionCard key={a.id32} a={a} now={now} />
          ))}
        </div>
        <JumpHistoryLink station={station.id32} />
        <StarOfPeace />
      </div>
      <DssHolo
        state={state}
        readout={[
          ["Orbit", p.name],
          [
            "Station",
            live.length > 0 ? (
              <span className="st-holo-icons" aria-label={live.map((a) => a.name).join(", ")}>
                {live.map((a) => (
                  <span key={a.id32} title={a.name}>
                    <ActionIcon name={a.name} size={22} />
                  </span>
                ))}
              </span>
            ) : (
              DSS_STATE_LABEL[state]
            ),
          ],
          ["Next jump", jump > now ? <Countdown to={jump} /> : "Vote closing"],
        ]}
      />
    </div>
  );
}

function PastStation({ s }: { s: StationRecord }) {
  return (
    <div className="st-layout">
      <div className="st">
        <section className="st-hero brackets st-past" style={{ "--tone": factionColor(s.faction) } as CSSProperties}>
          <div className="st-hero-text">
            <div className="st-planet">
              <StationEmblem station={s.art} size={34} color={factionColor(s.faction)} />
              <span>
                <b>{s.name}</b>
                <span className="caps">{s.status}</span>
              </span>
            </div>
            <p className="st-aka">Also called the {s.aka.join("; the ")}.</p>
            <p className="st-note">
              <PlanetText text={s.note} />
            </p>
          </div>
        </section>
        <div className="st-cols">
          <Panel title={s.operation} icon={History} tone={factionColor(s.faction)}>
            <Timeline events={s.timeline} />
          </Panel>
          <Panel title="Dossier" icon={FileText} tone={factionColor(s.faction)}>
            <div className="st-dossier">
              {s.sections.map((sec) => (
                <section key={sec.heading}>
                  <h3 className="st-sub caps">{sec.heading}</h3>
                  <p>
                    <PlanetText text={sec.text} />
                  </p>
                </section>
              ))}
              <Sources sources={s.sources} />
            </div>
          </Panel>
        </div>
      </div>
      {s.art === "css" && <CssHolo readout={s.readout} />}
    </div>
  );
}

interface Option {
  key: string;
  short: string;
  status: string;
  faction: string;
  live?: StationLive;
  past?: StationRecord;
}

/** Every station the war has had: the live ones from the API, then the ones only history remembers. */
export function StationsModule() {
  const { data, isLoading } = useSpaceStations();
  const [chosen, setChosen] = useState<string | null>(null);
  if (isLoading) return <Loading what="station telemetry" rows={3} />;
  const options: Option[] = [
    ...(data ?? []).map((s, i) => ({
      key: String(s.id32),
      short: i === 0 ? "DSS" : `Station ${i + 1}`,
      status: "Online",
      faction: "Humans",
      live: s,
    })),
    ...PAST_STATIONS.map((s) => ({ key: s.id, short: s.short, status: s.status, faction: s.faction, past: s })),
  ];
  const current = options.find((o) => o.key === chosen) ?? options[0];
  return (
    <div className="st-list">
      <div className="st-picker" role="tablist" aria-label="Space stations">
        {options.map((o) => (
          <button
            key={o.key}
            role="tab"
            aria-selected={o === current}
            className={`st-pick ${o === current ? "on" : ""}`}
            style={{ "--tone": factionColor(o.faction) } as CSSProperties}
            onClick={() => setChosen(o.key)}
          >
            <StationEmblem station={o.past ? "css" : "dss"} size={18} />
            <b className="caps">{o.short}</b>
            <span
              className="status-chip"
              style={{ "--tone": o.past ? "var(--bad)" : "var(--good)" } as CSSProperties}
            >
              {o.status}
            </span>
          </button>
        ))}
      </div>
      {!data?.length && (
        <Panel title="Democracy Space Station" icon={DssIcon}>
          <Empty>No station telemetry yet. It arrives with the next collector update.</Empty>
        </Panel>
      )}
      {current?.live && <StationPanel station={current.live} />}
      {current?.past && <PastStation s={current.past} />}
    </div>
  );
}
