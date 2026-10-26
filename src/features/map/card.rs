//! The map's hover card and the panel a clicked planet docks beside it.

use crate::features::map::poi::{PoiGlyph, PoiKind, poi_image, poi_kind, poi_origin};
use crate::features::planet::cities::CityList;
use crate::features::planet::tint::planet_tint;
use crate::features::planets::fronts::station_planets;
use crate::js::{self, num};
use crate::state::{HellpadModule, settings, ui};
use crate::ui::emblems::{DssIcon, SubfactionEmblem};
use crate::ui::icons::{self, Icon};
use crate::ui::primitives::{Meter, MeterSize};
use leptos::prelude::*;
use warroom_core::api::{Faction, Planet};
use warroom_core::biomes::{biome_tint, regen_pct_per_hour, resistance_level};
use warroom_core::effects::{EffectKind, PlanetEffect, has_pawn};
use warroom_core::factions::{faction_color, faction_label};
use warroom_core::format::{fmt_int, fmt_pct, to_fixed};
use warroom_core::galaxy::{Anomaly, Landmark, anomaly_of};
use warroom_core::progress::{liberation_percent, tracked_percent};

#[derive(Clone)]
pub enum Hovered {
    Planet(Box<Planet>),
    Landmark(&'static Landmark),
}

impl PartialEq for Hovered {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Hovered::Planet(a), Hovered::Planet(b)) => a == b,
            (Hovered::Landmark(a), Hovered::Landmark(b)) => std::ptr::eq(*a, *b),
            _ => false,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Hover {
    pub x: f64,
    pub y: f64,
    pub what: Hovered,
}

fn anomaly_kind(a: Anomaly) -> &'static str {
    match a {
        Anomaly::Wormhole => "Wormhole",
        Anomaly::Blackhole => "Black hole",
        Anomaly::Fractured => "Fractured planet",
        Anomaly::Scorched => "Scorched",
    }
}

fn enemy_of(p: &Planet) -> Option<Faction> {
    p.event.as_ref().map(|e| e.faction).or((p.current_owner != Faction::Humans).then_some(p.current_owner))
}

fn ids(effects: &[PlanetEffect]) -> Vec<i64> {
    effects.iter().map(|e| e.id).collect()
}

fn pois_of(effects: &[PlanetEffect]) -> Vec<PlanetEffect> {
    effects.iter().filter(|e| e.kind == EffectKind::Poi).cloned().collect()
}

/// The colours and picture a planet's card is drawn with.
fn card_look(p: &Planet, effects: &[PlanetEffect]) -> String {
    let tint = planet_tint(p, &ids(effects));
    // As in the game, a point of interest's (or an anomaly's) picture replaces the biome's.
    let art =
        poi_image(&pois_of(effects), anomaly_of(&p.name, &ids(effects))).map(|a| format!("url(\"{a}\")")).or(tint.art);
    let mut s = format!("--sky: {}; --ground: {}; ", tint.sky, tint.ground);
    if let Some(a) = art {
        s.push_str(&format!("--art: {a}; "));
    }
    s.push_str(&format!("--tone: {};", faction_color(Some(enemy_of(p).unwrap_or(Faction::Humans)))));
    s
}

fn poi_row(name: String, description: String, kind: PoiKind, faction: Faction) -> impl IntoView {
    view! {
        <div class="hc-poi">
            <div class="hc-poi-head caps">
                <svg viewBox="-12 -12 24 24" width="16" height="16" aria-hidden="true">
                    <PoiGlyph kind=kind faction=faction size=24.0 />
                </svg>
                {name}
            </div>
            <p>{description}</p>
        </div>
    }
}

