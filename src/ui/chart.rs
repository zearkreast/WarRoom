//! Shared chart styling: hairline solid grid and axes one step off the surface, text in label ink (never
//! series colours), a crosshair tooltip on every chart.
//!
//! The SVG mirrors what Recharts 3 drew (class names, tick placement and hiding, tooltip box), so the
//! stylesheets written against it still apply.

use crate::js::{self, num};
use leptos::html;
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
pub use warroom_core::chart::Curve;
use warroom_core::chart::{area_path, line_path};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

pub const HOUR: f64 = 3_600_000.0;

pub fn fmt_hour_tick(t: f64) -> String {
    js::fmt_time(t)
}

/// Whole-hour ticks every `step_hours` across [from, to].
pub fn hour_ticks(from: f64, to: f64, step_hours: f64) -> Vec<f64> {
    let step = step_hours * HOUR;
    let mut out = Vec::new();
    let mut t = (from / step).ceil() * step;
    while t <= to {
        out.push(t);
        t += step;
    }
    out
}

pub const GRID: &str = "var(--c-grid)";
pub const LABEL: &str = "var(--label)";
const TICK_SIZE: f64 = 6.0;
const TICK_MARGIN: f64 = 2.0;
const MIN_TICK_GAP: f64 = 5.0;
const X_AXIS_HEIGHT: f64 = 30.0;

pub type Fmt = Arc<dyn Fn(f64) -> String + Send + Sync>;

#[derive(Clone)]
pub struct XAxis {
    pub domain: (f64, f64),
    pub ticks: Vec<f64>,
    pub fmt: Fmt,
    /// Hidden axes take no room, like Recharts' `hide`.
    pub hide: bool,
    pub line: bool,
    pub stroke: String,
    pub tick_fill: String,
}

impl XAxis {
    /// A time axis styled like the app's charts.
    pub fn time(domain: (f64, f64), ticks: Vec<f64>) -> Self {
        XAxis {
            domain,
            ticks,
            fmt: Arc::new(fmt_hour_tick),
            hide: false,
            line: true,
            stroke: GRID.into(),
            tick_fill: LABEL.into(),
        }
    }
}

#[derive(Clone)]
pub struct YAxis {
    pub domain: (f64, f64),
    pub ticks: Vec<f64>,
    pub fmt: Fmt,
    pub width: f64,
    pub right: bool,
    pub line: bool,
    pub stroke: String,
    pub tick_fill: String,
    pub tick_class: Option<String>,
}

impl YAxis {
    /// A left axis styled like the app's charts: ticks in label ink, a grid-coloured axis line.
    pub fn new(domain: (f64, f64), ticks: Vec<f64>, fmt: Fmt) -> Self {
        YAxis {
            domain,
            ticks,
            fmt,
            width: 44.0,
            right: false,
            line: true,
            stroke: GRID.into(),
            tick_fill: LABEL.into(),
            tick_class: None,
        }
    }

    /// Recharts' automatic axis: `count` round ticks from zero past the data's highest value.
    pub fn auto(max: f64, fmt: Fmt) -> Self {
        let ticks = warroom_core::chart::nice_ticks(0.0_f64.min(max), max.max(0.0), 5, true);
        let domain = (ticks.first().copied().unwrap_or(0.0), ticks.last().copied().unwrap_or(0.0));
        YAxis::new(domain, ticks, fmt)
    }
}

#[derive(Clone)]
pub struct Series {
    pub name: String,
    pub values: Vec<Option<f64>>,
    /// Filled down to this, per point (stacked areas); otherwise down to the axis base.
    pub base: Option<Vec<f64>>,
    /// Area fill; a plain line when None.
    pub fill: Option<String>,
    pub stroke: String,
    pub stroke_width: f64,
    pub dash: Option<String>,
    pub stroke_opacity: Option<f64>,
    pub fill_opacity: f64,
    pub class: String,
    pub axis: usize,
    pub curve: Curve,
    /// The dot that marks the hovered point.
    pub active_dot: bool,
    /// Draw across gaps instead of breaking the line.
    pub connect_nulls: bool,
}

