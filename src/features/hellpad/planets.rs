//! The planets module: the faction control ring and every planet, searchable and grouped by sector.

use crate::api::fronts::{major_order_planets, station_planets};
use crate::api::queries::queries;
use crate::features::planet::tint::planet_tint;
use crate::js::num;
use crate::state::{OptionsAnchor, settings, ui};
use crate::ui::emblems::{DssIcon, FactionEmblem};
use crate::ui::hazard_icons::hazard_icon;
use crate::ui::holo_globe::HoloGlobe;
use crate::ui::icons::{self, Icon};
use crate::ui::neon_ring::{NeonRing, RingSegment};
use crate::ui::pointer_card::{CardRow, PointerAt, PointerCard, pointer_at};
use crate::ui::primitives::{Empty, Loading};
use leptos::html;
use leptos::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};
use warroom_core::api::{Faction, Planet};
use warroom_core::faction_planets::{
    CommunityPlanets, LIST_SIZE, ListedPlanet, busiest_planets, community_planets, super_earth_planets,
};
use warroom_core::factions::{faction_color, faction_label};
use warroom_core::format::{fmt_int, fmt_pct, js_round};
use warroom_core::planet_index::{
    PlanetFilter, SectorSort, control_shares, group_by_sector, matches_filter, matches_search,
};
use warroom_core::progress::{liberation_percent, tracked_percent};
use wasm_bindgen::JsCast;

const ORDER: [Faction; 4] = [Faction::Humans, Faction::Terminids, Faction::Automaton, Faction::Illuminate];

static COMMUNITY: LazyLock<CommunityPlanets> = LazyLock::new(community_planets);

#[component]
fn PlanetRow(
    planet: Memo<Option<Planet>>,
    active: Signal<bool>,
    mo: Signal<bool>,
    station: Signal<bool>,
    pinned: Signal<bool>,
) -> impl IntoView {
    move || {
        let p = planet.get()?;
        let active = active.get();
        let enemy =
            p.event.as_ref().map(|e| e.faction).or((p.current_owner != Faction::Humans).then_some(p.current_owner));
        let face = enemy.unwrap_or(p.current_owner);
        let tint = planet_tint(&p, &[]);
        let pct = if p.event.is_some() { tracked_percent(&p) } else { liberation_percent(p.health, p.max_health) };
        let status = if active {
            format!("{} campaign · {}", if p.event.is_some() { "Defense" } else { "Liberation" }, fmt_pct(pct, 1))
        } else {
            let owner = if p.current_owner == Faction::Humans {
                "Secured".to_string()
            } else {
                format!("{} control", faction_label(Some(p.current_owner)))
            };
            format!("{owner} · {}", if p.biome.name.is_empty() { "Unknown" } else { &p.biome.name })
        };
        let index = p.index;
        Some(view! {
            <li>
                <button
                    class=format!("pi-row {} {}", if active { "active" } else { "" }, if pinned.get() { "pinned" } else { "" })
                    style=format!("--k: {}; --sky: {}; --ground: {};", faction_color(Some(face)), tint.sky, tint.ground)
                    on:click=move |e| ui().open_options(Some(OptionsAnchor::at(index, &e)))
                >
                    <FactionEmblem faction=face.as_str() size=26.0 />
                    <span class="pi-row-text">
                        <b class="pi-row-name">{p.name.clone()}</b>
                        <span class="pi-row-status caps">
                            {active.then(|| view! { <span class="pi-dot"></span> })}
                            {status}
                        </span>
                    </span>
                    <span class="pi-row-icons">
                        {pinned.get().then(|| view! {
                            <span class="pi-icon pin" title="Pinned">
                                <Icon icon=icons::PIN size=13.0 />
                            </span>
                        })}
                        {mo.get().then(|| view! {
                            <span class="pi-icon mo" title="Major Order target">
                                <Icon icon=icons::TARGET size=13.0 />
                            </span>
                        })}
                        {station.get().then(|| view! {
                            <span class="pi-icon dss" title="Democracy Space Station">
                                <DssIcon size=13.0 />
                            </span>
                        })}
                        {p
                            .hazards
                            .iter()
                            .take(2)
                            .map(|h| view! {
                                <span class="pi-icon hz" title=h.name.clone()>
                                    <Icon icon=hazard_icon(&h.name) size=13.0 />
                                </span>
                            })
                            .collect_view()}
                        {(p.statistics.player_count > 0.0).then(|| view! {
                            <span class="pi-players num">
                                <Icon icon=icons::USERS size=12.0 />
                                {format!(" {}", fmt_int(p.statistics.player_count))}
                            </span>
                        })}
                    </span>
                    {active.then(|| view! { <span class="pi-row-progress" style=format!("width: {}%;", num(pct))></span> })}
                </button>
            </li>
        })
    }
}

