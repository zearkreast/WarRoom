//! d3-zoom as the map uses it: the wheel zooms about the pointer, a left drag pans, and moves made in
//! code either jump or fly along d3's smooth zoom path. Touch is left to the map's own handlers.

use crate::js;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use warroom_core::galaxy::Pt;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub k: f64,
    pub x: f64,
    pub y: f64,
}

impl Transform {
    pub const IDENTITY: Transform = Transform { k: 1.0, x: 0.0, y: 0.0 };

    /// `zoomIdentity.translate(x, y).scale(k)`.
    pub fn at(x: f64, y: f64, k: f64) -> Transform {
        Transform { k, x, y }
    }

    pub fn invert(&self, p: Pt) -> Pt {
        [(p[0] - self.x) / self.k, (p[1] - self.y) / self.k]
    }

    /// Moves so that map point `p1` lands on screen point `p0`.
    fn translate(self, p0: Pt, p1: Pt) -> Transform {
        Transform { k: self.k, x: p0[0] - p1[0] * self.k, y: p0[1] - p1[1] * self.k }
    }
}

/// d3's `interpolateZoom` (van Wijk and Nuij), ρ = √2, between views `[cx, cy, width]`.
fn interpolate_zoom(p0: [f64; 3], p1: [f64; 3]) -> impl Fn(f64) -> [f64; 3] {
    const RHO: f64 = std::f64::consts::SQRT_2;
    const RHO2: f64 = 2.0;
    const RHO4: f64 = 4.0;
    let [ux0, uy0, w0] = p0;
    let [ux1, uy1, w1] = p1;
    let (dx, dy) = (ux1 - ux0, uy1 - uy0);
    let d2 = dx * dx + dy * dy;
    let general = d2 >= 1e-12;
    let (d1, r0, s) = if general {
        let d1 = d2.sqrt();
        let b0 = (w1 * w1 - w0 * w0 + RHO4 * d2) / (2.0 * w0 * RHO2 * d1);
        let b1 = (w1 * w1 - w0 * w0 - RHO4 * d2) / (2.0 * w1 * RHO2 * d1);
        let r0 = ((b0 * b0 + 1.0).sqrt() - b0).ln();
        let r1 = ((b1 * b1 + 1.0).sqrt() - b1).ln();
        (d1, r0, (r1 - r0) / RHO)
    } else {
        (0.0, 0.0, (w1 / w0).ln() / RHO)
    };
    let cosh = |x: f64| {
        let e = x.exp();
        (e + 1.0 / e) / 2.0
    };
    let sinh = |x: f64| {
        let e = x.exp();
        (e - 1.0 / e) / 2.0
    };
    let tanh = |x: f64| {
        let e = (2.0 * x).exp();
        (e - 1.0) / (e + 1.0)
    };
    move |t: f64| {
        if !general {
            return [ux0 + t * dx, uy0 + t * dy, w0 * (RHO * t * s).exp()];
        }
        let s = t * s;
        let coshr0 = cosh(r0);
        let u = w0 / (RHO2 * d1) * (coshr0 * tanh(RHO * s + r0) - sinh(r0));
        [ux0 + u * dx, uy0 + u * dy, w0 * coshr0 / cosh(RHO * s + r0)]
    }
}

fn cubic_in_out(t: f64) -> f64 {
    let t = t * 2.0;
    if t <= 1.0 {
        t * t * t / 2.0
    } else {
        let t = t - 2.0;
        (t * t * t + 2.0) / 2.0
    }
}

fn noevent(e: &web_sys::Event) {
    e.prevent_default();
    e.stop_immediate_propagation();
}

type Listener = Closure<dyn Fn(web_sys::Event)>;

