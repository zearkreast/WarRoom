//! A planet's cities: their marks and the list.

use crate::features::major_order::order_bar::{OrderBar, use_count_up};
use crate::state::{CityRef, ui};
use crate::ui::icons::{self, Icon};
use leptos::prelude::*;
use std::f64::consts::PI;
use warroom_core::api::{Faction, Planet};
use warroom_core::biomes::resistance_level;
use warroom_core::cities::{BarMark, City, CityState, bar_marks, cities_of, city_hours_left, size_label};
use warroom_core::factions::faction_color;
use warroom_core::format::{fmt_duration, fmt_int, fmt_pct, to_fixed};

pub fn state_label(s: CityState) -> &'static str {
    match s {
        CityState::Liberating => "Liberating",
        CityState::Locked => "Locked",
        CityState::Held => "Enemy held",
        CityState::Secured => "Secured",
        CityState::Destroyed => "Destroyed",
        CityState::Rebuilt => "Rebuilt",
        CityState::Intact => "Intact",
    }
}

fn star(x: f64, y: f64, r: f64) -> String {
    (0..10)
        .map(|i| {
            let a = PI / 5.0 * i as f64 - PI / 2.0;
            let d = if i % 2 == 1 { r * 0.45 } else { r };
            format!("{},{}", to_fixed(x + a.cos() * d, 2), to_fixed(y + a.sin() * d, 2))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// A city's icon as the game draws it: towers under one star per size class.
#[component]
pub fn CityMark(
    stars: usize,
    state: CityState,
    #[prop(default = 32.0)] size: f64,
    #[prop(optional)] holder: Option<Faction>,
) -> impl IntoView {
    let size = crate::js::num(size);
    view! {
        <svg
            class=format!("city-mark {}", state.as_str())
            width=size.clone()
            height=size
            viewBox="0 0 24 24"
            style=holder.map(|h| format!("color: {};", faction_color(Some(h))))
            aria-hidden="true"
        >
            <g class="city-stars">
                {(0..stars)
                    .map(|i| view! { <polygon points=star(12.0 + (i as f64 - (stars as f64 - 1.0) / 2.0) * 5.6, 3.2, 2.7) /> })
                    .collect_view()}
            </g>
            <path class="city-towers" d="M4.5 23V12.5l3.5-2V23ZM10 23V9l2-3 2 3v14ZM16 23V10.5l3.5 2V23Z" />
            {(state == CityState::Destroyed).then(|| view! { <path class="city-razed" d="M3 9l18 14M21 9L3 23" /> })}
        </svg>
    }
}

/// Liberation % between two marks below which the second stacks over the first instead of overlapping it.
const MARK_GAP: f64 = 8.0;

/// A liberation bar with each city's mark standing where it opens; the map's cards and the planet page share it.
/// Wraps the bar it marks; none while the planet is defended.
#[component]
pub fn CityBar(#[prop(into)] planet: Signal<Planet>, children: Children) -> impl IntoView {
    let marks = Memo::new(move |_| planet.with(|p| bar_marks(p, MARK_GAP)));
    let rows = Memo::new(move |_| marks.with(|m| m.iter().map(|m| m.row + 1).max().unwrap_or(0)));
    view! {
        <div class="city-bar" style=move || format!("--rows: {};", rows.get())>
            {move || {
                marks
                    .get()
                    .into_iter()
                    .rev()
                    .map(|BarMark { city: c, at, row }| {
                        let open = c.state == CityState::Liberating;
                        let title = match (at > 0.0, open) {
                            (true, true) => format!("{} opened at {}", c.name, fmt_pct(at, 1)),
                            (true, false) => format!("{} opens at {}", c.name, fmt_pct(at, 1)),
                            (false, true) => format!("{}: open from the start", c.name),
                            (false, false) => format!("{}: {}", c.name, state_label(c.state)),
                        };
                        view! {
                            <span
                                class=format!("city-bar-mark {}", if open { "open" } else { "" })
                                style=format!("left: {}%; --row: {row}; --k: {};", crate::js::num(at), faction_color(Some(c.holder)))
                                title=title
                            >
                                <CityMark stars=c.stars state=c.state size=16.0 holder=c.holder />
                            </span>
                        }
                    })
                    .collect_view()
            }}
            {children()}
        </div>
    }
}

/// What a city's status line says: where it opens while it waits on its planet, else its state.
pub fn status_text(c: &City) -> String {
    match c.opens_at.filter(|_| c.waiting) {
        Some(at) => format!("Opens at {}", fmt_pct(at, 1)),
        None => state_label(c.state).to_string(),
    }
}

fn signed(n: f64) -> String {
    format!("{}{}/h", if n >= 0.0 { "+" } else { "−" }, fmt_pct(n.abs(), 2))
}

fn eta(c: &City) -> AnyView {
    if let Some(hours) = city_hours_left(c.percent, c.rate) {
        return fmt_duration(hours * 3_600_000.0).into_any();
    }
    if c.rate.is_some_and(|r| r < 0.0) {
        return view! { <span class="bad">"Losing ground"</span> }.into_any();
    }
    view! { <span class="muted">"Measuring…"</span> }.into_any()
}

#[component]
fn CityRow(city: Memo<Option<City>>, enemy: Memo<Option<Faction>>, full: bool, on_open: Callback<()>) -> impl IntoView {
    let fought = Memo::new(move |_| {
        enemy.get().is_some()
            && city.with(|c| {
                c.as_ref()
                    .is_some_and(|c| c.percent.is_some() && (c.state == CityState::Liberating || c.percent > Some(0.0)))
            })
    });
    let percent = Signal::derive(move || city.with(|c| c.as_ref().and_then(|c| c.percent).unwrap_or(0.0)));
    let shown = use_count_up(Signal::derive(move || if fought.get() { percent.get() } else { 0.0 }));
    move || {
        let c = city.get()?;
        let contested =
            enemy.get().is_some() && matches!(c.state, CityState::Liberating | CityState::Locked | CityState::Held);
        let res = c.resistance.map(resistance_level);
        let p = c.percent.unwrap_or(0.0);
        let rest = faction_color(enemy.get()).to_string();
        Some(view! {
            <li
                class=format!("city {} openable", c.state.as_str())
                role="button"
                tabindex="0"
                aria-label=format!("{}: open the city", c.name)
                on:click=move |_| on_open.run(())
                on:keydown=move |e| {
                    if e.key() == "Enter" || e.key() == " " {
                        e.prevent_default();
                        on_open.run(());
                    }
                }
            >
                <CityMark stars=c.stars state=c.state holder=c.holder />
                <div class="city-main">
                    <div class="city-head">
                        <b class="caps">{c.name.clone()}</b>
                        <span class="city-state caps">{status_text(&c)}</span>
                    </div>
                    <span class="city-size caps">
                        {size_label(&c.size)}
                        {c.formerly.clone().map(|f| format!(" · formerly {f}"))}
                        {(c.players > 0.0).then(|| view! {
                            " · "
                            <Icon icon=icons::USERS size=11.0 />
                            " "
                            <span class="num">{fmt_int(c.players)}</span>
                        })}
                    </span>
                    {fought.get().then(|| view! {
                        <OrderBar
                            slim=!full
                            shown=shown
                            value=percent
                            label=format!("{} liberated", c.name)
                            value_text=Signal::derive(move || fmt_pct(percent.get(), 1))
                            tone="var(--f-humans)".to_string()
                            rest=rest
                        />
                    })}
                    {(fought.get() && full).then(|| view! {
                        <dl class="city-stats">
                            <div>
                                <dt class="caps">"Liberated"</dt>
                                <dd class="num">{fmt_pct(p, 2)}</dd>
                            </div>
                            <div>
                                <dt class="caps">"Resistance"</dt>
                                <dd class="num">
                                    {match (res, c.resistance) {
                                        (Some((label, color)), Some(r)) => view! {
                                            {format!("{}/h ", fmt_pct(r, 2))}
                                            <span style=format!("color: {color};")>{label}</span>
                                        }
                                        .into_any(),
                                        _ => view! { <span class="muted">"—"</span> }.into_any(),
                                    }}
                                </dd>
                            </div>
                            <div>
                                <dt class="caps">"Progress"</dt>
                                <dd class=format!(
                                    "num {}",
                                    match c.rate {
                                        None => "muted",
                                        Some(r) if r >= 0.0 => "good",
                                        Some(_) => "bad",
                                    }
                                )>{c.rate.map_or("Measuring…".to_string(), signed)}</dd>
                            </div>
                            <div>
                                <dt class="caps">"Liberated in"</dt>
                                <dd class="num">{eta(&c)}</dd>
                            </div>
                        </dl>
                    })}
                    {(fought.get() && !full).then(|| view! {
                        <span class="city-line num caps">
                            {fmt_pct(p, 1)}
                            {c.rate.map(|r| format!(" · {}", signed(r)))}
                            {city_hours_left(c.percent, c.rate).map(|_| view! { " · " {eta(&c)} })}
                        </span>
                    })}
                    {(contested && c.boost.is_some()).then(|| view! {
                        <span class="city-boost caps">
                            <Icon icon=icons::TRENDING_UP size=12.0 />
                            <b class="num">{format!("+{}", fmt_pct(c.boost.unwrap_or(0.0), 1))}</b>
                            {if full { " planet liberation when freed" } else { " boost" }}
                        </span>
                    })}
                    {c.note.filter(|_| full).map(|n| view! { <p class="city-note">{n}</p> })}
                    {(full && !c.description.is_empty()).then(|| view! { <p>{c.description.clone()}</p> })}
                </div>
            </li>
        })
    }
}

pub fn enemy_of_cities(p: &Planet) -> Option<Faction> {
    p.event.as_ref().map(|e| e.faction).or((p.current_owner != Faction::Humans).then_some(p.current_owner))
}

/// A planet's cities, in the order their marks stand on the bar: size class, standing, and while one is fought
/// over its progress, pace and payoff. Each opens its own page.
#[component]
pub fn CityList(#[prop(into)] planet: Signal<Option<Planet>>, #[prop(optional)] full: bool) -> impl IntoView {
    let cities = Memo::new(move |_| {
        let mut list = planet.with(|p| p.as_ref().map(cities_of).unwrap_or_default());
        list.sort_by(|a, b| match (a.opens_at, b.opens_at) {
            (Some(x), Some(y)) => x.total_cmp(&y).then(a.id.cmp(&b.id)),
            (x, y) => y.is_some().cmp(&x.is_some()).then(a.id.cmp(&b.id)),
        });
        list
    });
    let enemy = Memo::new(move |_| planet.with(|p| p.as_ref().and_then(enemy_of_cities)));
    let index = Memo::new(move |_| planet.with(|p| p.as_ref().map(|p| p.index)));
    view! {
        <ul class=format!("cities {}", if full { "full" } else { "" })>
            <For each=move || cities.with(|l| l.iter().map(|c| c.id).collect::<Vec<_>>()) key=|id| *id let:id>
                <CityRow
                    city=Memo::new(move |_| cities.with(|l| l.iter().find(|c| c.id == id).cloned()))
                    enemy=enemy
                    full=full
                    on_open=Callback::new(move |_| {
                        if let Some(planet) = index.get_untracked() {
                            ui().open_city(Some(CityRef { planet, region: id }));
                        }
                    })
                />
            </For>
        </ul>
    }
}