impl Series {
    pub fn line(name: &str, values: Vec<Option<f64>>, stroke: &str, stroke_width: f64) -> Self {
        Series {
            name: name.into(),
            values,
            base: None,
            fill: None,
            stroke: stroke.into(),
            stroke_width,
            dash: None,
            stroke_opacity: None,
            fill_opacity: 0.6,
            class: String::new(),
            axis: 0,
            curve: Curve::Linear,
            active_dot: true,
            connect_nulls: false,
        }
    }

    pub fn area(name: &str, values: Vec<Option<f64>>, stroke: &str, stroke_width: f64, fill: &str) -> Self {
        Series { fill: Some(fill.into()), ..Series::line(name, values, stroke, stroke_width) }
    }
}

#[derive(Clone)]
pub struct RefLine {
    pub x: f64,
    pub stroke: String,
    pub label: Option<String>,
}

#[derive(Clone)]
pub struct Grid {
    pub stroke: String,
    pub opacity: Option<f64>,
    /// Lines only at the plot's top and bottom, as Recharts draws when no Y axis has the grid's id.
    pub edges_only: bool,
}

/// Everything one chart draws. Points are matched across series by index into `xs`.
#[derive(Clone)]
pub struct Plot {
    pub xs: Vec<f64>,
    pub x: XAxis,
    pub y: Vec<YAxis>,
    pub grid: Option<Grid>,
    pub series: Vec<Series>,
    pub ref_lines: Vec<RefLine>,
}

/// Pixel mapping of a laid-out chart, handed to overlays.
#[derive(Clone)]
pub struct Frame {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
    x_domain: (f64, f64),
    y_domains: Vec<(f64, f64)>,
}

impl Frame {
    pub fn x(&self, v: f64) -> f64 {
        let (a, b) = self.x_domain;
        if b == a { self.left + self.width / 2.0 } else { self.left + (v - a) / (b - a) * self.width }
    }

    pub fn y(&self, axis: usize, v: f64) -> f64 {
        let (a, b) = self.y_domains.get(axis).copied().unwrap_or((0.0, 1.0));
        if b == a { self.top + self.height / 2.0 } else { self.top + self.height - (v - a) / (b - a) * self.height }
    }
}

thread_local! {
    static SIZES: RefCell<HashMap<String, (f64, f64)>> = RefCell::new(HashMap::new());
}

/// A tick label's rendered size, measured the way Recharts does: in an off-screen span in the body.
fn text_size(text: &str) -> (f64, f64) {
    if let Some(s) = SIZES.with(|c| c.borrow().get(text).copied()) {
        return s;
    }
    let doc = js::document();
    let span = doc.get_element_by_id("chart-measure").unwrap_or_else(|| {
        let s = doc.create_element("span").expect("span");
        s.set_id("chart-measure");
        let _ = s.set_attribute("aria-hidden", "true");
        let _ = s.set_attribute(
            "style",
            "position: absolute; top: -20000px; left: 0; padding: 0; margin: 0; border: none; white-space: pre; font-size: 10px;",
        );
        if let Some(b) = doc.body() {
            let _ = b.append_child(&s);
        }
        s
    });
    span.set_text_content(Some(text));
    let r = span.get_bounding_client_rect();
    let size = (r.width(), r.height());
    SIZES.with(|c| c.borrow_mut().insert(text.to_string(), size));
    size
}

/// Recharts' default `preserveEnd` tick filter: walk back from the last tick, keeping each label that fits
/// clear of the one after it; the last is nudged inside the axis if it would overhang.
fn visible_ticks(coords: &[(f64, String)], start: f64, end: f64, vertical: bool) -> Vec<(f64, f64, String)> {
    let sign = if coords.len() >= 2 && coords[1].0 < coords[0].0 { -1.0 } else { 1.0 };
    let (start, mut end) = if sign > 0.0 { (start, end) } else { (end, start) };
    let size = |t: &str| {
        let (w, h) = text_size(t);
        if vertical { h } else { w }
    };
    let mut out = Vec::new();
    for (i, (c, label)) in coords.iter().enumerate().rev() {
        let s = size(label);
        let mut at = *c;
        if i == coords.len() - 1 {
            let gap = sign * (c + sign * s / 2.0 - end);
            if gap > 0.0 {
                at = c - gap * sign;
            }
        }
        let inside = !(sign * at < sign * start || sign * at > sign * end)
            && sign * (at - sign * s / 2.0 - start) >= 0.0
            && sign * (at + sign * s / 2.0 - end) <= 0.0;
        if inside {
            end = at - sign * (s / 2.0 + MIN_TICK_GAP);
            out.push((*c, at, label.clone()));
        }
    }
    out.reverse();
    out
}

