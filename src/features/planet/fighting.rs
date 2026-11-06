//! The last day of fighting on a planet or across the galaxy, beside the game's all-time totals.

use crate::api::cache::use_query;
use crate::api::combat::combat_readings;
use crate::js;
use crate::ui::emblems::FactionEmblem;
use leptos::prelude::*;
use warroom_core::api::Faction;
use warroom_core::factions::faction_color;
use warroom_core::fighting::last_day;
use warroom_core::format::{fmt_compact, fmt_int, fmt_pct};

/// Re-read every half hour; the collector keeps a reading an hour.
const REFRESH_MS: f64 = 30.0 * 60_000.0;
const HOUR_MS: f64 = 3_600_000.0;

#[component]
pub fn CombatTrendLine(#[prop(into)] planet: Signal<i64>) -> impl IntoView {
    let tick = js::use_now(REFRESH_MS);
    // Keyed by the half hour, so opening the page again reuses the last reading.
    let readings = use_query(
        move || Some(format!("combat:{}:{}", planet.get(), (tick.get() / REFRESH_MS).floor())),
        REFRESH_MS,
        |key: String| async move {
            let mut parts = key.split(':').skip(1);
            let planet: i64 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
            let slot: f64 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0.0);
            combat_readings(planet, slot * REFRESH_MS - 26.0 * HOUR_MS).await.map(|r| r.map(|l| last_day(&l)))
        },
    );
    let trend = Signal::derive(move || readings.data.get().flatten());
    move || {
        let t = trend.get()?;
        Some(match t {
            None => view! { <p class="ct-line caps">"Last 24 hours: measuring; the counters need half a day of readings."</p> }.into_any(),
            Some(t) => {
                let kills = Faction::ENEMIES.iter().zip(t.kills).filter(|(_, k)| *k > 0.0).map(|(f, k)| {
                    view! {
                        <span class="ct-kill" style=format!("--k: {};", faction_color(Some(*f)))>
                            <FactionEmblem faction=f.as_str() size=13.0 />
                            {format!("+{}", fmt_compact(k))}
                        </span>
                    }
                });
                view! {
                    <div class="ct-line">
                        <span class="caps ct-head">{format!("Last {} h", t.hours.round())}</span>
                        <span><b class="num good">{format!("+{}", fmt_int(t.won))}</b>" won"</span>
                        <span><b class="num bad">{format!("+{}", fmt_int(t.lost))}</b>" lost"</span>
                        {t.success_rate().map(|r| view! { <span><b class="num">{fmt_pct(r, 1)}</b>" success"</span> })}
                        {kills.collect_view()}
                        <span><b class="num">{format!("+{}", fmt_compact(t.deaths))}</b>" KIA"</span>
                    </div>
                }
                .into_any()
            }
        })
    }
}