struct Drag {
    moved: bool,
    origin: (i32, i32),
    listeners: Vec<(&'static str, Listener)>,
}

pub struct Zoom {
    el: web_sys::HtmlElement,
    extent: (f64, f64),
    t: Cell<Transform>,
    on_zoom: Box<dyn Fn(Transform)>,
    /// Screen point and the map point under it, held through a wheel or drag gesture.
    mouse: Cell<Option<(Pt, Pt)>>,
    /// Bumped by every wheel event; the gesture ends when the latest one has been still for 150 ms.
    wheel: Cell<(u64, bool)>,
    drag: RefCell<Option<Drag>>,
    flight: RefCell<Option<Closure<dyn Fn(f64)>>>,
    flight_frame: Cell<Option<i32>>,
    own: RefCell<Vec<(&'static str, Listener)>>,
}

impl Zoom {
    /// Starts zooming `el`; `on_zoom` hears every change of transform.
    pub fn attach(el: web_sys::HtmlElement, extent: (f64, f64), on_zoom: impl Fn(Transform) + 'static) -> Rc<Zoom> {
        let zoom = Rc::new(Zoom {
            el: el.clone(),
            extent,
            t: Cell::new(Transform::IDENTITY),
            on_zoom: Box::new(on_zoom),
            mouse: Cell::new(None),
            wheel: Cell::new((0, false)),
            drag: RefCell::new(None),
            flight: RefCell::new(None),
            flight_frame: Cell::new(None),
            own: RefCell::new(Vec::new()),
        });
        let weak = Rc::downgrade(&zoom);
        let on_wheel: Listener = Closure::new(move |e: web_sys::Event| {
            if let Some(z) = weak.upgrade() {
                Zoom::wheeled(&z, e.unchecked_ref());
            }
        });
        let opts = web_sys::AddEventListenerOptions::new();
        opts.set_passive(false);
        let _ = el.add_event_listener_with_callback_and_add_event_listener_options(
            "wheel",
            on_wheel.as_ref().unchecked_ref(),
            &opts,
        );
        let weak = Rc::downgrade(&zoom);
        let on_down: Listener = Closure::new(move |e: web_sys::Event| {
            if let Some(z) = weak.upgrade() {
                Zoom::mousedowned(&z, e.unchecked_ref());
            }
        });
        let _ = el.add_event_listener_with_callback("mousedown", on_down.as_ref().unchecked_ref());
        zoom.own.borrow_mut().extend([("wheel", on_wheel), ("mousedown", on_down)]);
        zoom
    }

    fn emit(&self, t: Transform, from_mouse: bool) {
        if !from_mouse && let Some((p0, _)) = self.mouse.get() {
            self.mouse.set(Some((p0, t.invert(p0))));
        }
        self.t.set(t);
        (self.on_zoom)(t);
    }

    fn clamp(&self, k: f64) -> f64 {
        k.min(self.extent.1).max(self.extent.0)
    }

    fn pointer(&self, e: &web_sys::MouseEvent) -> Pt {
        let r = self.el.get_bounding_client_rect();
        [
            e.client_x() as f64 - r.left() - self.el.client_left() as f64,
            e.client_y() as f64 - r.top() - self.el.client_top() as f64,
        ]
    }

    fn centroid(&self) -> (Pt, f64) {
        let (w, h) = (self.el.client_width() as f64, self.el.client_height() as f64);
        ([w / 2.0, h / 2.0], w.max(h))
    }

    /// Stops a flight under way.
    pub fn interrupt(&self) {
        if let Some(id) = self.flight_frame.take() {
            let _ = js::window().cancel_animation_frame(id);
        }
        self.flight.borrow_mut().take();
    }

    /// Jumps to `t`.
    pub fn set(&self, t: Transform) {
        self.interrupt();
        self.emit(t, false);
    }

    /// Zooms by `factor` about the middle, over `ms`.
    pub fn scale_by(self: &Rc<Self>, factor: f64, ms: f64) {
        let z = Rc::downgrade(self);
        self.fly(ms, move |a| {
            let Some(z) = z.upgrade() else { return a };
            let (p, _) = z.centroid();
            Transform { k: z.clamp(a.k * factor), ..a }.translate(p, a.invert(p))
        });
    }

    /// Flies to `t` over `ms`.
    pub fn fly_to(self: &Rc<Self>, t: Transform, ms: f64) {
        self.fly(ms, move |_| t);
    }

