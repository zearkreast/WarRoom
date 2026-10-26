use crate::assets;
use crate::js::{self, num};
use leptos::prelude::*;
use leptos::svg;
use leptos::task::spawn_local;
use send_wrapper::SendWrapper;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use warroom_core::data::{CityFateState, SUPER_EARTH_CITIES};
use warroom_core::format::to_fixed;
use warroom_core::geo::{MultiPolygon, Orthographic, distance, graticule10, topo_polygons};

/// Degrees the globe leans, north pole toward the viewer.
const TILT: f64 = -20.0;
/// Degrees of longitude turned per second.
const SPEED: f64 = 5.0;
/// Europe and Africa face the viewer first.
const START: f64 = -10.0;

thread_local! {
    static LAND: RefCell<Option<Rc<MultiPolygon>>> = const { RefCell::new(None) };
}

async fn load_land() -> Option<Rc<MultiPolygon>> {
    if let Some(l) = LAND.with(|l| l.borrow().clone()) {
        return Some(l);
    }
    let text = js::fetch_text(&assets::url("geo/land-110m.json")?).await.ok()?;
    let land = Rc::new(topo_polygons(&text, "land").ok()?);
    LAND.with(|l| *l.borrow_mut() = Some(land.clone()));
    Some(land)
}

fn state_class(s: CityFateState) -> &'static str {
    match s {
        CityFateState::Destroyed => "destroyed",
        CityFateState::Rebuilt => "rebuilt",
        CityFateState::Intact => "intact",
    }
}

/// Super Earth drawn as the Earth it is, turning, with its sector capitals lit or struck off.
#[component]
pub fn SuperEarthGlobe(r: f64) -> impl IntoView {
    let land_el = NodeRef::<svg::Path>::new();
    let grid_el = NodeRef::<svg::Path>::new();
    let city_els: Vec<NodeRef<svg::G>> = SUPER_EARTH_CITIES.iter().map(|_| NodeRef::new()).collect();
    let stop: SendWrapper<Rc<Cell<bool>>> = SendWrapper::new(Rc::new(Cell::new(false)));
    let halt = stop.clone();
    let els = city_els.clone();
    Effect::new(move |_| {
        let (Some(land_path), Some(grid_path)) = (land_el.get(), grid_el.get()) else { return };
        let stop = (*halt).clone();
        let els = els.clone();
        spawn_local(async move {
            let Some(land) = load_land().await else { return };
            if stop.get() {
                return;
            }
            let grid = graticule10();
            let draw = move |lon: f64| {
                let p = Orthographic { scale: r, rotate: [lon, TILT] };
                let _ = land_path.set_attribute("d", &p.polygons_path(&land));
                let _ = grid_path.set_attribute("d", &p.lines_path(&grid));
                for (c, el) in SUPER_EARTH_CITIES.iter().zip(&els) {
                    let Some(el) = el.get_untracked() else { continue };
                    let at = p.project(c.at);
                    let _ = el.set_attribute(
                        "transform",
                        &format!("translate({},{})", to_fixed(at[0], 1), to_fixed(at[1], 1)),
                    );
                    let shown = distance(c.at, [-lon, -TILT]) < std::f64::consts::FRAC_PI_2;
                    let _ = el.set_attribute("style", if shown { "" } else { "visibility: hidden;" });
                }
            };
            if js::prefers_reduced_motion() {
                draw(START);
                return;
            }
            let t0 = js::performance_now();
            let frames = js::animation_loop(move |t| {
                // Checked first: once the globe is gone, its city refs are disposed and reading them panics.
                if stop.get() {
                    return false;
                }
                draw(START + (t - t0) / 1000.0 * SPEED);
                true
            });
            let _ = frames;
        });
    });
    on_cleanup(move || stop.set(true));
    view! {
        <g class="se-globe">
            <defs>
                <radialGradient id="se-ocean" cx="38%" cy="32%" r="75%">
                    <stop offset="0%" stop-color="#2a6a9c" />
                    <stop offset="100%" stop-color="#0a2340" />
                </radialGradient>
            </defs>
            <circle r=num(r) fill="url(#se-ocean)" />
            <path node_ref=grid_el class="se-grid" />
            <path node_ref=land_el class="se-land" />
            {SUPER_EARTH_CITIES
                .iter()
                .zip(city_els)
                .map(|(c, el)| {
                    view! {
                        <g node_ref=el class=format!("se-city {}", state_class(c.state)) style="visibility: hidden;">
                            {if c.state == CityFateState::Destroyed {
                                view! { <path d="M-3 -3L3 3M3 -3L-3 3" /> }.into_any()
                            } else {
                                view! { <circle r="2.6" /> }.into_any()
                            }}
                        </g>
                    }
                })
                .collect_view()}
        </g>
    }
}
