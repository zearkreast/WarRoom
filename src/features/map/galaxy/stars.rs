//! The starfield behind the map, drawn on a canvas that follows the camera.

use crate::features::map::galaxy::controls::{Camera, EXTENT};
use crate::features::map::zoom::Transform;
use crate::js;
use crate::state::MapQuality;
use std::f64::consts::PI;
use warroom_core::galaxy::{GALAXY_SCALE, RINGS, ring_radius};
use wasm_bindgen::JsCast;

pub(super) fn star_count(q: MapQuality) -> usize {
    match q {
        MapQuality::Max => 4200,
        MapQuality::Normal => 2600,
        MapQuality::Min => 1600,
    }
}

/// Highest device pixel ratio the starfield canvas is drawn at; phones report up to 3 or 4.
pub(super) fn star_dpr(q: MapQuality) -> f64 {
    match q {
        MapQuality::Max => 4.0,
        MapQuality::Normal => 2.0,
        MapQuality::Min => 1.5,
    }
}
pub(super) struct Star {
    x: f64,
    y: f64,
    s: f64,
    o: f64,
    c: &'static str,
}

/// Deterministic background stars so the field doesn't reshuffle between renders.
pub(super) fn background_stars(n: usize) -> Vec<Star> {
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

pub(super) fn draw_stars(
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
        ctx.fill_rect(-cx, -cy / squash, width, height / squash);
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
