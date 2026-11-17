//! The front line charts shared by planets and cities: liberation with its projection over the next day, and
//! Helldivers deployed on the same time axis.

use crate::js::{self, fmt_date_time};
use crate::ui::chart::{
    Chart, GRID, Grid, HOUR, Overlay, Plot, RefLine, Series, TipRow, Tooltip, XAxis, YAxis, default_tooltip,
    hour_ticks, series_rows,
};
use leptos::prelude::*;
use std::sync::Arc;
use warroom_core::format::{fmt_compact, fmt_int, fmt_pct};

/// One row of the front line charts: a past sample, the projection ahead, or the sample where they meet.
#[derive(Clone, Copy, Default, PartialEq)]
pub struct FrontRow {
    pub at: f64,
    pub players: Option<f64>,
    pub percent: Option<f64>,
    pub projection: Option<f64>,
    /// A defense's enemy advance, up to the last sample and then ahead of it.
    pub enemy: Option<f64>,
    pub enemy_ahead: Option<f64>,
}

/// A defense's enemy advance at `t`, 0 to 100% across its `span`.
pub fn advance_at(span: Option<(f64, f64)>, t: f64) -> Option<f64> {
    span.filter(|(s, e)| t >= *s && t <= *e).map(|(s, e)| (t - s) / (e - s) * 100.0)
}

/// Adds the next 24 hours to `rows`, an hour apart: liberation from the last reading (or `percent` at `now`) at
/// `rate` %/h, and the enemy's advance across `span`.
pub fn add_projection(rows: &mut Vec<FrontRow>, percent: f64, rate: Option<f64>, span: Option<(f64, f64)>, now: f64) {
    let last = rows.last().copied();
    if let Some(l) = rows.last_mut() {
        l.projection = l.percent;
        l.enemy_ahead = l.enemy;
    }
    if rate.is_none() && span.is_none() {
        return;
    }
    let base = last.and_then(|l| l.percent).unwrap_or(percent);
    let t0 = last.map_or(now, |l| l.at);
    let mut ahead: Vec<f64> = (1..=24).map(|h| t0 + h as f64 * HOUR).collect();
    // A row at the deadline, so the enemy's line ends on 100% there.
    if let Some((_, end)) = span.filter(|(_, e)| *e > t0 && *e < t0 + 24.0 * HOUR) {
        ahead.push(end);
        ahead.sort_by(f64::total_cmp);
        ahead.dedup();
    }
    for at in ahead {
        rows.push(FrontRow {
            at,
            projection: rate.map(|r| (base + r * (at - t0) / HOUR).clamp(0.0, 100.0)),
            enemy_ahead: advance_at(span, at),
            ..Default::default()
        });
    }
}

/// The rows' span, ticked every four hours.
pub fn day_axis(rows: &[FrontRow], hide: bool) -> XAxis {
    let now = js::now();
    let domain = (rows.first().map_or(now, |r| r.at), rows.last().map_or(now, |r| r.at));
    XAxis { hide, ..XAxis::time(domain, hour_ticks(domain.0, domain.1, 4.0)) }
}

fn grid() -> Option<Grid> {
    Some(Grid { stroke: GRID.into(), opacity: None, edges_only: false })
}

/// Liberation and its dashed projection, 0 to 100%, followed by any `extra` series.
pub fn progress_plot(rows: &[FrontRow], x: XAxis, extra: Vec<Series>, ref_lines: Vec<RefLine>) -> Plot {
    let mut series = vec![
        Series::line("percent", rows.iter().map(|r| r.percent).collect(), "var(--c-humans)", 2.0),
        Series {
            dash: Some("4 4".into()),
            ..Series::line("projection", rows.iter().map(|r| r.projection).collect(), "var(--c-humans)", 2.0)
        },
    ];
    series.extend(extra);
    Plot {
        xs: rows.iter().map(|r| r.at).collect(),
        x,
        y: vec![YAxis::new((0.0, 100.0), vec![0.0, 25.0, 50.0, 75.0, 100.0], Arc::new(|v| format!("{}%", js::num(v))))],
        grid: grid(),
        series,
        ref_lines,
    }
}

pub fn players_plot(rows: &[FrontRow], x: XAxis, ref_lines: Vec<RefLine>) -> Plot {
    let values: Vec<Option<f64>> = rows.iter().map(|r| r.players).collect();
    let max = values.iter().flatten().copied().fold(0.0, f64::max);
    Plot {
        xs: rows.iter().map(|r| r.at).collect(),
        x,
        y: vec![YAxis::auto(max, Arc::new(fmt_compact))],
        grid: grid(),
        series: vec![Series::area("players", values, "var(--cta)", 2.0, "url(#pd-players)")],
        ref_lines,
    }
}

/// The progress chart's tooltip rows: progress, projection and the enemy's advance.
pub fn progress_rows(plot: &Plot, i: usize) -> Vec<TipRow> {
    let mut rows = series_rows(plot, i, |s, v| match s.name.as_str() {
        "percent" => (fmt_pct(v, 2), "Progress".into()),
        "projection" => (fmt_pct(v, 2), "Projection".into()),
        _ => (fmt_pct(v, 1), "Enemy advance".into()),
    });
    // Both enemy series hold the sample where the solid line turns dashed.
    rows.dedup_by(|a, b| a.name == b.name);
    rows
}

/// The Progress chart over the Helldivers deployed chart, moving together under the pointer.
#[component]
pub fn FrontCharts(
    #[prop(into)] progress: Signal<Arc<Plot>>,
    #[prop(into)] players: Signal<Arc<Plot>>,
    #[prop(optional)] progress_tip: Option<Tooltip>,
    #[prop(optional)] overlay: Option<Overlay>,
) -> impl IntoView {
    let hover = RwSignal::new(None::<usize>);
    let progress_tip = progress_tip.unwrap_or_else(|| {
        Arc::new(|plot, i| Some(default_tooltip(fmt_date_time(*plot.xs.get(i)?), progress_rows(plot, i))))
    });
    let overlay = overlay.unwrap_or_else(|| Arc::new(|_| ().into_any()));
    let players_tip: Tooltip = Arc::new(|plot, i| {
        let rows = series_rows(plot, i, |_, v| (fmt_int(v), "Helldivers".into()));
        Some(default_tooltip(fmt_date_time(*plot.xs.get(i)?), rows))
    });
    view! {
        <div class="pd-charts">
            <div class="pd-chart-label caps">"Progress"</div>
            <Chart
                plot=progress
                height=170.0
                margin=(8.0, 8.0, 0.0, 0.0)
                hover=hover
                tooltip=progress_tip
                overlay=overlay
            />
            <div class="pd-chart-label caps">"Helldivers deployed"</div>
            <Chart
                plot=players
                height=120.0
                margin=(4.0, 8.0, 0.0, 0.0)
                hover=hover
                tooltip=players_tip
                defs=ViewFn::from(|| view! {
                    <linearGradient id="pd-players" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stop-color="var(--cta)" stop-opacity="0.28" />
                        <stop offset="100%" stop-color="var(--cta)" stop-opacity="0.02" />
                    </linearGradient>
                })
            />
        </div>
    }
}