/// What the map's hover card and planet panel both show: picture, name, what's there and the fight.
fn planet_summary(p: Signal<Planet>, active: Signal<bool>, effects: Signal<Vec<PlanetEffect>>) -> impl IntoView {
    let enemy = Memo::new(move |_| p.with(enemy_of));
    let anomaly = Memo::new(move |_| p.with(|p| effects.with(|e| anomaly_of(&p.name, &ids(e)))));
    let kind = move || {
        p.with(|p| {
            if p.event.is_some() {
                "Defense"
            } else if active.get() {
                "Liberation"
            } else if let Some(a) = anomaly.get() {
                anomaly_kind(a)
            } else if p.current_owner == Faction::Humans {
                "Secured"
            } else {
                "Enemy held"
            }
        })
    };
    let art_class = move || {
        let poi = effects.with(|e| poi_image(&pois_of(e), anomaly.get()).is_some());
        format!("hc-art {}", if poi { "poi" } else { "" })
    };
    // Black holes and broken worlds list what they are too, as the game's tooltip does.
    let rows = Memo::new(move |_| {
        effects.with(|e| {
            e.iter()
                .filter(|e| e.kind == EffectKind::Look && poi_kind(&e.name) == PoiKind::Anomaly)
                .chain(e.iter().filter(|e| e.kind == EffectKind::Poi))
                .map(|e| (e.name.clone(), e.description.clone()))
                .collect::<Vec<_>>()
        })
    });
    let chips = Memo::new(move |_| {
        effects.with(|e| {
            e.iter()
                .filter(|e| has_pawn(e) && e.kind != EffectKind::Poi)
                .map(|e| (e.kind, e.name.clone()))
                .collect::<Vec<_>>()
        })
    });
    let desc = Memo::new(move |_| p.with(|p| p.biome.description.clone()));
    let pct = Signal::derive(move || if enemy.get().is_some() { p.with(tracked_percent) } else { 100.0 });
    let regen = Memo::new(move |_| p.with(|p| regen_pct_per_hour(p.regen_per_second, p.max_health)));
    let defended = Memo::new(move |_| p.with(|p| p.event.is_some()));
    let shown_pct =
        move || if defended.get() { pct.get() } else { p.with(|p| liberation_percent(p.health, p.max_health)) };
    view! {
        <div class=art_class></div>
        <div class="hc-head">
            <span class="hc-kind caps">{kind}</span>
            <b class="hc-name">{move || p.with(|p| p.name.clone())}</b>
            <span class="hc-sector caps">{move || p.with(|p| p.sector.clone())}" sector"</span>
        </div>
        {move || {
            let list = rows.get();
            if list.is_empty() {
                let d = desc.get();
                (!d.is_empty()).then(|| view! { <p class="hc-desc">{d}</p> }).into_any()
            } else {
                let enemy = enemy.get();
                list.into_iter()
                    .map(|(name, description)| {
                        let kind = poi_kind(&name);
                        poi_row(name, description, kind, poi_origin(kind, enemy))
                    })
                    .collect_view()
                    .into_any()
            }
        }}
        {move || {
            let list = chips.get();
            let enemy = enemy.get();
            (!list.is_empty()).then(|| {
                view! {
                    <ul class="hc-effects caps">
                        {list
                            .into_iter()
                            .map(|(kind, name)| {
                                view! {
                                    <li class=kind.as_str()>
                                        {(kind == EffectKind::Subfaction).then_some(enemy).flatten().map(|f| {
                                            view! {
                                                <SubfactionEmblem name=name.clone() faction=f.as_str() size=12.0 color="currentColor" />
                                            }
                                        })}
                                        {name}
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                }
            })
        }}
        {move || {
            enemy.get().map(|f| {
                view! {
                    <div class="hc-progress">
                        <Meter value=pct color="var(--f-humans)".to_string() rest=faction_color(Some(f)).to_string() size=MeterSize::M />
                        <span class="num caps">
                            {move || fmt_pct(shown_pct(), 4)}
                            " "
                            {move || if defended.get() { "defended" } else { "liberated" }}
                        </span>
                    </div>
                    <div class="hc-row caps">
                        <span>"Resistance"</span>
                        <b style=move || format!("color: {};", resistance_level(regen.get()).1)>
                            {move || resistance_level(regen.get()).0}
                            " "
                            <span class="num">"("{move || to_fixed(regen.get(), 2)}"%)"</span>
                        </b>
                    </div>
                }
            })
        }}
    }
}

pub fn hover_card(h: &Hover, active: Signal<bool>, effects: Signal<Vec<PlanetEffect>>) -> AnyView {
    let width = js::window().inner_width().ok().and_then(|w| w.as_f64()).unwrap_or(0.0);
    let place = format!("left: {}px; top: {}px; ", num((h.x + 18.0).min(width - 300.0)), num(8f64.max(h.y - 40.0)));
    match &h.what {
        Hovered::Landmark(l) => {
            let tint = biome_tint(Some(l.biome));
            let style = format!("{place}--sky: {}; --ground: {}; --tone: var(--f-humans);", tint.sky, tint.ground);
            view! {
                <div class="hover-card panel glass" style=style aria-hidden="true">
                    <div class="hc-art"></div>
                    <div class="hc-head">
                        <span class="hc-kind caps">{l.biome}</span>
                        <b class="hc-name">{l.name}</b>
                        <span class="hc-sector caps">{l.sector}" sector"</span>
                    </div>
                    <p class="hc-desc">{l.note}</p>
                    {(!l.pois.is_empty()).then(|| {
                        view! {
                            <div class="hc-pois-head caps">"Points of interest"</div>
                            {l.pois
                                .iter()
                                .map(|poi| {
                                    poi_row(poi.name.into(), poi.description.into(), poi_kind(poi.name), Faction::Humans)
                                })
                                .collect_view()}
                        }
                    })}
                    <div class="hc-foot caps">"No campaign data"</div>
                </div>
            }
            .into_any()
        }
        Hovered::Planet(p) => {
            let planet = (**p).clone();
            let look = {
                let planet = planet.clone();
                move || format!("{place}{}", effects.with(|e| card_look(&planet, e)))
            };
            view! {
                <div class="hover-card panel glass" style=look aria-hidden="true">
                    {planet_summary(Signal::stored(planet), active, effects)}
                    <div class="hc-foot caps">
                        <Icon icon=icons::MOUSE_POINTER_CLICK size=14.0 />
                        " Click for the planet panel"
                    </div>
                </div>
            }
            .into_any()
        }
    }
}

/// A clicked planet, docked beside the map (a sheet over it on phones) with its actions.
pub fn planet_panel(
    p: Signal<Planet>,
    active: Signal<bool>,
    effects: Signal<Vec<PlanetEffect>>,
    on_close: Callback<()>,
) -> impl IntoView {
    let index = Memo::new(move |_| p.with(|p| p.index));
    let pinned = Memo::new(move |_| settings().get(|s| s.pinned.contains(&index.get())));
    let stations = station_planets();
    let has_station = Memo::new(move |_| stations.with(|s| s.contains(&index.get())));
    let on_key = window_event_listener(leptos::ev::keydown, move |e| {
        if e.key() == "Escape" {
            on_close.run(());
        }
    });
    on_cleanup(move || on_key.remove());
    let has_cities = Memo::new(move |_| p.with(|p| !p.regions.is_empty()));
    let owner = Memo::new(move |_| p.with(|p| p.current_owner));
    view! {
        <aside
            class="planet-panel panel glass"
            style=move || p.with(|p| effects.with(|e| card_look(p, e)))
            aria-label=move || p.with(|p| format!("{} panel", p.name))
        >
            <button class="pp-close" on:click=move |_| on_close.run(()) aria-label="Close planet panel">
                <Icon icon=icons::X size=16.0 />
            </button>
            <div class="pp-scroll">
                {planet_summary(p, active, effects)}
                <Show when=move || has_cities.get()>
                    <section class="pp-cities" aria-label="Cities">
                        <h3 class="caps">"Cities"</h3>
                        <CityList planet=Signal::derive(move || Some(p.get())) />
                    </section>
                </Show>
                <dl class="pp-stats">
                    <div>
                        <dt class="caps">"Helldivers"</dt>
                        <dd class="num">{move || p.with(|p| fmt_int(p.statistics.player_count))}</dd>
                    </div>
                    <div>
                        <dt class="caps">"Held by"</dt>
                        <dd style=move || format!("color: {};", faction_color(Some(owner.get())))>
                            {move || faction_label(Some(owner.get()))}
                        </dd>
                    </div>
                </dl>
            </div>
            <div class="pp-actions">
                <button class="pp-action" on:click=move |_| settings().toggle_pin(index.get_untracked())>
                    {move || {
                        if pinned.get() {
                            view! { <Icon icon=icons::PIN_OFF size=16.0 /> }
                        } else {
                            view! { <Icon icon=icons::PIN size=16.0 /> }
                        }
                    }}
                    <span>{move || if pinned.get() { "Unpin" } else { "Pin" }}</span>
                </button>
                <button class="pp-action" on:click=move |_| ui().focus(index.get_untracked())>
                    <Icon icon=icons::CROSSHAIR size=16.0 />
                    <span>"Focus"</span>
                </button>
                <button class="pp-action" on:click=move |_| ui().open_detail(Some(index.get_untracked()))>
                    <Icon icon=icons::FILE_TEXT size=16.0 />
                    <span>"Details"</span>
                </button>
                <button
                    class="pp-action"
                    on:click=move |_| ui().open_module(Some(HellpadModule::Stations))
                    disabled=move || !has_station.get()
                    title=move || if has_station.get() { "Space station" } else { "No space station at this planet" }
                >
                    <DssIcon size=16.0 />
                    <span>"Station"</span>
                </button>
            </div>
        </aside>
    }
}
