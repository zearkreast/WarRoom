use crate::api::queries::queries;
use crate::features::map::poi::{PoiGlyph, poi_kind, poi_origin};
use crate::features::map::stratagem_art::{augmented_stratagem, stratagem_icon};
use crate::features::planet::circumvallation::{circumvallation_note, use_circumvallation};
use crate::features::planet::cities::CityList;
use crate::features::planet::record::PlanetRecord;
use crate::features::planets::fronts::{Front, FrontKind};
use crate::js::{self, fmt_date, fmt_date_time};
use crate::state::ui;
use crate::ui::chart::{
    Chart, Grid, HOUR, Plot, RefLine, Series, Tooltip, XAxis, YAxis, default_tooltip, hour_ticks, series_rows,
};
use crate::ui::emblems::{FactionEmblem, IconLike, SubfactionEmblem, icon_view};
use crate::ui::hazard_icons::hazard_icon;
use crate::ui::icons::{self, Icon};
use crate::ui::primitives::{Countdown, Empty, Meter, MeterSize, Panel, Tile};
use leptos::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use warroom_core::api::{Faction, Planet};
use warroom_core::biomes::regen_pct_per_hour;
use warroom_core::effects::EffectKind;
use warroom_core::factions::{faction_color, faction_label};
use warroom_core::format::{fmt_compact, fmt_int, fmt_pct, fmt_time_left, js_round, js_sum, to_fixed};
use warroom_core::progress::liberation_percent;
use warroom_core::time::parse_ms;
use warroom_core::war_history::{HistoryInput, LogEntry, build_war_history, defense_events, fmt_long_duration};

fn regen_of(p: &Planet) -> f64 {
    regen_pct_per_hour(p.regen_per_second, p.max_health)
}

fn enemy_of(p: &Planet) -> Option<Faction> {
    p.event.as_ref().map(|e| e.faction).or((p.current_owner != Faction::Humans).then_some(p.current_owner))
}

#[component]
pub fn LiberationSection(#[prop(into)] p: Signal<Planet>, front: Memo<Option<Front>>) -> impl IntoView {
    let get = move |f: fn(&Front) -> f64| Signal::derive(move || front.with(|x| x.as_ref().map_or(0.0, f)));
    let percent = get(|f| f.percent);
    let projected = Signal::derive(move || front.with(|f| f.as_ref().and_then(|f| f.projected)));
    let enemy = Signal::derive(move || faction_color(front.with(|f| f.as_ref().map(|f| f.enemy))).to_string());
    let defense = Memo::new(move |_| front.with(|f| f.as_ref().is_some_and(|f| f.kind == FrontKind::Defense)));
    let tiles = move || {
        let f = front.get()?;
        let p = p.get();
        let regen = regen_of(&p);
        let (health, max) = match (&p.event, f.kind) {
            (Some(e), FrontKind::Defense) => (e.health, e.max_health),
            _ => (p.health, p.max_health),
        };
        let defense = f.kind == FrontKind::Defense;
        let verb = if defense { "Defended" } else { "Liberated" };
        let victory = f.victory_at.filter(|v| *v != 0.0 && !v.is_nan());
        let fail = f.fail_at.filter(|v| *v != 0.0 && !v.is_nan());
        let prediction = match (victory, fail) {
            (Some(v), _) => view! { <Countdown to=v /> }.into_any(),
            (None, Some(d)) => view! { <Countdown to=d /> }.into_any(),
            _ => "Stalled".into_any(),
        };
        let sub = match (victory, fail) {
            (Some(_), _) => "until victory",
            (None, Some(_)) if defense => "until it falls",
            (None, Some(_)) => "until withdrawal",
            _ => "no progress",
        };
        let enemy = faction_color(Some(f.enemy));
        Some(view! {
            <div class="pd-tiles four">
                <Tile icon=icons::HEART label="Planet HP" value=format!("{} / {}", fmt_int(health), fmt_int(max)).into_any() />
                <Tile icon=icons::FLAG label=verb value=fmt_pct(f.percent, 4).into_any() />
                <Tile
                    icon=icons::TRENDING_UP
                    label=format!("{} rate", if defense { "Defense" } else { "Liberation" })
                    value=f
                        .rate
                        .map_or("Measuring…".to_string(), |r| format!("{}{}% / h", if r > 0.0 { "+" } else { "" }, to_fixed(r, 2)))
                        .into_any()
                    color=f.rate.map(|r| if r > 0.0 { "var(--good)".to_string() } else { "var(--bad)".to_string() })
                />
                <Tile
                    icon=icons::TIMER
                    label="Prediction"
                    value=prediction
                    sub=sub.into_any()
                    color=match (victory, fail) {
                        (Some(_), _) => Some("var(--good)".to_string()),
                        (None, Some(_)) => Some("var(--bad)".to_string()),
                        _ => None,
                    }
                />
            </div>
            <div class="pd-tiles three">
                <div class="pd-pill" style=format!("color: {enemy};")>
                    <FactionEmblem faction=f.enemy.as_str() size=18.0 />
                    " "
                    <span class="num">{format!("-{}% / h", to_fixed(regen, 2))}</span>
                    <span class="muted caps">"enemy regen"</span>
                </div>
                <div class="pd-pill" style="color: var(--f-humans);">
                    <FactionEmblem faction="Humans" size=18.0 />
                    <span class="num">{f.rate.map_or("—".to_string(), |r| format!("+{}% / h", to_fixed(r + regen, 2)))}</span>
                    <span class="muted caps">"Helldiver impact"</span>
                </div>
                <div class="pd-pill" style="color: var(--cta);">
                    <Icon icon=icons::USERS size=16.0 />
                    " "
                    <span class="num">{fmt_int(f.players)}</span>
                    <span class="muted caps">"deployed"</span>
                </div>
            </div>
        })
    };
    view! {
        <Panel
            title=Signal::derive(move || if defense.get() { "Defense progress".to_string() } else { "Liberation progress".to_string() })
            icon=icons::ACTIVITY
            tone=enemy.get_untracked()
        >
            <div class="pd-stack">
                <Meter value=percent color="var(--f-humans)".to_string() rest=enemy marker=projected size=MeterSize::L chevron=true label="Progress" />
                {tiles}
            </div>
        </Panel>
    }
}

