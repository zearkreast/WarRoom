import {
  Activity,
  CalendarClock,
  Crosshair,
  Flag,
  FlagOff,
  Heart,
  History,
  LineChart as LineIcon,
  Medal,
  Radio,
  Route,
  Shield,
  ShieldCheck,
  Skull,
  Timer,
  TrendingUp,
  TriangleAlert,
  Users,
  CloudSun,
  CloudFog,
  CloudLightning,
  Landmark,
  Orbit,
} from "lucide-react";
import { useMemo, type CSSProperties, type ReactNode } from "react";
import {
  Area,
  CartesianGrid,
  ComposedChart,
  Line,
  ReferenceLine,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { axisProps, fmtHourTick, gridProps, HOUR, hourTicks, tooltipProps } from "../../ui/chart";
import { useCampaigns, useEffects, useFreshPlanets } from "../../api/queries";
import type { EffectKind } from "../../domain/effects";
import type { Faction, Planet } from "../../api/types";
import { regenPctPerHour } from "../../domain/biomes";
import { factionColor, factionLabel } from "../../domain/factions";
import { fmtCompact, fmtDate, fmtDateTime, fmtInt, fmtPct, fmtTimeLeft } from "../../domain/format";
import { liberationPercent } from "../../domain/progress";
import { buildWarHistory, defenseEvents, fmtLongDuration, type LogEntry } from "../../domain/warHistory";
import { useUi } from "../../state";
import { DssIcon, FactionEmblem, type IconLike, SeafEmblem, SubfactionEmblem } from "../../ui/emblems";
import { hazardIcon } from "../../ui/hazardIcons";
import { Countdown, Empty, Meter, Panel, Tile, useNow } from "../../ui/primitives";
import type { Front } from "../planets/useFronts";
import { PoiGlyph, poiKind, poiOrigin } from "../map/poi";
import { augmentedStratagem, stratagemIcon } from "../map/stratagemArt";
import { CityList } from "./Cities";
import { CircumvallationNote, useCircumvallation } from "./Circumvallation";
import type { PlanetRecord } from "./usePlanetRecord";

export function LiberationSection({ p, front }: { p: Planet; front: Front }) {
  const enemy = factionColor(front.enemy);
  const regen = regenPctPerHour(p);
  const hp = front.kind === "defense" && p.event ? p.event : p;
  const verb = front.kind === "defense" ? "Defended" : "Liberated";
  return (
    <Panel title={front.kind === "defense" ? "Defense progress" : "Liberation progress"} icon={Activity} tone={enemy}>
      <div className="pd-stack">
        <Meter
          value={front.percent}
          color="var(--f-humans)"
          rest={enemy}
          marker={front.projected}
          size="l"
          chevron
          label="Progress"
        />
        <div className="pd-tiles four">
          <Tile icon={Heart} label="Planet HP" value={`${fmtInt(hp.health)} / ${fmtInt(hp.maxHealth)}`} />
          <Tile icon={Flag} label={verb} value={fmtPct(front.percent, 4)} />
          <Tile
            icon={TrendingUp}
            label={`${front.kind === "defense" ? "Defense" : "Liberation"} rate`}
            value={front.rate == null ? "Measuring…" : `${front.rate > 0 ? "+" : ""}${front.rate.toFixed(2)}% / h`}
            color={front.rate == null ? undefined : front.rate > 0 ? "var(--good)" : "var(--bad)"}
          />
          <Tile
            icon={Timer}
            label="Prediction"
            value={
              front.victoryAt ? (
                <Countdown to={front.victoryAt} />
              ) : front.failAt ? (
                <Countdown to={front.failAt} />
              ) : (
                "Stalled"
              )
            }
            sub={
              front.victoryAt
                ? "until victory"
                : front.failAt
                  ? front.kind === "defense"
                    ? "until it falls"
                    : "until withdrawal"
                  : "no progress"
            }
            color={front.victoryAt ? "var(--good)" : front.failAt ? "var(--bad)" : undefined}
          />
        </div>
        <div className="pd-tiles three">
          <div className="pd-pill" style={{ color: enemy }}>
            <FactionEmblem faction={front.enemy} size={18} /> <span className="num">-{regen.toFixed(2)}% / h</span>
            <span className="muted caps">enemy regen</span>
          </div>
          <div className="pd-pill" style={{ color: "var(--f-humans)" }}>
            <FactionEmblem faction="Humans" size={18} />
            <span className="num">{front.rate == null ? "—" : `+${(front.rate + regen).toFixed(2)}% / h`}</span>
            <span className="muted caps">Helldiver impact</span>
          </div>
          <div className="pd-pill" style={{ color: "var(--cta)" }}>
            <Users size={16} /> <span className="num">{fmtInt(front.players)}</span>
            <span className="muted caps">deployed</span>
          </div>
        </div>
      </div>
    </Panel>
  );
}

export function FrontLineSection({ p, front, record }: { p: Planet; front: Front; record: PlanetRecord }) {
  const now = Date.now();
  const data = useMemo(() => {
    const past = record.samples.map((s) => ({ at: s.at, players: s.players, percent: s.percent }));
    const last = past[past.length - 1];
    const future: { at: number; projection: number }[] = [];
    if (front.rate != null) {
      const base = last?.percent ?? front.percent;
      const t0 = last?.at ?? now;
      for (let h = 0; h <= 24; h++)
        future.push({ at: t0 + h * HOUR, projection: Math.max(0, Math.min(100, base + front.rate * h)) });
    }
    return [...past.map((d, i) => (i === past.length - 1 ? { ...d, projection: d.percent } : d)), ...future.slice(1)];
  }, [record.samples, front.rate, front.percent, now]);

  const nowAt = record.samples[record.samples.length - 1]?.at ?? now;
  const domain: [number, number] = [data[0]?.at ?? now, data[data.length - 1]?.at ?? now];
  const ticks = hourTicks(domain[0], domain[1]);
  const first = record.samples[0];
  const change = first ? front.percent - first.percent : null;
  const peak = record.samples.reduce((m, s) => Math.max(m, s.players), 0);
  const regen = regenPctPerHour(p);
  const proj = front.rate == null ? null : Math.max(0, Math.min(100, front.percent + front.rate * 24));

  return (
    <Panel title="Front line activity · last and next 24 h" icon={LineIcon}>
      <div className="pd-stack">
        <div className="pd-heads">
          <div>
            <span className="pd-key" style={{ "--k": "var(--f-humans)" } as CSSProperties}>
              {front.kind === "defense" ? "Defense" : "Liberation"} change
            </span>
            <b
              className="num"
              style={{ color: change == null ? undefined : change >= 0 ? "var(--good)" : "var(--bad)" }}
            >
              {change == null ? "—" : `${change >= 0 ? "+" : ""}${change.toFixed(2)}%`}
            </b>
          </div>
          <div>
            <span className="pd-key" style={{ "--k": "var(--cta)" } as CSSProperties}>
              Peak Helldivers
            </span>
            <b className="num">{peak ? fmtInt(peak) : "—"}</b>
          </div>
          <div>
            <span className="pd-key" style={{ "--k": "var(--bad)" } as CSSProperties}>
              Enemy impact
            </span>
            <b className="num" style={{ color: "var(--bad)" }}>
              -{regen.toFixed(2)}%/h
            </b>
          </div>
          <div>
            <span className="pd-key dashed" style={{ "--k": "var(--f-humans)" } as CSSProperties}>
              24 h projection
            </span>
            <b className="num">{proj == null ? "—" : fmtPct(proj, 2)}</b>
            {front.victoryAt && (
              <span className="pd-sub good caps">
                {front.kind === "defense" ? "Defended" : "Liberated"} in <Countdown to={front.victoryAt} />
              </span>
            )}
          </div>
        </div>
        {record.samples.length >= 2 ? (
          <div className="pd-charts">
            <div className="pd-chart-label caps">Progress</div>
            <ResponsiveContainer width="100%" height={170}>
              <ComposedChart data={data} syncId="pd-front" margin={{ top: 8, right: 8, bottom: 0, left: 0 }}>
                <CartesianGrid {...gridProps} />
                <XAxis
                  dataKey="at"
                  type="number"
                  scale="time"
                  domain={domain}
                  ticks={ticks}
                  tickFormatter={fmtHourTick}
                  hide
                />
                <YAxis
                  domain={[0, 100]}
                  ticks={[0, 25, 50, 75, 100]}
                  tickFormatter={(v) => `${v}%`}
                  {...axisProps}
                  width={44}
                />
                <Tooltip
                  {...tooltipProps}
                  formatter={(v, name) => [fmtPct(Number(v), 2), name === "percent" ? "Progress" : "Projection"]}
                />
                <Line
                  dataKey="percent"
                  stroke="var(--c-humans)"
                  strokeWidth={2}
                  dot={false}
                  isAnimationActive={false}
                />
                <Line
                  dataKey="projection"
                  stroke="var(--c-humans)"
                  strokeWidth={2}
                  strokeDasharray="4 4"
                  dot={false}
                  isAnimationActive={false}
                />
                <ReferenceLine
                  x={nowAt}
                  stroke="var(--text-dim)"
                  label={{ value: "Now", fill: "var(--text-dim)", fontSize: 10, position: "insideTopRight" }}
                />
              </ComposedChart>
            </ResponsiveContainer>
            <div className="pd-chart-label caps">Helldivers deployed</div>
            <ResponsiveContainer width="100%" height={120}>
              <ComposedChart data={data} syncId="pd-front" margin={{ top: 4, right: 8, bottom: 0, left: 0 }}>
                <defs>
                  <linearGradient id="pd-players" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stopColor="var(--cta)" stopOpacity={0.28} />
                    <stop offset="100%" stopColor="var(--cta)" stopOpacity={0.02} />
                  </linearGradient>
                </defs>
                <CartesianGrid {...gridProps} />
                <XAxis
                  dataKey="at"
                  type="number"
                  scale="time"
                  domain={domain}
                  ticks={ticks}
                  tickFormatter={fmtHourTick}
                  {...axisProps}
                />
                <YAxis tickFormatter={(v) => fmtCompact(Number(v))} {...axisProps} width={44} />
                <Tooltip {...tooltipProps} formatter={(v) => [fmtInt(Number(v)), "Helldivers"]} />
                <Area
                  dataKey="players"
                  stroke="var(--cta)"
                  strokeWidth={2}
                  fill="url(#pd-players)"
                  isAnimationActive={false}
                  connectNulls={false}
                />
                <ReferenceLine x={nowAt} stroke="var(--text-dim)" />
              </ComposedChart>
            </ResponsiveContainer>
          </div>
        ) : (
          <Empty>
            {record.loading
              ? "Loading telemetry…"
              : "Not enough readings yet; the chart fills in as the collector records this front."}
          </Empty>
        )}
      </div>
    </Panel>
  );
}

export function ServiceRecordSection({ p }: { p: Planet }) {
  const s = p.statistics;
  const missions = s.missionsWon + s.missionsLost;
  const kills: { f: Faction; n: number }[] = [
    { f: "Automaton", n: s.automatonKills },
    { f: "Terminids", n: s.terminidKills },
    { f: "Illuminate", n: s.illuminateKills },
  ].sort((a, b) => b.n - a.n) as { f: Faction; n: number }[];
  const totalKills = kills.reduce((n, k) => n + k.n, 0);
  const maxKills = Math.max(1, kills[0].n);
  const accuracy = s.bulletsFired ? (s.bulletsHit / s.bulletsFired) * 100 : 0;
  // Older collector builds trimmed these counters from the snapshot; they read as zero until it updates.
  const combat = s.bulletsFired > 0 || totalKills > 0;
  if (!missions && !totalKills) {
    return (
      <Panel title="Service record" icon={Medal}>
        <Empty>No missions recorded on this planet yet.</Empty>
      </Panel>
    );
  }
  return (
    <Panel title="Service record" icon={Medal}>
      <div className="pd-stack">
        <div className={combat ? "pd-tiles two" : "pd-tiles"}>
          <div className="pd-card">
            <div className="pd-card-head">
              <span className="tile-label">Missions</span>
              <span className="good caps">{fmtPct(missions ? (s.missionsWon / missions) * 100 : 0, 1)} success</span>
            </div>
            <div className="pd-big num">{fmtInt(missions)}</div>
            <Meter
              value={missions ? (s.missionsWon / missions) * 100 : 0}
              color="var(--good)"
              rest="var(--bad)"
              size="s"
            />
            <div className="pd-legend num">
              <span style={{ "--k": "var(--good)" } as CSSProperties}>Won {fmtInt(s.missionsWon)}</span>
              <span style={{ "--k": "var(--bad)" } as CSSProperties}>Lost {fmtInt(s.missionsLost)}</span>
            </div>
          </div>
          {combat && (
            <div className="pd-card">
              <div className="pd-card-head">
                <span className="tile-label">Accuracy</span>
              </div>
              <div className="pd-big num">{fmtPct(accuracy, 2)}</div>
              <Meter value={Math.min(100, accuracy)} color="var(--info)" rest="var(--line-strong)" size="s" />
              <div className="pd-legend num">
                <span style={{ "--k": "var(--cta)" } as CSSProperties}>Shots fired {fmtInt(s.bulletsFired)}</span>
                <span style={{ "--k": "var(--warn)" } as CSSProperties}>Projectile hits {fmtInt(s.bulletsHit)}</span>
              </div>
            </div>
          )}
        </div>
        {!combat && (
          <p className="pd-note caps">
            Combat counters (kills, shots, time deployed) arrive with the next collector update.
          </p>
        )}
        {combat && (
          <div className="pd-kills">
            <div className="pd-kills-head">
              <span className="tile-label">Enemies eliminated</span>
              <b className="num">{fmtInt(totalKills)}</b>
            </div>
            {kills.map((k) => (
              <div key={k.f} className="pd-kill" style={{ "--k": factionColor(k.f) } as CSSProperties}>
                <FactionEmblem faction={k.f} size={20} />
                <span className="caps">{factionLabel(k.f)}</span>
                <b className="num">{fmtInt(k.n)}</b>
                <div className="pd-kill-bar">
                  <span style={{ width: `${(k.n / maxKills) * 100}%` }} />
                </div>
              </div>
            ))}
          </div>
        )}
        {combat && (
          <div className="pd-tiles four">
            <Tile icon={Skull} label="Helldivers KIA" value={fmtInt(s.deaths)} color="var(--bad)" />
            <Tile
              icon={TriangleAlert}
              label="Accidentals"
              value={fmtInt(s.friendlies)}
              color="var(--warn)"
              sub={`${fmtPct(s.deaths ? (s.friendlies / s.deaths) * 100 : 0, 1)} of KIA`}
            />
            <Tile
              icon={Crosshair}
              label="Kills per death"
              value={s.deaths ? (totalKills / s.deaths).toFixed(1) : "—"}
            />
            <Tile icon={CalendarClock} label="Time deployed" value={fmtLongDuration(s.missionTime / 1000)} />
          </div>
        )}
      </div>
    </Panel>
  );
}

function LogRow({ e }: { e: LogEntry }) {
  if (e.kind === "start")
    return (
      <li className="wl-row">
        <span className="wl-mark" style={{ "--k": factionColor(e.owner) } as CSSProperties}>
          <FactionEmblem faction={e.owner} size={14} />
        </span>
        <div>
          <div className="wl-title caps">Record begins under {factionLabel(e.owner)}</div>
          <div className="wl-sub">{fmtDateTime(e.at)}</div>
        </div>
      </li>
    );
  if (e.kind === "defense")
    return (
      <li className="wl-row">
        <span className="wl-mark small" style={{ "--k": factionColor(e.enemy || "Humans") } as CSSProperties}>
          <Shield size={12} />
        </span>
        <div>
          <div className="wl-title caps">
            {e.ongoing ? "Defense under way" : e.held ? "Defense held" : "Defense failed"}
            {e.enemy && <> against {factionLabel(e.enemy)}</>} <span className="wl-battle">Battle {e.battle}</span>
          </div>
          <div className="wl-sub">{e.ongoing ? `Since ${fmtDateTime(e.at)}` : fmtDateTime(e.at)}</div>
        </div>
      </li>
    );
  const won = e.to === "Humans";
  const color = factionColor(won ? "Humans" : e.to);
  return (
    <li className="wl-row">
      <span className="wl-mark" style={{ "--k": color } as CSSProperties}>
        <FactionEmblem faction={won ? "Humans" : e.to} size={14} />
      </span>
      <div>
        <div className="wl-title caps" style={{ color }}>
          {won ? `Liberated from ${factionLabel(e.from)}` : `Fell to ${factionLabel(e.to)}`}{" "}
          <span className="wl-battle">Battle {e.battle}</span>
        </div>
        <div className="wl-sub">
          {fmtDateTime(e.at)} · {won ? "enemy held it for" : "after"}{" "}
          {e.heldDays < 1 ? `${Math.round(e.heldDays * 24)} hours` : `${Math.round(e.heldDays)} days`}
          {won ? "" : " of freedom"}
        </div>
      </div>
    </li>
  );
}

export function WarHistorySection({ p, record }: { p: Planet; record: PlanetRecord }) {
  const now = Date.now();
  const history = useMemo(() => {
    if (record.recordStart == null) return null;
    const defenses = defenseEvents(
      record.defendingAt,
      record.changes,
      p.event ? Date.parse(p.event.endTime) : null,
      now,
    );
    return buildWarHistory({
      changes: record.changes,
      defenses,
      currentOwner: p.currentOwner,
      recordStart: record.recordStart,
      now,
    });
    // `now` only matters at load time; the record refreshes every 5 minutes.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [record, p.currentOwner, p.event]);

  if (!history)
    return (
      <Panel title="War history" icon={History}>
        <Empty>{record.loading ? "Opening the war archive…" : "No record for this planet yet."}</Empty>
      </Panel>
    );
  const span = Math.max(1, now - history.recordStart);
  const owners = [...new Set(history.segments.map((s) => s.owner))];
  return (
    <Panel title="War history" icon={History}>
      <div className="pd-stack">
        <div className="pd-tiles four">
          <Tile icon={Flag} label="Liberated" value={history.liberated} color="var(--f-humans)" />
          <Tile icon={FlagOff} label="Lost" value={history.lost} color="var(--bad)" />
          <Tile
            icon={ShieldCheck}
            label="Defenses held"
            value={`${history.defensesHeld}/${history.defensesTotal}`}
            sub={
              history.defensesTotal
                ? `${fmtPct((history.defensesHeld / history.defensesTotal) * 100, 0)} held`
                : "none recorded"
            }
          />
          <Tile icon={Shield} label="Held free" value={fmtPct(history.heldFreePct, 0)} sub="of the recorded war" />
        </div>
        <div>
          <div className="tile-label">Control over time</div>
          <div className="wh-bar">
            {history.segments.map((s, i) => (
              <span
                key={i}
                style={{
                  left: `${((s.from - history.recordStart) / span) * 100}%`,
                  width: `${((s.to - s.from) / span) * 100}%`,
                  background: factionColor(s.owner),
                }}
                title={`${factionLabel(s.owner)}: ${fmtDateTime(s.from)} – ${fmtDateTime(s.to)}`}
              />
            ))}
          </div>
          <div className="wh-dots">
            {history.log
              .filter((e) => e.kind !== "start")
              .map((e, i) => (
                <span
                  key={i}
                  className={e.kind === "defense" ? "battle" : ""}
                  style={{
                    left: `${((e.at - history.recordStart) / span) * 100}%`,
                    background: e.kind === "change" ? factionColor(e.to) : undefined,
                  }}
                />
              ))}
          </div>
          <div className="wh-axis caps num">
            <span>{fmtDate(history.recordStart)}</span>
            <span>Now</span>
          </div>
          <div className="pd-legend">
            {owners.map((o) => (
              <span key={o} style={{ "--k": factionColor(o) } as CSSProperties}>
                {factionLabel(o)}
              </span>
            ))}
            {history.defenses.length > 0 && <span style={{ "--k": "var(--text-dim)" } as CSSProperties}>Defense</span>}
          </div>
        </div>
        <div>
          <div className="tile-label">War log</div>
          <ol className="wl">
            {history.log.map((e, i) => (
              <LogRow key={i} e={e} />
            ))}
          </ol>
          <p className="wl-since caps">On record since {fmtDateTime(history.recordStart)}.</p>
        </div>
      </div>
    </Panel>
  );
}

export function DispatchSection(props: { p: Planet; front: Front | null; inMajorOrder: boolean; station: boolean }) {
  const { p, front } = props;
  const owner = factionLabel(p.currentOwner);
  const lines: { text: string; tone?: "mo" | "dss" }[] = [];
  if (front?.kind === "defense")
    lines.push({
      text: `${p.name} is under attack by the ${factionLabel(front.enemy)}. Every mission slows their advance; hold the line.`,
    });
  else if (front)
    lines.push({
      text: `${p.name} remains under ${owner} control. Helldivers are dropping in to return its citizens to the loving embrace of Managed Democracy.`,
    });
  else if (p.currentOwner === "Humans")
    lines.push({
      text: `${p.name} is secure under Super Earth control. Citizens are reminded that vigilance is mandatory.`,
    });
  else lines.push({ text: `${p.name} is held by the ${owner}. No campaign is active here at present.` });
  if (props.inMajorOrder)
    lines.push({
      text: `${p.name} is a Major Order objective. High Command expects every Helldiver to answer the call.`,
      tone: "mo",
    });
  if (props.station)
    lines.push({
      text: `The Democracy Space Station is in orbit over ${p.name}, supporting Helldivers on the ground.`,
      tone: "dss",
    });
  return (
    <Panel title="Dispatch" icon={Radio}>
      <div className="pd-dispatch">
        {lines.map((l, i) => (
          <p key={i} className={l.tone ?? ""}>
            {l.text}
          </p>
        ))}
        <p className="pd-sign caps">Ministry of Truth</p>
      </div>
    </Panel>
  );
}

const EFFECT_ICON: Record<EffectKind, IconLike> = {
  subfaction: Skull,
  threat: TriangleAlert,
  seaf: SeafEmblem,
  poi: Landmark,
  dss: DssIcon,
  arsenal: Crosshair,
  condition: CloudLightning,
  event: Flag,
  look: Orbit,
  gloom: CloudFog,
  hidden: Flag,
};

/**
 * The planet's operational parameters in the game's order: the enemy units and allied forces on it,
 * its standing effects, then the environment.
 */
export function OperationsSection({ p }: { p: Planet }) {
  const now = useNow(60_000);
  const effects = (useEffects().data?.planets.get(p.index) ?? []).filter(
    (e) => e.kind !== "poi" && (e.until == null || e.until > now),
  );
  const enemy = p.event?.faction ?? (p.currentOwner === "Humans" ? undefined : p.currentOwner);
  const row = (key: string, icon: ReactNode, name: string, text: string, tone?: string, left?: number, look = "") => (
    <div key={key} className="pd-env-row" style={tone ? ({ "--k": tone } as CSSProperties) : undefined}>
      <span className={`pd-env-icon ${tone ? "toned" : ""} ${look}`}>{icon}</span>
      <div>
        <b className="caps">
          {name}
          {left != null && <span className="pd-env-left num"> · {fmtTimeLeft(left)} left</span>}
        </b>
        {text && <p>{text}</p>}
      </div>
    </div>
  );
  return (
    <Panel title="Operational parameters" icon={CloudSun}>
      <div className="pd-env">
        {effects.map((e) => {
          const Icon = EFFECT_ICON[e.kind];
          const stratagem = e.kind === "arsenal" ? stratagemIcon(augmentedStratagem(e.name)) : undefined;
          const icon =
            e.kind === "subfaction" && enemy ? (
              <SubfactionEmblem name={e.name} faction={enemy} size={16} color="currentColor" />
            ) : stratagem ? (
              <img src={stratagem} alt="" width={20} height={20} />
            ) : (
              <Icon size={16} />
            );
          const tone = e.kind === "subfaction" && enemy ? factionColor(enemy) : e.kind === "seaf" ? "var(--f-humans)" : undefined;
          const left = e.until != null ? e.until - now : undefined;
          return row(`e${e.id}`, icon, e.name, e.description, tone, left, stratagem ? "stratagem" : "");
        })}
        {p.hazards.map((h) => {
          const Icon = hazardIcon(h.name);
          return row(`h${h.name}`, <Icon size={16} />, h.name, h.description);
        })}
        {p.biome?.name && row("biome", <CloudSun size={16} />, p.biome.name, p.biome.description)}
        {!p.biome?.name && p.hazards.length === 0 && effects.length === 0 && <Empty>No environmental data.</Empty>}
      </div>
    </Panel>
  );
}

/** What stands on the planet: its points of interest, then its cities. */
export function PoisSection({ p }: { p: Planet }) {
  const pois = (useEffects().data?.planets.get(p.index) ?? []).filter((e) => e.kind === "poi");
  if (!pois.length && !p.regions.length) return null;
  const enemy = p.event?.faction ?? (p.currentOwner === "Humans" ? undefined : p.currentOwner);
  return (
    <Panel title="Points of interest" icon={Landmark} right={<span className="num">{pois.length + p.regions.length}</span>}>
      {pois.length > 0 && (
        <div className="pd-env">
          {pois.map((e) => (
            <div key={e.id} className="pd-env-row">
              <span className="pd-env-icon poi">
                <svg viewBox="-12 -12 24 24" width={18} height={18} aria-hidden>
                  <PoiGlyph kind={poiKind(e.name)} faction={poiOrigin(poiKind(e.name), enemy)} size={24} />
                </svg>
              </span>
              <div>
                <b className="caps">{e.name}</b>
                {e.description && <p>{e.description}</p>}
              </div>
            </div>
          ))}
        </div>
      )}
      {p.regions.length > 0 && (
        <>
          <div className={`pd-chart-label caps ${pois.length ? "pd-cities-head" : ""}`}>Cities</div>
          <CityList p={p} full />
        </>
      )}
    </Panel>
  );
}

export function SupplyLinesSection({ p }: { p: Planet }) {
  const planets = useFreshPlanets().data;
  const campaigns = useCampaigns().data;
  const openDetail = useUi((s) => s.openDetail);
  const linked = useMemo(() => {
    const byIdx = new Map((planets ?? []).map((x) => [x.index, x]));
    const ids = new Set([
      ...p.waypoints,
      ...(planets ?? []).filter((x) => x.waypoints.includes(p.index)).map((x) => x.index),
    ]);
    return [...ids].flatMap((i) => (byIdx.get(i) ? [byIdx.get(i)!] : []));
  }, [planets, p]);
  const active = new Set((campaigns ?? []).map((c) => c.planet.index));
  const siege = useCircumvallation(p);
  return (
    <Panel title="Supply lines" icon={Route} right={<span className="num">{linked.length}</span>}>
      {siege && <CircumvallationNote front={p} c={siege} className="pd-siege" />}
      {linked.length === 0 ? (
        <Empty>No supply lines from this planet.</Empty>
      ) : (
        <ul className="pd-lines">
          {linked.map((l) => {
            const campaign = active.has(l.index);
            const pct = l.currentOwner === "Humans" && !l.event ? 100 : liberationPercent(l);
            return (
              <li key={l.index}>
                <button
                  onClick={() => openDetail(l.index)}
                  style={{ "--k": factionColor(l.currentOwner) } as CSSProperties}
                >
                  <FactionEmblem faction={l.currentOwner} size={18} />
                  <span className="pd-line-name caps">{l.name}</span>
                  <span className={`pd-line-state caps ${campaign ? "on" : ""}`}>
                    {campaign
                      ? l.event
                        ? "Defense"
                        : "Campaign"
                      : l.currentOwner === "Humans"
                        ? "Secured"
                        : "Enemy held"}
                  </span>
                  <span className="num pd-line-pct">{fmtPct(pct, 0)}</span>
                </button>
              </li>
            );
          })}
        </ul>
      )}
    </Panel>
  );
}
