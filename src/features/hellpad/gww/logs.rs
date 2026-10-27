//! The Federation's logs: planets that changed hands, each day's peak and low, and the space stations' jumps.

use crate::api::history::history;
use crate::api::queries::queries;
use crate::api::stations::{JUMP_LIMIT, use_station_jumps};
use crate::api::war_samples::use_war_days;
use crate::features::hellpad::gww::blocks::{HeadingRef, banner, crumb, related};
use crate::features::hellpad::stations::jumps::JumpRoute;
use crate::js::{self, fmt_date, fmt_date_time, num};
use crate::state::{OptionsAnchor, ui};
use crate::ui::emblems::{DssIcon, FactionEmblem};
use crate::ui::icons::{self, Icon};
use crate::ui::primitives::{Empty, Loading};
use leptos::prelude::*;
use leptos::task::spawn_local;
use warroom_core::api::Faction;
use warroom_core::factions::{faction_color, faction_label};
use warroom_core::format::{fmt_int, sest_clock, sest_date};
use warroom_core::gww_nav::LOG_TITLE;

fn log_box(title: String, note: AnyView, icon: AnyView, body: AnyView) -> AnyView {
    view! {
        <details class="gww-fold gww-logbox" open>
            <summary>
                <span class="gww-logbox-icon" aria-hidden="true">{icon}</span>
                <b>{title}</b>
                <span class="caps num">{note}</span>
            </summary>
            <div class="gww-logbox-body">{body}</div>
        </details>
    }
    .into_any()
}

#[derive(Clone, Copy, PartialEq)]
enum LogFilter {
    All,
    Won,
    Lost,
}

const LOG_DAYS: usize = 7;