/// One row of the front line charts: a past sample, the projection ahead, or the sample where they meet.
#[derive(Clone, Copy, Default, PartialEq)]
struct FrontRow {
    at: f64,
    players: Option<f64>,
    percent: Option<f64>,
    projection: Option<f64>,
}

#[component]
pub fn FrontLineSection(
    #[prop(into)] p: Signal<Planet>,
    front: Memo<Option<Front>>,
    record: ReadSignal<PlanetRecord>,
) -> impl IntoView {
    let hover = RwSignal::new(None::<usize>);
    let rows = Memo::new(move |_| {
        let now = js::now();
        let (rate, percent) = front.with(|f| f.as_ref().map_or((None, 0.0), |f| (f.rate, f.percent)));
        record.with(|r| {
            let mut rows: Vec<FrontRow> = r
                .samples
                .iter()
                .map(|s| FrontRow { at: s.at, players: Some(s.players), percent: Some(s.percent), projection: None })
                .collect();
            let last = rows.last().copied();
            if let Some(l) = rows.last_mut() {
                l.projection = l.percent;
            }
            if let Some(rate) = rate {
                let base = last.and_then(|l| l.percent).unwrap_or(percent);
                let t0 = last.map_or(now, |l| l.at);
                for h in 1..=24 {
                    let v = (base + rate * h as f64).clamp(0.0, 100.0);
                    rows.push(FrontRow { at: t0 + h as f64 * HOUR, projection: Some(v), ..Default::default() });
                }
            }
            let now_at = r.samples.last().map_or(now, |s| s.at);
            Arc::new((rows, now_at))
        })
    });
    let x_axis = move |hide: bool| {
        let (rows, _) = &*rows.get();
        let now = js::now();
        let domain = (rows.first().map_or(now, |r| r.at), rows.last().map_or(now, |r| r.at));
        XAxis { hide, ..XAxis::time(domain, hour_ticks(domain.0, domain.1, 4.0)) }
    };
    let progress = Signal::derive(move || {
        let data = rows.get();
        let (rows, now_at) = &*data;
        Arc::new(Plot {
            xs: rows.iter().map(|r| r.at).collect(),
            x: x_axis(true),
            y: vec![YAxis::new(
                (0.0, 100.0),
                vec![0.0, 25.0, 50.0, 75.0, 100.0],
                Arc::new(|v| format!("{}%", js::num(v))),
            )],
            grid: Some(Grid { stroke: crate::ui::chart::GRID.into(), opacity: None, edges_only: false }),
            series: vec![
                Series::line("percent", rows.iter().map(|r| r.percent).collect(), "var(--c-humans)", 2.0),
                Series {
                    dash: Some("4 4".into()),
                    ..Series::line("projection", rows.iter().map(|r| r.projection).collect(), "var(--c-humans)", 2.0)
                },
            ],
            ref_lines: vec![RefLine { x: *now_at, stroke: "var(--text-dim)".into(), label: Some("Now".into()) }],
        })
    });
    let players = Signal::derive(move || {
        let data = rows.get();
        let (rows, now_at) = &*data;
        let values: Vec<Option<f64>> = rows.iter().map(|r| r.players).collect();
        let max = values.iter().flatten().copied().fold(0.0, f64::max);
        Arc::new(Plot {
            xs: rows.iter().map(|r| r.at).collect(),
            x: x_axis(false),
            y: vec![YAxis::auto(max, Arc::new(fmt_compact))],
            grid: Some(Grid { stroke: crate::ui::chart::GRID.into(), opacity: None, edges_only: false }),
            series: vec![Series::area("players", values, "var(--cta)", 2.0, "url(#pd-players)")],
            ref_lines: vec![RefLine { x: *now_at, stroke: "var(--text-dim)".into(), label: None }],
        })
    });
    let progress_tip: Tooltip = Arc::new(|plot, i| {
        let rows = series_rows(plot, i, |s, v| {
            (fmt_pct(v, 2), if s.name == "percent" { "Progress".into() } else { "Projection".into() })
        });
        Some(default_tooltip(fmt_date_time(plot.xs[i]), rows))
    });
    let players_tip: Tooltip = Arc::new(|plot, i| {
        let rows = series_rows(plot, i, |_, v| (fmt_int(v), "Helldivers".into()));
        Some(default_tooltip(fmt_date_time(plot.xs[i]), rows))
    });
    let heads = move || {
        let f = front.get()?;
        let p = p.get();
        let (change, peak) = record.with(|r| {
            (r.samples.first().map(|s| f.percent - s.percent), r.samples.iter().fold(0.0, |m: f64, s| m.max(s.players)))
        });
        let regen = regen_of(&p);
        let proj = f.rate.map(|r| (f.percent + r * 24.0).clamp(0.0, 100.0));
        let victory = f.victory_at.filter(|v| *v != 0.0 && !v.is_nan());
        Some(view! {
            <div class="pd-heads">
                <div>
                    <span class="pd-key" style="--k: var(--f-humans);">
                        {if f.kind == FrontKind::Defense { "Defense" } else { "Liberation" }}
                        " change"
                    </span>
                    <b
                        class="num"
                        style=change.map(|c| format!("color: {};", if c >= 0.0 { "var(--good)" } else { "var(--bad)" }))
                    >
                        {change.map_or("—".to_string(), |c| format!("{}{}%", if c >= 0.0 { "+" } else { "" }, to_fixed(c, 2)))}
                    </b>
                </div>
                <div>
                    <span class="pd-key" style="--k: var(--cta);">"Peak Helldivers"</span>
                    <b class="num">{if peak != 0.0 { fmt_int(peak) } else { "—".into() }}</b>
                </div>
                <div>
                    <span class="pd-key" style="--k: var(--bad);">"Enemy impact"</span>
                    <b class="num" style="color: var(--bad);">{format!("-{}%/h", to_fixed(regen, 2))}</b>
                </div>
                <div>
                    <span class="pd-key dashed" style="--k: var(--f-humans);">"24 h projection"</span>
                    <b class="num">{proj.map_or("—".to_string(), |v| fmt_pct(v, 2))}</b>
                    {victory.map(|v| view! {
                        <span class="pd-sub good caps">
                            {if f.kind == FrontKind::Defense { "Defended" } else { "Liberated" }}
                            " in "
                            <Countdown to=v />
                        </span>
                    })}
                </div>
            </div>
        })
    };
    let enough = Memo::new(move |_| record.with(|r| r.samples.len() >= 2));
    let loading = Memo::new(move |_| record.with(|r| r.loading));
    view! {
        <Panel title="Front line activity · last and next 24 h" icon=icons::LINE_CHART>
            <div class="pd-stack">
                {heads}
                <Show
                    when=move || enough.get()
                    fallback=move || {
                        view! {
                            <Empty>
                                {move || {
                                    if loading.get() {
                                        "Loading telemetry…"
                                    } else {
                                        "Not enough readings yet; the chart fills in as the collector records this front."
                                    }
                                }}
                            </Empty>
                        }
                    }
                >
                    <div class="pd-charts">
                        <div class="pd-chart-label caps">"Progress"</div>
                        <Chart plot=progress height=170.0 margin=(8.0, 8.0, 0.0, 0.0) hover=hover tooltip=progress_tip.clone() />
                        <div class="pd-chart-label caps">"Helldivers deployed"</div>
                        <Chart
                            plot=players
                            height=120.0
                            margin=(4.0, 8.0, 0.0, 0.0)
                            hover=hover
                            tooltip=players_tip.clone()
                            defs=ViewFn::from(|| view! {
                                <linearGradient id="pd-players" x1="0" y1="0" x2="0" y2="1">
                                    <stop offset="0%" stop-color="var(--cta)" stop-opacity="0.28" />
                                    <stop offset="100%" stop-color="var(--cta)" stop-opacity="0.02" />
                                </linearGradient>
                            })
                        />
                    </div>
                </Show>
            </div>
        </Panel>
    }
}

