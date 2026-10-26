use crate::api::queries::queries;
use crate::features::hellpad::gww::nav::use_gww_nav;
use crate::features::planets::fronts::{Front, FrontKind, fronts};
use crate::js;
use crate::state::{OptionsAnchor, ui};
use crate::ui::emblems::FactionEmblem;
use crate::ui::icons::{self, Icon};
use crate::ui::primitives::{Countdown, LiveDot};
use leptos::prelude::*;
use std::collections::HashMap;
use warroom_core::api::{Faction, Planet};
use warroom_core::biomes::regen_pct_per_hour;
use warroom_core::factions::{faction_color, faction_label};
use warroom_core::format::{fmt_compact, fmt_int, fmt_pct, fmt_rate, to_fixed};
use warroom_core::gww_nav::title_of;
use warroom_core::major_order::order_name;
use warroom_core::progress::liberation_percent;
use warroom_core::snapshot::EpisodeStatus;
use warroom_core::time::parse_ms;

/// `actions` are link targets.
fn live(title: &'static str, actions: &'static [&'static str], children: AnyView) -> AnyView {
    let nav = use_gww_nav();
    view! {
        <section class="gww-live" aria-label=format!("Live: {title}")>
            <header class="gww-live-head caps">
                <LiveDot />
                " Live from the front "
                <span>{format!("· {title}")}</span>
            </header>
            {children}
            {(!actions.is_empty()).then(|| view! {
                <div class="gww-live-actions">
                    {actions
                        .iter()
                        .map(|t| view! {
                            <button class="gww-chip" on:click=move |e| nav.follow(t, Some(&e))>
                                {format!("{} ", title_of(t))}
                                <Icon icon=icons::ARROW_UP_RIGHT size=12.0 />
                            </button>
                        })
                        .collect_view()}
                </div>
            })}
        </section>
    }
    .into_any()
}

fn waiting(what: &'static str) -> AnyView {
    view! { <p class="gww-live-empty">{what}</p> }.into_any()
}

fn planet_name(p: &Planet) -> AnyView {
    let index = p.index;
    view! {
        <button
            class="gww-live-planet"
            style=format!("color: {};", faction_color(Some(p.current_owner)))
            on:click=move |e| ui().open_options(Some(OptionsAnchor { planet: index, x: e.client_x() as f64, y: e.client_y() as f64 }))
        >
            <FactionEmblem faction=p.current_owner.as_str() size=14.0 />
            {p.name.clone()}
        </button>
    }
    .into_any()
}

fn stat(label: &'static str, value: AnyView, tone: Option<&str>) -> AnyView {
    view! {
        <div class="gww-live-stat">
            <span class="caps">{label}</span>
            <b class="num" style=tone.map(|t| format!("color: {t};"))>{value}</b>
        </div>
    }
    .into_any()
}

fn count(n: usize, one: &str) -> String {
    format!("{n} {}", if n == 1 { one.to_string() } else { format!("{one}s") })
}

fn rate_tone(rate: Option<f64>) -> Option<&'static str> {
    match rate {
        Some(r) if r > 0.0 => Some("var(--good)"),
        Some(r) if r < 0.0 => Some("var(--bad)"),
        _ => None,
    }
}

fn front_row(f: &Front, extra: AnyView) -> AnyView {
    view! {
        <li>
            {planet_name(&f.planet)}
            <span class="num">{fmt_pct(f.percent, 1)}</span>
            <span class="num" style=rate_tone(f.rate).map(|t| format!("color: {t};"))>
                {if f.rate.is_none() { "measuring".to_string() } else { fmt_rate(f.rate) }}
            </span>
            {extra}
        </li>
    }
    .into_any()
}

fn fronts_loading() -> bool {
    let q = queries();
    q.raw.loading.get() && q.campaigns.with(Option::is_none)
}