/// Tick labels are kept within the whole chart (`chart_h`), not just the axis, as Recharts does.
fn y_axis_view(a: &YAxis, x: f64, f: &Frame, axis: usize, chart_h: f64) -> AnyView {
    let (edge, inner, text_x, anchor) = if a.right {
        (x, x + TICK_SIZE, x + TICK_SIZE + TICK_MARGIN, "start")
    } else {
        (x + a.width, x + a.width - TICK_SIZE, x + a.width - TICK_SIZE - TICK_MARGIN, "end")
    };
    let _ = inner;
    let coords: Vec<(f64, String)> = a.ticks.iter().map(|t| (f.y(axis, *t), (a.fmt)(*t))).collect();
    let ticks = visible_ticks(&coords, 0.0, chart_h, true);
    let class =
        format!("recharts-text recharts-cartesian-axis-tick-value {}", a.tick_class.clone().unwrap_or_default());
    let side = if a.right { "right" } else { "left" };
    view! {
        <g class="recharts-layer recharts-cartesian-axis recharts-yAxis yAxis">
            {a.line.then(|| view! {
                <line
                    orientation=side
                    width=num(a.width)
                    height=num(f.height)
                    x=num(x)
                    y=num(f.top)
                    class="recharts-cartesian-axis-line"
                    stroke=a.stroke.clone()
                    fill="none"
                    x1=num(edge)
                    y1=num(f.top)
                    x2=num(edge)
                    y2=num(f.top + f.height)
                ></line>
            })}
            <g class="recharts-cartesian-axis-ticks recharts-yAxis-ticks">
                {ticks
                    .into_iter()
                    .map(|(_, at, label)| {
                        view! {
                            <g class="recharts-layer recharts-cartesian-axis-tick">
                                <text
                                    orientation=side
                                    width=num(a.width)
                                    height=num(f.height)
                                    stroke="none"
                                    x=num(text_x)
                                    y=num(at)
                                    class=class.clone()
                                    text-anchor=anchor
                                    fill=a.tick_fill.clone()
                                    font-size="10"
                                >
                                    <tspan x=num(text_x) dy="0.355em">{label}</tspan>
                                </text>
                            </g>
                        }
                    })
                    .collect_view()}
            </g>
        </g>
    }
    .into_any()
}

fn x_axis_view(a: &XAxis, f: &Frame, bottom: f64, chart_w: f64) -> AnyView {
    let y = bottom - X_AXIS_HEIGHT;
    let text_y = y + TICK_SIZE + TICK_MARGIN;
    let coords: Vec<(f64, String)> = a.ticks.iter().map(|t| (f.x(*t), (a.fmt)(*t))).collect();
    let ticks = visible_ticks(&coords, 0.0, chart_w, false);
    view! {
        <g class="recharts-layer recharts-cartesian-axis recharts-xAxis xAxis">
            {a.line.then(|| view! {
                <line
                    orientation="bottom"
                    width=num(f.width)
                    height=num(X_AXIS_HEIGHT)
                    x=num(f.left)
                    y=num(y)
                    class="recharts-cartesian-axis-line"
                    stroke=a.stroke.clone()
                    fill="none"
                    x1=num(f.left)
                    y1=num(y)
                    x2=num(f.left + f.width)
                    y2=num(y)
                ></line>
            })}
            <g class="recharts-cartesian-axis-ticks recharts-xAxis-ticks">
                {ticks
                    .into_iter()
                    .map(|(_, at, label)| {
                        view! {
                            <g class="recharts-layer recharts-cartesian-axis-tick">
                                <text
                                    orientation="bottom"
                                    width=num(f.width)
                                    height=num(X_AXIS_HEIGHT)
                                    stroke="none"
                                    x=num(at)
                                    y=num(text_y)
                                    class="recharts-text recharts-cartesian-axis-tick-value"
                                    text-anchor="middle"
                                    fill=a.tick_fill.clone()
                                    font-size="10"
                                >
                                    <tspan x=num(at) dy="0.71em">{label}</tspan>
                                </text>
                            </g>
                        }
                    })
                    .collect_view()}
            </g>
        </g>
    }
    .into_any()
}

