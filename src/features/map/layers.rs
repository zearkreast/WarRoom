//! Map layers: sector tiles and the Gloom, shared with the Theater of War, and the TCS+ haze.

use crate::js::{self, num};
use crate::state::{MapQuality, settings};
use leptos::prelude::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;
use warroom_core::api::{Faction, Planet};
use warroom_core::effects::tcs_standing;
use warroom_core::factions::faction_color;
use warroom_core::galaxy::{PolarSectors, Pt, Territory, project, sector_territory};
use warroom_core::planets::ownership_key;
use wasm_bindgen::{Clamped, JsCast};

/// Paths per faction, in the order factions first appear.
type ByFaction = Vec<(Faction, String)>;

fn add(m: &mut ByFaction, f: Faction, d: &str) {
    match m.iter_mut().find(|(g, _)| *g == f) {
        Some((_, s)) => s.push_str(d),
        None => m.push((f, d.to_string())),
    }
}

#[derive(Clone, Default, PartialEq)]
struct Shapes {
    grid: String,
    seams: String,
    fronts: ByFaction,
    cells: ByFaction,
    splits: ByFaction,
    held: ByFaction,
    contested: ByFaction,
}

fn shapes(geo: &PolarSectors, territory: &HashMap<String, Option<Territory>>) -> Shapes {
    let mut s = Shapes::default();
    let get = |k: &str| territory.get(k).copied().flatten();
    for e in &geo.edges {
        let ta = get(&e.a);
        let tb = e.b.as_deref().and_then(get);
        match (ta, tb) {
            (None, None) => s.grid.push_str(&e.d),
            (Some(a), Some(b)) if a.faction == b.faction => {
                s.seams.push_str(&e.d);
                add(&mut s.cells, a.faction, &e.d);
                if a.held != b.held {
                    add(&mut s.splits, a.faction, &e.d);
                }
            }
            _ => {
                for t in [ta, tb].into_iter().flatten() {
                    add(&mut s.fronts, t.faction, &e.d);
                }
            }
        }
    }
    for (sector, d) in &geo.fills {
        if let Some(t) = get(sector) {
            add(if t.held { &mut s.held } else { &mut s.contested }, t.faction, d);
        }
    }
    s
}

/// Enemy territory as the game's war table draws it: only enemy sectors are painted, each a hatched
/// tile set apart by a dark seam, faint where Super Earth still holds some of its planets (contested).
/// The galaxy map and the Theater of War share it.
#[component]
pub fn SectorLayer(
    #[prop(into)] geo: Signal<Arc<PolarSectors>>,
    #[prop(into)] planets: Signal<Arc<Vec<Planet>>>,
    /// Prefix for pattern ids, unique per SVG.
    #[prop(into)]
    id: String,
    /// Hatch spacing in map units.
    hatch: f64,
) -> impl IntoView {
    let owners = Memo::new(move |_| planets.with(|p| ownership_key(p)));
    let territory = Memo::new(move |_| {
        owners.with(|_| ());
        Arc::new(planets.with_untracked(|p| sector_territory(p).into_iter().collect::<HashMap<_, _>>()))
    });
    let shapes = Memo::new(move |_| geo.with(|g| territory.with(|t| shapes(g, t))));
    let pattern_id = id.clone();
    let defs = Faction::ENEMIES
        .iter()
        .flat_map(|f| {
            let id = pattern_id.clone();
            ["held", "contested"].map(move |kind| {
                let color = faction_color(Some(*f));
                let held = kind == "held";
                view! {
                    <pattern
                        id=format!("{id}-hatch-{}-{kind}", f.as_str())
                        class="hatch"
                        width=num(hatch)
                        height=num(hatch)
                        patternUnits="userSpaceOnUse"
                        patternTransform="rotate(45)"
                    >
                        <rect width=num(hatch) height=num(hatch) fill=color fill-opacity=if held { "0.3" } else { "0.08" } />
                        <line
                            x1=num(hatch * 0.15)
                            y1="0"
                            x2=num(hatch * 0.15)
                            y2=num(hatch)
                            stroke=color
                            stroke-opacity=if held { "0.75" } else { "0.32" }
                            stroke-width=num(hatch * 0.28)
                        />
                    </pattern>
                }
            })
        })
        .collect_view();
    let fills = move |kind: &'static str, id: String| {
        move || {
            shapes.with(|s| {
                let list = if kind == "held" { &s.held } else { &s.contested };
                list.iter()
                    .map(|(f, d)| {
                        view! { <path d=d.clone() class="tf-fill" fill=format!("url(#{id}-hatch-{}-{kind})", f.as_str()) /> }
                    })
                    .collect_view()
            })
        }
    };
    let strokes = move |class: &'static str, pick: fn(&Shapes) -> &ByFaction| {
        move || {
            shapes.with(|s| {
                pick(s)
                    .iter()
                    .map(|(f, d)| view! { <path d=d.clone() class=class style=format!("color: {};", faction_color(Some(*f))) /> })
                    .collect_view()
            })
        }
    };
    view! {
        <g class="territory">
            <defs>{defs}</defs>
            <path d=move || shapes.with(|s| s.grid.clone()) class="sg-grid" />
            {fills("held", id.clone())}
            {fills("contested", id.clone())}
            <path d=move || shapes.with(|s| s.seams.clone()) class="tf-seam" />
            {strokes("tf-cell", |s| &s.cells)}
            {strokes("tf-split", |s| &s.splits)}
            {strokes("tf-front", |s| &s.fronts)}
            <circle r=move || num(geo.with(|g| g.rim)) class="sg-rim" />
        </g>
    }
}