#[component]
fn LiveBasics() -> impl IntoView {
    let q = queries();
    let all = fronts();
    move || {
        let war = q.war.get();
        let Some(war) = war.filter(|_| !fronts_loading()) else {
            return live("The war today", &[], waiting("Waiting for the collector…"));
        };
        let list = all.get();
        let defenses = list.iter().filter(|f| f.kind == FrontKind::Defense).count();
        let order = q.assignments.with(|a| a.as_ref().and_then(|a| a.first().map(|o| parse_ms(&o.expiration))));
        live(
            "The war today",
            &["app:campaigns", "app:overview"],
            view! {
                <div class="gww-live-stats">
                    {stat("Helldivers online", fmt_int(war.statistics.player_count).into_any(), None)}
                    {stat("Fronts", format!("{} · {}", count(list.len() - defenses, "liberation"), count(defenses, "defense")).into_any(), None)}
                    {stat("Impact modifier", to_fixed(war.impact_multiplier, 4).into_any(), Some("var(--impact)"))}
                    {order.map(|o| stat("Major Order ends in", view! { <Countdown to=o /> }.into_any(), Some("var(--cta)")))}
                </div>
            }
            .into_any(),
        )
    }
}

#[component]
fn LiveSupplyLines() -> impl IntoView {
    let q = queries();
    move || {
        let attacks: Option<Vec<(Planet, Planet)>> = q.fresh_planets.with(|p| {
            let p = p.as_ref()?;
            let by_index: HashMap<i64, &Planet> = p.iter().map(|x| (x.index, x)).collect();
            Some(
                p.iter()
                    .flat_map(|from| {
                        from.attacking.iter().filter_map(|i| by_index.get(i).map(|to| (from.clone(), (*to).clone())))
                    })
                    .collect(),
            )
        });
        let body = match attacks {
            None => waiting("Waiting for the collector…"),
            Some(a) if a.is_empty() => waiting("No planet is attacking another right now."),
            Some(a) => view! {
                <ul class="gww-live-rows">
                    {a.iter()
                        .take(8)
                        .map(|(from, to)| view! {
                            <li>
                                {planet_name(from)}
                                <Icon icon=icons::ARROW_RIGHT size=14.0 label="attacks" />
                                {planet_name(to)}
                            </li>
                        })
                        .collect_view()}
                </ul>
            }
            .into_any(),
        };
        live("Attacks along the lines", &["app:map"], body)
    }
}

#[component]
fn LiveCampaignTypes() -> impl IntoView {
    let all = fronts();
    move || {
        if fronts_loading() {
            return live("Campaigns now", &[], waiting("Waiting for the collector…"));
        }
        let list = all.get();
        let defenses = list.iter().filter(|f| f.kind == FrontKind::Defense).count();
        live(
            "Campaigns now",
            &["app:campaigns"],
            view! {
                <div class="gww-live-stats">
                    {stat("Liberations", (list.len() - defenses).into_any(), Some("var(--f-humans)"))}
                    {stat("Defenses", defenses.into_any(), (defenses > 0).then_some("var(--bad)"))}
                    {stat("Major Order targets", list.iter().filter(|f| f.in_major_order).count().into_any(), Some("var(--cta)"))}
                </div>
            }
            .into_any(),
        )
    }
}

#[component]
fn LiveLiberation() -> impl IntoView {
    let all = fronts();
    move || {
        let mut top: Vec<Front> = all.get().iter().filter(|f| f.kind == FrontKind::Liberation).cloned().collect();
        top.sort_by(|a, b| b.players.total_cmp(&a.players));
        top.truncate(5);
        let body = if fronts_loading() {
            waiting("Waiting for the collector…")
        } else if top.is_empty() {
            waiting("No liberation campaign is open right now.")
        } else {
            view! {
                <ul class="gww-live-rows">
                    {top
                        .iter()
                        .map(|f| {
                            let extra = match f.victory_at.filter(|v| *v != 0.0 && !v.is_nan()) {
                                Some(v) => view! { <span class="muted num">"falls in " <Countdown to=v /></span> }.into_any(),
                                None => view! { <span class="muted num">{format!("{} Helldivers", fmt_compact(f.players))}</span> }.into_any(),
                            };
                            front_row(f, extra)
                        })
                        .collect_view()}
                </ul>
            }
            .into_any()
        };
        live("Busiest liberations", &["app:campaigns"], body)
    }
}