fn war_log() -> AnyView {
    let h = history();
    let show = RwSignal::new(LogFilter::All);
    let all_days = RwSignal::new(false);
    let changes = h.changes;
    let ready = h.ready;
    let older = h.older;
    let fetching = RwSignal::new(false);
    let days = Memo::new(move |_| {
        let s = show.get();
        let mut days: Vec<(String, Vec<_>)> = Vec::new();
        changes.with(|c| {
            for c in
                c.iter().filter(|c| s == LogFilter::All || (s == LogFilter::Won) == (c.change.to == Faction::Humans))
            {
                let day = fmt_date(c.change.at);
                match days.last_mut() {
                    Some((d, items)) if *d == day => items.push(c.clone()),
                    _ => days.push((day, vec![c.clone()])),
                }
            }
        });
        days
    });
    let body = move || {
        if !ready.get() {
            return view! { <Loading what="the war archive" rows=3 /> }.into_any();
        }
        let list = days.get();
        if list.is_empty() {
            let span = if older.get() {
                format!("in the latest {} changes", changes.with(Vec::len))
            } else {
                "since records began".into()
            };
            let text = match show.get() {
                LogFilter::All => format!("No planet has changed hands {span}."),
                LogFilter::Won => format!("No planet has been liberated {span}."),
                LogFilter::Lost => format!("No planet has been lost {span}."),
            };
            return view! { <Empty>{text}</Empty> }.into_any();
        }
        let shown = if all_days.get() { list.len() } else { LOG_DAYS };
        list.into_iter()
            .take(shown)
            .map(|(day, items)| view! {
                <section class="gww-log-day">
                    <h3 class="caps num">{day}</h3>
                    <ol class="wl">
                        {items
                            .into_iter()
                            .map(|c| {
                                let ch = c.change;
                                let won = ch.to == Faction::Humans;
                                let color = faction_color(Some(if won { Faction::Humans } else { ch.to }));
                                let planet = ch.planet;
                                view! {
                                    <li class="wl-row">
                                        <span class="wl-mark" style=format!("--k: {color};")>
                                            <FactionEmblem faction=if won { "Humans" } else { ch.to.as_str() } size=14.0 />
                                        </span>
                                        <div>
                                            <div class="wl-title caps">
                                                <button
                                                    class="link"
                                                    style="color: var(--text-hi);"
                                                    on:click=move |e| ui().open_options(Some(OptionsAnchor::at(planet, &e)))
                                                >
                                                    {ch.planet_name.clone()}
                                                </button>
                                                " "
                                                <span style=format!("color: {color};")>
                                                    {if won { format!("liberated from {}", faction_label(Some(ch.from))) } else { format!("fell to {}", faction_label(Some(ch.to))) }}
                                                </span>
                                            </div>
                                            <div class="wl-sub">{fmt_date_time(ch.at)}</div>
                                        </div>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ol>
                </section>
            })
            .collect_view()
            .into_any()
    };
    let more =
        move || {
            let n = days.with(Vec::len);
            (ready.get() && n > LOG_DAYS).then(|| view! {
            <button class="gww-chip gww-more" on:click=move |_| all_days.update(|a| *a = !*a)>
                {move || if all_days.get() { "Show fewer days".to_string() } else { format!("Show all {n} days") }}
            </button>
        })
        };
    let load_older = move || {
        let every_day = all_days.get() || days.with(Vec::len) <= LOG_DAYS;
        (ready.get() && older.get() && every_day).then(|| {
            view! {
                <button
                    class="gww-chip gww-more"
                    disabled=move || fetching.get()
                    on:click=move |_| {
                        fetching.set(true);
                        spawn_local(async move {
                            let _ = h.load_older().await;
                            fetching.set(false);
                        });
                    }
                >
                    "Show older changes"
                </button>
            }
        })
    };
    view! {
        <div class="gww-filter" role="group" aria-label="Show">
            {[(LogFilter::All, "Everything"), (LogFilter::Won, "Liberated"), (LogFilter::Lost, "Lost")]
                .map(|(k, label)| view! {
                    <button
                        class=move || format!("gww-chip {}", if show.get() == k { "on" } else { "" })
                        aria-pressed=move || (show.get() == k).to_string()
                        on:click=move |_| show.set(k)
                    >
                        {label}
                    </button>
                })
                .collect_view()}
        </div>
        {body}
        {more}
        {load_older}
    }
    .into_any()
}

const ACTIVITY_DAYS: usize = 14;

fn day_stat(peak: bool, value: f64, at: String, top: f64) -> AnyView {
    view! {
        <span class=format!("gww-day-stat {}", if peak { "peak" } else { "low" })>
            <span class="gww-day-num num">
                <b>{fmt_int(value)}</b>
                <span class="caps">
                    <span class="sr-only">{if peak { "peak" } else { "lowest" }}" "</span>
                    "at "
                    {at}
                </span>
            </span>
            <span class="gww-day-bar" aria-hidden="true">
                <i style=format!("width: {}%;", num(value / top * 100.0))></i>
            </span>
        </span>
    }
    .into_any()
}

fn activity_days(now: Memo<f64>) -> AnyView {
    let q = use_war_days();
    let all = RwSignal::new(false);
    let hm = |ms: f64| sest_clock(ms).chars().take(5).collect::<String>();
    (move || {
        if q.loading.get() && q.data.with(Option::is_none) {
            return view! { <Loading what="the activity log" rows=2 /> }.into_any();
        }
        let days = q.data.get().unwrap_or_default();
        if days.is_empty() {
            return view! { <Empty>"No days logged yet. Logging starts with the next telemetry update."</Empty> }
                .into_any();
        }
        let shown: Vec<_> = days.iter().take(if all.get() { days.len() } else { ACTIVITY_DAYS }).copied().collect();
        let top = shown.iter().fold(1.0, |m: f64, d| m.max(d.peak));
        let today = (now.get() / 86_400_000.0).floor() * 86_400_000.0;
        let n = days.len();
        view! {
            <div class="gww-days-head caps" aria-hidden="true">
                <span>"SEST day"</span>
                <span>"Peak online"</span>
                <span>"Lowest online"</span>
            </div>
            <ol class="gww-days">
                {shown
                    .into_iter()
                    .map(|d| view! {
                        <li class=(d.day == today).then_some("today")>
                            <span class="gww-day-date num">
                                {sest_date(d.day)}
                                {(d.day == today).then(|| view! { <span class="caps">"Today, so far"</span> })}
                            </span>
                            {day_stat(true, d.peak, hm(d.peak_at), top)}
                            {day_stat(false, d.low, hm(d.low_at), top)}
                        </li>
                    })
                    .collect_view()}
            </ol>
            <p class="gww-note">"Each bar is the day's count against the busiest day listed."</p>
            {(n > ACTIVITY_DAYS).then(|| view! {
                <button class="gww-chip gww-more" on:click=move |_| all.update(|a| *a = !*a)>
                    {if all.get_untracked() { "Show fewer days".to_string() } else { format!("Show all {n} days") }}
                </button>
            })}
        }
        .into_any()
    })
    .into_any()
}

#[component]
fn StationJumps(station: i64, name: String, now: Memo<f64>) -> impl IntoView {
    let q = use_station_jumps(move || station);
    let jumps = Signal::derive(move || q.data.get().unwrap_or_default());
    let note = move || match jumps.with(|j| j.len()) {
        1 => "1 jump".to_string(),
        n if n >= JUMP_LIMIT => format!("Latest {n} jumps"),
        n => format!("{n} jumps"),
    };
    let waiting = Memo::new(move |_| q.loading.get() && q.data.with(Option::is_none));
    let empty = Memo::new(move |_| jumps.with(|j| j.is_empty()));
    let body = move || {
        if waiting.get() {
            return view! { <Loading what="the jump log" rows=2 /> }.into_any();
        }
        if empty.get() {
            return view! { <Empty>"No jumps logged yet. Each jump is logged as telemetry comes in."</Empty> }
                .into_any();
        }
        view! { <JumpRoute jumps=jumps now=now /> }.into_any()
    };
    log_box(format!("{name} FTL jumps"), note.into_any(), view! { <DssIcon size=18.0 /> }.into_any(), body.into_any())
}

/// Every planet that changed hands, each day's Helldivers online and every station jump.
pub fn logs_page(heading: HeadingRef) -> AnyView {
    let h = history();
    let changes = h.changes;
    let ready = h.ready;
    let older = h.older;
    let clock = js::use_now(60_000.0);
    let now = Memo::new(move |_| clock.get());
    let stations = Memo::new(move |_| {
        queries()
            .stations
            .with(|s| s.as_ref().map(|s| s.iter().map(|s| s.id32).collect::<Vec<_>>()).unwrap_or_default())
    });
    let war_note = move || {
        if !ready.get() {
            return "Planets that changed hands".into_any();
        }
        let (lib, total) = changes.with(|c| (c.iter().filter(|c| c.change.to == Faction::Humans).count(), c.len()));
        view! {
            {older.get().then(|| format!("Latest {total} · "))}
            <span style="color: var(--f-humans);">{lib}" liberated"</span>
            " ·"
            " "
            <span style="color: var(--bad);">{total - lib}" lost"</span>
        }
        .into_any()
    };
    view! {
        <article class="gww-page" style="--site: var(--f-humans);">
            {banner(
                Some(view! { <Icon icon=icons::HISTORY size=40.0 stroke_width=1.5 /> }.into_any()),
                vec![crumb("Super Earth", Some("dir:gov"))],
                LOG_TITLE.into(),
                Some(
                    "The war log, Helldivers online day by day and the space stations' FTL jumps, since records began."
                        .into_any(),
                ),
                heading,
                false,
            )}
            {log_box(
                "War log".into(),
                war_note.into_any(),
                view! { <Icon icon=icons::GLOBE_2 size=18.0 /> }.into_any(),
                war_log(),
            )}
            {log_box(
                "Helldivers online".into(),
                "Each SEST day, midnight to midnight".into_any(),
                view! { <Icon icon=icons::USERS size=18.0 /> }.into_any(),
                activity_days(now),
            )}
            <For
                each=move || stations.get().into_iter().enumerate()
                key=|(i, id)| (*i, *id)
                children=move |(i, id)| {
                    let name = if i == 0 { "DSS".to_string() } else { format!("Station {}", i + 1) };
                    view! { <StationJumps station=id name=name now=now /> }
                }
            />
            {related(vec!["manual:liberation", "manual:dss", "app:stations", "app:past"], "Read next")}
        </article>
    }
    .into_any()
}
