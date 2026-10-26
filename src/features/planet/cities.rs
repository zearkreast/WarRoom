use crate::features::major_order::order_bar::{OrderBar, use_count_up};
use crate::ui::icons::{self, Icon};
use leptos::prelude::*;
use std::f64::consts::PI;
use warroom_core::api::{Faction, Planet};
use warroom_core::biomes::resistance_level;
use warroom_core::cities::{City, CityState, cities_of, city_hours_left, size_label};
use warroom_core::factions::faction_color;
use warroom_core::format::{fmt_duration, fmt_int, fmt_pct, to_fixed};

fn state_label(s: CityState) -> &'static str {
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
pub fn CityMark(stars: usize, state: CityState, #[prop(default = 32.0)] size: f64) -> impl IntoView {
    let size = crate::js::num(size);
    view! {
        <svg class=format!("city-mark {}", state.as_str()) width=size.clone() height=size viewBox="0 0 24 24" aria-hidden="true">
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
fn CityRow(city: Memo<Option<City>>, enemy: Memo<Option<Faction>>, full: bool) -> impl IntoView {
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
            <li class=format!("city {}", c.state.as_str())>
                <CityMark stars=c.stars state=c.state />
                <div class="city-main">
                    <div class="city-head">
                        <b class="caps">{c.name.clone()}</b>
                        <span class="city-state caps">{state_label(c.state)}</span>
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

/// A planet's cities: size class, standing, and while one is fought over its progress, pace and payoff.
#[component]
pub fn CityList(#[prop(into)] planet: Signal<Option<Planet>>, #[prop(optional)] full: bool) -> impl IntoView {
    let cities = Memo::new(move |_| planet.with(|p| p.as_ref().map(cities_of).unwrap_or_default()));
    let enemy = Memo::new(move |_| {
        planet.with(|p| {
            let p = p.as_ref()?;
            p.event.as_ref().map(|e| e.faction).or((p.current_owner != Faction::Humans).then_some(p.current_owner))
        })
    });
    view! {
        <ul class=format!("cities {}", if full { "full" } else { "" })>
            <For each=move || cities.with(|l| l.iter().map(|c| c.id).collect::<Vec<_>>()) key=|id| *id let:id>
                <CityRow city=Memo::new(move |_| cities.with(|l| l.iter().find(|c| c.id == id).cloned())) enemy=enemy full=full />
            </For>
        </ul>
    }
}
