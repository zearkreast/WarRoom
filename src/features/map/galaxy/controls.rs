//! The map camera: rotation and tilt, zoom, the frame loop that repaints, and mouse, keyboard and touch gestures.

use crate::features::map::camera::write_camera;
use crate::features::map::galaxy::stars::{Star, draw_stars, star_dpr};
use crate::features::map::zoom::{Transform, Zoom};
use crate::js::{self, num};
use crate::state::{MapOverlay, MapQuality};
use leptos::html;
use leptos::prelude::*;
use std::cell::{Cell, RefCell};
use std::f64::consts::PI;
use std::rc::{Rc, Weak};
use warroom_core::galaxy::{GALAXY_SCALE, Pt};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

pub(super) const EXTENT: f64 = GALAXY_SCALE * 1.1;
/// Map rotation (radians, clockwise on screen) and tilt (radians, 0 = straight down).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Camera {
    pub(super) rot: f64,
    pub(super) tilt: f64,
}

const MAX_TILT: f64 = (55.0 * PI) / 180.0;
pub(super) const TILT_PRESET: f64 = (45.0 * PI) / 180.0;
pub(super) const ROT_STEP: f64 = PI / 12.0;
pub(super) const TILT_STEP: f64 = PI / 18.0;
pub(super) const DRAG_ROT: f64 = 0.006;
pub(super) const DRAG_TILT: f64 = 0.005;
const TOUCH_TILT: f64 = 0.008;
/// Finger travel (px) before a touch counts as a drag rather than a tap.
const TOUCH_SLOP: f64 = 6.0;
/// Two fingers must twist this far (radians) before the map turns, so a pinch doesn't wobble it.
const TWIST_START: f64 = 0.2;
pub(super) const SCALE_EXTENT: (f64, f64) = (0.5, 14.0);
/// How long the buttons and keys take to turn or tilt the map.
const CAMERA_MS: f64 = 320.0;
/// How long the camera must be still before the costly glow filters come back.
const SETTLE_MS: i32 = 180;
/// An angle in (-π, π].
fn wrap(a: f64) -> f64 {
    a.sin().atan2(a.cos())
}

fn turn([x, y]: Pt, a: f64) -> Pt {
    [x * a.cos() - y * a.sin(), x * a.sin() + y * a.cos()]
}

/// Map units to the zoom layer's own units: rotated, then squashed by the tilt.
pub(super) fn to_layer(p: Pt, cam: Camera) -> Pt {
    let [x, y] = turn(p, cam.rot);
    [x, y * cam.tilt.cos()]
}

pub(super) fn from_layer([x, y]: Pt, cam: Camera) -> Pt {
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
pub(super) fn scale_hatches(svg: &web_sys::Element, k: f64) {
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
/// Where everything about the camera lives between frames; none of it is reactive state.
pub(super) struct Ctl {
    pub(super) stage: NodeRef<html::Div>,
    pub(super) svg: NodeRef<leptos::svg::Svg>,
    pub(super) canvas: NodeRef<html::Canvas>,
    pub(super) layer: NodeRef<leptos::svg::G>,
    pub(super) compass: NodeRef<html::Button>,
    pub(super) tilt: NodeRef<html::Button>,
    pub(super) zoom: RefCell<Option<Rc<Zoom>>>,
    pub(super) t: Cell<Transform>,
    pub(super) cam: Cell<Camera>,
    pub(super) stars: RefCell<Rc<Vec<Star>>>,
    pub(super) overlay: Cell<MapOverlay>,
    pub(super) quality: Cell<MapQuality>,
    frame: Cell<Option<i32>>,
    paint_fn: RefCell<Option<Closure<dyn Fn(f64)>>>,
    settle: Cell<Option<i32>>,
    settle_fn: RefCell<Option<Closure<dyn Fn()>>>,
    written: RefCell<String>,
    tween: Cell<Option<i32>>,
    tween_fn: RefCell<Option<Closure<dyn Fn(f64)>>>,
    /// Where the buttons and keys are taking the camera, so a key pressed mid-turn adds to the turn.
    goal: Cell<Option<Camera>>,
    pub(super) drag: Cell<Option<(f64, f64, Camera)>>,
    pub(super) cleanup: RefCell<Vec<Box<dyn FnOnce()>>>,
}

impl Ctl {
    pub(super) fn new() -> Rc<Ctl> {
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

    pub(super) fn zoom(&self) -> Option<Rc<Zoom>> {
        self.zoom.borrow().clone()
    }

    pub(super) fn draw_stars(&self) {
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

    /// Queues a paint for the next frame.
    pub(super) fn repaint(&self) {
        if self.frame.get().is_none() {
            let id = self
                .paint_fn
                .borrow()
                .as_ref()
                .and_then(|f| js::window().request_animation_frame(f.as_ref().unchecked_ref()).ok());
            self.frame.set(id);
        }
    }

    /// Queues a paint and marks the map as moving, which drops the glow filters until it settles.
    pub(super) fn schedule(&self) {
        let Some(svg) = self.svg.get_untracked() else { return };
        let w = js::window();
        self.repaint();
        let _ = svg.class_list().add_1("moving");
        if let Some(id) = self.settle.take() {
            w.clear_timeout_with_handle(id);
        }
        let id = self.settle_fn.borrow().as_ref().and_then(|f| {
            w.set_timeout_with_callback_and_timeout_and_arguments_0(f.as_ref().unchecked_ref(), SETTLE_MS).ok()
        });
        self.settle.set(id);
    }

    pub(super) fn rect(&self) -> Option<(f64, f64)> {
        let r = self.stage.get_untracked()?.get_bounding_client_rect();
        Some((r.width(), r.height()))
    }

    pub(super) fn fit(&self) {
        let (Some(z), Some((w, h))) = (self.zoom(), self.rect()) else { return };
        let k = w.min(h) / (EXTENT * 2.0);
        z.set(Transform::at(w / 2.0, h / 2.0, k));
    }

    pub(super) fn zoom_by(&self, f: f64) {
        if let Some(z) = self.zoom() {
            z.scale_by(f, 250.0);
        }
    }

    /// Rotates and tilts about the middle of the screen, keeping whatever is there in place.
    pub(super) fn set_camera(&self, rot: Option<f64>, tilt: Option<f64>) {
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
    pub(super) fn settle_camera(&self) {
        if let Some(id) = self.tween.take() {
            let _ = js::window().cancel_animation_frame(id);
        }
        self.goal.set(None);
        if let Some(z) = self.zoom() {
            z.interrupt();
        }
    }

    pub(super) fn aim(&self) -> Camera {
        self.goal.get().unwrap_or(self.cam.get())
    }

    /// Turns and tilts to the target over CAMERA_MS, the short way round.
    pub(super) fn animate_camera(self: &Rc<Self>, rot: Option<f64>, tilt: Option<f64>) {
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

    pub(super) fn listen(
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

    pub(super) fn teardown(&self) {
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
pub(super) fn touch_gestures(ctl: &Rc<Ctl>, stage: &web_sys::HtmlElement) {
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