const NOISE_SIZE: usize = 256;

/// A cloud image's box in map units: x, y, width, height.
type Bx = (f64, f64, f64, f64);

/// A Gloom image by the spots and density it was made for.
type GloomCache = Option<(Vec<Pt>, f64, Option<(Bx, String)>)>;

thread_local! {
    static FIELD: RefCell<Option<Rc<Vec<f32>>>> = const { RefCell::new(None) };
    static NOISE: RefCell<Option<web_sys::HtmlCanvasElement>> = const { RefCell::new(None) };
    /// So returning to the map doesn't encode the Gloom again.
    static GLOOM_IMAGE: RefCell<GloomCache> = const { RefCell::new(None) };
    static TCS_IMAGES: RefCell<Option<(f64, Option<[String; 3]>)>> = const { RefCell::new(None) };
}

fn canvas(w: u32, h: u32) -> Option<(web_sys::HtmlCanvasElement, web_sys::CanvasRenderingContext2d)> {
    let c: web_sys::HtmlCanvasElement = js::document().create_element("canvas").ok()?.dyn_into().ok()?;
    c.set_width(w);
    c.set_height(h);
    let ctx = c.get_context("2d").ok()??.dyn_into().ok()?;
    Some((c, ctx))
}

/// A Uint8ClampedArray store: clamped, rounded half to even.
fn clamp_u8(v: f64) -> u8 {
    if v.is_nan() { 0 } else { v.clamp(0.0, 255.0).round_ties_even() as u8 }
}

/// Tileable fractal value noise, one value per map unit, each 0 to 1; made once per session.
fn field() -> Rc<Vec<f32>> {
    if let Some(f) = FIELD.with(|f| f.borrow().clone()) {
        return f;
    }
    let mut seed: f64 = 11.0;
    let mut rnd = move || {
        seed = (seed * 16807.0) % 2147483647.0;
        seed / 2147483647.0
    };
    let mut field = vec![0f32; NOISE_SIZE * NOISE_SIZE];
    let mut amp = 1.0;
    let mut cell = 64usize;
    while cell >= 4 {
        let n = NOISE_SIZE / cell;
        let grid: Vec<f64> = (0..n * n).map(|_| rnd()).collect();
        let at = |gx: usize, gy: usize| grid[((gy + n) % n) * n + ((gx + n) % n)];
        for y in 0..NOISE_SIZE {
            for x in 0..NOISE_SIZE {
                let (gx, gy) = (x / cell, y / cell);
                let fx = (x % cell) as f64 / cell as f64;
                let fy = (y % cell) as f64 / cell as f64;
                let sx = fx * fx * (3.0 - 2.0 * fx);
                let sy = fy * fy * (3.0 - 2.0 * fy);
                let top = at(gx, gy) + (at(gx + 1, gy) - at(gx, gy)) * sx;
                let bottom = at(gx, gy + 1) + (at(gx + 1, gy + 1) - at(gx, gy + 1)) * sx;
                let i = y * NOISE_SIZE + x;
                field[i] = (field[i] as f64 + (top + (bottom - top) * sy) * amp) as f32;
            }
        }
        amp /= 2.0;
        cell /= 2;
    }
    for f in &mut field {
        *f = ((*f as f64 / 1.9 - 0.2) * 1.8).clamp(0.0, 1.0) as f32;
    }
    let field = Rc::new(field);
    FIELD.with(|f| *f.borrow_mut() = Some(field.clone()));
    field
}

