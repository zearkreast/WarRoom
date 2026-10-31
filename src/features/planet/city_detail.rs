//! A city's own page.

use crate::api::history::{self, CityReading, PlayerReading};
use crate::api::queries::{self, queries};
use crate::features::map::card::LiberationBar;
use crate::features::planet::activity::{ActivitySection, ProgressChart};
use crate::features::planet::cities::{CityMark, enemy_of_cities, state_label};
use crate::features::planet::tint::planet_tint;
use crate::js;
use crate::state::{CityRef, HellpadModule, ui};
use crate::ui::icons::{self, Icon};
use crate::ui::primitives::{Empty, Loading, Meter, MeterSize, Overlay, Panel, Tile};
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use warroom_core::api::{Faction, Planet};
use warroom_core::biomes::resistance_level;
use warroom_core::cities::{City, CityState, cities_of, city_hours_left, size_label};
use warroom_core::factions::{faction_color, faction_label};
use warroom_core::format::{fmt_duration, fmt_int, fmt_pct};
use warroom_core::progress::tracked_percent;

const DAY_MS: f64 = 86_400_000.0;
const REFRESH_MS: f64 = 5.0 * 60_000.0;

#[derive(Clone, Default, PartialEq)]
struct Readings {
    loading: bool,
    rows: Arc<Vec<CityReading>>,
}

/// A city's last 24 hours of readings, re-read every 5 minutes.
fn use_city_readings(at: CityRef) -> ReadSignal<Readings> {
    let (state, set_state) = signal(Readings { loading: true, ..Default::default() });
    let tick = js::use_now(REFRESH_MS);
    let last_update = queries().last_update;
    let generation = Arc::new(AtomicU64::new(0));
    let alive = generation.clone();
    on_cleanup(move || {
        alive.fetch_add(1, Ordering::SeqCst);
    });
    Effect::new(move |_| {
        tick.track();
        let since = last_update.get().unwrap_or_else(js::now) - DAY_MS;
        let run = generation.fetch_add(1, Ordering::SeqCst) + 1;
        let generation = generation.clone();
        spawn_local(async move {
            let result = history::city_readings(at.planet, at.region, since).await;
            if generation.load(Ordering::SeqCst) != run {
                return;
            }
            match result {
                Ok(rows) => set_state.set(Readings { loading: false, rows: Arc::new(rows) }),
                Err(_) => set_state.update(|s| s.loading = false),
            }
        });
    });
    state
}

fn signed(n: f64) -> String {
    format!("{}{}/h", if n >= 0.0 { "+" } else { "−" }, fmt_pct(n.abs(), 2))
}

fn fought(c: &City, enemy: Option<Faction>) -> bool {
    enemy.is_some() && c.percent.is_some() && (c.state == CityState::Liberating || c.percent > Some(0.0))
}

fn opens_at(c: &City) -> Option<f64> {
    c.opens_at.filter(|a| *a != 0.0)
}