    fn request_flight_frame(&self) {
        let id = self
            .flight
            .borrow()
            .as_ref()
            .and_then(|f| js::window().request_animation_frame(f.as_ref().unchecked_ref()).ok());
        self.flight_frame.set(id);
    }

    /// A d3 transition on cubic-in-out easing; where it ends is worked out on its first frame, from wherever
    /// the map then is.
    fn fly(self: &Rc<Self>, ms: f64, target: impl Fn(Transform) -> Transform + 'static) {
        self.interrupt();
        let start = js::performance_now();
        let weak = Rc::downgrade(self);
        let path: RefCell<Option<Box<dyn Fn(f64) -> Transform>>> = RefCell::new(None);
        *self.flight.borrow_mut() = Some(Closure::new(move |_: f64| {
            let Some(z) = weak.upgrade() else { return };
            if path.borrow().is_none() {
                let a = z.t.get();
                let b = target(a);
                let (p, w) = z.centroid();
                let (ia, ib) = (a.invert(p), b.invert(p));
                let i = interpolate_zoom([ia[0], ia[1], w / a.k], [ib[0], ib[1], w / b.k]);
                *path.borrow_mut() = Some(Box::new(move |t: f64| {
                    if t >= 1.0 {
                        return b;
                    }
                    let [x, y, l] = i(t);
                    let k = w / l;
                    Transform { k, x: p[0] - x * k, y: p[1] - y * k }
                }));
            }
            let elapsed = js::performance_now() - start;
            let t = if elapsed < ms { cubic_in_out(elapsed / ms) } else { 1.0 };
            let next = path.borrow().as_ref().map_or(z.t.get(), |f| f(t));
            if t < 1.0 {
                z.request_flight_frame();
            } else {
                z.flight_frame.set(None);
                z.flight.borrow_mut().take();
            }
            z.emit(next, false);
        }));
        self.request_flight_frame();
    }

    fn wheeled(z: &Rc<Zoom>, e: &web_sys::WheelEvent) {
        if e.button() != 0 {
            return;
        }
        let t = z.t.get();
        let unit = match e.delta_mode() {
            1 => 0.05,
            0 => 0.002,
            _ => 1.0,
        };
        let delta = -e.delta_y() * unit * if e.ctrl_key() { 10.0 } else { 1.0 };
        let k = z.clamp(t.k * 2f64.powf(delta));
        let p = z.pointer(e);
        let (seq, wheeling) = z.wheel.get();
        if wheeling {
            if let Some((p0, _)) = z.mouse.get()
                && (p0[0] != p[0] || p0[1] != p[1])
            {
                z.mouse.set(Some((p, t.invert(p))));
            }
        } else if t.k == k {
            return;
        } else {
            z.mouse.set(Some((p, t.invert(p))));
            z.interrupt();
        }
        noevent(e);
        let seq = seq + 1;
        z.wheel.set((seq, true));
        let weak = Rc::downgrade(z);
        let idle = Closure::once_into_js(move || {
            let Some(z) = weak.upgrade() else { return };
            if z.wheel.get() == (seq, true) {
                z.wheel.set((seq, false));
                if z.drag.borrow().is_none() {
                    z.mouse.set(None);
                }
            }
        });
        let _ = js::window().set_timeout_with_callback_and_timeout_and_arguments_0(idle.unchecked_ref(), 150);
        let Some((p0, p1)) = z.mouse.get() else { return };
        z.emit(Transform { k, ..t }.translate(p0, p1), true);
    }

