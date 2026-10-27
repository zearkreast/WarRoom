//! Planet charts: liberation and Helldivers over time.

use crate::api::history::PlayerReading;
use crate::js::{self, fmt_date_time, fmt_time};
use crate::ui::chart::{
    Chart, GRID, Grid, Plot, RefLine, Series, Tooltip, XAxis, YAxis, default_tooltip, hour_ticks, series_rows,
};
use crate::ui::icons;
use crate::ui::primitives::{Empty, Panel};
use leptos::prelude::*;
use std::sync::Arc;
use warroom_core::format::{fmt_compact, fmt_int, fmt_pct};

const HALF_HOUR: f64 = 1_800_000.0;

/// The readings' span, or half an hour either side of a few of them, each ticked as Recharts does then.
fn time_axis(xs: &[f64]) -> XAxis {
    let domain = (xs.first().copied().unwrap_or(0.0), xs.last().copied().unwrap_or(0.0));
    if xs.len() < 4 {
        XAxis::time((domain.0 - HALF_HOUR, domain.1 + HALF_HOUR), xs.to_vec())
    } else {
        XAxis::time(domain, hour_ticks(domain.0, domain.1, 4.0))
    }
}

fn dots(xs: &[f64], color: &str) -> Option<(f64, String)> {
    (xs.len() < 4).then(|| (3.0, color.to_string()))
}

/// Helldivers over time, however few the readings: a lone one still shows as a point.
#[component]
pub fn ActivityChart(
    #[prop(into)] points: Signal<Arc<Vec<PlayerReading>>>,
    #[prop(default = 140.0)] height: f64,
) -> impl IntoView {
    let gradient = js::unique_id("activity");
    let fill = format!("url(#{gradient})");
    let plot = Signal::derive(move || {
        let points = points.get();
        let xs: Vec<f64> = points.iter().map(|r| r.at).collect();
        let max = points.iter().fold(0.0, |m: f64, r| m.max(r.players));
        Arc::new(Plot {
            x: time_axis(&xs),
            y: vec![YAxis::auto_whole(max, Arc::new(fmt_compact))],
            grid: Some(Grid { stroke: GRID.into(), opacity: None, edges_only: false }),
            series: vec![Series {
                dots: dots(&xs, "var(--cta)"),
                ..Series::area("players", points.iter().map(|r| Some(r.players)).collect(), "var(--cta)", 2.0, &fill)
            }],
            ref_lines: xs
                .last()
                .map(|x| RefLine { x: *x, stroke: "var(--text-dim)".into(), label: None })
                .into_iter()
                .collect(),
            xs,
        })
    });
    let tip: Tooltip = Arc::new(|plot, i| {
        let rows = series_rows(plot, i, |_, v| (fmt_int(v), "Helldivers".into()));
        Some(default_tooltip(fmt_date_time(*plot.xs.get(i)?), rows))
    });
    view! {
        <Chart
            plot=plot
            height=height
            margin=(8.0, 8.0, 0.0, 0.0)
            tooltip=tip
            defs=ViewFn::from(move || view! {
                <linearGradient id=gradient.clone() x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stop-color="var(--cta)" stop-opacity="0.28" />
                    <stop offset="100%" stop-color="var(--cta)" stop-opacity="0.02" />
                </linearGradient>
            })
        />
    }
}

/// Liberation over time, 0 to 100%.
#[component]
pub fn ProgressChart(
    /// (at, percent)
    #[prop(into)]
    points: Signal<Arc<Vec<(f64, f64)>>>,
    #[prop(default = 140.0)] height: f64,
) -> impl IntoView {
    let plot = Signal::derive(move || {
        let points = points.get();
        let xs: Vec<f64> = points.iter().map(|p| p.0).collect();
        Arc::new(Plot {
            x: time_axis(&xs),
            y: vec![YAxis::new(
                (0.0, 100.0),
                vec![0.0, 25.0, 50.0, 75.0, 100.0],
                Arc::new(|v| format!("{}%", js::num(v))),
            )],
            grid: Some(Grid { stroke: GRID.into(), opacity: None, edges_only: false }),
            series: vec![Series {
                dots: dots(&xs, "var(--c-humans)"),
                ..Series::line("percent", points.iter().map(|p| Some(p.1)).collect(), "var(--c-humans)", 2.0)
            }],
            ref_lines: Vec::new(),
            xs,
        })
    });
    let tip: Tooltip = Arc::new(|plot, i| {
        let rows = series_rows(plot, i, |_, v| (fmt_pct(v, 2), "Liberated".into()));
        Some(default_tooltip(fmt_date_time(*plot.xs.get(i)?), rows))
    });
    view! { <Chart plot=plot height=height margin=(8.0, 8.0, 0.0, 0.0) tooltip=tip /> }
}

fn head(label: &'static str, tone: &'static str, value: String, sub: Option<String>) -> impl IntoView {
    view! {
        <div>
            <span class="pd-key" style=format!("--k: {tone};")>
                {label}
            </span>
            <b class="num">{value}</b>
            {sub.map(|s| view! { <span class="pd-sub caps num">{s}</span> })}
        </div>
    }
}

/// Helldivers somewhere over the last 24 hours, with the busiest and quietest readings.
#[component]
pub fn ActivitySection(
    #[prop(into)] title: Signal<String>,
    #[prop(into)] now: Signal<f64>,
    #[prop(into)] points: Signal<Arc<Vec<PlayerReading>>>,
    #[prop(into)] loading: Signal<bool>,
    #[prop(into)] empty: Signal<String>,
    #[prop(optional)] children: Option<ChildrenFn>,
) -> impl IntoView {
    let heads = move || {
        let points = points.get();
        let peak = points.iter().fold(None::<&PlayerReading>, |m, r| match m {
            Some(m) if r.players <= m.players => Some(m),
            _ => Some(r),
        });
        let low = points.iter().fold(None::<&PlayerReading>, |m, r| match m {
            Some(m) if r.players >= m.players => Some(m),
            _ => Some(r),
        });
        let value = |r: Option<&PlayerReading>| r.map_or("—".to_string(), |r| fmt_int(r.players));
        view! {
            <div class="pd-heads">
                {head("Now", "var(--cta)", fmt_int(now.get()), None)}
                {head("Peak", "var(--cta)", value(peak), peak.map(|r| fmt_time(r.at)))}
                {head("Lowest", "var(--text-dim)", value(low), low.map(|r| fmt_time(r.at)))}
                {head("Readings", "var(--text-dim)", points.len().to_string(), None)}
            </div>
        }
    };
    let any = Memo::new(move |_| points.with(|p| !p.is_empty()));
    view! {
        <Panel title=move || title.get() icon=icons::USERS>
            <div class="pd-stack">
                {heads}
                <Show
                    when=move || any.get()
                    fallback=move || view! { <Empty>{move || if loading.get() { "Loading telemetry…".to_string() } else { empty.get() }}</Empty> }
                >
                    <ActivityChart points=points />
                </Show>
                {children.as_ref().map(|c| c())}
            </div>
        </Panel>
    }
}
