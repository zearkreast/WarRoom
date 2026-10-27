//! The Galaxy map page: title, overlay switch, dispatch drawer and the map.

use crate::api::queries::queries;
use crate::features::major_order::panel::MajorOrderList;
use crate::features::map::galaxy::GalaxyMap;
use crate::features::map::options::overlay_options;
use crate::js;
use crate::state::{View, settings, ui};
use crate::ui::emblems::FactionEmblem;
use crate::ui::hint::Hint;
use crate::ui::icons::{self, Icon};
use crate::ui::primitives::{KeyButton, Keycap};
use leptos::prelude::*;
use warroom_core::api::Faction;
use warroom_core::dispatch::{dispatch_body, latest_dispatches};
use warroom_core::factions::{faction_color, faction_label};
use warroom_core::format::{dispatch_title, fmt_ago};
use warroom_core::time::parse_ms;
use wasm_bindgen::JsCast;

#[derive(Clone, Copy, PartialEq)]
enum Drawer {
    Dispatches,
    Orders,
}

#[component]
fn TitleStack(#[prop(into)] hovered: Signal<Option<String>>) -> impl IntoView {
    let fresh = queries().fresh_planets;
    let sector = Memo::new(move |_| {
        let name = hovered.get()?;
        fresh.with(|planets| {
            let planets = planets.as_ref()?;
            let mut counts: Vec<(Faction, usize)> = Vec::new();
            for p in planets.iter().filter(|p| p.sector == name) {
                match counts.iter_mut().find(|(f, _)| *f == p.current_owner) {
                    Some(c) => c.1 += 1,
                    None => counts.push((p.current_owner, 1)),
                }
            }
            counts.sort_by_key(|c| std::cmp::Reverse(c.1));
            Some((name, counts.first().map_or(Faction::Humans, |c| c.0)))
        })
    });
    view! {
        <div class="map-title">
            {move || match sector.get() {
                Some((_, owner)) => view! { <FactionEmblem faction=owner.as_str() size=34.0 /> }.into_any(),
                None => view! { <FactionEmblem faction="Humans" size=34.0 color="var(--text-hi)" /> }.into_any(),
            }}
            <div>
                <h1 class=move || if sector.with(Option::is_some) { "sector" } else { "" }>
                    {move || sector.get().map_or_else(|| "Galactic war".to_string(), |(name, _)| format!("{name} sector"))}
                </h1>
                <div
                    class="map-subtitle caps"
                    style=move || sector.get().map(|(_, owner)| format!("color: {};", faction_color(Some(owner))))
                >
                    {move || {
                        sector
                            .get()
                            .map_or_else(|| "Galaxy overview".to_string(), |(_, owner)| format!("{} control", faction_label(Some(owner))))
                    }}
                </div>
            </div>
        </div>
    }
}

