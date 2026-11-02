//! Galactic control over time: the planets each side held day by day, and this week's gains and losses per front.

use crate::api::history::all_changes;
use crate::api::queries::queries;
use crate::js::{self, fmt_date};
use crate::ui::chart::{Chart, GRID, Grid, Plot, Series, TipRow, Tooltip, XAxis, YAxis, default_tooltip, series_rows};
use crate::ui::emblems::FactionEmblem;
use crate::ui::primitives::{Empty, Loading};
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::HashMap;
use std::sync::Arc;
use warroom_core::api::Faction;
use warroom_core::control_history::{DAY_MS, control_by_day, front_tally};
use warroom_core::factions::{faction_color, faction_label};
use warroom_core::owner_changes::OwnerChange;

const RANGES: [(Option<usize>, &str); 4] =
    [(Some(30), "30 days"), (Some(90), "90 days"), (Some(365), "1 year"), (None, "Whole war")];

/// About six dates along the bottom, the last one today.
fn day_ticks(xs: &[f64]) -> Vec<f64> {
    let step = (xs.len() / 6).max(1);
    xs.iter().rev().step_by(step).rev().copied().collect()
}

#[component]
pub fn ControlHistory() -> impl IntoView {
    let q = queries();
    let changes = RwSignal::new(None::<Result<Arc<Vec<OwnerChange>>, String>>);
    spawn_local(async move {
        let got = all_changes().await.map(|l| Arc::new(l.into_iter().map(|c| c.change).collect::<Vec<_>>()));
        changes.set(Some(got));
    });
    let range = RwSignal::new(Some(90_usize));
    let now = js::now();
    let days = Memo::new(move |_| {
        let list = changes.get()?.ok()?;
        let owners: HashMap<i64, Faction> =
            q.fresh_planets.with(|p| p.iter().flat_map(|l| l.iter()).map(|p| (p.index, p.current_owner)).collect());
        let span = range.get().unwrap_or_else(|| {
            let first = list.iter().map(|c| c.at).fold(now, f64::min);
            ((now - first) / DAY_MS).ceil() as usize + 1
        });
        Some(Arc::new(control_by_day(&owners, &list, span, now)))
    });
    let plot = Signal::derive(move || {
        let days = days.get().unwrap_or_default();
        let xs: Vec<f64> = days.iter().map(|d| d.at).collect();
        // Super Earth holds most of the galaxy; on one scale its line would flatten the enemies'.
        let max = days.iter().flat_map(|d| d.held.into_iter().skip(1)).max().unwrap_or(0) as f64;
        let mut x =
            XAxis::time((xs.first().copied().unwrap_or(0.0), xs.last().copied().unwrap_or(0.0)), day_ticks(&xs));
        x.fmt = Arc::new(fmt_date);
        Arc::new(Plot {
            x,
            y: vec![YAxis::auto_whole(max, Arc::new(js::num))],
            grid: Some(Grid { stroke: GRID.into(), opacity: None, edges_only: false }),
            series: Faction::ALL
                .iter()
                .enumerate()
                .skip(1)
                .map(|(i, f)| {
                    Series::line(
                        faction_label(Some(*f)),
                        days.iter().map(|d| Some(d.held[i] as f64)).collect(),
                        faction_color(Some(*f)),
                        2.0,
                    )
                })
                .collect(),
            ref_lines: Vec::new(),
            xs,
        })
    });
    let tip: Tooltip = Arc::new(move |plot, i| {
        let held = days.with_untracked(|d| d.as_ref()?.get(i).map(|d| d.held[0]))?;
        let mut rows = vec![TipRow {
            name: faction_label(Some(Faction::Humans)).into(),
            value: js::num(held as f64),
            color: faction_color(Some(Faction::Humans)).into(),
        }];
        rows.extend(series_rows(plot, i, |s, v| (js::num(v), s.name.clone())));
        Some(default_tooltip(fmt_date(*plot.xs.get(i)?), rows))
    });
    let week =
        Memo::new(move |_| changes.with(|c| c.as_ref()?.as_ref().ok().map(|l| front_tally(l, now - 7.0 * DAY_MS))));
    view! {
        <div class="gc-history">
            <div class="gc-history-head">
                <h3 class="caps">"Planets each enemy held, day by day"</h3>
                <div class="filter-row" role="toolbar" aria-label="Range">
                    {RANGES
                        .map(|(k, label)| view! {
                            <button
                                class=move || format!("filter-chip {}", if range.get() == k { "on" } else { "" })
                                aria-pressed=move || (range.get() == k).to_string()
                                on:click=move |_| range.set(k)
                            >
                                {label}
                            </button>
                        })
                        .collect_view()}
                </div>
            </div>
            {move || match changes.get() {
                None => view! { <Loading what="the war's changes of hands" rows=2 /> }.into_any(),
                Some(Err(_)) => view! { <Empty>"Could not reach the war archive."</Empty> }.into_any(),
                Some(Ok(_)) => view! { <Chart plot=plot height=200.0 margin=(8.0, 8.0, 0.0, 0.0) tooltip=tip.clone() /> }.into_any(),
            }}
            <p class="gc-hint">
                "Worked back from today's owners through every recorded change of hands, so a planet with none counts as its owner's throughout."
            </p>
            {move || {
                week.get().map(|t| {
                    view! {
                        <ul class="gc-week">
                            <li class="caps">"Last 7 days"</li>
                            {Faction::ENEMIES
                                .iter()
                                .zip(t)
                                .map(|(f, (won, lost))| {
                                    view! {
                                        <li style=format!("--k: {};", faction_color(Some(*f)))>
                                            <FactionEmblem faction=f.as_str() size=14.0 />
                                            <span>{faction_label(Some(*f))}</span>
                                            <b class="num" style="color: var(--good);" title="Liberated">{format!("+{won}")}</b>
                                            <b class="num" style="color: var(--bad);" title="Lost">{format!("−{lost}")}</b>
                                        </li>
                                    }
                                })
                                .collect_view()}
                        </ul>
                    }
                })
            }}
        </div>
    }
}