fn regen(p: &Planet) -> f64 {
    regen_pct_per_hour(p.regen_per_second, p.max_health)
}

#[component]
fn LiveResistance() -> impl IntoView {
    let all = fronts();
    move || {
        let liberations: Vec<Front> = all.get().iter().filter(|f| f.kind == FrontKind::Liberation).cloned().collect();
        let losing = liberations.iter().filter(|f| f.rate.is_some_and(|r| r < 0.0)).count();
        let mut stiffest = liberations.clone();
        stiffest.sort_by(|a, b| regen(&b.planet).total_cmp(&regen(&a.planet)));
        stiffest.truncate(4);
        let body = if fronts_loading() {
            waiting("Waiting for the collector…")
        } else if liberations.is_empty() {
            waiting("No liberation campaign is open right now.")
        } else {
            view! {
                <div class="gww-live-stats">
                    {stat("Fronts losing ground", losing.into_any(), Some(if losing > 0 { "var(--bad)" } else { "var(--good)" }))}
                    {stat("Liberation fronts", liberations.len().into_any(), None)}
                </div>
                <p class="gww-live-sub caps">"Stiffest resistance"</p>
                <ul class="gww-live-rows">
                    {stiffest
                        .iter()
                        .map(|f| {
                            front_row(
                                f,
                                view! {
                                    <span class="num" style=format!("color: {};", faction_color(Some(f.enemy)))>
                                        {format!("−{}%/h", to_fixed(regen(&f.planet), 2))}
                                    </span>
                                }
                                .into_any(),
                            )
                        })
                        .collect_view()}
                </ul>
            }
            .into_any()
        };
        live("Resistance now", &["app:campaigns"], body)
    }
}

fn spaced_size(size: &str) -> String {
    let mut out = String::new();
    let mut prev: Option<char> = None;
    for c in size.chars() {
        if c.is_ascii_uppercase() && prev.is_some_and(|p| p.is_ascii_lowercase()) {
            out.push(' ');
        }
        out.push(c);
        prev = Some(c);
    }
    out
}

#[component]
fn LiveRegions() -> impl IntoView {
    let all = fronts();
    move || {
        let mut regions: Vec<(Front, warroom_core::api::Region, f64)> = all
            .get()
            .iter()
            .flat_map(|f| {
                f.planet
                    .regions
                    .iter()
                    .filter(|r| r.is_available && r.health.is_some() && r.max_health > 0.0)
                    .map(|r| (f.clone(), r.clone(), 100.0 - r.health.unwrap_or(0.0) / r.max_health * 100.0))
                    .collect::<Vec<_>>()
            })
            .collect();
        regions.sort_by(|a, b| b.1.players.total_cmp(&a.1.players));
        regions.truncate(6);
        let body = if fronts_loading() {
            waiting("Waiting for the collector…")
        } else if regions.is_empty() {
            waiting("No city is being fought over right now.")
        } else {
            view! {
                <ul class="gww-live-rows">
                    {regions
                        .iter()
                        .map(|(f, r, freed)| view! {
                            <li>
                                <b class="caps">{r.name.clone()}</b>
                                <span class="muted caps">{spaced_size(&r.size)}</span>
                                {planet_name(&f.planet)}
                                <span class="num">{format!("{} freed", fmt_pct(*freed, 1))}</span>
                                <span class="muted num">{format!("{} Helldivers", fmt_compact(r.players))}</span>
                            </li>
                        })
                        .collect_view()}
                </ul>
            }
            .into_any()
        };
        live("Cities in the fight", &["app:campaigns"], body)
    }
}

