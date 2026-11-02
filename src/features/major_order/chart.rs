//! A Major Order's whole run: each objective's progress from the archive's readings.

use crate::api::orders::order_progress;
use crate::js::{self, fmt_date, fmt_date_time};
use crate::ui::chart::{
    Chart, GRID, Grid, HOUR, Plot, Series, Tooltip, XAxis, YAxis, default_tooltip, hour_ticks, series_rows,
};
use crate::ui::primitives::{Empty, Loading};
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::sync::Arc;
use warroom_core::format::fmt_pct;
use warroom_core::major_order::{TaskView, task_name};
use warroom_core::snapshot::OrderSample;

/// One colour per objective: several often fight the same enemy, so the faction's colour can't tell them apart.
const COLORS: [&str; 5] = ["var(--cta)", "var(--info)", "var(--good)", "var(--warn)", "var(--bad)"];

/// A task's reading as 0-100: a count's share of its goal, a planet's 0 or 100 once taken.
fn percent(t: &TaskView, v: f64) -> f64 {
    if t.is_planet {
        if v >= 1.0 { 100.0 } else { 0.0 }
    } else if t.goal > 0.0 {
        (v / t.goal * 100.0).min(100.0)
    } else {
        0.0
    }
}

#[component]
pub fn OrderChart(id: i64, #[prop(into)] tasks: Signal<Arc<Vec<TaskView>>>) -> impl IntoView {
    let readings = RwSignal::new(None::<Result<Option<Arc<Vec<OrderSample>>>, String>>);
    let tick = js::use_now(5.0 * 60_000.0);
    Effect::new(move |_| {
        tick.track();
        spawn_local(async move {
            readings.set(Some(order_progress(id).await.map(|r| r.map(Arc::new))));
        });
    });
    let plot = Signal::derive(move || {
        let list = readings.with(|r| r.as_ref().and_then(|r| r.as_ref().ok().cloned().flatten())).unwrap_or_default();
        let tasks = tasks.get();
        let xs: Vec<f64> = list.iter().map(|s| s.at).collect();
        let domain = (xs.first().copied().unwrap_or(0.0), xs.last().copied().unwrap_or(0.0));
        let days = (domain.1 - domain.0) / (24.0 * HOUR);
        let x = if days > 2.0 {
            XAxis {
                fmt: Arc::new(fmt_date),
                ..XAxis::time(domain, hour_ticks(domain.0, domain.1, 24.0 * (days / 6.0).ceil()))
            }
        } else {
            XAxis::time(domain, hour_ticks(domain.0, domain.1, 6.0))
        };
        Arc::new(Plot {
            x,
            y: vec![YAxis::new(
                (0.0, 100.0),
                vec![0.0, 25.0, 50.0, 75.0, 100.0],
                Arc::new(|v| format!("{}%", js::num(v))),
            )],
            grid: Some(Grid { stroke: GRID.into(), opacity: None, edges_only: false }),
            series: tasks
                .iter()
                .enumerate()
                .map(|(i, t)| {
                    Series::line(
                        &task_name(t),
                        list.iter().map(|s| s.progress.get(i).map(|v| percent(t, *v))).collect(),
                        COLORS[i % COLORS.len()],
                        2.0,
                    )
                })
                .collect(),
            ref_lines: Vec::new(),
            xs,
        })
    });
    let tip: Tooltip = Arc::new(|plot, i| {
        let rows = series_rows(plot, i, |s, v| (fmt_pct(v, 1), s.name.clone()));
        Some(default_tooltip(fmt_date_time(*plot.xs.get(i)?), rows))
    });
    move || match readings.get() {
        None => view! { <Loading what="the order's progress" rows=2 /> }.into_any(),
        Some(Err(_)) => view! { <Empty>"Could not reach the war archive."</Empty> }.into_any(),
        Some(Ok(None)) => {
            view! { <Empty>"The order chart arrives with the order archive's database update."</Empty> }.into_any()
        }
        Some(Ok(Some(list))) if list.len() < 2 => {
            view! { <Empty>"Not enough readings of this order yet."</Empty> }.into_any()
        }
        Some(Ok(Some(_))) => view! {
            <Chart plot=plot height=160.0 margin=(8.0, 8.0, 0.0, 0.0) tooltip=tip.clone() />
            <ul class="mo-chart-key">
                {tasks
                    .get()
                    .iter()
                    .enumerate()
                    .map(|(i, t)| view! { <li style=format!("--k: {};", COLORS[i % COLORS.len()])>{task_name(t)}</li> })
                    .collect_view()}
            </ul>
        }
        .into_any(),
    }
}