/// The noise at a point in map units, interpolated and wrapping like the tile.
fn sample(field: &[f32], x: f64, y: f64) -> f64 {
    let size = NOISE_SIZE as f64;
    let (x, y) = (x.rem_euclid(size), y.rem_euclid(size));
    let (x0, y0) = (x.floor() as usize, y.floor() as usize);
    let (x1, y1) = ((x0 + 1) % NOISE_SIZE, (y0 + 1) % NOISE_SIZE);
    let (fx, fy) = (x - x.floor(), y - y.floor());
    let v = |x: usize, y: usize| field[y * NOISE_SIZE + x] as f64;
    let top = v(x0, y0) + (v(x1, y0) - v(x0, y0)) * fx;
    let bottom = v(x0, y1) + (v(x1, y1) - v(x0, y1)) * fx;
    top + (bottom - top) * fy
}

fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The noise coloured like the Gloom, as a canvas to tile; made once per session.
fn gloom_noise() -> Option<web_sys::HtmlCanvasElement> {
    if let Some(n) = NOISE.with(|n| n.borrow().clone()) {
        return Some(n);
    }
    let (canvas, ctx) = canvas(NOISE_SIZE as u32, NOISE_SIZE as u32)?;
    let field = field();
    let mut data = vec![0u8; field.len() * 4];
    for (i, f) in field.iter().enumerate() {
        let v = *f as f64;
        data[i * 4] = 232;
        data[i * 4 + 1] = clamp_u8(170.0 + v * 50.0);
        data[i * 4 + 2] = clamp_u8(60.0 + v * 50.0);
        data[i * 4 + 3] = clamp_u8(warroom_core::format::js_round(v.powf(1.3) * 240.0));
    }
    put(&ctx, &data, NOISE_SIZE as u32, NOISE_SIZE as u32)?;
    NOISE.with(|n| *n.borrow_mut() = Some(canvas.clone()));
    Some(canvas)
}

fn put(ctx: &web_sys::CanvasRenderingContext2d, data: &[u8], w: u32, h: u32) -> Option<()> {
    let img = web_sys::ImageData::new_with_u8_clamped_array_and_sh(Clamped(data), w, h).ok()?;
    ctx.put_image_data(&img, 0.0, 0.0).ok()
}

/// Pixels per map unit for a cloud image: half at Minimal, and on touch screens below Maximum, which the soft
/// clouds don't show.
fn cloud_density(full: f64) -> f64 {
    let coarse = js::window().match_media("(pointer: coarse)").ok().flatten().is_some_and(|m| m.matches());
    match settings().get(|s| s.map_quality) {
        MapQuality::Min => full / 2.0,
        MapQuality::Normal if coarse => full / 2.0,
        _ => full,
    }
}

/// Map units around a Gloom planet that its cloud covers.
const GLOOM_REACH: f64 = 78.0;