#[component]
pub fn ServiceRecordSection(#[prop(into)] p: Signal<Planet>) -> impl IntoView {
    move || {
        let s = p.with(|p| p.statistics.clone());
        let missions = s.missions_won + s.missions_lost;
        let mut kills = [
            (Faction::Automaton, s.automaton_kills),
            (Faction::Terminids, s.terminid_kills),
            (Faction::Illuminate, s.illuminate_kills),
        ];
        kills.sort_by(|a, b| b.1.total_cmp(&a.1));
        let total_kills = js_sum(kills.iter().map(|k| k.1));
        let max_kills = kills[0].1.max(1.0);
        let accuracy = if s.bullets_fired != 0.0 { s.bullets_hit / s.bullets_fired * 100.0 } else { 0.0 };
        // Older collector builds trimmed these counters from the snapshot; they read as zero until it updates.
        let combat = s.bullets_fired > 0.0 || total_kills > 0.0;
        if missions == 0.0 && total_kills == 0.0 {
            return view! {
                <Panel title="Service record" icon=icons::MEDAL>
                    <Empty>"No missions recorded on this planet yet."</Empty>
                </Panel>
            }
            .into_any();
        }
        let won_pct = if missions != 0.0 { s.missions_won / missions * 100.0 } else { 0.0 };
        view! {
            <Panel title="Service record" icon=icons::MEDAL>
                <div class="pd-stack">
                    <div class=if combat { "pd-tiles two" } else { "pd-tiles" }>
                        <div class="pd-card">
                            <div class="pd-card-head">
                                <span class="tile-label">"Missions"</span>
                                <span class="good caps">{format!("{} success", fmt_pct(won_pct, 1))}</span>
                            </div>
                            <div class="pd-big num">{fmt_int(missions)}</div>
                            <Meter value=won_pct color="var(--good)".to_string() rest="var(--bad)".to_string() size=MeterSize::S />
                            <div class="pd-legend num">
                                <span style="--k: var(--good);">{format!("Won {}", fmt_int(s.missions_won))}</span>
                                <span style="--k: var(--bad);">{format!("Lost {}", fmt_int(s.missions_lost))}</span>
                            </div>
                        </div>
                        {combat.then(|| view! {
                            <div class="pd-card">
                                <div class="pd-card-head">
                                    <span class="tile-label">"Accuracy"</span>
                                </div>
                                <div class="pd-big num">{fmt_pct(accuracy, 2)}</div>
                                <Meter value=accuracy.min(100.0) color="var(--info)".to_string() rest="var(--line-strong)".to_string() size=MeterSize::S />
                                <div class="pd-legend num">
                                    <span style="--k: var(--cta);">{format!("Shots fired {}", fmt_int(s.bullets_fired))}</span>
                                    <span style="--k: var(--warn);">{format!("Projectile hits {}", fmt_int(s.bullets_hit))}</span>
                                </div>
                            </div>
                        })}
                    </div>
                    {(!combat).then(|| view! {
                        <p class="pd-note caps">"Combat counters (kills, shots, time deployed) arrive with the next collector update."</p>
                    })}
                    {combat.then(|| view! {
                        <div class="pd-kills">
                            <div class="pd-kills-head">
                                <span class="tile-label">"Enemies eliminated"</span>
                                <b class="num">{fmt_int(total_kills)}</b>
                            </div>
                            {kills
                                .iter()
                                .map(|(f, n)| view! {
                                    <div class="pd-kill" style=format!("--k: {};", faction_color(Some(*f)))>
                                        <FactionEmblem faction=f.as_str() size=20.0 />
                                        <span class="caps">{faction_label(Some(*f))}</span>
                                        <b class="num">{fmt_int(*n)}</b>
                                        <div class="pd-kill-bar">
                                            <span style=format!("width: {}%;", js::num(n / max_kills * 100.0))></span>
                                        </div>
                                    </div>
                                })
                                .collect_view()}
                        </div>
                        <div class="pd-tiles four">
                            <Tile icon=icons::SKULL label="Helldivers KIA" value=fmt_int(s.deaths).into_any() color="var(--bad)" />
                            <Tile
                                icon=icons::TRIANGLE_ALERT
                                label="Accidentals"
                                value=fmt_int(s.friendlies).into_any()
                                color="var(--warn)"
                                sub=format!(
                                    "{} of KIA",
                                    fmt_pct(if s.deaths != 0.0 { s.friendlies / s.deaths * 100.0 } else { 0.0 }, 1)
                                )
                                .into_any()
                            />
                            <Tile
                                icon=icons::CROSSHAIR
                                label="Kills per death"
                                value=(if s.deaths != 0.0 { to_fixed(total_kills / s.deaths, 1) } else { "—".into() }).into_any()
                            />
                            <Tile icon=icons::CALENDAR_CLOCK label="Time deployed" value=fmt_long_duration(s.mission_time / 1000.0).into_any() />
                        </div>
                    })}
                </div>
            </Panel>
        }
        .into_any()
    }
}