/// Where the city stands: its liberation while it's fought over, and what freeing it is worth.
#[component]
fn CityStatus(c: Signal<City>, enemy: Signal<Option<Faction>>) -> impl IntoView {
    let is_fought = Memo::new(move |_| c.with(|c| fought(c, enemy.get())));
    let tone = Memo::new(move |_| faction_color(Some(c.with(|c| c.holder))).to_string());
    let state = move || {
        c.with(|c| match c.opens_at.filter(|_| c.waiting) {
            Some(at) => format!("Opens at {}", fmt_pct(at, 1)),
            None => state_label(c.state).to_string(),
        })
    };
    let tiles = move || {
        let c = c.get();
        let fought = is_fought.get();
        let res = c.resistance.map(resistance_level);
        let hours = city_hours_left(c.percent, c.rate);
        let tone = tone.get();
        view! {
            <div class="pd-tiles four">
                <Tile
                    icon=icons::FLAG
                    label="Liberated"
                    value=c.percent.map_or("—".to_string(), |p| fmt_pct(p, 2)).into_any()
                    color=fought.then(|| "var(--f-humans)".to_string())
                />
                {match (c.resistance, res) {
                    (Some(r), Some((label, color))) => view! {
                        <Tile
                            icon=icons::GAUGE
                            label="Resistance"
                            value=format!("{}/h", fmt_pct(r, 2)).into_any()
                            sub=view! { <span style=format!("color: {color};")>{label}</span> }.into_any()
                        />
                    }
                    .into_any(),
                    _ => view! { <Tile icon=icons::GAUGE label="Resistance" value="—".into_any() /> }.into_any(),
                }}
                <Tile
                    icon=icons::TRENDING_UP
                    label="Progress"
                    value=match (fought, c.rate) {
                        (false, _) => "—".to_string(),
                        (true, None) => "Measuring…".to_string(),
                        (true, Some(r)) => signed(r),
                    }
                    .into_any()
                    color=c
                        .rate
                        .filter(|_| fought)
                        .map(|r| if r >= 0.0 { "var(--good)".to_string() } else { "var(--bad)".to_string() })
                />
                <Tile
                    icon=icons::TIMER
                    label="Liberated in"
                    value=match hours {
                        Some(h) => fmt_duration(h * 3_600_000.0),
                        None if fought && c.rate.is_some_and(|r| r < 0.0) => "Losing ground".to_string(),
                        None => "—".to_string(),
                    }
                    .into_any()
                />
            </div>
            <div class="pd-tiles three">
                <Tile icon=icons::USERS label="Helldivers" value=fmt_int(c.players).into_any() color="var(--cta)".to_string() />
                <Tile
                    icon=icons::TRENDING_UP
                    label="Planet boost"
                    value=c.boost.map_or("—".to_string(), |b| format!("+{}", fmt_pct(b, 1))).into_any()
                    sub=if c.defense { "defense when freed" } else { "planet liberation when freed" }.into_any()
                    color=c.boost.map(|_| "var(--good)".to_string())
                />
                {match opens_at(&c) {
                    Some(at) => view! {
                        <Tile
                            icon=icons::LOCK
                            label="Opens at"
                            value=fmt_pct(at, 1).into_any()
                            sub="planet liberation".into_any()
                        />
                    }
                    .into_any(),
                    None => view! {
                        <Tile icon=icons::SHIELD label="Held by" value=faction_label(Some(c.holder)).into_any() color=tone />
                    }
                    .into_any(),
                }}
            </div>
            {(!c.description.is_empty() || c.note.is_some()).then(|| view! {
                <div class="cd-about">
                    {(!c.description.is_empty()).then(|| view! { <p>{c.description.clone()}</p> })}
                    {c.note.map(|n| view! { <p class="cd-note">{n}</p> })}
                </div>
            })}
        }
    };
    move || {
        let tone = tone.get();
        view! {
            <Panel
                title="City status"
                icon=icons::BUILDING_2
                tone=tone.clone()
                right=view! { <span style=format!("color: {tone};")>{state}</span> }.into_any()
            >
                <div class="pd-stack">
                    <Show when=move || is_fought.get()>
                        <Meter
                            value=Signal::derive(move || c.with(|c| c.percent.unwrap_or(0.0)))
                            color="var(--f-humans)".to_string()
                            rest=Signal::derive(move || faction_color(enemy.get()).to_string())
                            size=MeterSize::L
                            label="City liberated"
                        />
                    </Show>
                    {tiles}
                </div>
            </Panel>
        }
    }
}

fn hero_style(p: &Planet, enemy: Option<Faction>) -> String {
    let t = planet_tint(p, &[]);
    format!(
        "--tone: {}; --sky: {}; --ground: {};{}",
        faction_color(Some(enemy.unwrap_or(Faction::Humans))),
        t.sky,
        t.ground,
        t.art.map(|a| format!(" --art: {a};")).unwrap_or_default()
    )
}

