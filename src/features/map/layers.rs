use crate::js::{self, num};
use leptos::prelude::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use wasm_bindgen::{Clamped, JsCast};
use warroom_core::api::{Faction, Planet};
use warroom_core::factions::faction_color;
use warroom_core::galaxy::{PolarSectors, Pt, Territory, project, sector_territory};
use warroom_core::planets::ownership_key;

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
        owners.track();
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

thread_local! {
    static NOISE: RefCell<Option<web_sys::HtmlCanvasElement>> = const { RefCell::new(None) };
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

/// Tileable fractal value noise, coloured like the Gloom; made once per session.
fn gloom_noise() -> Option<web_sys::HtmlCanvasElement> {
    if let Some(n) = NOISE.with(|n| n.borrow().clone()) {
        return Some(n);
    }
    let (canvas, ctx) = canvas(NOISE_SIZE as u32, NOISE_SIZE as u32)?;
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
    let mut data = vec![0u8; field.len() * 4];
    for (i, f) in field.iter().enumerate() {
        let v = ((*f as f64 / 1.9 - 0.2) * 1.8).clamp(0.0, 1.0);
        data[i * 4] = 232;
        data[i * 4 + 1] = clamp_u8(170.0 + v * 50.0);
        data[i * 4 + 2] = clamp_u8(60.0 + v * 50.0);
        data[i * 4 + 3] = clamp_u8(warroom_core::format::js_round(v.powf(1.3) * 240.0));
    }
    let img = web_sys::ImageData::new_with_u8_clamped_array_and_sh(Clamped(&data), NOISE_SIZE as u32, NOISE_SIZE as u32).ok()?;
    ctx.put_image_data(&img, 0.0, 0.0).ok()?;
    NOISE.with(|n| *n.borrow_mut() = Some(canvas.clone()));
    Some(canvas)
}

/// Map units around a Gloom planet that its cloud covers.
const GLOOM_REACH: f64 = 78.0;

/// The tiled noise cut to soft discs around each spot, as one PNG at one pixel per map unit (the
/// noise's own resolution).
fn gloom_image(spots: &[Pt], bx: (f64, f64, f64, f64)) -> Option<String> {
    let tex = gloom_noise()?;
    let (x0, y0, w, h) = bx;
    let (canvas, ctx) = canvas(w.ceil() as u32, h.ceil() as u32)?;
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
    canvas.to_data_url().ok()
}

/// The Gloom: a drifting amber cloud over the planets the game marks as in or on the edge of it, on
/// every overlay. It is one pre-rendered image, so panning doesn't re-run a mask or filter.
#[component]
pub fn GloomLayer(#[prop(into)] planets: Signal<Arc<Vec<Planet>>>, #[prop(into)] gloom: Signal<Arc<HashSet<i64>>>) -> impl IntoView {
    // Planets don't move, so the image only needs redrawing when the set of Gloom planets changes,
    // not on every data refresh.
    let spots = Memo::new(move |_| {
        gloom.with(|g| {
            planets.with(|p| p.iter().filter(|p| g.contains(&p.index)).map(|p| (p.index, project(p.position))).collect::<Vec<_>>())
        })
    });
    let image = Memo::new(move |_| {
        let spots: Vec<Pt> = spots.with(|s| s.iter().map(|(_, p)| *p).collect());
        if spots.is_empty() {
            return None;
        }
        let xs = spots.iter().map(|p| p[0]);
        let ys = spots.iter().map(|p| p[1]);
        let (min_x, max_x) = xs.fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), x| (a.min(x), b.max(x)));
        let (min_y, max_y) = ys.fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), y| (a.min(y), b.max(y)));
        let pad = GLOOM_REACH * 1.2;
        let bx = (min_x - pad, min_y - pad, max_x - min_x + 2.0 * pad, max_y - min_y + 2.0 * pad);
        gloom_image(&spots, bx).map(|href| (bx, href))
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