fn log_row(e: &LogEntry) -> AnyView {
    match e {
        LogEntry::Start { at, owner } => view! {
            <li class="wl-row">
                <span class="wl-mark" style=format!("--k: {};", faction_color(Some(*owner)))>
                    <FactionEmblem faction=owner.as_str() size=14.0 />
                </span>
                <div>
                    <div class="wl-title caps">{format!("Record begins under {}", faction_label(Some(*owner)))}</div>
                    <div class="wl-sub">{fmt_date_time(*at)}</div>
                </div>
            </li>
        }
        .into_any(),
        LogEntry::Defense { at, enemy, battle, ongoing, held } => view! {
            <li class="wl-row">
                <span class="wl-mark small" style=format!("--k: {};", faction_color(Some(enemy.unwrap_or(Faction::Humans))))>
                    <Icon icon=icons::SHIELD size=12.0 />
                </span>
                <div>
                    <div class="wl-title caps">
                        {if *ongoing {
                            "Defense under way"
                        } else if *held {
                            "Defense held"
                        } else {
                            "Defense failed"
                        }}
                        {enemy.map(|e| format!(" against {}", faction_label(Some(e))))}
                        " "
                        <span class="wl-battle">{format!("Battle {battle}")}</span>
                    </div>
                    <div class="wl-sub">{if *ongoing { format!("Since {}", fmt_date_time(*at)) } else { fmt_date_time(*at) }}</div>
                </div>
            </li>
        }
        .into_any(),
        LogEntry::Change { at, from, to, battle, held_days } => {
            let won = *to == Faction::Humans;
            let color = faction_color(Some(if won { Faction::Humans } else { *to }));
            let held = if *held_days < 1.0 {
                format!("{} hours", js::num(js_round(held_days * 24.0)))
            } else {
                format!("{} days", js::num(js_round(*held_days)))
            };
            view! {
                <li class="wl-row">
                    <span class="wl-mark" style=format!("--k: {color};")>
                        <FactionEmblem faction=if won { "Humans" } else { to.as_str() } size=14.0 />
                    </span>
                    <div>
                        <div class="wl-title caps" style=format!("color: {color};")>
                            {if won {
                                format!("Liberated from {}", faction_label(Some(*from)))
                            } else {
                                format!("Fell to {}", faction_label(Some(*to)))
                            }}
                            " "
                            <span class="wl-battle">{format!("Battle {battle}")}</span>
                        </div>
                        <div class="wl-sub">
                            {format!(
                                "{} · {} {held}{}",
                                fmt_date_time(*at),
                                if won { "enemy held it for" } else { "after" },
                                if won { "" } else { " of freedom" }
                            )}
                        </div>
                    </div>
                </li>
            }
            .into_any()
        }
    }
}

