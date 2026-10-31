//! The galaxy map: planets, supply lines, overlays and markers over a starfield, with the hover card and docked planet panel.

mod controls;
mod rail;
mod stars;

use crate::api::fronts::major_order_planets;
use crate::api::queries::queries;
use crate::features::map::card::{Hover, Hovered, hover_card, planet_panel};
use crate::features::map::galaxy::controls::{
    Ctl, DRAG_ROT, DRAG_TILT, ROT_STEP, SCALE_EXTENT, TILT_PRESET, TILT_STEP, from_layer, scale_hatches, to_layer,
    touch_gestures,
};
use crate::features::map::galaxy::rail::{MapSettings, Side, rail_button};
use crate::features::map::galaxy::stars::{background_stars, star_count};
use crate::features::map::layers::{GloomLayer, SectorLayer};
use crate::features::map::legend::MapLegend;
use crate::features::map::marks::{Body, POI_MAX, halo_defs, planet_body, sphere_color};
use crate::features::map::pawn_art::{DssInfo, dss_info};
use crate::features::map::poi::{PoiKind, poi_kind};
use crate::features::map::zoom::{Transform, Zoom};
use crate::features::planet::tint::planet_tint;
use crate::js::{self, num};
use crate::state::{MapOverlay, MapPlanets, settings, ui};
use crate::ui::emblems::EmblemArt;
use crate::ui::icons;
use crate::ui::primitives::Loading;
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;
use std::rc::Rc;
use std::sync::Arc;
use warroom_core::api::{Faction, Planet};
use warroom_core::defense::attackers;
use warroom_core::effects::{EffectKind, PlanetEffect, has_pawn};
use warroom_core::faction_marks::faction_marks;
use warroom_core::factions::faction_color;
use warroom_core::galaxy::{GALAXY_SCALE, LANDMARKS, Pt, anomaly_of, project, sector_at, supply_lines};
use warroom_core::planets::{by_index, ownership_key};
use warroom_core::regions::build_regions;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

/// Hatch spacing in screen pixels; the pattern is counter-scaled on zoom to keep it.
const HATCH: f64 = 7.0;

thread_local! {
    /// The last `focus_seq` handled, so a remounted map doesn't replay it.
    static FOCUS_DONE: Cell<u64> = const { Cell::new(0) };
}

#[derive(Clone, Debug, PartialEq)]
struct PlanetLook {
    x: f64,
    y: f64,
    class: String,
    name: String,
    /// The active-planet tag's colour and word.
    tag: Option<(&'static str, &'static str)>,
    body: Body,
}

#[allow(clippy::too_many_arguments)]
fn planet_look(
    p: &Planet,
    effects: Option<&Vec<PlanetEffect>>,
    active: bool,
    mo: bool,
    pinned: bool,
    selected: bool,
    dss: Option<DssInfo>,
    style: MapPlanets,
    now: f64,
) -> PlanetLook {
    let [x, y] = project(p.position);
    let faction = p.event.as_ref().map(|e| e.faction).unwrap_or(p.current_owner);
    let color = faction_color(Some(faction));
    let empty = Vec::new();
    let effects = effects.unwrap_or(&empty);
    let ids: Vec<i64> = effects.iter().map(|e| e.id).collect();
    let mut pois: Vec<PoiKind> = Vec::new();
    for e in effects.iter().filter(|e| e.kind == EffectKind::Poi) {
        let k = poi_kind(&e.name);
        if !pois.contains(&k) {
            pois.push(k);
        }
    }
    pois.truncate(POI_MAX);
    let flags = [
        (active, "active"),
        (mo, "mo"),
        (pinned, "pinned"),
        (selected, "selected"),
        (p.current_owner == Faction::Humans, "se"),
        (p.index == 0, "home"),
        (!pois.is_empty(), "poi"),
    ];
    let mut class = String::from("planet");
    for (on, name) in flags {
        if on {
            class.push(' ');
            class.push_str(name);
        }
    }
    let home = p.index == 0;
    PlanetLook {
        x,
        y,
        class,
        name: p.name.clone(),
        tag: active.then(|| (color, if p.event.is_some() { "Defend" } else { "Liberate" })),
        body: Body {
            style,
            faction,
            color: color.to_string(),
            fill: if home { "#3f8cc2".to_string() } else { sphere_color(planet_tint(p, &ids).ground) },
            size: if home { 1.6 } else { 1.0 },
            anomaly: anomaly_of(&p.name, &ids),
            active,
            defense: p.event.is_some(),
            mo,
            enemy: p.current_owner != Faction::Humans || p.event.is_some(),
            pinned,
            pawns: effects.iter().filter(|e| has_pawn(e) && e.until.is_none_or(|u| u > now)).cloned().collect(),
            dss,
            seed: p.index,
            pois,
        },
    }
}