#[derive(Clone)]
struct FactionHover {
    faction: Faction,
    at: PointerAt,
    list: Vec<ListedPlanet>,
}

/// Collapsible faction-control ring for the planet index; hovering an arc maps that faction's planets.
#[component]
fn GalacticControl(#[prop(into)] planets: Signal<Arc<Vec<Planet>>>, on_filter: Callback<Faction>) -> impl IntoView {
    let open = Memo::new(move |_| settings().get(|s| s.planets_control_open));
    let shares = Memo::new(move |_| planets.with(|p| control_shares(p)));
    let active = RwSignal::new(None::<Faction>);
    let map = RwSignal::new(None::<FactionHover>);
    let list_for = move |f: Faction| {
        planets.with_untracked(|p| {
            if f == Faction::Humans {
                super_earth_planets(p, &COMMUNITY, js_sys::Math::random)
            } else {
                busiest_planets(p, f, LIST_SIZE)
            }
        })
    };
    let focus =
        Memo::new(move |_| active.get().and_then(|a| shares.with(|s| s.iter().find(|s| s.faction == a).copied())));
    let pct = |p: f64| num(js_round(p));
    view! {
        <section class=move || format!("gc {}", if open.get() { "open" } else { "" })>
            <button
                class="gc-toggle"
                aria-expanded=move || open.get().to_string()
                on:click=move |_| settings().set(|s| s.planets_control_open = !s.planets_control_open)
            >
                <h2>"Galactic control"</h2>
                <Show when=move || !open.get()>
                    <span class="gc-peek num">
                        {move || {
                            shares
                                .get()
                                .into_iter()
                                .map(|s| view! {
                                    <span style=format!("color: {};", faction_color(Some(s.faction)))>
                                        <FactionEmblem faction=s.faction.as_str() size=13.0 />
                                        {format!("{}%", pct(s.pct))}
                                    </span>
                                })
                                .collect_view()
                        }}
                    </span>
                </Show>
                <Icon icon=icons::CHEVRON_DOWN size=18.0 class="gc-chevron" />
            </button>
            <Show when=move || open.get()>
                <div class="gc-body">
                    <NeonRing
                        label=Signal::derive(move || {
                            shares.with(|s| {
                                format!(
                                    "Planet control: {}",
                                    s.iter().map(|s| format!("{} {}%", faction_label(Some(s.faction)), pct(s.pct))).collect::<Vec<_>>().join(", ")
                                )
                            })
                        })
                        segments=Signal::derive(move || {
                            shares
                                .get()
                                .into_iter()
                                .map(|s| RingSegment {
                                    key: s.faction.as_str().into(),
                                    label: faction_label(Some(s.faction)).into(),
                                    value: s.planets as f64,
                                    color: faction_color(Some(s.faction)).into(),
                                })
                                .collect::<Vec<_>>()
                        })
                        active=Signal::derive(move || active.get().map(|a| a.as_str().to_string()))
                        on_active=Callback::new(move |key: Option<String>| {
                            let f = key.as_deref().and_then(Faction::parse);
                            active.set(f);
                            if f.is_none() {
                                map.set(None);
                            }
                        })
                        on_pointer_move=Callback::new(move |(key, e): (String, web_sys::MouseEvent)| {
                            let Some(f) = Faction::parse(&key) else { return };
                            let svg = e
                                .current_target()
                                .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                                .and_then(|el| el.closest("svg").ok().flatten());
                            let Some(svg) = svg else { return };
                            let at = pointer_at(&e, &svg);
                            let same = map.with_untracked(|m| m.as_ref().is_some_and(|m| m.faction == f));
                            if same {
                                map.update(|m| {
                                    if let Some(m) = m {
                                        m.at = at;
                                    }
                                });
                            } else {
                                map.set(Some(FactionHover { faction: f, at, list: list_for(f) }));
                            }
                        })
                        on_select=Callback::new(move |key: String| {
                            if let Some(f) = Faction::parse(&key) {
                                on_filter.run(f);
                            }
                        })
                        center=Arc::new(move || {
                            (move || match focus.get() {
                                Some(f) => view! {
                                    <div class="nr-readout" style=format!("color: {};", faction_color(Some(f.faction)))>
                                        <FactionEmblem faction=f.faction.as_str() size=30.0 />
                                        <span class="caps">{faction_label(Some(f.faction))}</span>
                                        <b class="num">{format!("{}%", pct(f.pct))}</b>
                                        <span class="sub num">{format!("{} planets", f.planets)}</span>
                                        <span class="pl num">
                                            <Icon icon=icons::USERS size=11.0 />
                                            {format!(" {}", fmt_int(f.players))}
                                        </span>
                                    </div>
                                }
                                .into_any(),
                                None => view! { <HoloGlobe /> }.into_any(),
                            })
                            .into_any()
                        })
                    />
                    <div class="gc-legend">
                        {move || {
                            shares
                                .get()
                                .into_iter()
                                .map(|s| {
                                    let f = s.faction;
                                    view! {
                                        <button
                                            class=move || format!("gc-chip {}", if active.get() == Some(f) { "on" } else { "" })
                                            style=format!("--k: {};", faction_color(Some(f)))
                                            on:pointerenter=move |_| active.set(Some(f))
                                            on:pointerleave=move |_| active.set(None)
                                            on:focus=move |_| active.set(Some(f))
                                            on:blur=move |_| active.set(None)
                                            on:click=move |_| on_filter.run(f)
                                        >
                                            <FactionEmblem faction=f.as_str() size=18.0 />
                                            <span class="gc-chip-name">{faction_label(Some(f))}</span>
                                            <b class="num">{format!("{}%", pct(s.pct))}</b>
                                            <span class="gc-chip-sub num">{format!("{} planets", s.planets)}</span>
                                        </button>
                                    }
                                })
                                .collect_view()
                        }}
                        <p class="gc-hint">"Hover or tap the ring to map a faction's planets; pick a faction to filter the list."</p>
                    </div>
                </div>
            </Show>
            {move || {
                if !open.get() {
                    return None;
                }
                let m = map.get()?;
                let f = focus.get()?;
                let rows: Vec<CardRow> = m
                    .list
                    .iter()
                    .map(|l| CardRow {
                        key: l.name.clone(),
                        name: l.name.clone(),
                        value: l.planet.and_then(|i| planets.with(|p| p.iter().find(|p| p.index == i).map(|p| p.statistics.player_count))),
                        note: l.lore.map(String::from),
                        color: None,
                    })
                    .collect();
                Some(view! {
                    <PointerCard
                        at=m.at
                        tone=faction_color(Some(m.faction))
                        icon=view! { <FactionEmblem faction=m.faction.as_str() size=18.0 /> }.into_any()
                        title=faction_label(Some(m.faction))
                        badge=format!("{}% · {}", pct(f.pct), f.planets).into_any()
                        rows=rows
                        footer=(m.faction != Faction::Humans && f.planets > LIST_SIZE)
                            .then(|| format!("+{} more · busiest first", f.planets - LIST_SIZE))
                    />
                })
            }}
        </section>
    }
}