fn series_view(s: &Series, xs: &[f64], f: &Frame) -> AnyView {
    let mut pts: Vec<Option<(f64, f64)>> =
        xs.iter().zip(&s.values).map(|(x, v)| v.filter(|v| v.is_finite()).map(|v| (f.x(*x), f.y(s.axis, v)))).collect();
    if s.connect_nulls {
        pts.retain(Option::is_some);
    }
    let line = line_path(&pts, s.curve);
    let dash = s.dash.clone();
    let curve = view! {
        <path
            stroke=s.stroke.clone()
            stroke-width=num(s.stroke_width)
            stroke-dasharray=dash
            stroke-opacity=s.stroke_opacity.map(num)
            fill="none"
            width=num(f.width)
            height=num(f.height)
            class=if s.fill.is_some() { "recharts-curve recharts-area-curve" } else { "recharts-curve recharts-line-curve" }
            d=line
        ></path>
    };
    match &s.fill {
        None => view! { <g class=format!("recharts-layer recharts-line {}", s.class)>{curve}</g> }.into_any(),
        Some(fill) => {
            let (lo, hi) = f.y_domains.get(s.axis).copied().unwrap_or((0.0, 1.0));
            let floor = if lo >= 0.0 && hi >= 0.0 {
                lo.min(hi)
            } else if lo <= 0.0 && hi <= 0.0 {
                lo.max(hi)
            } else {
                0.0
            };
            let area: Vec<Option<(f64, f64, f64)>> = xs
                .iter()
                .enumerate()
                .zip(&s.values)
                .map(|((i, x), v)| {
                    let v = v.filter(|v| v.is_finite())?;
                    let base = s.base.as_ref().and_then(|b| b.get(i).copied()).unwrap_or(floor);
                    Some((f.x(*x), f.y(s.axis, v), f.y(s.axis, base)))
                })
                .collect();
            view! {
                <g class=format!("recharts-layer recharts-area {}", s.class)>
                    <g class="recharts-layer">
                        <path
                            fill=fill.clone()
                            fill-opacity=num(s.fill_opacity)
                            stroke="none"
                            width=num(f.width)
                            height=num(f.height)
                            class="recharts-curve recharts-area-area"
                            d=area_path(&area, s.curve)
                        ></path>
                        {curve}
                    </g>
                </g>
            }
            .into_any()
        }
    }
}

/// One row of the default tooltip.
pub struct TipRow {
    pub name: String,
    pub value: String,
    pub color: String,
}