#[component]
pub fn WarHistorySection(#[prop(into)] p: Signal<Planet>, record: ReadSignal<PlanetRecord>) -> impl IntoView {
    let key = Memo::new(move |_| p.with(|p| (p.current_owner, p.event.as_ref().map(|e| e.end_time.clone()))));
    let history = Memo::new(move |_| {
        let (owner, event_end) = key.get();
        record.with(|r| {
            let start = r.record_start?;
            // `now` only matters at load time; the record refreshes every 5 minutes.
            let now = js::now();
            let defenses = defense_events(&r.defending_at, &r.changes, event_end.map(|e| parse_ms(&e)), now);
            Some((
                build_war_history(HistoryInput {
                    changes: &r.changes,
                    defenses: &defenses,
                    current_owner: owner,
                    record_start: start,
                    now,
                }),
                now,
            ))
        })
    });
    move || {
        let Some((h, now)) = history.get() else {
            let loading = record.with(|r| r.loading);
            return view! {
                <Panel title="War history" icon=icons::HISTORY>
                    <Empty>{if loading { "Opening the war archive…" } else { "No record for this planet yet." }}</Empty>
                </Panel>
            }
            .into_any();
        };
        let span = (now - h.record_start).max(1.0);
        let mut owners: Vec<Faction> = Vec::new();
        for s in &h.segments {
            if !owners.contains(&s.owner) {
                owners.push(s.owner);
            }
        }
        let pos = move |t: f64| js::num((t - h.record_start) / span * 100.0);
        view! {
            <Panel title="War history" icon=icons::HISTORY>
                <div class="pd-stack">
                    <div class="pd-tiles four">
                        <Tile icon=icons::FLAG label="Liberated" value=h.liberated.into_any() color="var(--f-humans)" />
                        <Tile icon=icons::FLAG_OFF label="Lost" value=h.lost.into_any() color="var(--bad)" />
                        <Tile
                            icon=icons::SHIELD_CHECK
                            label="Defenses held"
                            value=format!("{}/{}", h.defenses_held, h.defenses_total).into_any()
                            sub=(if h.defenses_total > 0 {
                                format!("{} held", fmt_pct(h.defenses_held as f64 / h.defenses_total as f64 * 100.0, 0))
                            } else {
                                "none recorded".into()
                            })
                            .into_any()
                        />
                        <Tile icon=icons::SHIELD label="Held free" value=fmt_pct(h.held_free_pct, 0).into_any() sub="of the recorded war".into_any() />
                    </div>
                    <div>
                        <div class="tile-label">"Control over time"</div>
                        <div class="wh-bar">
                            {h
                                .segments
                                .iter()
                                .map(|s| view! {
                                    <span
                                        style=format!(
                                            "left: {}%; width: {}%; background: {};",
                                            pos(s.from),
                                            js::num((s.to - s.from) / span * 100.0),
                                            faction_color(Some(s.owner))
                                        )
                                        title=format!("{}: {} – {}", faction_label(Some(s.owner)), fmt_date_time(s.from), fmt_date_time(s.to))
                                    ></span>
                                })
                                .collect_view()}
                        </div>
                        <div class="wh-dots">
                            {h
                                .log
                                .iter()
                                .filter(|e| !matches!(e, LogEntry::Start { .. }))
                                .map(|e| {
                                    let bg = match e {
                                        LogEntry::Change { to, .. } => format!(" background: {};", faction_color(Some(*to))),
                                        _ => String::new(),
                                    };
                                    view! {
                                        <span
                                            class=if matches!(e, LogEntry::Defense { .. }) { "battle" } else { "" }
                                            style=format!("left: {}%;{bg}", pos(e.at()))
                                        ></span>
                                    }
                                })
                                .collect_view()}
                        </div>
                        <div class="wh-axis caps num">
                            <span>{fmt_date(h.record_start)}</span>
                            <span>"Now"</span>
                        </div>
                        <div class="pd-legend">
                            {owners
                                .iter()
                                .map(|o| view! { <span style=format!("--k: {};", faction_color(Some(*o)))>{faction_label(Some(*o))}</span> })
                                .collect_view()}
                            {(!h.defenses.is_empty()).then(|| view! { <span style="--k: var(--text-dim);">"Defense"</span> })}
                        </div>
                    </div>
                    <div>
                        <div class="tile-label">"War log"</div>
                        <ol class="wl">{h.log.iter().map(log_row).collect_view()}</ol>
                        <p class="wl-since caps">{format!("On record since {}.", fmt_date_time(h.record_start))}</p>
                    </div>
                </div>
            </Panel>
        }
        .into_any()
    }
}