#[component]
fn LiveDefense() -> impl IntoView {
    let all = fronts();
    let q = queries();
    move || {
        let list = all.get();
        let defenses: Vec<&Front> = list.iter().filter(|f| f.kind == FrontKind::Defense).collect();
        let front_of: HashMap<i64, &Front> = list.iter().map(|f| (f.planet.index, f)).collect();
        let body = if fronts_loading() {
            waiting("Waiting for the collector…")
        } else if defenses.is_empty() {
            waiting("No Super Earth planet is under attack right now.")
        } else {
            let planets = q.fresh_planets.get();
            view! {
                <ul class="gww-live-rows">
                    {defenses
                        .iter()
                        .map(|f| {
                            let attackers: Vec<Planet> = planets
                                .iter()
                                .flat_map(|p| p.iter())
                                .filter(|p| p.current_owner != Faction::Humans && p.attacking.contains(&f.planet.index))
                                .cloned()
                                .collect();
                            view! {
                                <li class="gww-live-defense">
                                    {planet_name(&f.planet)}
                                    <span class="num">{format!("{} defended", fmt_pct(f.percent, 1))}</span>
                                    {f.fail_at.filter(|v| *v != 0.0 && !v.is_nan()).map(|d| view! {
                                        <span class="num" style="color: var(--bad);">"deadline in " <Countdown to=d /></span>
                                    })}
                                    {f.projected.map(|p| view! {
                                        <span class="num" style=format!("color: {};", if p >= 100.0 { "var(--good)" } else { "var(--bad)" })>
                                            {if p >= 100.0 { "on pace to hold".to_string() } else { format!("on pace for {}", fmt_pct(p.max(0.0), 0)) }}
                                        </span>
                                    })}
                                    {(!attackers.is_empty()).then(|| view! {
                                        <span class="gww-live-gambit">
                                            <span class="caps">"Gambit"</span>
                                            {attackers
                                                .iter()
                                                .map(|a| {
                                                    let pct = front_of.get(&a.index).map_or_else(|| liberation_percent(a.health, a.max_health), |f| f.percent);
                                                    view! {
                                                        <span class="gww-live-gambit-planet">
                                                            {planet_name(a)}
                                                            <span class="muted num">{format!("{} liberated", fmt_pct(pct, 1))}</span>
                                                        </span>
                                                    }
                                                })
                                                .collect_view()}
                                        </span>
                                    })}
                                </li>
                            }
                        })
                        .collect_view()}
                </ul>
            }
            .into_any()
        };
        live("Defenses and their gambits", &["app:campaigns", "app:map"], body)
    }
}

#[component]
fn LiveImpact() -> impl IntoView {
    let q = queries();
    move || {
        let body = match q.war.get() {
            None => waiting("Waiting for the collector…"),
            Some(w) => view! {
                <div class="gww-live-stats">
                    {stat("Impact modifier", to_fixed(w.impact_multiplier, 4).into_any(), Some("var(--impact)"))}
                    {stat("Helldivers online", fmt_int(w.statistics.player_count).into_any(), None)}
                    {stat("Missions won, all war", fmt_compact(w.statistics.missions_won).into_any(), None)}
                </div>
            }
            .into_any(),
        };
        live("The modifier now", &["app:overview"], body)
    }
}

#[component]
fn LiveDss() -> impl IntoView {
    let q = queries();
    let now = js::use_now(30_000.0);
    move || {
        let stations = q.stations.get();
        let body = match stations.as_ref().map(|s| s.first().cloned()) {
            None => waiting("Waiting for the collector…"),
            Some(None) => waiting("The Democracy Space Station is out of service."),
            Some(Some(s)) => {
                let planet = q
                    .fresh_planets
                    .with(|p| p.as_ref().and_then(|p| p.iter().find(|p| p.index == s.planet_index).cloned()));
                let n = now.get();
                let running: Vec<String> = s
                    .tactical_actions
                    .iter()
                    .filter(|a| a.status == 2 && parse_ms(&a.status_expire) > n)
                    .map(|a| a.name.clone())
                    .collect();
                view! {
                    <div class="gww-live-stats">
                        {stat("In orbit over", planet.as_ref().map_or_else(|| "Unknown planet".into_any(), planet_name), None)}
                        {stat("Next jump in", view! { <Countdown to=parse_ms(&s.election_end) /> }.into_any(), Some("var(--cta)"))}
                        {stat(
                            "Running",
                            (if running.is_empty() { "No tactical action".to_string() } else { running.join(", ") }).into_any(),
                            (!running.is_empty()).then_some("var(--good)"),
                        )}
                    </div>
                }
                .into_any()
            }
        };
        live("The station now", &["app:stations"], body)
    }
}