/// Recharts' default tooltip content in the app's chart style.
pub fn default_tooltip(label: String, rows: Vec<TipRow>) -> AnyView {
    view! {
        <div
            class="recharts-default-tooltip"
            style="margin: 0px; padding: 10px; background: var(--surface-1); border: 1px solid var(--line-strong); white-space: nowrap; border-radius: 0px; font-size: 12px; box-shadow: var(--shadow-2);"
        >
            <p class="recharts-tooltip-label" style="margin: 0px 0px 4px; color: var(--text-dim);">{label}</p>
            <ul class="recharts-tooltip-item-list" style="padding: 0px; margin: 0px;">
                {rows
                    .into_iter()
                    .map(|r| {
                        view! {
                            <li
                                class="recharts-tooltip-item"
                                style=format!("display: block; padding: 0px; color: var(--text-hi); --series: {};", r.color)
                            >
                                <span class="recharts-tooltip-item-name">{r.name}</span>
                                <span class="recharts-tooltip-item-separator">" : "</span>
                                <span class="recharts-tooltip-item-value">{r.value}</span>
                                <span class="recharts-tooltip-item-unit"></span>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
        </div>
    }
    .into_any()
}

/// Tooltip rows from each series' value at a point, as Recharts lists them (gaps left out).
pub fn series_rows(plot: &Plot, i: usize, fmt: impl Fn(&Series, f64) -> (String, String)) -> Vec<TipRow> {
    plot.series
        .iter()
        .filter_map(|s| {
            let v = s.values.get(i).copied().flatten()?;
            let (value, name) = fmt(s, v);
            Some(TipRow { name, value, color: s.stroke.clone() })
        })
        .collect()
}

/// Width of an element, kept current as it resizes.
pub fn use_width(node: NodeRef<html::Div>) -> ReadSignal<f64> {
    let (width, set_width) = signal(0.0);
    let observer: SendWrapper<RefCell<Option<(web_sys::ResizeObserver, Closure<dyn Fn(js_sys::Array)>)>>> =
        SendWrapper::new(RefCell::new(None));
    let observer = Arc::new(observer);
    let o = observer.clone();
    Effect::new(move |_| {
        let Some(el) = node.get() else { return };
        set_width.set(el.get_bounding_client_rect().width());
        let el2 = el.clone();
        let cb = Closure::<dyn Fn(js_sys::Array)>::new(move |_| set_width.set(el2.get_bounding_client_rect().width()));
        if let Ok(ro) = web_sys::ResizeObserver::new(cb.as_ref().unchecked_ref()) {
            ro.observe(&el);
            if let Some((old, _)) = o.borrow_mut().replace((ro, cb)) {
                old.disconnect();
            }
        }
    });
    on_cleanup(move || {
        if let Some((ro, _)) = observer.borrow_mut().take() {
            ro.disconnect();
        }
    });
    width
}

pub type Overlay = Arc<dyn Fn(&Frame) -> AnyView + Send + Sync>;
pub type Tooltip = Arc<dyn Fn(&Plot, usize) -> Option<AnyView> + Send + Sync>;

/// A responsive cartesian chart: grid, areas, lines, reference lines, axes, then `overlay` (reference dots),
/// with a crosshair, active dots and a tooltip following the pointer. Charts sharing `hover` move together.
#[component]
pub fn Chart(
    #[prop(into)] plot: Signal<Arc<Plot>>,
    height: f64,
    /// Top, right, bottom, left.
    #[prop(default = (5.0, 5.0, 5.0, 5.0))]
    margin: (f64, f64, f64, f64),
    #[prop(optional)] hover: Option<RwSignal<Option<usize>>>,
    #[prop(optional)] tooltip: Option<Tooltip>,
    #[prop(optional)] overlay: Option<Overlay>,
    #[prop(optional)] defs: Option<ViewFn>,
    #[prop(default = "var(--text-dim)".into(), into)] cursor: String,
    #[prop(optional)] cursor_opacity: Option<f64>,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let node = NodeRef::<html::Div>::new();
    let wrap = NodeRef::<html::Div>::new();
    let tip_node = NodeRef::<html::Div>::new();
    let width = use_width(node);
    let hover = hover.unwrap_or_else(|| RwSignal::new(None));
    let pointer_y = RwSignal::new(None::<f64>);
    let tip_size = RwSignal::new((0.0, 0.0));
    let frame = Memo::new(move |_| {
        let w = width.get();
        plot.with(|p| {
            let (top, right, bottom, left) = margin;
            let lefts: f64 = p.y.iter().filter(|a| !a.right).map(|a| a.width).sum();
            let rights: f64 = p.y.iter().filter(|a| a.right).map(|a| a.width).sum();
            let x_h = if p.x.hide { 0.0 } else { X_AXIS_HEIGHT };
            let f = Frame {
                left: left + lefts,
                top,
                width: (w - left - lefts - right - rights).max(0.0),
                height: (height - top - bottom - x_h).max(0.0),
                x_domain: p.x.domain,
                y_domains: p.y.iter().map(|a| a.domain).collect(),
            };
            (f.left, f.top, f.width, f.height, f.x_domain, f.y_domains.clone())
        })
    });
    let frame_of = move || {
        let (left, top, width, height, x_domain, y_domains) = frame.get();
        Frame { left, top, width, height, x_domain, y_domains }
    };

    let on_move = move |cx: f64, cy: f64| {
        let f = frame_of();
        if cx < f.left || cx > f.left + f.width || cy < f.top || cy > f.top + f.height {
            hover.set(None);
            return;
        }
        let best = plot.with_untracked(|p| {
            p.xs.iter()
                .enumerate()
                .min_by(|a, b| (f.x(*a.1) - cx).abs().total_cmp(&(f.x(*b.1) - cx).abs()))
                .map(|(i, _)| i)
        });
        pointer_y.set(Some(cy));
        hover.set(best);
    };
    let local = move |e: &web_sys::PointerEvent| {
        let el = wrap.get_untracked()?;
        let r = el.get_bounding_client_rect();
        Some((e.client_x() as f64 - r.left(), e.client_y() as f64 - r.top()))
    };

    Effect::new(move |_| {
        hover.track();
        if let Some(el) = tip_node.get() {
            let r = el.get_bounding_client_rect();
            let size = (r.width(), r.height());
            if tip_size.get_untracked() != size {
                tip_size.set(size);
            }
        }
    });

    let svg = move || {
        let cursor = cursor.clone();
        let w = width.get();
        if w <= 0.0 {
            return None;
        }
        let f = frame_of();
        let p = plot.get();
        let (_, right, bottom, left) = margin;
        let mut left_x = left;
        let mut right_x = w - right;
        let axes: Vec<AnyView> =
            p.y.iter()
                .enumerate()
                .map(|(i, a)| {
                    let x = if a.right {
                        right_x -= a.width;
                        right_x
                    } else {
                        let x = left_x;
                        left_x += a.width;
                        x
                    };
                    y_axis_view(a, x, &f, i, height)
                })
                .collect();
        let grid = p.grid.clone().map(|g| {
            let ys: Vec<f64> = if g.edges_only {
                vec![f.top, f.top + f.height]
            } else {
                p.y.first().map(|a| a.ticks.iter().map(|t| f.y(0, *t)).collect()).unwrap_or_default()
            };
            view! {
                <g class="recharts-cartesian-grid">
                    <g class="recharts-cartesian-grid-horizontal">
                        {ys.into_iter()
                            .map(|y| {
                                view! {
                                    <line
                                        stroke=g.stroke.clone()
                                        stroke-opacity=g.opacity.map(num)
                                        fill="none"
                                        x=num(f.left)
                                        y=num(f.top)
                                        width=num(f.width)
                                        height=num(f.height)
                                        x1=num(f.left)
                                        y1=num(y)
                                        x2=num(f.left + f.width)
                                        y2=num(y)
                                    ></line>
                                }
                            })
                            .collect_view()}
                    </g>
                </g>
            }
        });
        let (areas, lines): (Vec<&Series>, Vec<&Series>) = p.series.iter().partition(|s| s.fill.is_some());
        let areas = areas.into_iter().map(|s| series_view(s, &p.xs, &f)).collect_view();
        let lines = lines.into_iter().map(|s| series_view(s, &p.xs, &f)).collect_view();
        let refs = p
            .ref_lines
            .iter()
            .filter(|r| r.x >= f.x_domain.0.min(f.x_domain.1) && r.x <= f.x_domain.0.max(f.x_domain.1))
            .map(|r| {
                let x = f.x(r.x);
                view! {
                    <g class="recharts-layer recharts-reference-line">
                        <line
                            stroke=r.stroke.clone()
                            fill="none"
                            fill-opacity="1"
                            stroke-width="1"
                            x1=num(x)
                            y1=num(f.top + f.height)
                            x2=num(x)
                            y2=num(f.top)
                            class="recharts-reference-line-line"
                        ></line>
                    </g>
                }
            })
            .collect_view();
        let labels = p
            .ref_lines
            .iter()
            .filter(|r| {
                r.label.is_some() && r.x >= f.x_domain.0.min(f.x_domain.1) && r.x <= f.x_domain.0.max(f.x_domain.1)
            })
            .map(|r| {
                let x = f.x(r.x) - 5.0;
                let y = f.top + 5.0;
                view! {
                    <text
                        offset="5"
                        x=num(x)
                        y=num(y)
                        class="recharts-text recharts-label"
                        text-anchor="end"
                        fill="var(--text-dim)"
                        font-size="10"
                    >
                        <tspan x=num(x) dy="0.71em">{r.label.clone()}</tspan>
                    </text>
                }
            })
            .collect_view();
        let x_axis = (!p.x.hide).then(|| x_axis_view(&p.x, &f, height - bottom, w));
        let over = overlay.as_ref().map(|o| o(&f));
        let defs = defs.as_ref().map(|d| d.run());
        Some(view! {
            <svg
                class="recharts-surface"
                width=num(w)
                height=num(height)
                viewBox=format!("0 0 {} {}", num(w), num(height))
                style="width: 100%; height: 100%;"
            >
                <title></title>
                <desc></desc>
                {defs.map(|d| view! { <defs>{d}</defs> })}
                {grid}
                {areas}
                {lines}
                {refs}
                {axes}
                {x_axis}
                {over}
                {move || {
                    let i = hover.get()?;
                    let p = plot.get();
                    let f = frame_of();
                    let x = f.x(*p.xs.get(i)?);
                    let dots = p
                        .series
                        .iter()
                        .filter(|s| s.active_dot)
                        .filter_map(|s| {
                            let v = s.values.get(i).copied().flatten()?;
                            Some(view! {
                                <g class="recharts-layer recharts-active-dot">
                                    <circle
                                        cx=num(x)
                                        cy=num(f.y(s.axis, v))
                                        r="4"
                                        fill=s.stroke.clone()
                                        stroke-width="2"
                                        stroke="#fff"
                                        class="recharts-dot"
                                    ></circle>
                                </g>
                            })
                        })
                        .collect_view();
                    Some(view! {
                        <path
                            stroke=cursor.clone()
                            stroke-opacity=cursor_opacity.map(num)
                            stroke-width="1"
                            fill="none"
                            pointer-events="none"
                            class="recharts-curve recharts-tooltip-cursor"
                            d=format!("M{},{}L{},{}", num(x), num(f.top), num(x), num(f.top + f.height))
                        ></path>
                        {dots}
                    })
                }}
                {labels}
            </svg>
        })
    };

    let tip = move || {
        let tooltip = tooltip.clone()?;
        let i = hover.get()?;
        let p = plot.get();
        let content = tooltip(&p, i)?;
        let f = frame_of();
        let x = f.x(*p.xs.get(i)?);
        let y = pointer_y.get().unwrap_or(f.top).clamp(f.top, f.top + f.height);
        let (tw, th) = tip_size.get();
        let place = |c: f64, size: f64, lo: f64, span: f64| {
            let positive = c + 10.0;
            if positive + size > lo + span { (c - size - 10.0).max(lo) } else { positive.max(lo) }
        };
        let style = if tw > 0.0 && th > 0.0 {
            format!(
                "pointer-events: none; visibility: visible; position: absolute; top: 0px; left: 0px; transition: transform 400ms ease; transform: translate({}px, {}px);",
                num(place(x, tw, f.left, f.width)),
                num(place(y, th, f.top, f.height))
            )
        } else {
            "pointer-events: none; visibility: hidden; position: absolute; top: 0px; left: 0px;".to_string()
        };
        Some(view! { <div class="recharts-tooltip-wrapper" style=style node_ref=tip_node>{content}</div> })
    };

    view! {
        <div
            class=format!("recharts-responsive-container {}", class.unwrap_or_default())
            style=format!("width: 100%; height: {}px; min-width: 0px;", num(height))
            node_ref=node
        >
            <div
                class="recharts-wrapper"
                style=move || format!("position: relative; cursor: default; width: 100%; height: 100%; max-height: {}px; max-width: {}px;", num(height), num(width.get()))
                node_ref=wrap
                on:pointermove=move |e| {
                    if let Some((x, y)) = local(&e) {
                        on_move(x, y)
                    }
                }
                on:pointerdown=move |e| {
                    if let Some((x, y)) = local(&e) {
                        on_move(x, y)
                    }
                }
                on:pointerleave=move |_| hover.set(None)
            >
                {svg}
                {tip}
            </div>
        </div>
    }
}