fn empty_text(filter: PlanetFilter, query: &str) -> String {
    if !query.is_empty() {
        return format!("No planets match “{query}”.");
    }
    match filter {
        PlanetFilter::Pinned => "No pinned planets yet. Pick a planet and choose Pin to top.".into(),
        PlanetFilter::Active => "No front is open right now.".into(),
        PlanetFilter::Owner(f) => format!("No planet is under {} control.", faction_label(Some(f))),
        PlanetFilter::All => "No planets charted yet.".into(),
    }
}

#[component]
pub fn PlanetsModule() -> impl IntoView {
    let q = queries();
    let mo = major_order_planets();
    let stations = station_planets();
    let pinned_list = Memo::new(move |_| settings().get(|s| s.pinned.clone()));
    let pinned = Memo::new(move |_| pinned_list.with(|l| l.iter().copied().collect::<HashSet<_>>()));
    let query = RwSignal::new(String::new());
    let filter = RwSignal::new(PlanetFilter::All);
    let sort = RwSignal::new(SectorSort::Players);
    let list_ref = NodeRef::<html::Div>::new();
    let active = Memo::new(move |_| {
        q.campaigns.with(|c| c.iter().flat_map(|c| c.iter()).map(|c| c.planet.index).collect::<HashSet<_>>())
    });
    let planets = Signal::derive(move || q.fresh_planets.get().unwrap_or_default());
    let by_index = Memo::new(move |_| {
        planets.with(|p| Arc::new(p.iter().map(|p| (p.index, p.clone())).collect::<HashMap<_, _>>()))
    });

    let groups = Memo::new(move |_| {
        let (qs, f, s) = (query.get(), filter.get(), sort.get());
        active.with(|a| {
            pinned.with(|pn| {
                planets.with(|p| {
                    let shown: Vec<Planet> =
                        p.iter().filter(|p| matches_search(p, &qs) && matches_filter(p, f, a, pn)).cloned().collect();
                    group_by_sector(&shown, a, s)
                        .into_iter()
                        .map(|g| {
                            (
                                g.name,
                                g.owner,
                                g.hottest,
                                g.active,
                                g.players,
                                g.planets.iter().map(|p| p.index).collect::<Vec<_>>(),
                            )
                        })
                        .collect::<Vec<_>>()
                })
            })
        })
    });
    // Pinned planets also get a shelf above the sectors, in the order they were pinned.
    let shelf = Memo::new(move |_| {
        if filter.get() == PlanetFilter::Pinned {
            return Vec::new();
        }
        let (qs, f) = (query.get(), filter.get());
        by_index.with(|m| {
            active.with(|a| {
                pinned.with(|pn| {
                    pinned_list.with(|l| {
                        l.iter()
                            .filter(|i| m.get(i).is_some_and(|p| matches_search(p, &qs) && matches_filter(p, f, a, pn)))
                            .copied()
                            .collect::<Vec<_>>()
                    })
                })
            })
        })
    });
    let count = move |f: PlanetFilter| {
        let qs = query.get();
        active.with(|a| {
            pinned.with(|pn| {
                planets.with(|p| p.iter().filter(|p| matches_search(p, &qs) && matches_filter(p, f, a, pn)).count())
            })
        })
    };
    let row = move |index: i64| {
        let planet = Memo::new(move |_| by_index.with(|m| m.get(&index).cloned()));
        view! {
            <PlanetRow
                planet=planet
                active=Signal::derive(move || active.with(|a| a.contains(&index)))
                mo=Signal::derive(move || mo.with(|m| m.contains(&index)))
                station=Signal::derive(move || stations.with(|s| s.contains(&index)))
                pinned=Signal::derive(move || pinned.with(|p| p.contains(&index)))
            />
        }
    };
    let chip = move |f: PlanetFilter, label: AnyView| {
        view! {
            <button class=move || format!("filter-chip {}", if filter.get() == f { "on" } else { "" }) on:click=move |_| filter.set(f)>
                {label}
                " "
                <span class="count">{move || count(f)}</span>
            </button>
        }
    };
    let ready = Memo::new(move |_| q.fresh_planets.with(Option::is_some));
    let empty = Memo::new(move |_| groups.with(Vec::is_empty));

    view! {
        <Show when=move || ready.get() fallback=|| view! { <Loading what="the star chart" rows=4 /> }>
            <div class="pi">
                <div class="page-head">
                    <h1 class="page-title">
                        "Planets "
                        <span class="pi-count num">{move || planets.with(|p| p.len())}</span>
                    </h1>
                </div>
                <GalacticControl
                    planets=planets
                    on_filter=Callback::new(move |f: Faction| {
                        filter.set(PlanetFilter::Owner(f));
                        if let Some(el) = list_ref.get_untracked() {
                            let opts = web_sys::ScrollIntoViewOptions::new();
                            opts.set_behavior(web_sys::ScrollBehavior::Smooth);
                            opts.set_block(web_sys::ScrollLogicalPosition::Start);
                            el.scroll_into_view_with_scroll_into_view_options(&opts);
                        }
                    })
                />
                <div class="pi-controls" node_ref=list_ref>
                    <label class="pi-search">
                        <Icon icon=icons::SEARCH size=16.0 />
                        <input
                            prop:value=move || query.get()
                            on:input=move |e| query.set(event_target_value(&e))
                            placeholder="Search by planet, sector, biome or hazard"
                            aria-label="Search planets"
                        />
                    </label>
                    <label class="pi-sort">
                        <Icon icon=icons::ARROW_DOWN_WIDE_NARROW size=16.0 />
                        <select
                            prop:value=move || match sort.get() {
                                SectorSort::Players => "players",
                                SectorSort::Active => "active",
                                SectorSort::Az => "az",
                            }
                            on:change=move |e| {
                                sort.set(match event_target_value(&e).as_str() {
                                    "active" => SectorSort::Active,
                                    "az" => SectorSort::Az,
                                    _ => SectorSort::Players,
                                })
                            }
                            aria-label="Sort sectors"
                        >
                            <option value="players">"Most Helldivers"</option>
                            <option value="active">"Most active"</option>
                            <option value="az">"A–Z"</option>
                        </select>
                    </label>
                </div>
                <div class="filter-row" role="toolbar" aria-label="Filter planets">
                    {chip(PlanetFilter::All, "All".into_any())}
                    {chip(PlanetFilter::Active, view! { <Icon icon=icons::SWORDS size=14.0 /> " Active fronts" }.into_any())}
                    {chip(PlanetFilter::Pinned, view! { <Icon icon=icons::PIN size=14.0 /> " Pinned" }.into_any())}
                    {ORDER
                        .map(|f| chip(PlanetFilter::Owner(f), view! { <FactionEmblem faction=f.as_str() size=14.0 /> {format!(" {}", faction_label(Some(f)))} }.into_any()))
                        .collect_view()}
                </div>
                <Show when=move || shelf.with(|s| !s.is_empty())>
                    <section class="pi-sector pi-pinned" aria-label="Pinned planets">
                        <header class="pi-sector-head" style="--k: var(--cta);">
                            <span class="pi-sector-glyph">
                                <Icon icon=icons::PIN size=15.0 />
                            </span>
                            <h2>"Pinned"</h2>
                            <span class="pi-sector-rule"></span>
                            <span class="pi-sector-meta caps num">
                                <span>{move || shelf.with(|s| format!("{} planet{}", s.len(), if s.len() == 1 { "" } else { "s" }))}</span>
                            </span>
                        </header>
                        <ul class="pi-grid">
                            <For each=move || shelf.get() key=|i| *i let:i>
                                {row(i)}
                            </For>
                        </ul>
                    </section>
                </Show>
                {move || {
                    if empty.get() {
                        let text = query.with(|q| empty_text(filter.get(), q));
                        return view! { <Empty>{text}</Empty> }.into_any();
                    }
                    view! {
                        <For each=move || groups.get() key=|g| g.0.clone() let:g>
                            {
                                let name = g.0.clone();
                                let meta = Memo::new(move |_| groups.with(|gs| gs.iter().find(|x| x.0 == name).cloned()));
                                view! {
                                    <section class="pi-sector" aria-label=format!("{} sector", g.0)>
                                        {move || meta.get().map(|(name, owner, hottest, active_n, players, list)| view! {
                                            <header class="pi-sector-head" style=format!("--k: {};", faction_color(Some(owner)))>
                                                <span class="pi-sector-glyph">
                                                    <FactionEmblem faction=owner.as_str() size=16.0 />
                                                </span>
                                                <h2>{format!("{name} sector")}</h2>
                                                {hottest.then(|| view! {
                                                    <span class="pi-hot caps">
                                                        <Icon icon=icons::FLAME size=12.0 />
                                                        " Hottest front"
                                                    </span>
                                                })}
                                                <span class="pi-sector-rule"></span>
                                                <span class="pi-sector-meta caps num">
                                                    {(active_n > 0).then(|| view! { <span class="on">{format!("{active_n} active")}</span> })}
                                                    <span>{format!("{} planet{}", list.len(), if list.len() == 1 { "" } else { "s" })}</span>
                                                    {(players > 0.0).then(|| view! {
                                                        <span class="pl">
                                                            <Icon icon=icons::USERS size=12.0 />
                                                            {format!(" {}", fmt_int(players))}
                                                        </span>
                                                    })}
                                                </span>
                                            </header>
                                        })}
                                        <ul class="pi-grid">
                                            <For each=move || meta.get().map(|m| m.5).unwrap_or_default() key=|i| *i let:i>
                                                {row(i)}
                                            </For>
                                        </ul>
                                    </section>
                                }
                            }
                        </For>
                    }
                    .into_any()
                }}
            </div>
        </Show>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text_names_the_filter_without_a_search() {
        assert_eq!(empty_text(PlanetFilter::Owner(Faction::Illuminate), ""), "No planet is under Illuminate control.");
        assert_eq!(empty_text(PlanetFilter::Active, ""), "No front is open right now.");
        assert_eq!(empty_text(PlanetFilter::Active, "mars"), "No planets match “mars”.");
        assert!(empty_text(PlanetFilter::Pinned, "").starts_with("No pinned planets yet."));
    }
}