/// The tiled noise cut to soft discs around each spot, as one PNG of the box at `density` pixels per map unit;
/// the noise keeps one texel per map unit at any density.
fn gloom_image(spots: &[Pt], density: f64) -> Option<(Bx, String)> {
    if let Some(hit) = GLOOM_IMAGE.with(|c| {
        c.borrow().as_ref().filter(|(s, d, _)| s == spots && *d == density).map(|(_, _, image)| image.clone())
    }) {
        return hit;
    }
    let xs = spots.iter().map(|p| p[0]);
    let ys = spots.iter().map(|p| p[1]);
    let (min_x, max_x) = xs.fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), x| (a.min(x), b.max(x)));
    let (min_y, max_y) = ys.fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), y| (a.min(y), b.max(y)));
    let pad = GLOOM_REACH * 1.2;
    let bx = (min_x - pad, min_y - pad, max_x - min_x + 2.0 * pad, max_y - min_y + 2.0 * pad);
    let image = (|| {
        let tex = gloom_noise()?;
        let (x0, y0, w, h) = bx;
        let (canvas, ctx) = canvas((w * density).ceil() as u32, (h * density).ceil() as u32)?;
        ctx.scale(density, density).ok()?;
        ctx.translate(-x0, -y0).ok()?;
        for [x, y] in spots {
            let g = ctx.create_radial_gradient(*x, *y, 0.0, *x, *y, GLOOM_REACH).ok()?;
            g.add_color_stop(0.0, "#fff").ok()?;
            g.add_color_stop(0.55, "rgba(255,255,255,0.75)").ok()?;
            g.add_color_stop(1.0, "rgba(255,255,255,0)").ok()?;
            ctx.set_fill_style_canvas_gradient(&g);
            ctx.fill_rect(x - GLOOM_REACH, y - GLOOM_REACH, GLOOM_REACH * 2.0, GLOOM_REACH * 2.0);
        }
        ctx.set_global_composite_operation("source-in").ok()?;
        // Tiled from the map origin, so the texture doesn't shift when the box changes.
        let pattern = ctx.create_pattern_with_html_canvas_element(&tex, "repeat").ok()??;
        ctx.set_fill_style_canvas_pattern(&pattern);
        ctx.fill_rect(x0, y0, w, h);
        Some((bx, canvas.to_data_url().ok()?))
    })();
    GLOOM_IMAGE.with(|c| *c.borrow_mut() = Some((spots.to_vec(), density, image.clone())));
    image
}

/// The Gloom: an amber cloud over the planets the game marks as in or on the edge of it, on every overlay. It is
/// one pre-rendered image, so panning doesn't re-run a mask or filter.
#[component]
pub fn GloomLayer(
    #[prop(into)] planets: Signal<Arc<Vec<Planet>>>,
    #[prop(into)] gloom: Signal<Arc<HashSet<i64>>>,
) -> impl IntoView {
    // Planets don't move, so the image only needs redrawing when the set of Gloom planets changes,
    // not on every data refresh.
    let spots = Memo::new(move |_| {
        gloom.with(|g| {
            planets.with(|p| {
                p.iter().filter(|p| g.contains(&p.index)).map(|p| (p.index, project(p.position))).collect::<Vec<_>>()
            })
        })
    });
    let image = Memo::new(move |_| {
        let spots: Vec<Pt> = spots.with(|s| s.iter().map(|(_, p)| *p).collect());
        if spots.is_empty() {
            return None;
        }
        gloom_image(&spots, cloud_density(1.0))
    });
    move || {
        image.get().map(|((x, y, w, h), href)| {
            view! {
                <g class="gloom" aria-hidden="true">
                    <image href=href x=num(x) y=num(y) width=num(w.ceil()) height=num(h.ceil()) />
                </g>
            }
        })
    }
}

/// Map units around a TCS+ planet that its haze reaches; under the usual distance to a neighbour, so neighbouring
/// hazes stay apart.
const TCS_REACH: f64 = 36.0;

/// Opacity of the TCS+ haze's lower layer; keep in sync with .tcs-under in layers.css.
const TCS_UNDER_ALPHA: f64 = 0.75;

/// One layer of the haze around a TCS+ planet, centred in its image. `swirl` winds the noise into a spiral (radians
/// from centre to rim); `at` is where in the noise the layer is cut from.
fn tcs_haze(density: f64, at: Pt, swirl: f64) -> Option<web_sys::HtmlCanvasElement> {
    let size = (TCS_REACH * 2.0 * density).ceil() as u32;
    let (canvas, ctx) = canvas(size, size)?;
    let field = field();
    let mut data = vec![0u8; (size * size * 4) as usize];
    // Keep the colours in step with --tcs in theme.css.
    let (thin, dense) = ([38.0, 52.0, 214.0], [104.0, 136.0, 255.0]);
    for py in 0..size {
        for px in 0..size {
            let ux = (px as f64 + 0.5) / density - TCS_REACH;
            let uy = (py as f64 + 0.5) / density - TCS_REACH;
            let r = ux.hypot(uy) / TCS_REACH;
            if r >= 1.0 {
                continue;
            }
            let theta = uy.atan2(ux);
            // The outline: noise read around a circle, so it closes on itself.
            let lobe = sample(&field, at[0] + 60.0 + 14.0 * theta.cos(), at[1] + 60.0 + 14.0 * theta.sin());
            let edge = 0.5 + 0.5 * lobe;
            let wound = theta + swirl * r;
            let n = sample(&field, at[0] + r * TCS_REACH * wound.cos(), at[1] + r * TCS_REACH * wound.sin());
            let fade = 1.0 - smoothstep(edge * 0.2, edge, r);
            let wisp = smoothstep(0.3, 0.8, n);
            let core = 0.35 * (-(r / 0.18).powi(2)).exp();
            let alpha = (fade * (0.12 + 0.88 * wisp) + core).min(1.0);
            let t = (0.6 * wisp + 0.4 * (1.0 - r / edge).max(0.0)).clamp(0.0, 1.0);
            let i = ((py * size + px) * 4) as usize;
            for c in 0..3 {
                data[i + c] = clamp_u8(thin[c] + (dense[c] - thin[c]) * t);
            }
            data[i + 3] = clamp_u8(alpha * 255.0);
        }
    }
    put(&ctx, &data, size, size)?;
    Some(canvas)
}

