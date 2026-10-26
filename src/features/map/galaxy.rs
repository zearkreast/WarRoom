use crate::api::queries::queries;
use crate::features::map::camera::write_camera;
use crate::features::map::card::{Hover, Hovered, hover_card, planet_panel};
use crate::features::map::layers::{GloomLayer, SectorLayer};
use crate::features::map::legend::{MapLegend, close_outside};
use crate::features::map::marks::{Body, POI_MAX, halo_defs, planet_body, sphere_color};
use crate::features::map::options::{overlay_options, planet_options, quality_options};
use crate::features::map::pawn_art::{DssInfo, dss_info};
use crate::features::map::poi::{PoiKind, poi_kind};
use crate::features::map::zoom::{Transform, Zoom};
use crate::features::planet::tint::planet_tint;
use crate::features::planets::fronts::major_order_planets;
use crate::js::{self, num};
use crate::state::{MapOverlay, MapPlanets, MapQuality, View, settings, ui};
use crate::ui::emblems::EmblemArt;
use crate::ui::icons::{self, Icon, IconData};
use crate::ui::primitives::{Loading, Stepper};
use leptos::html;
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use warroom_core::api::{Faction, Planet};
use warroom_core::effects::{EffectKind, PlanetEffect, has_pawn};
use warroom_core::faction_marks::faction_marks;
use warroom_core::factions::faction_color;
use warroom_core::galaxy::{
    GALAXY_SCALE, LANDMARKS, Pt, RINGS, anomaly_of, project, ring_radius, sector_at, supply_lines,
};
use warroom_core::planets::{by_index, ownership_key};
use warroom_core::regions::build_regions;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

const EXTENT: f64 = GALAXY_SCALE * 1.1;

fn star_count(q: MapQuality) -> usize {
    match q {
        MapQuality::Max => 4200,
        MapQuality::Normal => 2600,
        MapQuality::Min => 1600,
    }
}

/// Highest device pixel ratio the starfield canvas is drawn at; phones report up to 3 or 4.
fn star_dpr(q: MapQuality) -> f64 {
    match q {
        MapQuality::Max => 4.0,
        MapQuality::Normal => 2.0,
        MapQuality::Min => 1.5,
    }
}

/// Map rotation (radians, clockwise on screen) and tilt (radians, 0 = straight down).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Camera {
    rot: f64,
    tilt: f64,
}

const MAX_TILT: f64 = (55.0 * PI) / 180.0;
const TILT_PRESET: f64 = (45.0 * PI) / 180.0;
const ROT_STEP: f64 = PI / 12.0;
const TILT_STEP: f64 = PI / 18.0;
const DRAG_ROT: f64 = 0.006;
const DRAG_TILT: f64 = 0.005;
const TOUCH_TILT: f64 = 0.008;
/// Finger travel (px) before a touch counts as a drag rather than a tap.
const TOUCH_SLOP: f64 = 6.0;
/// Two fingers must twist this far (radians) before the map turns, so a pinch doesn't wobble it.
const TWIST_START: f64 = 0.2;
const SCALE_EXTENT: (f64, f64) = (0.5, 14.0);
/// How long the buttons and keys take to turn or tilt the map.
const CAMERA_MS: f64 = 320.0;
/// How long the camera must be still before the costly glow filters come back.
const SETTLE_MS: i32 = 180;
/// Hatch spacing in screen pixels; the pattern is counter-scaled on zoom to keep it.
const HATCH: f64 = 7.0;

/// An angle in (-π, π].
fn wrap(a: f64) -> f64 {
    a.sin().atan2(a.cos())
}

fn turn([x, y]: Pt, a: f64) -> Pt {
    [x * a.cos() - y * a.sin(), x * a.sin() + y * a.cos()]
}

/// Map units to the zoom layer's own units: rotated, then squashed by the tilt.
fn to_layer(p: Pt, cam: Camera) -> Pt {
    let [x, y] = turn(p, cam.rot);
    [x, y * cam.tilt.cos()]
}

fn from_layer([x, y]: Pt, cam: Camera) -> Pt {
    turn([x, y / cam.tilt.cos()], -cam.rot)
}

