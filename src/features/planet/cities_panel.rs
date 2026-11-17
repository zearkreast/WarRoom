//! A planet's cities on a page of their own, over the planet's detail page; each opens its city page.

use crate::api::queries;
use crate::features::planet::cities::{CityList, enemy_of_cities};
use crate::features::planet::tint::planet_tint;
use crate::state::{HellpadModule, ui};
use crate::ui::icons::{self, Icon};
use crate::ui::primitives::{Empty, Loading, Overlay, Panel};
use leptos::prelude::*;
use warroom_core::api::{Faction, Planet};
use warroom_core::factions::faction_color;

fn hero_style(p: &Planet) -> String {
    let t = planet_tint(p, &[]);
    format!(
        "--tone: {}; --sky: {}; --ground: {};{}",
        faction_color(Some(enemy_of_cities(p).unwrap_or(Faction::Humans))),
        t.sky,
        t.ground,
        t.art.map(|a| format!(" --art: {a};")).unwrap_or_default()
    )
}

fn count_label(n: usize) -> String {
    if n == 1 { "1 city".into() } else { format!("{n} cities") }
}

#[component]
fn Body(index: i64) -> impl IntoView {
    let planet = Memo::new(move |_| queries::planet(Some(index)));
    let ready = Memo::new(move |_| planet.with(Option::is_some));
    let count = Memo::new(move |_| planet.with(|p| p.as_ref().map_or(0, |p| p.regions.len())));
    let open_planet = move |_| ui().open_detail(Some(index));
    view! {
        <Show when=move || ready.get() fallback=|| view! { <Loading what="cities" /> }>
            {move || {
                let p = Signal::derive(move || planet.get().unwrap_or_default());
                let has_cities = move || count.get() != 0;
                let no_cities = move || format!("{} has no cities.", p.with(|p| p.name.clone()));
                view! {
                    <div class="pd">
                        <section class="pd-hero brackets" style=move || p.with(hero_style)>
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
                                    <h1 class="pd-name">{move || p.with(|p| format!("{} cities", p.name))}</h1>
                                </div>
                                <div class="pd-chips">
                                    <span class="chip num">
                                        <Icon icon=icons::BUILDING_2 size=13.0 />
                                        {move || format!(" {}", count_label(count.get()))}
                                    </span>
                                    <button class="chip" on:click=open_planet>
                                        <Icon icon=icons::COMPASS size=13.0 />
                                        {move || p.with(|p| format!(" {} · {} sector", p.name, p.sector))}
                                    </button>
                                </div>
                            </div>
                        </section>
                        <Show when=has_cities fallback=move || view! { <Empty>{no_cities}</Empty> }>
                            <Panel title="Cities" icon=icons::BUILDING_2 right=view! { <span class="num">{move || count.get()}</span> }.into_any()>
                                <CityList planet=Signal::derive(move || Some(p.get())) full=true />
                            </Panel>
                        </Show>
                    </div>
                }
            }}
        </Show>
    }
}

/// The cities panel; it has its own place in Back's history and its own address (`?cities=<planet>`).
#[component]
pub fn CitiesPanel() -> impl IntoView {
    let cities = ui().cities;
    move || {
        let index = cities.get()?;
        Some(view! {
            <Overlay on_close=Callback::new(|_| ui().open_cities(None)) label="Planet cities">
                <Body index=index />
            </Overlay>
        })
    }
}
