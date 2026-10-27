use crate::api::queries::queries;
use crate::api::stations::StationJump;
use crate::js::fmt_date_time;
use crate::state::ui;
use crate::ui::emblems::FactionEmblem;
use leptos::prelude::*;
use std::sync::Arc;
use warroom_core::factions::faction_color;
use warroom_core::format::fmt_ago;
use warroom_core::war_history::fmt_long_duration;

fn stop(index: Option<i64>) -> AnyView {
    let planet = index.and_then(|i| {
        queries().fresh_planets.with(|p| {
            let p = p.as_ref()?.iter().find(|x| x.index == i)?;
            Some((p.index, p.name.clone(), p.sector.clone(), p.event.as_ref().map_or(p.current_owner, |e| e.faction)))
        })
    });
    match planet {
        None => view! {
            <span class="st-hop-planet dim caps">{index.map_or("Unknown".to_string(), |i| format!("Planet {i}"))}</span>
        }
        .into_any(),
        Some((index, name, sector, owner)) => view! {
            <button
                class="st-hop-planet caps"
                style=format!("--tone: {};", faction_color(Some(owner)))
                on:click=move |_| ui().open_detail(Some(index))
            >
                <FactionEmblem faction=owner.as_str() size=16.0 />
                <b>{name}</b>
                <span>{sector}</span>
            </button>
        }
        .into_any(),
    }
}

/// A station's FTL jumps as a route, newest first; past `limit` the rest wait behind a button.
#[component]
pub fn JumpRoute(
    #[prop(into)] jumps: Signal<Arc<Vec<StationJump>>>,
    #[prop(into)] now: Signal<f64>,
    #[prop(default = 8)] limit: usize,
) -> impl IntoView {
    let all = RwSignal::new(false);
    let hops = move || {
        let jumps = jumps.get();
        let now = now.get();
        let shown = if all.get() { jumps.len() } else { limit };
        jumps
            .iter()
            .take(shown)
            .enumerate()
            .map(|(i, j)| {
                // Newest first, so the stay at a destination ends with the jump before it in the list.
                let left = if i == 0 { now } else { jumps[i - 1].at };
                view! {
                    <li class=if i == 0 { "now" } else { "" }>
                        <div class="st-hop-when">
                            <span class="num">{fmt_date_time(j.at)}</span>
                            <span class="caps">{fmt_ago(j.at, now)}</span>
                        </div>
                        <i class="st-hop-node" aria-hidden="true"></i>
                        <div class="st-hop-route">
                            {stop(j.from)}
                            <span class="st-hop-arrow" aria-label="to"></span>
                            {stop(Some(j.to))}
                        </div>
                        <span class="st-hop-stay caps num">
                            {if i == 0 { "In orbit · " } else { "Stayed " }}
                            {fmt_long_duration(((left - j.at) / 1000.0).max(0.0))}
                        </span>
                    </li>
                }
            })
            .collect_view()
    };
    let more = move || {
        let n = jumps.with(|j| j.len());
        (n > limit).then(|| {
            view! {
                <button class="gww-chip gww-more" on:click=move |_| all.update(|a| *a = !*a)>
                    {move || if all.get() { "Show fewer".to_string() } else { format!("Show all {n} jumps") }}
                </button>
            }
        })
    };
    view! {
        <ol class="st-hops">{hops}</ol>
        {more}
    }
}