fn angle_of(a: Pt, b: Pt) -> f64 {
    (b[1] - a[1]).atan2(b[0] - a[0])
}

fn distance(a: Pt, b: Pt) -> f64 {
    (b[0] - a[0]).hypot(b[1] - a[1])
}

fn midpoint(a: Pt, b: Pt) -> Pt {
    [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0]
}

struct Star {
    x: f64,
    y: f64,
    s: f64,
    o: f64,
    c: &'static str,
}

/// Deterministic background stars so the field doesn't reshuffle between renders.
fn background_stars(n: usize) -> Vec<Star> {
    let mut seed = 7.0f64;
    let mut rnd = move || {
        seed = (seed * 16807.0) % 2147483647.0;
        seed / 2147483647.0
    };
    (0..n)
        .map(|_| {
            let a = rnd() * PI * 2.0;
            // Denser towards the core.
            let r = rnd().powf(0.8) * EXTENT * 1.35;
            let hue = rnd();
            let s = if rnd() < 0.06 { 1.6 } else { rnd() * 0.9 + 0.4 };
            let o = rnd() * 0.55 + 0.15;
            let c = if hue < 0.12 {
                "#9fd4ff"
            } else if hue < 0.18 {
                "#ffd9a0"
            } else {
                "#ffffff"
            };
            Star { x: a.cos() * r, y: a.sin() * r, s, o, c }
        })
        .collect()
}

fn draw_stars(
    canvas: &web_sys::HtmlCanvasElement,
    stars: &[Star],
    t: Transform,
    cam: Camera,
    rings: bool,
    max_dpr: f64,
) {
    let dpr = js::window().device_pixel_ratio();
    let dpr = if dpr > 0.0 { dpr } else { 1.0 }.min(max_dpr);
    let rect = canvas.get_bounding_client_rect();
    let (width, height) = (rect.width(), rect.height());
    let (w, h) = (warroom_core::format::js_round(width * dpr), warroom_core::format::js_round(height * dpr));
    if canvas.width() as f64 != w {
        canvas.set_width(w as u32);
    }
    if canvas.height() as f64 != h {
        canvas.set_height(h as u32);
    }
    let Some(ctx) =
        canvas.get_context("2d").ok().flatten().and_then(|c| c.dyn_into::<web_sys::CanvasRenderingContext2d>().ok())
    else {
        return;
    };
    let _ = ctx.set_transform(dpr, 0.0, 0.0, dpr, 0.0, 0.0);
    ctx.clear_rect(0.0, 0.0, width, height);
    let squash = cam.tilt.cos();

    // Core glow around Super Earth, flattened with the disc.
    let (cx, cy) = (t.x, t.y);
    ctx.save();
    let _ = ctx.translate(cx, cy);
    let _ = ctx.scale(1.0, squash);
    if let Ok(glow) = ctx.create_radial_gradient(0.0, 0.0, 0.0, 0.0, 0.0, 140.0 * t.k) {
        let _ = glow.add_color_stop(0.0, "rgba(120, 200, 255, 0.28)");
        let _ = glow.add_color_stop(0.35, "rgba(70, 164, 255, 0.08)");
        let _ = glow.add_color_stop(1.0, "rgba(70, 164, 255, 0)");
        ctx.set_fill_style_canvas_gradient(&glow);
        ctx.fill_rect(-width * 2.0, (-height * 2.0) / squash, width * 4.0, (height * 4.0) / squash);
    }
    ctx.restore();

    // Stars drift a little slower than the map, so zooming has depth.
    let k = t.k.powf(0.85);
    let (cos, sin) = (cam.rot.cos(), cam.rot.sin());
    for s in stars {
        let x = (s.x * cos - s.y * sin) * k + t.x;
        let y = (s.x * sin + s.y * cos) * k + t.y;
        if x < -2.0 || y < -2.0 || x > width + 2.0 || y > height + 2.0 {
            continue;
        }
        ctx.set_global_alpha(s.o);
        ctx.set_fill_style_str(s.c);
        ctx.fill_rect(x, y, s.s, s.s);
    }
    ctx.set_global_alpha(1.0);

    // The game's sector rings, so planets read against the same bands; the Sectors overlay draws its own borders.
    if !rings {
        return;
    }
    ctx.set_stroke_style_str("rgba(120, 170, 230, 0.09)");
    ctx.set_line_width(1.0);
    for i in 0..RINGS {
        let r = ring_radius(i) * GALAXY_SCALE * t.k;
        ctx.begin_path();
        let _ = ctx.ellipse(cx, cy, r, r * squash, 0.0, 0.0, PI * 2.0);
        ctx.stroke();
    }
}