#[component]
fn Found(p: Signal<Planet>, c: Signal<City>, readings: ReadSignal<Readings>) -> impl IntoView {
    let enemy = Signal::derive(move || p.with(enemy_of_cities));
    let index = move || p.with(|p| p.index);
    let open_planet = move |_| ui().open_detail(Some(index()));
    let status = move || {
        c.with(|c| match c.opens_at.filter(|_| c.waiting) {
            Some(at) => format!("Opens at {} planet liberation", fmt_pct(at, 1)),
            None => format!(
                "{} · {}",
                size_label(&c.size),
                match enemy.get() {
                    Some(e) => format!("{} front", faction_label(Some(e))),
                    None => format!("{} control", faction_label(Some(p.with(|p| p.current_owner)))),
                }
            ),
        })
    };
    let planet_pct = Signal::derive(move || if enemy.get().is_some() { p.with(tracked_percent) } else { 100.0 });
    let progress = Signal::derive(move || {
        Arc::new(readings.with(|r| r.rows.iter().filter_map(|r| r.percent.map(|v| (r.at, v))).collect::<Vec<_>>()))
    });
    let has_progress = Memo::new(move |_| progress.with(|p| !p.is_empty()));
    let look = Memo::new(move |_| c.with(|c| (c.stars, c.state, c.holder)));
    let mark = move || {
        let (stars, state, holder) = look.get();
        view! { <CityMark stars=stars state=state size=46.0 holder=holder /> }
    };
    let opening = Memo::new(move |_| {
        Some((enemy.get()?, c.with(opens_at)?, c.with(|c| c.name.clone()), p.with(|p| p.name.clone())))
    });
    let liberation = move || {
        let (e, at, name, planet) = opening.get()?;
        Some(view! {
            <Panel title=format!("{planet} liberation") icon=icons::COMPASS tone=faction_color(Some(e)).to_string()>
                <div class="pd-stack">
                    <LiberationBar p=p pct=planet_pct enemy=Signal::derive(move || enemy.get().unwrap_or(e)) />
                    <p class="muted">
                        {format!("{name} opens once {planet} is {} liberated; the planet stands at", fmt_pct(at, 1))}
                        " "
                        {move || fmt_pct(planet_pct.get(), 2)}
                        "."
                    </p>
                </div>
            </Panel>
        })
    };
    view! {
        <div class="pd">
            <section class="pd-hero brackets" style=move || p.with(|p| hero_style(p, enemy.get()))>
                <div class="pd-hero-main">
                    <nav class="pd-crumbs caps" aria-label="Breadcrumb">
                        <button class="link" on:click=move |_| ui().open_module(Some(HellpadModule::Planets))>
                            "Planets"
                        </button>
                        <span>"/"</span>
                        <button class="link" on:click=open_planet>
                            {move || p.with(|p| p.name.clone())}
                        </button>
                        <span>"/"</span>
                        <span>"Cities"</span>
                    </nav>
                    <div class="pd-name-row">
                        {mark}
                        <h1 class="pd-name">{move || c.with(|c| c.name.clone())}</h1>
                    </div>
                    <div class="pd-status caps">{status}</div>
                    <div class="pd-chips">
                        <span class="chip">
                            <Icon icon=icons::BUILDING_2 size=13.0 />
                            " "
                            {move || c.with(|c| size_label(&c.size))}
                        </span>
                        <button class="chip" on:click=open_planet>
                            <Icon icon=icons::COMPASS size=13.0 />
                            " "
                            {move || p.with(|p| format!("{} · {} sector", p.name, p.sector))}
                        </button>
                        <span class="chip num">
                            <Icon icon=icons::USERS size=13.0 />
                            " "
                            {move || fmt_int(c.with(|c| c.players))}
                            " "
                            <span class="muted">"deployed"</span>
                        </span>
                    </div>
                </div>
            </section>
            <CityStatus c=c enemy=enemy />
            {liberation}
            <ActivitySection
                title="City activity · last 24 h".to_string()
                now=Signal::derive(move || c.with(|c| c.players))
                points=Signal::derive(move || {
                    Arc::new(
                        readings.with(|r| {
                            r.rows.iter().map(|r| PlayerReading { at: r.at, players: r.players }).collect::<Vec<_>>()
                        }),
                    )
                })
                loading=Signal::derive(move || readings.with(|r| r.loading))
                empty=Signal::derive(move || {
                    format!(
                        "No readings of {} in the last 24 hours. One is kept every few minutes while Helldivers are there or it's being liberated.",
                        c.with(|c| c.name.clone())
                    )
                })
            >
                <Show when=move || has_progress.get()>
                    <div class="pd-chart-label caps">"Liberation"</div>
                    <ProgressChart points=progress />
                </Show>
            </ActivitySection>
        </div>
    }
}

#[component]
fn Body(at: CityRef) -> impl IntoView {
    let planet = Memo::new(move |_| queries::planet(Some(at.planet)));
    let city = Memo::new(move |_| {
        planet.with(|p| p.as_ref().and_then(|p| cities_of(p).into_iter().find(|c| c.id == at.region)))
    });
    let readings = use_city_readings(at);
    let ready = Memo::new(move |_| planet.with(Option::is_some));
    let found = Memo::new(move |_| city.with(Option::is_some));
    view! {
        <Show when=move || ready.get() fallback=|| view! { <Loading what="city record" /> }>
            <Show
                when=move || found.get()
                fallback=move || {
                    view! {
                        <Empty>
                            "This city is no longer listed on "
                            {move || planet.with(|p| p.as_ref().map(|p| p.name.clone()))}
                            "."
                        </Empty>
                    }
                }
            >
                {move || {
                    let p = Signal::derive(move || planet.get().unwrap_or_default());
                    let c = Signal::derive(move || city.get().unwrap_or_default());
                    view! { <Found p=p c=c readings=readings /> }
                }}
            </Show>
        </Show>
    }
}

#[component]
pub fn CityDetail() -> impl IntoView {
    let city = Memo::new(move |_| ui().city.get());
    move || {
        let at = city.get()?;
        Some(view! {
            <Overlay on_close=Callback::new(|_| ui().open_city(None)) label="City details">
                <Body at=at />
            </Overlay>
        })
    }
}