/// Hatches for a planet under attack inside its attacker's region, in the attacker's colour.
fn contested_hatches() -> impl IntoView {
    Faction::ENEMIES
        .iter()
        .map(|f| {
            let c = faction_color(Some(*f));
            view! {
                <pattern
                    id=format!("region-hatch-{}", f.as_str())
                    class="hatch"
                    width=num(HATCH)
                    height=num(HATCH)
                    patternUnits="userSpaceOnUse"
                    patternTransform="rotate(45)"
                >
                    <rect width=num(HATCH) height=num(HATCH) fill=c fill-opacity="0.08"></rect>
                    <line
                        x1=num(HATCH * 0.15)
                        y1="0"
                        x2=num(HATCH * 0.15)
                        y2=num(HATCH)
                        stroke=c
                        stroke-opacity="0.6"
                        stroke-width=num(HATCH * 0.3)
                    ></line>
                </pattern>
            }
        })
        .collect_view()
}

/// Chevrons along an attack line: one per this many map units, moving this many map units a second.
const CHEVRON_GAP: f64 = 16.0;
const CHEVRON_SPEED: f64 = 9.0;

/// The line from an attacking world to the planet it attacks, with chevrons running along it to the target.
fn attack_line(from: Pt, to: Pt, color: &'static str) -> impl IntoView {
    let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
    let len = dx.hypot(dy);
    let n = (len / CHEVRON_GAP).floor().max(1.0);
    let run = len / CHEVRON_SPEED;
    view! {
        <g
            class="attack-line"
            transform=format!("translate({},{}) rotate({})", num(from[0]), num(from[1]), num(dy.atan2(dx).to_degrees()))
            style=format!("color: {color}; --len: {}px; --run: {}s;", num(len), num(run))
        >
            <line x2=num(len)></line>
            {(0..n as usize)
                .map(|i| {
                    let at = (i as f64 + 0.5) / n;
                    view! {
                        <g class="attack-chev" style=format!("--at: {}px; animation-delay: {}s;", num(len * at), num(-run * at))>
                            <path class="attack-arrow" d="M-4.5,-5L2.5,0L-4.5,5L-2,0Z"></path>
                        </g>
                    }
                })
                .collect_view()}
        </g>
    }
}

/// `on_hover_sector` hears the sector under the pointer, whatever the overlay, or None off the disc.
#[component]
pub fn GalaxyMap(on_hover_sector: Callback<Option<String>>) -> impl IntoView {
    let q = queries();
    let fresh = q.fresh_planets;
    let loaded = Memo::new(move |_| fresh.with(Option::is_some));
    move || {
        if loaded.get() {
            view! { <MapView on_hover_sector=on_hover_sector /> }.into_any()
        } else {
            view! { <Loading what="galactic war data" /> }.into_any()
        }
    }
}