fn scale_hatches(svg: &web_sys::Element, k: f64) {
    let Ok(list) = svg.query_selector_all("pattern.hatch") else { return };
    for i in 0..list.length() {
        if let Some(p) = list.item(i).and_then(|n| n.dyn_into::<web_sys::Element>().ok()) {
            let _ = p.set_attribute(
                "patternTransform",
                &format!("rotate(45) scale({})", warroom_core::format::to_fixed(1.0 / k, 4)),
            );
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Side {
    Settings,
    Legend,
}

#[component]
fn MapSettings(on_close: Callback<()>) -> impl IntoView {
    let panel = NodeRef::<html::Div>::new();
    close_outside(panel, on_close);
    let s = settings();
    view! {
        <div node_ref=panel class="map-settings panel glass" role="dialog" aria-label="Map settings">
            <div class="ms-head caps">
                <Icon icon=icons::SETTINGS size=14.0 />
                " Map settings"
            </div>
            <div class="ms-row">
                <span class="caps">"Overlay"</span>
                <Stepper
                    label="Map overlay"
                    options=overlay_options()
                    value=Signal::derive(move || s.get(|s| s.map_overlay))
                    on_change=move |v| s.set(|s| s.map_overlay = v)
                />
            </div>
            <div class="ms-row">
                <span class="caps">"Planets"</span>
                <Stepper
                    label="Planet style"
                    options=planet_options()
                    value=Signal::derive(move || s.get(|s| s.map_planets))
                    on_change=move |v| s.set(|s| s.map_planets = v)
                />
            </div>
            <div class="ms-row">
                <span class="caps">"Quality"</span>
                <Stepper
                    label="Render quality"
                    options=quality_options()
                    value=Signal::derive(move || s.get(|s| s.map_quality))
                    on_change=move |v| s.set(|s| s.map_quality = v)
                />
            </div>
            <button class="link ms-more caps" on:click=move |_| ui().set_view(View::Settings)>
                "All settings "
                <Icon icon=icons::ARROW_UP_RIGHT size=12.0 />
            </button>
        </div>
    }
}

#[allow(clippy::too_many_arguments)]
fn rail_button(
    icon: IconData,
    label: &'static str,
    title: Option<&'static str>,
    key_hint: Option<&'static str>,
    active: Option<Signal<bool>>,
    class: &'static str,
    icon_style: Option<String>,
    node_ref: NodeRef<html::Button>,
    on_click: impl Fn() + 'static,
) -> impl IntoView {
    let title = title.unwrap_or(label);
    let full = match key_hint {
        Some(k) => format!("{title} ({k})"),
        None => title.to_string(),
    };
    view! {
        <button
            node_ref=node_ref
            class=move || format!("rail-btn {} {class}", if active.is_some_and(|a| a.get()) { "on" } else { "" })
            on:click=move |_| on_click()
            aria-label=title
            aria-pressed=move || active.map(|a| a.get().to_string())
            title=full
        >
            <Icon icon=icon size=18.0 stroke_width=1.75 style=icon_style />
            <span class="caps">{label}</span>
        </button>
    }
}

/// Where everything about the camera lives between frames; none of it is reactive state.
struct Ctl {
    stage: NodeRef<html::Div>,
    svg: NodeRef<leptos::svg::Svg>,
    canvas: NodeRef<html::Canvas>,
    layer: NodeRef<leptos::svg::G>,
    compass: NodeRef<html::Button>,
    tilt: NodeRef<html::Button>,
    zoom: RefCell<Option<Rc<Zoom>>>,
    t: Cell<Transform>,
    cam: Cell<Camera>,
    stars: RefCell<Rc<Vec<Star>>>,
    overlay: Cell<MapOverlay>,
    quality: Cell<MapQuality>,
    frame: Cell<Option<i32>>,
    paint_fn: RefCell<Option<Closure<dyn Fn(f64)>>>,
    settle: Cell<Option<i32>>,
    settle_fn: RefCell<Option<Closure<dyn Fn()>>>,
    written: RefCell<String>,
    tween: Cell<Option<i32>>,
    tween_fn: RefCell<Option<Closure<dyn Fn(f64)>>>,
    /// Where the buttons and keys are taking the camera, so a key pressed mid-turn adds to the turn.
    goal: Cell<Option<Camera>>,
    drag: Cell<Option<(f64, f64, Camera)>>,
    cleanup: RefCell<Vec<Box<dyn FnOnce()>>>,
}

impl Ctl {
    fn new() -> Rc<Ctl> {
        let ctl = Rc::new(Ctl {
            stage: NodeRef::new(),
            svg: NodeRef::new(),
            canvas: NodeRef::new(),
            layer: NodeRef::new(),
            compass: NodeRef::new(),
            tilt: NodeRef::new(),
            zoom: RefCell::new(None),
            t: Cell::new(Transform::IDENTITY),
            cam: Cell::new(Camera::default()),
            stars: RefCell::new(Rc::new(Vec::new())),
            overlay: Cell::new(MapOverlay::Regions),
            quality: Cell::new(MapQuality::Normal),
            frame: Cell::new(None),
            paint_fn: RefCell::new(None),
            settle: Cell::new(None),
            settle_fn: RefCell::new(None),
            written: RefCell::new(String::new()),
            tween: Cell::new(None),
            tween_fn: RefCell::new(None),
            goal: Cell::new(None),
            drag: Cell::new(None),
            cleanup: RefCell::new(Vec::new()),
        });
        let weak = Rc::downgrade(&ctl);
        *ctl.paint_fn.borrow_mut() = Some(Closure::new(move |_: f64| {
            if let Some(c) = weak.upgrade() {
                c.paint();
            }
        }));
        let weak = Rc::downgrade(&ctl);
        *ctl.settle_fn.borrow_mut() = Some(Closure::new(move || {
            let Some(c) = weak.upgrade() else { return };
            c.settle.set(None);
            if let Some(svg) = c.svg.get_untracked() {
                let _ = svg.class_list().remove_1("moving");
                scale_hatches(&svg, c.t.get().k);
            }
        }));
        ctl
    }

    fn zoom(&self) -> Option<Rc<Zoom>> {
        self.zoom.borrow().clone()
    }

    fn draw_stars(&self) {
        if let Some(canvas) = self.canvas.get_untracked() {
            draw_stars(
                &canvas,
                &self.stars.borrow(),
                self.t.get(),
                self.cam.get(),
                self.overlay.get() != MapOverlay::Sectors,
                star_dpr(self.quality.get()),
            );
        }
    }

    /// Writes the camera to the DOM once per frame.
    fn paint(&self) {
        self.frame.set(None);
        let (Some(svg), Some(layer)) = (self.svg.get_untracked(), self.layer.get_untracked()) else { return };
        let t = self.t.get();
        let Camera { rot, tilt } = self.cam.get();
        let transform = format!(
            "translate({},{}) scale({}) scale(1,{}) rotate({})",
            num(t.x),
            num(t.y),
            num(t.k),
            num(tilt.cos()),
            num((rot * 180.0) / PI)
        );
        if *self.written.borrow() != transform {
            let _ = layer.set_attribute("transform", &transform);
            *self.written.borrow_mut() = transform;
        }
        write_camera(t.k, rot, tilt);
        let classes = svg.class_list();
        let _ = classes.toggle_with_force("z-labels", t.k > 0.9);
        let _ = classes.toggle_with_force("z-sectors", t.k > 1.6);
        let _ = classes.toggle_with_force("z-planets", t.k > 3.0);
        let _ = classes.toggle_with_force("z-units", t.k > 2.0);
        if let Some(c) = self.compass.get_untracked() {
            let _ = c.class_list().toggle_with_force("on", rot != 0.0 || tilt != 0.0);
            if let Some(icon) =
                c.query_selector("svg").ok().flatten().and_then(|e| e.dyn_into::<web_sys::SvgElement>().ok())
            {
                let _ = icon.style().set_property("transform", &format!("rotate({}rad)", num(rot - PI / 4.0)));
            }
        }
        if let Some(b) = self.tilt.get_untracked() {
            let _ = b.class_list().toggle_with_force("on", tilt > 0.0);
        }
        self.draw_stars();
    }

    /// Queues a paint and marks the map as moving, which drops the glow filters until it settles.
    fn schedule(&self) {
        let Some(svg) = self.svg.get_untracked() else { return };
        let w = js::window();
        if self.frame.get().is_none() {
            let id = self
                .paint_fn
                .borrow()
                .as_ref()
                .and_then(|f| w.request_animation_frame(f.as_ref().unchecked_ref()).ok());
            self.frame.set(id);
        }
        let _ = svg.class_list().add_1("moving");
        if let Some(id) = self.settle.take() {
            w.clear_timeout_with_handle(id);
        }
        let id = self.settle_fn.borrow().as_ref().and_then(|f| {
            w.set_timeout_with_callback_and_timeout_and_arguments_0(f.as_ref().unchecked_ref(), SETTLE_MS).ok()
        });
        self.settle.set(id);
    }

    fn rect(&self) -> Option<(f64, f64)> {
        let r = self.stage.get_untracked()?.get_bounding_client_rect();
        Some((r.width(), r.height()))
    }

    fn fit(&self) {
        let (Some(z), Some((w, h))) = (self.zoom(), self.rect()) else { return };
        let k = w.min(h) / (EXTENT * 2.0);
        z.set(Transform::at(w / 2.0, h / 2.0, k));
    }

    fn zoom_by(&self, f: f64) {
        if let Some(z) = self.zoom() {
            z.scale_by(f, 250.0);
        }
    }

    /// Rotates and tilts about the middle of the screen, keeping whatever is there in place.
    fn set_camera(&self, rot: Option<f64>, tilt: Option<f64>) {
        let (Some(z), Some((w, h))) = (self.zoom(), self.rect()) else { return };
        // A zoom step or fly-to still running would keep pulling towards a spot worked out for the old camera.
        z.interrupt();
        let t = self.t.get();
        let c = [w / 2.0, h / 2.0];
        let cam = self.cam.get();
        let world = from_layer(t.invert(c), cam);
        let next = Camera { rot: wrap(rot.unwrap_or(cam.rot)), tilt: tilt.unwrap_or(cam.tilt).max(0.0).min(MAX_TILT) };
        self.cam.set(next);
        let [x, y] = to_layer(world, next);
        z.set(Transform::at(c[0] - t.k * x, c[1] - t.k * y, t.k));
    }

    /// Stops a camera tween and any zoom transition, for a gesture that takes over.
    fn settle_camera(&self) {
        if let Some(id) = self.tween.take() {
            let _ = js::window().cancel_animation_frame(id);
        }
        self.goal.set(None);
        if let Some(z) = self.zoom() {
            z.interrupt();
        }
    }

    fn aim(&self) -> Camera {
        self.goal.get().unwrap_or(self.cam.get())
    }

    /// Turns and tilts to the target over CAMERA_MS, the short way round.
    fn animate_camera(self: &Rc<Self>, rot: Option<f64>, tilt: Option<f64>) {
        if let Some(id) = self.tween.take() {
            let _ = js::window().cancel_animation_frame(id);
        }
        let from = self.cam.get();
        let goal = self.goal.get();
        let to = Camera {
            rot: rot.or(goal.map(|g| g.rot)).unwrap_or(from.rot),
            tilt: tilt.or(goal.map(|g| g.tilt)).unwrap_or(from.tilt).max(0.0).min(MAX_TILT),
        };
        self.goal.set(Some(to));
        let turn_by = wrap(to.rot - from.rot);
        let reduced = js::prefers_reduced_motion();
        let start = js::performance_now();
        let weak: Weak<Ctl> = Rc::downgrade(self);
        *self.tween_fn.borrow_mut() = Some(Closure::new(move |now: f64| {
            let Some(c) = weak.upgrade() else { return };
            let p = if reduced { 1.0 } else { ((now - start) / CAMERA_MS).min(1.0) };
            let eased = 1.0 - (1.0 - p).powi(3);
            c.set_camera(Some(from.rot + turn_by * eased), Some(from.tilt + (to.tilt - from.tilt) * eased));
            if p < 1.0 {
                let id = c
                    .tween_fn
                    .borrow()
                    .as_ref()
                    .and_then(|f| js::window().request_animation_frame(f.as_ref().unchecked_ref()).ok());
                c.tween.set(id);
            } else {
                c.tween.set(None);
                c.goal.set(None);
            }
        }));
        let id = self
            .tween_fn
            .borrow()
            .as_ref()
            .and_then(|f| js::window().request_animation_frame(f.as_ref().unchecked_ref()).ok());
        self.tween.set(id);
    }

    fn listen(
        &self,
        target: &web_sys::EventTarget,
        name: &'static str,
        capture: bool,
        f: Closure<dyn Fn(web_sys::Event)>,
    ) {
        let opts = web_sys::AddEventListenerOptions::new();
        opts.set_capture(capture);
        let _ = target.add_event_listener_with_callback_and_add_event_listener_options(
            name,
            f.as_ref().unchecked_ref(),
            &opts,
        );
        let target = target.clone();
        self.cleanup.borrow_mut().push(Box::new(move || {
            let _ = target.remove_event_listener_with_callback_and_bool(name, f.as_ref().unchecked_ref(), capture);
        }));
    }

    fn teardown(&self) {
        let w = js::window();
        if let Some(id) = self.frame.take() {
            let _ = w.cancel_animation_frame(id);
        }
        if let Some(id) = self.settle.take() {
            w.clear_timeout_with_handle(id);
        }
        if let Some(id) = self.tween.take() {
            let _ = w.cancel_animation_frame(id);
        }
        for f in self.cleanup.borrow_mut().drain(..) {
            f();
        }
        self.zoom.borrow_mut().take();
    }
}

/// Touch, which the zoom leaves alone: one finger pans; two fingers pinch to zoom and twist to turn, or
/// slide up and down together to tilt.
fn touch_gestures(ctl: &Rc<Ctl>, stage: &web_sys::HtmlElement) {
    #[derive(Clone, Copy, PartialEq)]
    enum Mode {
        Still,
        Pan,
        Pinch,
        Tilt,
    }
    struct Gesture {
        fingers: Vec<(i32, Pt)>,
        start: Option<(Vec<Pt>, Transform, Camera)>,
        mode: Mode,
        twist: Option<f64>,
        dragged: bool,
    }
    let g = Rc::new(RefCell::new(Gesture {
        fingers: Vec::new(),
        start: None,
        mode: Mode::Still,
        twist: None,
        dragged: false,
    }));
    let local = {
        let stage = stage.clone();
        move |e: &web_sys::PointerEvent| -> Pt {
            let b = stage.get_bounding_client_rect();
            [e.client_x() as f64 - b.left(), e.client_y() as f64 - b.top()]
        }
    };
    // Each change in the number of fingers starts the gesture afresh from where the camera is.
    let restart = {
        let ctl = Rc::downgrade(ctl);
        move |g: &mut Gesture| {
            let Some(c) = ctl.upgrade() else { return };
            let pts: Vec<Pt> = g.fingers.iter().take(2).map(|(_, p)| *p).collect();
            g.start = (!pts.is_empty()).then(|| (pts, c.t.get(), c.cam.get()));
            g.mode = Mode::Still;
            g.twist = None;
        }
    };
    let set_finger = |g: &mut Gesture, id: i32, p: Pt| match g.fingers.iter_mut().find(|(i, _)| *i == id) {
        Some(f) => f.1 = p,
        None => g.fingers.push((id, p)),
    };

    let on_down = {
        let (g, ctl, local, restart) = (g.clone(), Rc::downgrade(ctl), local.clone(), restart.clone());
        Closure::<dyn Fn(web_sys::Event)>::new(move |e: web_sys::Event| {
            let e: &web_sys::PointerEvent = e.unchecked_ref();
            if e.pointer_type() != "touch" {
                return;
            }
            let Some(c) = ctl.upgrade() else { return };
            c.settle_camera();
            let mut g = g.borrow_mut();
            if g.fingers.is_empty() {
                g.dragged = false;
            }
            set_finger(&mut g, e.pointer_id(), local(e));
            restart(&mut g);
        })
    };
    let on_move = {
        let (g, ctl, local) = (g.clone(), Rc::downgrade(ctl), local.clone());
        Closure::<dyn Fn(web_sys::Event)>::new(move |e: web_sys::Event| {
            let e: &web_sys::PointerEvent = e.unchecked_ref();
            let Some(c) = ctl.upgrade() else { return };
            let mut g = g.borrow_mut();
            if !g.fingers.iter().any(|(i, _)| *i == e.pointer_id()) || g.start.is_none() {
                return;
            }
            set_finger(&mut g, e.pointer_id(), local(e));
            let now: Vec<Pt> = g.fingers.iter().take(2).map(|(_, p)| *p).collect();
            let (s_pts, s_t, s_cam) = g.start.clone().unwrap();
            let apply = |x: f64, y: f64, k: f64| {
                if let Some(z) = c.zoom() {
                    z.set(Transform::at(x, y, k));
                }
            };
            if now.len() == 1 {
                let (dx, dy) = (now[0][0] - s_pts[0][0], now[0][1] - s_pts[0][1]);
                if g.mode == Mode::Still && dx.hypot(dy) < TOUCH_SLOP {
                    return;
                }
                g.mode = Mode::Pan;
                g.dragged = true;
                drop(g);
                apply(s_t.x + dx, s_t.y + dy, s_t.k);
                return;
            }
            if s_pts.len() < 2 {
                return;
            }
            let (a0, b0) = (s_pts[0], s_pts[1]);
            let (a, b) = (now[0], now[1]);
            if g.mode == Mode::Still {
                let da = [a[0] - a0[0], a[1] - a0[1]];
                let db = [b[0] - b0[0], b[1] - b0[1]];
                let (ma, mb) = (da[0].hypot(da[1]), db[0].hypot(db[1]));
                if ma.max(mb) < TOUCH_SLOP {
                    return;
                }
                // Fingers report moves one at a time; wait for the other before telling a tilt from a pinch.
                if ma.min(mb) < TOUCH_SLOP / 2.0 && ma.max(mb) < 3.0 * TOUCH_SLOP {
                    return;
                }
                g.dragged = true;
                let side_by_side = (b0[1] - a0[1]).abs() < (b0[0] - a0[0]).abs();
                let vertical = |d: Pt| d[1].abs() > 2.0 * d[0].abs();
                g.mode = if side_by_side && da[1] * db[1] > 0.0 && vertical(da) && vertical(db) {
                    Mode::Tilt
                } else {
                    Mode::Pinch
                };
            }
            if g.mode == Mode::Tilt {
                let dy = (a[1] - a0[1] + b[1] - b0[1]) / 2.0;
                drop(g);
                c.set_camera(None, Some(s_cam.tilt - dy * TOUCH_TILT));
                return;
            }
            let turned = angle_of(a, b) - angle_of(a0, b0);
            let turned = turned.sin().atan2(turned.cos());
            if g.twist.is_none() && turned.abs() > TWIST_START {
                g.twist = Some(turned);
            }
            let cam = Camera { rot: s_cam.rot + g.twist.map_or(0.0, |tw| turned - tw), tilt: s_cam.tilt };
            drop(g);
            c.cam.set(cam);
            // A small window fits the galaxy below the zoom-out limit; don't snap to the limit on pinch.
            let least = SCALE_EXTENT.0.min(s_t.k);
            let k = SCALE_EXTENT.1.min(least.max((s_t.k * distance(a, b)) / distance(a0, b0)));
            // What was under the fingers' midpoint stays under it.
            let world = from_layer(s_t.invert(midpoint(a0, b0)), s_cam);
            let [x, y] = to_layer(world, cam);
            let m = midpoint(a, b);
            apply(m[0] - k * x, m[1] - k * y, k);
        })
    };
    let up: Rc<dyn Fn(web_sys::Event)> = {
        let (g, restart) = (g.clone(), restart.clone());
        Rc::new(move |e: web_sys::Event| {
            let e: &web_sys::PointerEvent = e.unchecked_ref();
            let mut g = g.borrow_mut();
            let before = g.fingers.len();
            g.fingers.retain(|(i, _)| *i != e.pointer_id());
            if g.fingers.len() != before {
                restart(&mut g);
            }
        })
    };
    let on_up = {
        let up = up.clone();
        Closure::<dyn Fn(web_sys::Event)>::new(move |e| up(e))
    };
    let on_cancel = Closure::<dyn Fn(web_sys::Event)>::new(move |e| up(e));
    // A drag that ends over a planet shouldn't open it.
    let on_click = {
        let g = g.clone();
        Closure::<dyn Fn(web_sys::Event)>::new(move |e: web_sys::Event| {
            let mut g = g.borrow_mut();
            if !g.dragged {
                return;
            }
            g.dragged = false;
            e.stop_propagation();
        })
    };
    let w: web_sys::EventTarget = js::window().into();
    let s: web_sys::EventTarget = stage.clone().into();
    ctl.listen(&s, "pointerdown", false, on_down);
    ctl.listen(&w, "pointermove", false, on_move);
    ctl.listen(&w, "pointerup", false, on_up);
    ctl.listen(&w, "pointercancel", false, on_cancel);
    ctl.listen(&s, "click", true, on_click);
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
            pawns: effects.iter().filter(|e| has_pawn(e)).cloned().collect(),
            dss,
            seed: p.index,
            pois,
        },
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
    let pinned = Memo::new(move |_| s.get(|s| s.pinned.iter().copied().collect::<HashSet<i64>>()));
    let mo = major_order_planets();
    let active = Memo::new(move |_| {
        q.campaigns.with(|c| {
            c.as_ref().map(|c| c.iter().map(|c| c.planet.index).collect::<HashSet<i64>>()).unwrap_or_default()
        })
    });
    let dss = Memo::new(move |_| {
        let now = js::now();
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

    // The zoom, resizes and touch, once the stage is in place.
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
            let size: Rc<Cell<Option<(f64, f64)>>> = Rc::new(Cell::new(None));
            let weak = Rc::downgrade(&*ctl);
            let on_resize = Closure::<dyn Fn()>::new(move || {
                let Some(c) = weak.upgrade() else { return };
                let Some((w, h)) = c.rect() else { return };
                let before = size.replace(Some((w, h)));
                let Some((bw, bh)) = before else { return c.fit() };
                if w == 0.0 || h == 0.0 {
                    return;
                }
                let t = c.t.get();
                if let Some(z) = c.zoom() {
                    z.set(Transform::at(t.x + (w - bw) / 2.0, t.y + (h - bh) / 2.0, t.k));
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
            if !ready.get() || seq == 0 {
                return;
            }
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
        active.with(|active| {
            planets.with(|p| {
                let lookup = by_index(p);
                p.iter()
                    .flat_map(|p| {
                        p.attacking.iter().filter(|t| active.contains(t)).filter_map(|t| {
                            let target = lookup.get(t)?;
                            let [x1, y1] = project(p.position);
                            let [x2, y2] = project(target.position);
                            Some((x1, y1, x2, y2, faction_color(Some(p.current_owner))))
                        })
                    })
                    .collect::<Vec<_>>()
            })
        })
    });
    let indices = Memo::new(move |_| planets.with(|p| p.iter().map(|p| p.index).collect::<Vec<_>>()));
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
        <div class=move || format!("map-wrap q-{}", quality.get().as_str())>
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
                                            {move || {
                                                regions
                                                    .get()
                                                    .into_iter()
                                                    .map(|r| {
                                                        let c = faction_color(Some(r.owner));
                                                        view! {
                                                            <g>
                                                                <path d=r.path.clone() fill=c fill-opacity="0.05" stroke="none"></path>
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
                            {move || {
                                attacks
                                    .get()
                                    .into_iter()
                                    .map(|(x1, y1, x2, y2, c)| {
                                        view! { <line x1=num(x1) y1=num(y1) x2=num(x2) y2=num(y2) stroke=c></line> }
                                    })
                                    .collect_view()
                            }}
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