/// The TCS+ haze's two layers and both of them in one image, each centred, shared by every planet.
fn tcs_images(density: f64) -> Option<[String; 3]> {
    if let Some(hit) =
        TCS_IMAGES.with(|c| c.borrow().as_ref().filter(|(d, _)| *d == density).map(|(_, images)| images.clone()))
    {
        return hit;
    }
    let images = (|| {
        let top = tcs_haze(density, [0.0, 0.0], 2.4)?;
        let under = tcs_haze(density, [131.0, 77.0], -1.7)?;
        let (both, ctx) = canvas(top.width(), top.height())?;
        ctx.set_global_alpha(TCS_UNDER_ALPHA);
        ctx.draw_image_with_html_canvas_element(&under, 0.0, 0.0).ok()?;
        ctx.set_global_alpha(1.0);
        ctx.draw_image_with_html_canvas_element(&top, 0.0, 0.0).ok()?;
        Some([top.to_data_url().ok()?, under.to_data_url().ok()?, both.to_data_url().ok()?])
    })();
    TCS_IMAGES.with(|c| *c.borrow_mut() = Some((density, images.clone())));
    images
}

/// The Termicide haze around each TCS+ planet whose towers stand. Its two layers turn opposite ways so it churns;
/// Minimal quality, and touch screens below Maximum, turn both as one image.
#[component]
pub fn TcsLayer(
    #[prop(into)] planets: Signal<Arc<Vec<Planet>>>,
    #[prop(into)] tcs: Signal<Arc<HashSet<i64>>>,
) -> impl IntoView {
    let spots = Memo::new(move |_| {
        tcs.with(|t| {
            planets.with(|p| {
                let standing = tcs_standing(p, t);
                p.iter()
                    .filter(|p| standing.contains(&p.index))
                    .map(|p| (p.index, project(p.position)))
                    .collect::<Vec<_>>()
            })
        })
    });
    // Pixels per map unit; the disc is small enough to stay sharp up close.
    let images = Memo::new(move |_| tcs_images(cloud_density(3.0)));
    let d = num(TCS_REACH * 2.0);
    let r = num(-TCS_REACH);
    move || {
        let [top, under, both] = images.get()?;
        let spots = spots.get();
        (!spots.is_empty()).then(|| {
            view! {
                <g class="tcs" aria-hidden="true">
                    <defs>
                        <image id="tcs-top" href=top x=r.clone() y=r.clone() width=d.clone() height=d.clone() />
                        <image id="tcs-under" href=under x=r.clone() y=r.clone() width=d.clone() height=d.clone() />
                        <image id="tcs-both" href=both x=r.clone() y=r.clone() width=d.clone() height=d.clone() />
                    </defs>
                    {spots
                        .into_iter()
                        .map(|(index, [x, y])| {
                            // Each planet starts at its own point of the turn, so the hazes don't move as one.
                            let delay = format!("animation-delay: -{}s;", index * 37 % 180);
                            let [swell, under, top] = [delay.clone(), delay.clone(), delay.clone()];
                            view! {
                                <g transform=format!("translate({},{})", num(x), num(y))>
                                    <g class="tcs-swell" style=swell>
                                        <g class="tcs-spin tcs-under" style=under>
                                            <use href="#tcs-under" />
                                        </g>
                                        <g class="tcs-spin tcs-top" style=top>
                                            <use href="#tcs-top" />
                                        </g>
                                        <g class="tcs-spin tcs-both" style=delay>
                                            <use href="#tcs-both" />
                                        </g>
                                    </g>
                                </g>
                            }
                        })
                        .collect_view()}
                </g>
            }
        })
    }
}