#[component]
pub fn DispatchSection(
    #[prop(into)] p: Signal<Planet>,
    front: Memo<Option<Front>>,
    #[prop(into)] in_major_order: Signal<bool>,
    #[prop(into)] station: Signal<bool>,
) -> impl IntoView {
    let lines = move || {
        let p = p.get();
        let owner = faction_label(Some(p.current_owner));
        let mut lines: Vec<(String, &'static str)> = Vec::new();
        let first = match front.get() {
            Some(f) if f.kind == FrontKind::Defense => format!(
                "{} is under attack by the {}. Every mission slows their advance; hold the line.",
                p.name,
                faction_label(Some(f.enemy))
            ),
            Some(_) => format!(
                "{} remains under {owner} control. Helldivers are dropping in to return its citizens to the loving embrace of Managed Democracy.",
                p.name
            ),
            None if p.current_owner == Faction::Humans => {
                format!(
                    "{} is secure under Super Earth control. Citizens are reminded that vigilance is mandatory.",
                    p.name
                )
            }
            None => format!("{} is held by the {owner}. No campaign is active here at present.", p.name),
        };
        lines.push((first, ""));
        if in_major_order.get() {
            lines.push((
                format!(
                    "{} is a Major Order objective. High Command expects every Helldiver to answer the call.",
                    p.name
                ),
                "mo",
            ));
        }
        if station.get() {
            lines.push((
                format!(
                    "The Democracy Space Station is in orbit over {}, supporting Helldivers on the ground.",
                    p.name
                ),
                "dss",
            ));
        }
        lines.into_iter().map(|(text, tone)| view! { <p class=tone>{text}</p> }).collect_view()
    };
    view! {
        <Panel title="Dispatch" icon=icons::RADIO>
            <div class="pd-dispatch">
                {lines}
                <p class="pd-sign caps">"Ministry of Truth"</p>
            </div>
        </Panel>
    }
}

fn effect_icon(kind: EffectKind) -> IconLike {
    match kind {
        EffectKind::Subfaction => icons::SKULL.into(),
        EffectKind::Threat => icons::TRIANGLE_ALERT.into(),
        EffectKind::Seaf => IconLike::Seaf,
        EffectKind::Poi => icons::LANDMARK.into(),
        EffectKind::Dss => IconLike::Dss,
        EffectKind::Arsenal => icons::CROSSHAIR.into(),
        EffectKind::Condition => icons::CLOUD_LIGHTNING.into(),
        EffectKind::Event | EffectKind::Hidden => icons::FLAG.into(),
        EffectKind::Look => icons::ORBIT.into(),
        EffectKind::Gloom => icons::CLOUD_FOG.into(),
    }
}

fn env_row(icon: AnyView, name: String, text: String, tone: Option<String>) -> AnyView {
    env_row_timed(icon, name, text, tone, None, "")
}

/// `left` is the time left on a timed effect, ms; `look` an extra class for its icon.
fn env_row_timed(
    icon: AnyView,
    name: String,
    text: String,
    tone: Option<String>,
    left: Option<f64>,
    look: &str,
) -> AnyView {
    view! {
        <div class="pd-env-row" style=tone.clone().map(|t| format!("--k: {t};"))>
            <span class=format!("pd-env-icon {} {look}", if tone.is_some() { "toned" } else { "" })>{icon}</span>
            <div>
                <b class="caps">
                    {name}
                    {left.map(|l| view! { <span class="pd-env-left num">{format!(" · {} left", fmt_time_left(l))}</span> })}
                </b>
                {(!text.is_empty()).then(|| view! { <p>{text}</p> })}
            </div>
        </div>
    }
    .into_any()
}

/// The planet's operational parameters in the game's order: the enemy units and allied forces on it,
/// its standing effects, then the environment.
#[component]
pub fn OperationsSection(#[prop(into)] p: Signal<Planet>) -> impl IntoView {
    let effects = queries().effects;
    let clock = js::use_now(60_000.0);
    move || {
        let p = p.get();
        let now = clock.get();
        let list: Vec<_> = effects.with(|e| {
            e.as_ref()
                .and_then(|e| e.planets.get(&p.index).cloned())
                .unwrap_or_default()
                .into_iter()
                .filter(|e| e.kind != EffectKind::Poi && e.until.is_none_or(|u| u > now))
                .collect()
        });
        let enemy = enemy_of(&p);
        let rows = list
            .iter()
            .map(|e| {
                let sub = e.kind == EffectKind::Subfaction;
                let stratagem =
                    (e.kind == EffectKind::Arsenal).then(|| stratagem_icon(&augmented_stratagem(&e.name))).flatten();
                let look = if stratagem.is_some() { "stratagem" } else { "" };
                let icon = match (sub, enemy, stratagem) {
                    (true, Some(en), _) => {
                        view! { <SubfactionEmblem name=e.name.clone() faction=en.as_str() size=16.0 color="currentColor" /> }.into_any()
                    }
                    (_, _, Some(src)) => view! { <img src=src alt="" width="20" height="20" /> }.into_any(),
                    _ => icon_view(effect_icon(e.kind), 16.0, 2.0, None),
                };
                let tone = match (sub, enemy) {
                    (true, Some(en)) => Some(faction_color(Some(en)).to_string()),
                    _ if e.kind == EffectKind::Seaf => Some("var(--f-humans)".to_string()),
                    _ => None,
                };
                env_row_timed(icon, e.name.clone(), e.description.clone(), tone, e.until.map(|u| u - now), look)
            })
            .collect_view();
        let hazards = p
            .hazards
            .iter()
            .map(|h| {
                env_row(
                    view! { <Icon icon=hazard_icon(&h.name) size=16.0 /> }.into_any(),
                    h.name.clone(),
                    h.description.clone(),
                    None,
                )
            })
            .collect_view();
        let biome = (!p.biome.name.is_empty()).then(|| {
            env_row(
                view! { <Icon icon=icons::CLOUD_SUN size=16.0 /> }.into_any(),
                p.biome.name.clone(),
                p.biome.description.clone(),
                None,
            )
        });
        let empty = p.biome.name.is_empty() && p.hazards.is_empty() && list.is_empty();
        view! {
            <Panel title="Operational parameters" icon=icons::CLOUD_SUN>
                <div class="pd-env">
                    {rows}
                    {hazards}
                    {biome}
                    {empty.then(|| view! { <Empty>"No environmental data."</Empty> })}
                </div>
            </Panel>
        }
    }
}

/// What stands on the planet: its points of interest, then its cities.
#[component]
pub fn PoisSection(#[prop(into)] p: Signal<Planet>) -> impl IntoView {
    let effects = queries().effects;
    let pois = Memo::new(move |_| {
        let index = p.with(|p| p.index);
        effects.with(|e| {
            e.as_ref()
                .and_then(|e| e.planets.get(&index).cloned())
                .unwrap_or_default()
                .into_iter()
                .filter(|e| e.kind == EffectKind::Poi)
                .collect::<Vec<_>>()
        })
    });
    let regions = Memo::new(move |_| p.with(|p| p.regions.len()));
    let enemy = Memo::new(move |_| p.with(enemy_of));
    let planet = Signal::derive(move || Some(p.get()));
    move || {
        let list = pois.get();
        let n_regions = regions.get();
        if list.is_empty() && n_regions == 0 {
            return None;
        }
        let enemy = enemy.get();
        let has_pois = !list.is_empty();
        Some(view! {
            <Panel title="Points of interest" icon=icons::LANDMARK right=view! { <span class="num">{list.len() + n_regions}</span> }.into_any()>
                {has_pois.then(|| view! {
                    <div class="pd-env">
                        {list
                            .iter()
                            .map(|e| {
                                let kind = poi_kind(&e.name);
                                view! {
                                    <div class="pd-env-row">
                                        <span class="pd-env-icon poi">
                                            <svg viewBox="-12 -12 24 24" width="18" height="18" aria-hidden="true">
                                                <PoiGlyph kind=kind faction=poi_origin(kind, enemy) size=24.0 />
                                            </svg>
                                        </span>
                                        <div>
                                            <b class="caps">{e.name.clone()}</b>
                                            {(!e.description.is_empty()).then(|| view! { <p>{e.description.clone()}</p> })}
                                        </div>
                                    </div>
                                }
                            })
                            .collect_view()}
                    </div>
                })}
                {(n_regions > 0).then(|| view! {
                    <div class=format!("pd-chart-label caps {}", if has_pois { "pd-cities-head" } else { "" })>"Cities"</div>
                    <CityList planet=planet full=true />
                })}
            </Panel>
        })
    }
}

#[component]
pub fn SupplyLinesSection(#[prop(into)] p: Signal<Planet>) -> impl IntoView {
    let q = queries();
    let linked = Memo::new(move |_| {
        let (index, waypoints) = p.with(|p| (p.index, p.waypoints.clone()));
        q.fresh_planets.with(|planets| {
            let planets = planets.as_deref().map(|p| p.as_slice()).unwrap_or_default();
            let by_index: HashMap<i64, &Planet> = planets.iter().map(|x| (x.index, x)).collect();
            let mut ids: Vec<i64> = Vec::new();
            for i in waypoints
                .iter()
                .copied()
                .chain(planets.iter().filter(|x| x.waypoints.contains(&index)).map(|x| x.index))
            {
                if !ids.contains(&i) {
                    ids.push(i);
                }
            }
            ids.into_iter().filter_map(|i| by_index.get(&i).map(|x| (*x).clone())).collect::<Vec<_>>()
        })
    });
    let active = Memo::new(move |_| {
        q.campaigns.with(|c| c.iter().flat_map(|l| l.iter()).map(|c| c.planet.index).collect::<HashSet<_>>())
    });
    let siege = use_circumvallation(p);
    move || {
        let list = linked.get();
        let active = active.get();
        view! {
            <Panel title="Supply lines" icon=icons::ROUTE right=view! { <span class="num">{list.len()}</span> }.into_any()>
                {move || siege.get().map(|c| circumvallation_note(&p.get(), &c, "pd-siege"))}
                {if list.is_empty() {
                    view! { <Empty>"No supply lines from this planet."</Empty> }.into_any()
                } else {
                    view! {
                        <ul class="pd-lines">
                            {list
                                .iter()
                                .map(|l| {
                                    let campaign = active.contains(&l.index);
                                    let pct = if l.current_owner == Faction::Humans && l.event.is_none() {
                                        100.0
                                    } else {
                                        liberation_percent(l.health, l.max_health)
                                    };
                                    let state = match (campaign, l.event.is_some(), l.current_owner == Faction::Humans) {
                                        (true, true, _) => "Defense",
                                        (true, false, _) => "Campaign",
                                        (false, _, true) => "Secured",
                                        (false, _, false) => "Enemy held",
                                    };
                                    let index = l.index;
                                    view! {
                                        <li>
                                            <button
                                                on:click=move |_| ui().open_detail(Some(index))
                                                style=format!("--k: {};", faction_color(Some(l.current_owner)))
                                            >
                                                <FactionEmblem faction=l.current_owner.as_str() size=18.0 />
                                                <span class="pd-line-name caps">{l.name.clone()}</span>
                                                <span class=format!("pd-line-state caps {}", if campaign { "on" } else { "" })>{state}</span>
                                                <span class="num pd-line-pct">{fmt_pct(pct, 0)}</span>
                                            </button>
                                        </li>
                                    }
                                })
                                .collect_view()}
                        </ul>
                    }
                    .into_any()
                }}
            </Panel>
        }
    }
}