fn controls() -> Vec<(AnyView, &'static str)> {
    let icon = |i| view! { <Icon icon=i size=15.0 /> }.into_any();
    let keys = |ks: &[&'static str]| ks.iter().map(|k| view! { <Keycap k=*k /> }).collect_view().into_any();
    vec![
        (icon(icons::MOVE), "Drag to pan"),
        (icon(icons::MOUSE), "Scroll to zoom"),
        (icon(icons::MOUSE_POINTER_CLICK), "Planet options"),
        (keys(&["+", "−"]), "Zoom"),
        (keys(&["0"]), "Fit galaxy"),
        (icon(icons::MOUSE_POINTER_2), "Right-drag: turn & tilt"),
        (keys(&["Q", "E"]), "Rotate"),
        (keys(&["R", "F"]), "Tilt"),
        (keys(&["Z"]), "Dispatches"),
        (keys(&["C"]), "Orders"),
        (keys(&["H"]), "Hide controls"),
    ]
}

/// The three map overlays side by side, one click each.
#[component]
fn OverlaySwitch() -> impl IntoView {
    let s = settings();
    let overlay = Memo::new(move |_| s.get(|s| s.map_overlay));
    view! {
        <div class="map-overlays" role="radiogroup" aria-label="Map overlay">
            {overlay_options()
                .into_iter()
                .map(|(value, label)| {
                    view! {
                        <button
                            role="radio"
                            aria-checked=move || (overlay.get() == value).to_string()
                            class=move || if overlay.get() == value { "on" } else { "" }
                            on:click=move |_| s.set(|s| s.map_overlay = value)
                        >
                            {label}
                        </button>
                    }
                })
                .collect_view()}
        </div>
    }
}

fn controls_panel() -> impl IntoView {
    view! {
        <dl class="map-keys panel glass" aria-label="Map controls">
            {controls()
                .into_iter()
                .map(|(glyph, label)| {
                    view! {
                        <div>
                            <dt>{glyph}</dt>
                            <dd>{label}</dd>
                        </div>
                    }
                })
                .collect_view()}
        </dl>
    }
}

#[component]
fn DispatchDrawer(now: ReadSignal<f64>) -> impl IntoView {
    let q = queries();
    let count = Memo::new(move |_| settings().get(|s| s.map_dispatches));
    let recent = Memo::new(move |_| {
        let n = count.get();
        q.dispatches.with(|d| latest_dispatches(d.as_deref().map_or(&[], |d| d.as_slice()), n.as_str(), now.get()))
    });
    view! {
        <div class="map-drawer panel glass">
            <div class="md-head caps">
                "Dispatches "
                <span>
                    {move || {
                        let n = count.get().as_str();
                        if n == "72h" { "last 72 hours".to_string() } else { format!("latest {n}") }
                    }}
                </span>
            </div>
            {move || {
                let list = recent.get();
                if list.is_empty() {
                    view! { <p class="md-empty caps">"No dispatches in the last 72 hours."</p> }.into_any()
                } else {
                    let now = now.get();
                    view! {
                        <ul class="md-list">
                            {list
                                .into_iter()
                                .map(|d| {
                                    view! {
                                        <li>
                                            <b>{dispatch_title(&d.message)}</b>
                                            <p>{dispatch_body(&d.message).into_iter().map(|s| s.text).collect::<String>()}</p>
                                            <span class="num">{fmt_ago(parse_ms(&d.published), now)}</span>
                                        </li>
                                    }
                                })
                                .collect_view()}
                        </ul>
                    }
                    .into_any()
                }
            }}
            <button class="md-more caps" on:click=move |_| ui().set_view(View::News)>
                "See all dispatches "
                <Icon icon=icons::ARROW_UP_RIGHT size=14.0 />
            </button>
        </div>
    }
}

#[component]
pub fn MapPage() -> impl IntoView {
    let drawer = RwSignal::new(None::<Drawer>);
    let controls_open = RwSignal::new(false);
    let hovered = RwSignal::new(None::<String>);
    let now = js::use_now(60_000.0);
    let q = queries();
    let count = Memo::new(move |_| settings().get(|s| s.map_dispatches));
    let dispatches = Memo::new(move |_| {
        let n = count.get();
        q.dispatches
            .with(|d| latest_dispatches(d.as_deref().map_or(&[], |d| d.as_slice()), n.as_str(), now.get()).len())
    });
    let toggle = move |d: Drawer| drawer.update(|o| *o = if *o == Some(d) { None } else { Some(d) });
    let on_key = window_event_listener(leptos::ev::keydown, move |e| {
        let in_field = e
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
            .is_some_and(|el| el.closest("input, textarea").ok().flatten().is_some());
        if e.ctrl_key() || e.meta_key() || in_field {
            return;
        }
        match e.key().to_lowercase().as_str() {
            "h" => controls_open.update(|c| *c = !*c),
            "z" => toggle(Drawer::Dispatches),
            "c" => toggle(Drawer::Orders),
            _ => {}
        }
    });
    on_cleanup(move || on_key.remove());
    let has_badge = Memo::new(move |_| dispatches.get() > 0);
    view! {
        <div class="map-page">
            <GalaxyMap on_hover_sector=Callback::new(move |s| hovered.set(s)) />
            <TitleStack hovered=hovered />
            <div class="map-touch-hint">
                <Hint id="map-touch">
                    "One finger pans. Two fingers pinch to zoom, twist to turn, or slide up and down together to tilt."
                </Hint>
            </div>
            <div class="map-corner">
                <OverlaySwitch />
                <div class="map-controls-hint">
                    <KeyButton
                        k="H"
                        active=Signal::derive(move || controls_open.get())
                        on_click=Callback::new(move |_| controls_open.update(|c| *c = !*c))
                    >
                        "Controls"
                    </KeyButton>
                    <Show when=move || controls_open.get()>{controls_panel()}</Show>
                </div>
            </div>
            <div class="map-toggles">
                {move || match drawer.get() {
                    Some(Drawer::Dispatches) => view! { <DispatchDrawer now=now /> }.into_any(),
                    Some(Drawer::Orders) => {
                        view! {
                            <div class="map-drawer orders">
                                <MajorOrderList compact=true />
                            </div>
                        }
                            .into_any()
                    }
                    None => ().into_any(),
                }}
                <div class="map-toggle-row">
                    {move || {
                        let active = Signal::derive(move || drawer.get() == Some(Drawer::Dispatches));
                        let on_click = Callback::new(move |_| toggle(Drawer::Dispatches));
                        if has_badge.get() {
                            view! {
                                <KeyButton k="Z" active=active badge=(move || dispatches.get()).into_any() on_click=on_click>
                                    "Dispatches"
                                </KeyButton>
                            }
                                .into_any()
                        } else {
                            view! {
                                <KeyButton k="Z" active=active on_click=on_click>
                                    "Dispatches"
                                </KeyButton>
                            }
                                .into_any()
                        }
                    }}
                    <KeyButton
                        k="C"
                        active=Signal::derive(move || drawer.get() == Some(Drawer::Orders))
                        on_click=Callback::new(move |_| toggle(Drawer::Orders))
                    >
                        "Orders"
                    </KeyButton>
                </div>
            </div>
        </div>
    }
}