#[component]
fn MapView(on_hover_sector: Callback<Option<String>>) -> impl IntoView {
    let q = queries();
    let fresh = q.fresh_planets;
    let planets = Signal::derive(move || fresh.get().unwrap_or_default());
    let geo = q.geometry;
    let effects = q.effects;
    let s = settings();
    let u = ui();
    let overlay = Memo::new(move |_| s.get(|s| s.map_overlay));
    let style = Memo::new(move |_| s.get(|s| s.map_planets));
    let quality = Memo::new(move |_| s.get(|s| s.map_quality));
    let beacon = Memo::new(move |_| s.get(|s| s.map_beacon));
    let pinned = Memo::new(move |_| s.get(|s| s.pinned.iter().copied().collect::<HashSet<i64>>()));
    let mo = major_order_planets();
    let active = Memo::new(move |_| {
        q.campaigns.with(|c| {
            c.as_ref().map(|c| c.iter().map(|c| c.planet.index).collect::<HashSet<i64>>()).unwrap_or_default()
        })
    });
    let clock = js::use_now(30_000.0);
    let dss = Memo::new(move |_| {
        let now = clock.get();
        q.stations.with(|s| {
            s.as_ref()
                .map(|s| {
                    s.iter()
                        .map(|s| (s.planet_index, dss_info(&s.tactical_actions, now)))
                        .collect::<HashMap<i64, DssInfo>>()
                })
                .unwrap_or_default()
        })
    });
    let hover = RwSignal::new(None::<Hover>);
    let sector = StoredValue::new(None::<String>);
    let hover_sector = move |next: Option<String>| {
        if sector.with_value(|s| *s == next) {
            return;
        }
        sector.set_value(next.clone());
        on_hover_sector.run(next);
    };
    let side = RwSignal::new(None::<Side>);
    let close_side = Callback::new(move |_| side.set(None));
    let panel = RwSignal::new(None::<i64>);
    let close_panel = Callback::new(move |_| panel.set(None));

    let ctl = Ctl::new();
    let stars = Memo::new(move |_| quality.get());
    {
        let ctl = SendWrapper::new(ctl.clone());
        Effect::new(move |_| {
            *ctl.stars.borrow_mut() = Rc::new(background_stars(star_count(stars.get())));
            ctl.overlay.set(overlay.get());
            ctl.quality.set(quality.get());
            ctl.draw_stars();
        });
    }
    {
        let ctl = SendWrapper::new(ctl.clone());
        Effect::new(move |_| {
            overlay.with(|_| ());
            fresh.with(|_| ());
            if let Some(svg) = ctl.svg.get_untracked() {
                scale_hatches(&svg, ctl.t.get().k);
            }
        });
    }
    // A new overlay or planet style rewrites the map's class, dropping the zoom classes the paint sets.
    {
        let ctl = SendWrapper::new(ctl.clone());
        Effect::new(move |_| {
            overlay.with(|_| ());
            style.with(|_| ());
            ctl.repaint();
        });
    }

    let ready = RwSignal::new(false);
    {
        let ctl = SendWrapper::new(ctl.clone());
        Effect::new(move |_| {
            let Some(stage) = ctl.stage.get() else { return };
            if ctl.zoom.borrow().is_some() {
                return;
            }
            let weak = Rc::downgrade(&*ctl);
            let stage: web_sys::HtmlElement = stage.into();
            let z = Zoom::attach(stage.clone(), SCALE_EXTENT, move |t| {
                let Some(c) = weak.upgrade() else { return };
                c.t.set(t);
                c.schedule();
                if hover.with_untracked(Option::is_some) {
                    hover.set(None);
                }
            });
            *ctl.zoom.borrow_mut() = Some(z.clone());
            ctl.fit();
            // A resize (a phone turned, a browser bar sliding away) keeps the middle in place instead of refitting.
            let size: Rc<Cell<Option<(f64, f64)>>> = Rc::new(Cell::new(ctl.rect()));
            let weak = Rc::downgrade(&*ctl);
            let on_resize = Closure::<dyn Fn()>::new(move || {
                let Some(c) = weak.upgrade() else { return };
                let Some((w, h)) = c.rect() else { return };
                if w == 0.0 || h == 0.0 {
                    return;
                }
                match size.replace(Some((w, h))) {
                    // The observer's first delivery; setting the transform would cancel a flight just started.
                    Some((bw, bh)) if bw == w && bh == h => {}
                    Some((bw, bh)) if bw > 0.0 && bh > 0.0 => {
                        let t = c.t.get();
                        if let Some(z) = c.zoom() {
                            z.set(Transform::at(t.x + (w - bw) / 2.0, t.y + (h - bh) / 2.0, t.k));
                        }
                    }
                    _ => c.fit(),
                }
            });
            if let Ok(ro) = web_sys::ResizeObserver::new(on_resize.as_ref().unchecked_ref()) {
                ro.observe(&stage);
                ctl.cleanup.borrow_mut().push(Box::new(move || {
                    ro.disconnect();
                    drop(on_resize);
                }));
            }
            touch_gestures(&ctl, &stage);
            ready.set(true);
        });
    }

    // Fly to a planet chosen elsewhere ("Show on map") or from the map's own options ("Focus").
    {
        let ctl = SendWrapper::new(ctl.clone());
        Effect::new(move |_| {
            let seq = u.focus_seq.get();
            if !ready.get() || seq == 0 || FOCUS_DONE.get() >= seq {
                return;
            }
            FOCUS_DONE.set(seq);
            let Some(index) = u.selected_planet.get_untracked() else { return };
            let Some(p) =
                fresh.with_untracked(|f| f.as_ref().and_then(|f| f.iter().find(|p| p.index == index).cloned()))
            else {
                return;
            };
            let (Some(z), Some((w, h))) = (ctl.zoom(), ctl.rect()) else { return };
            panel.set(Some(p.index));
            let [x, y] = to_layer(project(p.position), ctl.cam.get());
            let k = 3f64.max(w.min(h) / 280.0);
            z.fly_to(Transform::at(w / 2.0 - x * k, h / 2.0 - y * k, k), 700.0);
        });
    }

    // H, Z and C belong to the page.
    {
        let keys = ctl.clone();
        let on_key = Closure::<dyn Fn(web_sys::Event)>::new(move |e: web_sys::Event| {
            let ctl = &keys;
            let e: &web_sys::KeyboardEvent = e.unchecked_ref();
            let in_field = e
                .target()
                .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                .is_some_and(|el| el.closest("input, textarea, select").ok().flatten().is_some());
            if e.ctrl_key() || e.meta_key() || in_field {
                return;
            }
            match e.key().to_lowercase().as_str() {
                "+" | "=" => ctl.zoom_by(1.5),
                "-" => ctl.zoom_by(1.0 / 1.5),
                "0" => ctl.fit(),
                "q" => ctl.animate_camera(Some(ctl.aim().rot - ROT_STEP), None),
                "e" => ctl.animate_camera(Some(ctl.aim().rot + ROT_STEP), None),
                "r" => ctl.animate_camera(None, Some(ctl.aim().tilt + TILT_STEP)),
                "f" => ctl.animate_camera(None, Some(ctl.aim().tilt - TILT_STEP)),
                _ => {}
            }
        });
        let w: web_sys::EventTarget = js::window().into();
        ctl.listen(&w, "keydown", false, on_key);
    }
    {
        let ctl = SendWrapper::new(ctl.clone());
        on_cleanup(move || ctl.teardown());
    }

    // Right-drag (or Ctrl + drag) turns the map sideways and tilts it up and down; the zoom ignores both.
    let on_pointer_down = {
        let ctl = ctl.clone();
        move |e: web_sys::PointerEvent| {
            if e.button() != 2 && !(e.button() == 0 && e.ctrl_key()) {
                return;
            }
            ctl.settle_camera();
            if let Some(t) = e.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) {
                let _ = t.set_pointer_capture(e.pointer_id());
            }
            ctl.drag.set(Some((e.client_x() as f64, e.client_y() as f64, ctl.cam.get())));
        }
    };
    let on_pointer_move = {
        let ctl = ctl.clone();
        move |e: web_sys::PointerEvent| {
            if let Some((x, y, cam)) = ctl.drag.get() {
                ctl.set_camera(
                    Some(cam.rot + (e.client_x() as f64 - x) * DRAG_ROT),
                    Some(cam.tilt - (e.client_y() as f64 - y) * DRAG_TILT),
                );
                return;
            }
            if e.buttons() != 0 {
                return;
            }
            let Some(stage) = ctl.stage.get_untracked() else { return };
            let found = geo.with_untracked(|g| {
                let g = g.as_ref()?;
                let b = stage.get_bounding_client_rect();
                let at = ctl.t.get().invert([e.client_x() as f64 - b.left(), e.client_y() as f64 - b.top()]);
                let [x, y] = from_layer(at, ctl.cam.get());
                Some(sector_at(g, x / GALAXY_SCALE, -y / GALAXY_SCALE).map(str::to_string))
            });
            if let Some(found) = found {
                hover_sector(found);
            }
        }
    };
    let end_drag = {
        let ctl = ctl.clone();
        move || ctl.drag.set(None)
    };
    let end_drag2 = end_drag.clone();

    let owners = Memo::new(move |_| planets.with(|p| ownership_key(p)));
    let regions = Memo::new(move |_| {
        owners.with(|_| ());
        planets.with_untracked(|p| build_regions(p))
    });
    let marks = Memo::new(move |_| {
        owners.with(|_| ());
        let o = overlay.get().core();
        geo.with(|g| planets.with_untracked(|p| faction_marks(p, o, g.as_deref())))
    });
    let lanes = Memo::new(move |_| {
        planets.with(|p| {
            let lookup = by_index(p);
            let mut d = String::new();
            for (a, b) in supply_lines(p) {
                let (Some(pa), Some(pb)) = (lookup.get(&a), lookup.get(&b)) else { continue };
                let [x1, y1] = project(pa.position);
                let [x2, y2] = project(pb.position);
                d.push_str(&format!("M{},{}L{},{}", num(x1), num(y1), num(x2), num(y2)));
            }
            d
        })
    });
    let attacks = Memo::new(move |_| {
        planets.with(|p| {
            p.iter()
                .filter(|t| t.event.is_some())
                .flat_map(|t| {
                    attackers(t, p)
                        .into_iter()
                        .map(|a| (project(a.position), project(t.position), faction_color(Some(a.current_owner))))
                })
                .collect::<Vec<_>>()
        })
    });
    // Planets under attack go last, so their beacons draw over their neighbours.
    let indices = Memo::new(move |_| {
        planets.with(|p| {
            let (attacked, rest): (Vec<&Planet>, Vec<&Planet>) = p.iter().partition(|p| p.event.is_some());
            rest.into_iter().chain(attacked).map(|p| p.index).collect::<Vec<_>>()
        })
    });
    let gloom =
        Signal::derive(move || effects.with(|e| Arc::new(e.as_ref().map(|e| e.gloom.clone()).unwrap_or_default())));

    let open = Callback::new(move |index: i64| {
        panel.set(Some(index));
        u.select(Some(index));
    });
    let planet_view = move |index: i64| {
        let look = Memo::new(move |_| {
            planets.with(|list| {
                let p = list.iter().find(|p| p.index == index)?;
                Some(effects.with(|e| {
                    planet_look(
                        p,
                        e.as_ref().and_then(|e| e.planets.get(&index)),
                        active.with(|a| a.contains(&index)),
                        mo.with(|m| m.contains(&index)),
                        pinned.with(|s| s.contains(&index)),
                        u.selected_planet.get() == Some(index),
                        dss.with(|d| d.get(&index).cloned()),
                        style.get(),
                        clock.get(),
                    )
                }))
            })
        });
        let class = Memo::new(move |_| look.with(|l| l.as_ref().map(|l| l.class.clone()).unwrap_or_default()));
        let at = Memo::new(move |_| look.with(|l| l.as_ref().map(|l| format!("translate({},{})", num(l.x), num(l.y)))));
        let body = Memo::new(move |_| look.with(|l| l.as_ref().map(|l| l.body.clone())));
        let name = Memo::new(move |_| look.with(|l| l.as_ref().map(|l| l.name.clone()).unwrap_or_default()));
        let tag = Memo::new(move |_| look.with(|l| l.as_ref().and_then(|l| l.tag)));
        view! {
            <g
                class=move || class.get()
                transform=move || at.get()
                on:click=move |_| open.run(index)
                on:pointerenter=move |e: web_sys::PointerEvent| {
                    let p = planets.with_untracked(|l| l.iter().find(|p| p.index == index).cloned());
                    if let Some(p) = p {
                        hover.set(Some(Hover { x: e.client_x() as f64, y: e.client_y() as f64, what: Hovered::Planet(Box::new(p)) }));
                    }
                }
                on:pointerleave=move |_| hover.set(None)
            >
                <g class="upright">
                    {move || body.with(|b| b.as_ref().map(planet_body))}
                    <text class="planet-label">{move || name.get()}</text>
                    {move || tag.get().map(|(color, word)| view! { <text class="planet-tag" fill=color>{word}</text> })}
                </g>
            </g>
        }
    };

    let ctl_compass = ctl.clone();
    let ctl_tilt = ctl.clone();
    let ctl_in = ctl.clone();
    let ctl_out = ctl.clone();
    let (stage_ref, svg_ref, canvas_ref, layer_ref, compass_ref, tilt_ref) =
        (ctl.stage, ctl.svg, ctl.canvas, ctl.layer, ctl.compass, ctl.tilt);
    let panel_planet =
        Memo::new(move |_| panel.get().and_then(|i| planets.with(|p| p.iter().find(|p| p.index == i).cloned())));
    let panel_open = Memo::new(move |_| panel_planet.with(Option::is_some));
    let effects_of = move |index: Signal<Option<i64>>| {
        Signal::derive(move || {
            let Some(i) = index.get() else { return Vec::new() };
            effects.with(|e| e.as_ref().and_then(|e| e.planets.get(&i).cloned()).unwrap_or_default())
        })
    };
    let panel_index = Signal::derive(move || panel_planet.with(|p| p.as_ref().map(|p| p.index)));
    let hover_index = Signal::derive(move || {
        hover.with(|h| match h.as_ref().map(|h| &h.what) {
            Some(Hovered::Planet(p)) => Some(p.index),
            _ => None,
        })
    });

    view! {
        <div class=move || format!("map-wrap q-{} bc-{}", quality.get().as_str(), beacon.get().as_str())>
            <canvas node_ref=canvas_ref class="starfield-canvas" aria-hidden="true"></canvas>
            <div node_ref=stage_ref class="map-stage">
                <svg
                    node_ref=svg_ref
                    class=move || format!("galaxy ov-{} pl-{}", overlay.get().as_str(), style.get().as_str())
                    role="img"
                    aria-label="Galactic war map"
                    on:pointerdown=on_pointer_down
                    on:pointermove=on_pointer_move
                    on:pointerup=move |_| end_drag()
                    on:pointercancel=move |_| end_drag2()
                    on:contextmenu=|e| e.prevent_default()
                    on:pointerleave=move |_| {
                        hover.set(None);
                        hover_sector(None);
                    }
                >
                    {halo_defs()}
                    <g node_ref=layer_ref>
                        {move || {
                            (overlay.get() == MapOverlay::Regions)
                                .then(|| {
                                    view! {
                                        <g class="regions">
                                            <defs>{contested_hatches()}</defs>
                                            {move || {
                                                regions
                                                    .get()
                                                    .into_iter()
                                                    .enumerate()
                                                    .map(|(i, r)| {
                                                        let c = faction_color(Some(r.owner));
                                                        let contested = (!r.contested.is_empty())
                                                            .then(|| {
                                                                let clip = format!("region-clip-{i}");
                                                                view! {
                                                                    <clipPath id=clip.clone()>
                                                                        <path d=r.path.clone()></path>
                                                                    </clipPath>
                                                                    <path
                                                                        d=r.contested.clone()
                                                                        class="region-contested"
                                                                        fill=format!("url(#region-hatch-{})", r.owner.as_str())
                                                                        clip-path=format!("url(#{clip})")
                                                                    ></path>
                                                                }
                                                            });
                                                        view! {
                                                            <g>
                                                                <path d=r.path.clone() fill=c fill-opacity="0.05" stroke="none"></path>
                                                                {contested}
                                                                <path d=r.path.clone() class="region-glow" stroke=c></path>
                                                                <path d=r.path class="region-line" stroke=c></path>
                                                            </g>
                                                        }
                                                    })
                                                    .collect_view()
                                            }}
                                        </g>
                                    }
                                })
                        }}
                        {move || {
                            (overlay.get() == MapOverlay::Sectors && geo.with(Option::is_some))
                                .then(|| {
                                    let g = Signal::derive(move || geo.get().unwrap_or_default());
                                    view! { <SectorLayer geo=g planets=planets id="map" hatch=HATCH /> }
                                })
                        }}
                        <g class=move || {
                            format!(
                                "lanes {}",
                                match overlay.get() {
                                    MapOverlay::Sectors => "front",
                                    MapOverlay::Galaxy => "bright",
                                    MapOverlay::Regions => "dim",
                                },
                            )
                        }>
                            <path d=move || lanes.get()></path>
                        </g>
                        {move || effects.with(Option::is_some).then(|| view! { <GloomLayer planets=planets gloom=gloom /> })}
                        <g class="faction-marks" aria-hidden="true">
                            {move || {
                                marks
                                    .get()
                                    .into_iter()
                                    .map(|m| {
                                        view! {
                                            <g transform=format!(
                                                "translate({},{})",
                                                warroom_core::format::to_fixed(m.x, 1),
                                                warroom_core::format::to_fixed(m.y, 1),
                                            )>
                                                <g class="upright">
                                                    <EmblemArt art=m.faction.as_str() size=72.0 />
                                                </g>
                                            </g>
                                        }
                                    })
                                    .collect_view()
                            }}
                        </g>
                        <g class="attacks">
                            {move || attacks.get().into_iter().map(|(from, to, c)| attack_line(from, to, c)).collect_view()}
                        </g>
                        <g class="planets">
                            <For each=move || indices.get() key=|i| *i children=planet_view />
                            {LANDMARKS
                                .iter()
                                .map(|l| {
                                    let [x, y] = project(l.position);
                                    view! {
                                        <g
                                            class="planet landmark"
                                            transform=format!("translate({},{})", num(x), num(y))
                                            on:pointerenter=move |e: web_sys::PointerEvent| {
                                                hover.set(Some(Hover { x: e.client_x() as f64, y: e.client_y() as f64, what: Hovered::Landmark(l) }));
                                            }
                                            on:pointerleave=move |_| hover.set(None)
                                        >
                                            <g class="upright">
                                                {move || {
                                                    let st = style.get();
                                                    planet_body(&Body {
                                                        style: st,
                                                        faction: Faction::Humans,
                                                        color: "#c98a6a".into(),
                                                        fill: "#b5532f".into(),
                                                        size: if st == MapPlanets::Spheres { 0.85 } else { 0.9 },
                                                        anomaly: Some(warroom_core::galaxy::Anomaly::Scorched),
                                                        active: false,
                                                        defense: false,
                                                        mo: false,
                                                        enemy: false,
                                                        pinned: false,
                                                        pawns: Vec::new(),
                                                        dss: None,
                                                        seed: 0,
                                                        pois: Vec::new(),
                                                    })
                                                }}
                                                <text class="planet-label">{l.name}</text>
                                            </g>
                                        </g>
                                    }
                                })
                                .collect_view()}
                        </g>
                    </g>
                </svg>
            </div>
            <div class="map-rail">
                {rail_button(icons::ZOOM_IN, "Zoom in", None, Some("+"), None, "", None, NodeRef::new(), move || ctl_in.zoom_by(1.5))}
                {rail_button(icons::ZOOM_OUT, "Zoom out", None, Some("-"), None, "", None, NodeRef::new(), move || ctl_out.zoom_by(1.0 / 1.5))}
                {rail_button(
                    icons::COMPASS,
                    "North",
                    Some("Turn north up and flatten the map"),
                    None,
                    None,
                    "map-compass",
                    Some(format!("transform: rotate({}rad);", num(-PI / 4.0))),
                    compass_ref,
                    move || ctl_compass.animate_camera(Some(0.0), Some(0.0)),
                )}
                {rail_button(
                    icons::ROTATE_3D,
                    "Tilt",
                    Some("Tilt or flatten the map"),
                    Some("R / F"),
                    None,
                    "",
                    None,
                    tilt_ref,
                    move || {
                        let tilt = if ctl_tilt.aim().tilt > 0.0 { 0.0 } else { TILT_PRESET };
                        ctl_tilt.animate_camera(None, Some(tilt));
                    },
                )}
                {rail_button(
                    icons::BOOK_OPEN,
                    "Legend",
                    None,
                    None,
                    Some(Signal::derive(move || side.get() == Some(Side::Legend))),
                    "",
                    None,
                    NodeRef::new(),
                    move || side.update(|s| *s = if *s == Some(Side::Legend) { None } else { Some(Side::Legend) }),
                )}
                {rail_button(
                    icons::SETTINGS,
                    "Settings",
                    Some("Map settings"),
                    None,
                    Some(Signal::derive(move || side.get() == Some(Side::Settings))),
                    "",
                    None,
                    NodeRef::new(),
                    move || side.update(|s| *s = if *s == Some(Side::Settings) { None } else { Some(Side::Settings) }),
                )}
            </div>
            {move || match side.get() {
                Some(Side::Settings) => view! { <MapSettings on_close=close_side /> }.into_any(),
                Some(Side::Legend) => view! { <MapLegend on_close=close_side /> }.into_any(),
                None => ().into_any(),
            }}
            {move || {
                panel_open
                    .get()
                    .then(|| {
                        let p = Signal::derive(move || panel_planet.get().unwrap_or_default());
                        let is_active = Signal::derive(move || panel_index.get().is_some_and(|i| active.with(|a| a.contains(&i))));
                        planet_panel(p, is_active, effects_of(panel_index), close_panel)
                    })
            }}
            {move || {
                hover
                    .get()
                    .map(|h| {
                        let is_active = Signal::derive(move || hover_index.get().is_some_and(|i| active.with(|a| a.contains(&i))));
                        hover_card(&h, is_active, effects_of(hover_index))
                    })
            }}
        </div>
    }
}