#[component]
fn LiveCampaigns() -> impl IntoView {
    let q = queries();
    move || {
        let episodes = q.episodes.get();
        let orders = q.assignments.get();
        let body = if episodes.is_none() && orders.is_none() {
            waiting("Waiting for the collector…")
        } else {
            let running = episodes.as_ref().and_then(|e| e.iter().find(|e| e.status == EpisodeStatus::Active).cloned());
            let order = orders.as_ref().and_then(|o| o.first().cloned());
            let name = order.as_ref().map(|o| {
                order_name(episodes.as_deref().map(|e| e.as_slice()))
                    .or_else(|| o.title.clone())
                    .unwrap_or_else(|| "Major Order".into())
            });
            view! {
                <div class="gww-live-stats">
                    {stat(
                        "Galactic Campaign",
                        running.as_ref().map_or("Between campaigns".to_string(), |r| r.title.clone()).into_any(),
                        running.as_ref().map(|r| faction_color(Some(r.faction))),
                    )}
                    {stat("Major Order", name.unwrap_or_else(|| "None issued".into()).into_any(), order.as_ref().map(|_| "var(--cta)"))}
                    {order.map(|o| stat("Ends in", view! { <Countdown to=parse_ms(&o.expiration) /> }.into_any(), None))}
                </div>
            }
            .into_any()
        };
        live("The story now", &["app:overview", "app:past"], body)
    }
}

#[component]
fn LiveGloom() -> impl IntoView {
    let q = queries();
    let all = fronts();
    move || {
        let body = match q.effects.get() {
            None => waiting("Waiting for the collector…"),
            Some(e) => {
                let in_gloom: Vec<Front> =
                    all.get().iter().filter(|f| e.gloom.contains(&f.planet.index)).cloned().collect();
                view! {
                    <div class="gww-live-stats">
                        {stat("Planets in or near the Gloom", e.gloom.len().into_any(), Some("var(--f-terminids)"))}
                        {stat("Of them, fronts", in_gloom.len().into_any(), None)}
                    </div>
                    {(!in_gloom.is_empty()).then(|| view! {
                        <ul class="gww-live-rows">
                            {in_gloom
                                .iter()
                                .take(6)
                                .map(|f| front_row(f, view! { <span class="muted caps">{faction_label(Some(f.enemy))}</span> }.into_any()))
                                .collect_view()}
                        </ul>
                    })}
                }
                .into_any()
            }
        };
        live("The Gloom now", &["app:map"], body)
    }
}

pub fn live_panel(manual: &str) -> Option<AnyView> {
    Some(match manual {
        "basics" => view! { <LiveBasics /> }.into_any(),
        "supply-lines" => view! { <LiveSupplyLines /> }.into_any(),
        "campaign-types" => view! { <LiveCampaignTypes /> }.into_any(),
        "liberation" => view! { <LiveLiberation /> }.into_any(),
        "resistance" => view! { <LiveResistance /> }.into_any(),
        "regions" => view! { <LiveRegions /> }.into_any(),
        "defense" => view! { <LiveDefense /> }.into_any(),
        "impact" => view! { <LiveImpact /> }.into_any(),
        "dss" => view! { <LiveDss /> }.into_any(),
        "campaigns" => view! { <LiveCampaigns /> }.into_any(),
        "gloom" => view! { <LiveGloom /> }.into_any(),
        _ => return None,
    })
}