    fn mousedowned(z: &Rc<Zoom>, e: &web_sys::MouseEvent) {
        if e.ctrl_key() || e.button() != 0 {
            return;
        }
        let w = js::window();
        let mut listeners: Vec<(&'static str, Listener)> = Vec::new();
        let weak = Rc::downgrade(z);
        listeners.push((
            "mousemove",
            Closure::new(move |e: web_sys::Event| {
                if let Some(z) = weak.upgrade() {
                    z.mousemoved(e.unchecked_ref());
                }
            }),
        ));
        let weak = Rc::downgrade(z);
        listeners.push((
            "mouseup",
            Closure::new(move |e: web_sys::Event| {
                if let Some(z) = weak.upgrade() {
                    z.mouseupped(e.unchecked_ref());
                }
            }),
        ));
        listeners.push(("dragstart", Closure::new(|e: web_sys::Event| noevent(&e))));
        listeners.push(("selectstart", Closure::new(|e: web_sys::Event| noevent(&e))));
        for (name, f) in &listeners {
            let opts = web_sys::AddEventListenerOptions::new();
            opts.set_capture(true);
            opts.set_passive(false);
            let _ = w.add_event_listener_with_callback_and_add_event_listener_options(
                name,
                f.as_ref().unchecked_ref(),
                &opts,
            );
        }
        e.stop_immediate_propagation();
        let p = z.pointer(e);
        z.mouse.set(Some((p, z.t.get().invert(p))));
        z.interrupt();
        if let Some(old) = z.drag.replace(Some(Drag { moved: false, origin: (e.client_x(), e.client_y()), listeners }))
        {
            Zoom::unlisten(old.listeners);
        }
    }

    fn unlisten(listeners: Vec<(&'static str, Listener)>) {
        let w = js::window();
        for (name, f) in listeners {
            let _ = w.remove_event_listener_with_callback_and_bool(name, f.as_ref().unchecked_ref(), true);
        }
    }

    fn mousemoved(&self, e: &web_sys::MouseEvent) {
        noevent(e);
        if let Some(d) = self.drag.borrow_mut().as_mut()
            && !d.moved
        {
            let (dx, dy) = (e.client_x() - d.origin.0, e.client_y() - d.origin.1);
            d.moved = dx * dx + dy * dy > 0;
        }
        let Some((_, p1)) = self.mouse.get() else { return };
        let p0 = self.pointer(e);
        self.mouse.set(Some((p0, p1)));
        self.emit(self.t.get().translate(p0, p1), true);
    }

    fn mouseupped(&self, e: &web_sys::MouseEvent) {
        let Some(d) = self.drag.borrow_mut().take() else { return };
        Zoom::unlisten(d.listeners);
        if !self.wheel.get().1 {
            self.mouse.set(None);
        }
        // A drag that moved swallows the click it ends in.
        if d.moved {
            let w = js::window();
            let swallow: Listener = Closure::new(|e: web_sys::Event| noevent(&e));
            let opts = web_sys::AddEventListenerOptions::new();
            opts.set_capture(true);
            opts.set_passive(false);
            let _ = w.add_event_listener_with_callback_and_add_event_listener_options(
                "click",
                swallow.as_ref().unchecked_ref(),
                &opts,
            );
            let remove = Closure::once_into_js(move || {
                let _ = js::window().remove_event_listener_with_callback_and_bool(
                    "click",
                    swallow.as_ref().unchecked_ref(),
                    true,
                );
            });
            let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(remove.unchecked_ref(), 0);
        }
        noevent(e);
    }
}

impl Drop for Zoom {
    fn drop(&mut self) {
        self.interrupt();
        if let Some(d) = self.drag.borrow_mut().take() {
            Zoom::unlisten(d.listeners);
        }
        for (name, f) in self.own.borrow_mut().drain(..) {
            let _ = self.el.remove_event_listener_with_callback(name, f.as_ref().unchecked_ref());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eases_and_interpolates_like_d3() {
        assert_eq!(cubic_in_out(0.0), 0.0);
        assert_eq!(cubic_in_out(0.5), 0.5);
        assert_eq!(cubic_in_out(1.0), 1.0);
        assert!((cubic_in_out(0.25) - 0.0625).abs() < 1e-12);
        // d3.interpolateZoom([0, 0, 100], [100, 0, 100])(0.5)
        let i = interpolate_zoom([0.0, 0.0, 100.0], [100.0, 0.0, 100.0]);
        let [x, y, w] = i(0.5);
        assert!((x - 50.0).abs() < 1e-9 && y == 0.0);
        assert!((w - 141.42135623730948).abs() < 1e-6, "{w}");
        assert_eq!(i(1.0)[0].round(), 100.0);
    }
}
