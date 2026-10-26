use crate::api::queries::queries;
use crate::features::map::layers::{GloomLayer, SectorLayer};
use crate::features::overview::activity::chart_color;
use crate::js::{self, num};
use crate::state::{MapOverlay, View, settings, ui};
use crate::ui::emblems::EmblemArt;
use crate::ui::icons::{self, Icon};
use crate::ui::primitives::Panel;
use leptos::html;
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;
use warroom_core::api::Faction;
use warroom_core::faction_marks::faction_marks;
use warroom_core::format::to_fixed;
use warroom_core::galaxy::{project, supply_lines};
use warroom_core::planets::{by_index, ownership_key};
use warroom_core::regions::build_regions;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

const SPIN_DEG_PER_S: f64 = 4.0;
const DRAG_DEG_PER_PX: f64 = 0.35;
const KEY_STEP_DEG: f64 = 10.0;
const ORDER: [Faction; 4] = [Faction::Humans, Faction::Terminids, Faction::Illuminate, Faction::Automaton];
/// Hatch spacing in disc units; the disc is drawn at about a third of its size.
const HATCH: f64 = 18.0;

/// The galaxy disc seen at an angle, drawn like the map's chosen overlay; it turns slowly and can be dragged round.
#[component]
pub fn TheaterOfWar() -> impl IntoView {
    let q = queries();
    let overlay = Memo::new(move |_| settings().get(|s| s.map_overlay));
    let spin = Memo::new(move |_| settings().get(|s| s.tw_spin));
    let sector = RwSignal::new(None::<String>);
    let owners = Memo::new(move |_| q.fresh_planets.with(|p| p.as_ref().map(|p| ownership_key(p)).unwrap_or_default()));
    let planets = Signal::derive(move || q.fresh_planets.get().unwrap_or_default());
    let by_owner = Memo::new(move |_| {
        owners.with(|_| ());
        if overlay.get() != MapOverlay::Regions {
            return Vec::new();
        }
        let mut groups: Vec<(Faction, String)> =
            planets.with_untracked(|p| build_regions(p)).into_iter().map(|r| (r.owner, r.path)).collect();
        groups.sort_by_key(|(o, _)| ORDER.iter().position(|x| x == o).unwrap_or(ORDER.len()));
        groups
    });
    let marks = Memo::new(move |_| {
        owners.with(|_| ());
        let o = overlay.get().core();
        q.geometry.with(|g| planets.with_untracked(|p| faction_marks(p, o, g.as_deref())))
    });
    let lanes = Memo::new(move |_| {
        q.geometry.with(|_| ());
        if overlay.get() != MapOverlay::Galaxy {
            return String::new();
        }
        planets.with_untracked(|p| {
            let lookup = by_index(p);
            supply_lines(p)
                .into_iter()
                .filter_map(|(a, b)| {
                    let (pa, pb) = (lookup.get(&a)?, lookup.get(&b)?);
                    let ([x1, y1], [x2, y2]) = (project(pa.position), project(pb.position));
                    Some(format!("M{},{}L{},{}", to_fixed(x1, 1), to_fixed(y1, 1), to_fixed(x2, 1), to_fixed(y2, 1)))
                })
                .collect::<String>()
        })
    });

    let stage = NodeRef::<html::Div>::new();
    let disc = NodeRef::<leptos::svg::G>::new();
    let angle = StoredValue::new(0.0f64);
    let drag = StoredValue::new(None::<(f64, f64)>);
    let turn = move |deg: f64| {
        let a = (deg + 360.0) % 360.0;
        angle.set_value(a);
        if let Some(d) = disc.get_untracked() {
            let _ = d.set_attribute("transform", &format!("rotate({})", to_fixed(a, 2)));
            let _ = d.style().set_property("--unrot", &format!("{}deg", to_fixed(-a, 2)));
        }
    };

    let ready = Memo::new(move |_| q.fresh_planets.with(Option::is_some) && q.geometry.with(Option::is_some));
    let stop: SendWrapper<Rc<Cell<bool>>> = SendWrapper::new(Rc::new(Cell::new(false)));
    let halt = stop.clone();
    Effect::new(move |started: Option<bool>| {
        if started == Some(true) {
            return true;
        }
        let Some(el) = stage.get() else { return false };
        let reduced = js::prefers_reduced_motion();
        let on_screen = Rc::new(Cell::new(true));
        let seen = on_screen.clone();
        let cb = Closure::<dyn Fn(js_sys::Array)>::new(move |entries: js_sys::Array| {
            if let Some(e) = entries.get(0).dyn_ref::<web_sys::IntersectionObserverEntry>() {
                seen.set(e.is_intersecting());
            }
        });
        let io = web_sys::IntersectionObserver::new(cb.as_ref().unchecked_ref()).ok();
        if let Some(io) = &io {
            io.observe(&el);
        }
        cb.forget();
        let last = Rc::new(Cell::new(js::performance_now()));
        let stop = (*halt).clone();
        let io = io.map(SendWrapper::new);
        let frames = js::animation_loop(move |t| {
            // Frames stop while the window is hidden; don't jump by the whole gap when they resume.
            let dt = ((t - last.get()) / 1000.0).min(0.1);
            last.set(t);
            let spinning = settings().untracked(|s| s.tw_spin);
            if !reduced && spinning && on_screen.get() && drag.with_value(Option::is_none) && !js::document().hidden() {
                turn(angle.get_value() + SPIN_DEG_PER_S * dt);
            }
            if stop.get() {
                if let Some(io) = &io {
                    io.disconnect();
                }
                return false;
            }
            true
        });
        let _ = frames;
        true
    });
    on_cleanup(move || stop.set(true));

    let right = move || {
        view! {
            <button
                class="hud-btn icon tw-spin"
                on:click=move |_| settings().set(|s| s.tw_spin = !s.tw_spin)
                aria-pressed=move || (!spin.get()).to_string()
                aria-label=move || if spin.get() { "Stop turning" } else { "Turn the disc" }
                title=move || if spin.get() { "Stop turning" } else { "Turn the disc" }
            >
                {move || view! { <Icon icon=if spin.get() { icons::PAUSE } else { icons::PLAY } size=14.0 /> }}
            </button>
            <button class="hud-btn tw-open" on:click=move |_| ui().set_view(View::Map)>
                "Open map "
                <Icon icon=icons::ARROW_UP_RIGHT size=14.0 />
            </button>
        }
        .into_any()
    };

    let geo = Signal::derive(move || q.geometry.get().unwrap_or_default());
    let gloom =
        Signal::derive(move || q.effects.with(|e| Arc::new(e.as_ref().map(|e| e.gloom.clone()).unwrap_or_default())));
    view! {
        <Show when=move || ready.get()>
            <Panel class="tw" title="Theater of war" icon=icons::MAP body_class="panel-body tw-body" right=right()>
                <div
                    node_ref=stage
                    class="tw-stage"
                    tabindex="0"
                    role="img"
                    aria-label=move || format!("Galaxy {} view. Drag or use the arrow keys to rotate.", MapOverlay::as_str(overlay.get()))
                    on:pointerdown=move |e: web_sys::PointerEvent| {
                        if let Some(t) = e.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) {
                            let _ = t.set_pointer_capture(e.pointer_id());
                        }
                        drag.set_value(Some((e.client_x() as f64, angle.get_value())));
                        if let Some(s) = stage.get_untracked() {
                            let _ = s.class_list().add_1("dragging");
                        }
                    }
                    on:pointermove=move |e: web_sys::PointerEvent| {
                        if let Some((x, from)) = drag.get_value() {
                            turn(from + (e.client_x() as f64 - x) * DRAG_DEG_PER_PX);
                        }
                    }
                    on:pointerup=move |_| {
                        drag.set_value(None);
                        if let Some(s) = stage.get_untracked() {
                            let _ = s.class_list().remove_1("dragging");
                        }
                    }
                    on:pointercancel=move |_| {
                        drag.set_value(None);
                        if let Some(s) = stage.get_untracked() {
                            let _ = s.class_list().remove_1("dragging");
                        }
                    }
                    on:pointerleave=move |_| sector.set(None)
                    on:keydown=move |e: web_sys::KeyboardEvent| {
                        match e.key().as_str() {
                            "ArrowLeft" => turn(angle.get_value() - KEY_STEP_DEG),
                            "ArrowRight" => turn(angle.get_value() + KEY_STEP_DEG),
                            _ => {}
                        }
                    }
                >
                    <svg viewBox="-560 -560 1120 1120" aria-hidden="true">
                        <circle r="545" class="tw-rim" />
                        <g node_ref=disc>
                            {move || (overlay.get() != MapOverlay::Sectors).then(|| view! {
                                <g class="tw-owner se" style=format!("--c: {};", chart_color(Faction::Humans))>
                                    <circle r=move || num(geo.with(|g| g.rim)) />
                                </g>
                            })}
                            {move || (overlay.get() == MapOverlay::Sectors).then(|| view! {
                                <SectorLayer geo=geo planets=planets id="tw" hatch=HATCH />
                            })}
                            {move || lanes.with(|l| (!l.is_empty()).then(|| view! { <path d=l.clone() class="tw-lanes" /> }))}
                            {move || {
                                by_owner
                                    .get()
                                    .into_iter()
                                    .map(|(owner, path)| view! {
                                        <g
                                            class=format!("tw-owner {} ", if owner == Faction::Humans { "se" } else { "enemy" })
                                            style=format!("--c: {};", chart_color(owner))
                                        >
                                            <path d=path />
                                        </g>
                                    })
                                    .collect_view()
                            }}
                            {move || q.effects.with(Option::is_some).then(|| view! { <GloomLayer planets=planets gloom=gloom /> })}
                            <g class="tw-hits">
                                {move || {
                                    geo.with(|g| {
                                        g.fills
                                            .iter()
                                            .map(|(name, d)| {
                                                let name = name.clone();
                                                view! { <path d=d.clone() on:pointerenter=move |_| sector.set(Some(name.clone())) /> }
                                            })
                                            .collect_view()
                                    })
                                }}
                            </g>
                            {move || {
                                marks
                                    .get()
                                    .into_iter()
                                    .map(|m| view! {
                                        <g transform=format!("translate({},{})", to_fixed(m.x, 1), to_fixed(m.y, 1))>
                                            <g class="tw-mark" style=format!("--unsquash: {};", num(1.0 / (56.0f64).to_radians().cos()))>
                                                <EmblemArt art=m.faction.as_str() size=84.0 />
                                            </g>
                                        </g>
                                    })
                                    .collect_view()
                            }}
                            <line x1="0" y1="-545" x2="0" y2="-470" class="tw-tick" />
                        </g>
                        <circle r="70" class="tw-core-glow" />
                        <g class="tw-core-mark">
                            <g transform="scale(1,1.778)">
                                <EmblemArt art="Humans" size=86.0 />
                            </g>
                        </g>
                    </svg>
                    <span class="tw-hint caps">
                        {move || sector.get().map_or("Drag to rotate".to_string(), |s| format!("{s} sector"))}
                    </span>
                </div>
            </Panel>
        </Show>
    }
}
