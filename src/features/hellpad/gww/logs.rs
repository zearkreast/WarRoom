//! The Federation's logs: planets that changed hands, each day's peak and low, and the space stations' jumps.

use crate::api::history::history;
use crate::api::queries::queries;
use crate::api::stations::{JUMP_LIMIT, use_station_jumps};
use crate::api::war_samples::{WarDay, use_war_days};
use crate::features::hellpad::gww::blocks::{HeadingRef, banner, crumb, related};
use crate::features::hellpad::stations::jumps::JumpRoute;
use crate::js::{self, fmt_date, num};
use crate::state::{OptionsAnchor, ui};
use crate::ui::chart::{
    Chart, GRID, Grid, Plot, Series, Tooltip, XAxis, YAxis, default_tooltip, hour_ticks, series_rows,
};
use crate::ui::emblems::{DssIcon, FactionEmblem};
use crate::ui::icons::{self, Icon};
use crate::ui::primitives::{Empty, Loading};
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;
use warroom_core::api::Faction;
use warroom_core::factions::{faction_color, faction_label};
use warroom_core::format::{fmt_compact, fmt_int, sest_clock, sest_date};
use warroom_core::gww_nav::LOG_TITLE;
use warroom_core::owner_changes::{LogQuery, log_matches};

#[derive(Clone, Copy, PartialEq)]
enum LogFilter {
    All,
    Won,
    Lost,
}

/// War log days shown at first, and added by each press of "more".
const LOG_DAYS: usize = 14;

fn war_log() -> AnyView {
    let h = history();
    let q = queries();
    let show = RwSignal::new(LogFilter::All);
    let days_shown = RwSignal::new(LOG_DAYS);
    let changes = h.changes;
    let ready = h.ready;
    let older = h.older;
    let fetching = RwSignal::new(false);
    let enemy = RwSignal::new(None::<Faction>);
    let sector = RwSignal::new(None::<String>);
    let text = RwSignal::new(String::new());
    let query = Memo::new(move |_| LogQuery { enemy: enemy.get(), sector: sector.get(), text: text.get() });
    let sector_of =
        Memo::new(move |_| {
            Arc::new(q.planets.with(|p| {
                p.iter().flat_map(|l| l.iter()).map(|p| (p.index, p.sector.clone())).collect::<HashMap<_, _>>()
            }))
        });
    let sectors = Memo::new(move |_| sector_of.with(|m| m.values().cloned().collect::<BTreeSet<_>>()));
    let searching = RwSignal::new(false);
    // A filter searches the whole war, not only the latest page.
    Effect::new(move |_| {
        if query.with(LogQuery::is_empty) || !ready.get() || !older.get() || searching.get_untracked() {
            return;
        }
        searching.set(true);
        spawn_local(async move {
            let _ = h.load_all().await;
            searching.set(false);
        });
    });
    let days = Memo::new(move |_| {
        let s = show.get();
        let query = query.get();
        let sector_of = sector_of.get();
        let mut days: Vec<(String, Vec<_>)> = Vec::new();
        changes.with(|c| {
            for c in c.iter().filter(|c| {
                (s == LogFilter::All || (s == LogFilter::Won) == (c.change.to == Faction::Humans))
                    && log_matches(&c.change, sector_of.get(&c.change.planet).map(String::as_str), &query)
            }) {
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
        if !ready.get() || searching.get() {
            return view! { <Loading what="the war archive" rows=3 /> }.into_any();
        }
        let list = days.get();
        if list.is_empty() {
            if !query.with(LogQuery::is_empty) {
                return view! { <Empty>"No change of hands matches these filters."</Empty> }.into_any();
            }
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
        let sector_of = sector_of.get();
        list.into_iter()
            .take(days_shown.get())
            .map(|(day, items)| {
                let n = items.len();
                view! {
                    <section class="lg-day">
                        <h3 class="lg-day-head caps num">
                            {day}
                            <span>{if n == 1 { "1 change".to_string() } else { format!("{n} changes") }}</span>
                        </h3>
                        <ol class="lg-rows">
                            {items
                                .into_iter()
                                .map(|c| {
                                    let ch = c.change;
                                    let won = ch.to == Faction::Humans;
                                    let color = faction_color(Some(if won { Faction::Humans } else { ch.to }));
                                    let planet = ch.planet;
                                    let sector = sector_of.get(&planet).cloned().unwrap_or_default();
                                    view! {
                                        <li class=format!("lg-row {}", if won { "won" } else { "lost" }) style=format!("--k: {color};")>
                                            <time class="num">{sest_clock(ch.at).chars().take(5).collect::<String>()}</time>
                                            <span class="lg-mark">
                                                <FactionEmblem faction=if won { "Humans" } else { ch.to.as_str() } size=14.0 />
                                            </span>
                                            <span class="lg-what">
                                                <button
                                                    class="link lg-planet"
                                                    on:click=move |e| ui().open_options(Some(OptionsAnchor::at(planet, &e)))
                                                >
                                                    {ch.planet_name.clone()}
                                                </button>
                                                <span class="lg-event">
                                                    {if won { format!("liberated from {}", faction_label(Some(ch.from))) } else { format!("fell to {}", faction_label(Some(ch.to))) }}
                                                </span>
                                            </span>
                                            <span class="lg-sector caps">{sector}</span>
                                        </li>
                                    }
                                })
                                .collect_view()}
                        </ol>
                    </section>
                }
            })
            .collect_view()
            .into_any()
    };
    let more = move || {
        let n = days.with(Vec::len);
        let shown = days_shown.get();
        (ready.get() && n > shown).then(|| {
            let step = LOG_DAYS.min(n - shown);
            view! {
                <button class="gww-chip gww-more" on:click=move |_| days_shown.update(|d| *d += LOG_DAYS)>
                    {format!("Show {step} more {}", if step == 1 { "day" } else { "days" })}
                </button>
            }
        })
    };
    let load_older = move || {
        let every_day = days_shown.get() >= days.with(Vec::len);
        (ready.get() && older.get() && every_day && query.with(LogQuery::is_empty)).then(|| {
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
    let week = Memo::new(move |_| {
        let since = js::now() - 7.0 * 86_400_000.0;
        changes.with(|c| {
            let recent = c.iter().filter(|c| c.change.at >= since);
            recent.fold((0, 0), |(w, l), c| if c.change.to == Faction::Humans { (w + 1, l) } else { (w, l + 1) })
        })
    });
    view! {
        <Show when=move || ready.get()>
            <p class="lg-week caps num">
                "Last 7 days "
                <b style="color: var(--f-humans);">{move || week.get().0}" liberated"</b>
                " · "
                <b style="color: var(--bad);">{move || week.get().1}" lost"</b>
            </p>
        </Show>
        <div class="gww-filter" role="group" aria-label="Show">
            {[(LogFilter::All, "Everything"), (LogFilter::Won, "Liberated"), (LogFilter::Lost, "Lost")]
                .map(|(k, label)| view! {
                    <button
                        class=move || format!("gww-chip {}", if show.get() == k { "on" } else { "" })
                        aria-pressed=move || (show.get() == k).to_string()
                        on:click=move |_| {
                            show.set(k);
                            days_shown.set(LOG_DAYS);
                        }
                    >
                        {label}
                    </button>
                })
                .collect_view()}
        </div>
        <div class="gww-log-tools">
            <label class="gww-search">
                <Icon icon=icons::SEARCH size=14.0 />
                <input
                    id="war-log-search"
                    type="search"
                    placeholder="Search planets or sectors"
                    prop:value=move || text.get()
                    on:input=move |e| text.set(event_target_value(&e))
                    aria-label="Search the war log"
                />
            </label>
            <select
                id="war-log-enemy"
                class="gww-select"
                aria-label="Enemy"
                on:change=move |e| {
                    let v = event_target_value(&e);
                    enemy.set(Faction::ENEMIES.into_iter().find(|f| f.as_str() == v));
                }
            >
                <option value="" selected=move || enemy.get().is_none()>"Every enemy"</option>
                {Faction::ENEMIES
                    .map(|f| view! {
                        <option value=f.as_str() selected=move || enemy.get() == Some(f)>{faction_label(Some(f))}</option>
                    })
                    .collect_view()}
            </select>
            <select
                id="war-log-sector"
                class="gww-select"
                aria-label="Sector"
                on:change=move |e| {
                    let v = event_target_value(&e);
                    sector.set((!v.is_empty()).then_some(v));
                }
            >
                <option value="" selected=move || sector.get().is_none()>"Every sector"</option>
                {move || {
                    sectors
                        .get()
                        .into_iter()
                        .map(|s| {
                            let pick = s.clone();
                            view! {
                                <option value=s.clone() selected=move || sector.with(|x| x.as_deref() == Some(pick.as_str()))>
                                    {s.clone()}
                                </option>
                            }
                        })
                        .collect_view()
                }}
            </select>
        </div>
        {body}
        {more}
        {load_older}
    }
    .into_any()
}

/// Days of the online table shown at first, and added by each press of "more".
const ACTIVITY_DAYS: usize = 14;
/// Days the online chart spans.
const CHART_DAYS: usize = 90;

#[derive(Clone, Copy, PartialEq)]
enum DayColumn {
    Day,
    Peak,
    Low,
}

#[derive(Clone, Copy, PartialEq)]
struct DayOrder {
    column: DayColumn,
    highest_first: bool,
}

impl DayOrder {
    const NEWEST_FIRST: Self = Self { column: DayColumn::Day, highest_first: true };

    /// A column starts with its highest value on top; pressing it again turns the list over.
    fn press(self, column: DayColumn) -> Self {
        Self { column, highest_first: self.column != column || !self.highest_first }
    }
}

/// Equal counts keep the newest day on top, the order `days` arrive in.
fn sort_days(days: &[WarDay], order: DayOrder) -> Vec<WarDay> {
    let value = |d: &WarDay| match order.column {
        DayColumn::Day => d.day,
        DayColumn::Peak => d.peak,
        DayColumn::Low => d.low,
    };
    let mut sorted = days.to_vec();
    if order.highest_first {
        sorted.sort_by(|a, b| value(b).total_cmp(&value(a)));
    } else {
        sorted.sort_by(|a, b| value(a).total_cmp(&value(b)));
    }
    sorted
}

fn day_sort(order: RwSignal<DayOrder>, column: DayColumn, label: &'static str) -> AnyView {
    let on = move || order.get().column == column;
    let direction = move || match (column, order.get().highest_first) {
        _ if !on() => "",
        (DayColumn::Day, true) => ", newest first",
        (DayColumn::Day, false) => ", oldest first",
        (_, true) => ", highest first",
        (_, false) => ", lowest first",
    };
    view! {
        <button
            class=move || {
                let up = !order.get().highest_first;
                format!("gww-days-sort {}", if !on() { "" } else if up { "on up" } else { "on" })
            }
            aria-pressed=move || on().to_string()
            on:click=move |_| order.update(|o| *o = o.press(column))
        >
            {label}
            <span class="sr-only">{direction}</span>
            <Icon icon=icons::CHEVRON_DOWN size=12.0 />
        </button>
    }
    .into_any()
}

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

/// Each day's peak and low over the last few months.
fn online_chart(days: &[WarDay]) -> AnyView {
    let mut list: Vec<WarDay> = days.iter().take(CHART_DAYS).cloned().collect();
    list.sort_by(|a, b| a.day.total_cmp(&b.day));
    if list.len() < 2 {
        return ().into_any();
    }
    let xs: Vec<f64> = list.iter().map(|d| d.day).collect();
    let domain = (xs[0], xs[xs.len() - 1]);
    let span_days = (domain.1 - domain.0) / 86_400_000.0;
    let step = 24.0 * (span_days / 6.0).ceil().max(1.0);
    let top = list.iter().fold(1.0, |m: f64, d| m.max(d.peak));
    let plot = Arc::new(Plot {
        x: XAxis { fmt: Arc::new(fmt_date), ..XAxis::time(domain, hour_ticks(domain.0, domain.1, step)) },
        y: vec![YAxis::auto_whole(top, Arc::new(fmt_compact))],
        grid: Some(Grid { stroke: GRID.into(), opacity: None, edges_only: false }),
        series: vec![
            Series {
                fill_opacity: 0.12,
                ..Series::area(
                    "Peak online",
                    list.iter().map(|d| Some(d.peak)).collect(),
                    "var(--cta)",
                    2.0,
                    "var(--cta)",
                )
            },
            Series::line("Lowest online", list.iter().map(|d| Some(d.low)).collect(), "var(--text-dim)", 1.5),
        ],
        ref_lines: Vec::new(),
        xs,
    });
    let tip: Tooltip = Arc::new(|plot, i| {
        let rows = series_rows(plot, i, |s, v| (fmt_int(v), s.name.clone()));
        Some(default_tooltip(sest_date(*plot.xs.get(i)?), rows))
    });
    view! {
        <div class="lg-chart">
            <Chart plot=Signal::stored(plot) height=170.0 margin=(8.0, 8.0, 0.0, 0.0) tooltip=tip />
            <ul class="lg-chart-key caps">
                <li style="--k: var(--cta);">"Peak online"</li>
                <li style="--k: var(--text-dim);">"Lowest online"</li>
            </ul>
        </div>
    }
    .into_any()
}

fn activity_days(now: Memo<f64>) -> AnyView {
    let q = use_war_days();
    let shown_days = RwSignal::new(ACTIVITY_DAYS);
    let order = RwSignal::new(DayOrder::NEWEST_FIRST);
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
        let n = days.len();
        let chart = online_chart(&days);
        // In its own closure, so pressing a heading keeps the focus on it.
        let rows = move || {
            let shown: Vec<_> = sort_days(&days, order.get()).into_iter().take(shown_days.get()).collect();
            let top = shown.iter().fold(1.0, |m: f64, d| m.max(d.peak));
            let today = (now.get() / 86_400_000.0).floor() * 86_400_000.0;
            shown
                .into_iter()
                .map(|d| {
                    view! {
                        <li class=(d.day == today).then_some("today")>
                            <span class="gww-day-date num">
                                {sest_date(d.day)}
                                {(d.day == today).then(|| view! { <span class="caps">"Today, so far"</span> })}
                            </span>
                            {day_stat(true, d.peak, hm(d.peak_at), top)}
                            {day_stat(false, d.low, hm(d.low_at), top)}
                        </li>
                    }
                })
                .collect_view()
        };
        view! {
            {chart}
            <div class="gww-days-head caps" role="group" aria-label="Sort days by">
                {day_sort(order, DayColumn::Day, "SEST day")}
                {day_sort(order, DayColumn::Peak, "Peak online")}
                {day_sort(order, DayColumn::Low, "Lowest online")}
            </div>
            <ol class="gww-days">{rows}</ol>
            <p class="gww-note">"Each bar is the day's count against the busiest day listed."</p>
            {move || {
                let shown = shown_days.get();
                (n > shown).then(|| {
                    let step = (ACTIVITY_DAYS * 2).min(n - shown);
                    view! {
                        <button class="gww-chip gww-more" on:click=move |_| shown_days.update(|d| *d += ACTIVITY_DAYS * 2)>
                            {format!("Show {step} more {}", if step == 1 { "day" } else { "days" })}
                        </button>
                    }
                })
            }}
        }
        .into_any()
    })
    .into_any()
}

#[component]
fn StationJumps(station: i64, now: Memo<f64>) -> impl IntoView {
    let q = use_station_jumps(move || station);
    let jumps = Signal::derive(move || q.data.get().unwrap_or_default());
    let note = move || match jumps.with(|j| j.len()) {
        1 => "1 jump".to_string(),
        n if n >= JUMP_LIMIT => format!("Latest {n} jumps"),
        n => format!("{n} jumps"),
    };
    let waiting = Memo::new(move |_| q.loading.get() && q.data.with(Option::is_none));
    let empty = Memo::new(move |_| jumps.with(|j| j.is_empty()));
    move || {
        if waiting.get() {
            return view! { <Loading what="the jump log" rows=2 /> }.into_any();
        }
        if empty.get() {
            return view! { <Empty>"No jumps logged yet. Each jump is logged as telemetry comes in."</Empty> }
                .into_any();
        }
        view! {
            <p class="lg-note caps num">{note}</p>
            <JumpRoute jumps=jumps now=now />
        }
        .into_any()
    }
}

#[derive(Clone, Copy, PartialEq)]
enum LogTab {
    War,
    Online,
    /// A space station's jumps, by its place in the station list.
    Jumps(usize),
}

fn station_name(i: usize) -> String {
    if i == 0 { "DSS".to_string() } else { format!("Station {}", i + 1) }
}

fn tab_button(tab: RwSignal<LogTab>, me: LogTab, icon: AnyView, label: String) -> AnyView {
    view! {
        <button
            class=move || format!("lg-tab {}", if tab.get() == me { "on" } else { "" })
            role="tab"
            aria-selected=move || (tab.get() == me).to_string()
            on:click=move |_| tab.set(me)
        >
            {icon}
            <span>{label}</span>
        </button>
    }
    .into_any()
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
    let tab = RwSignal::new(LogTab::War);
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
        <article class="gww-page lg" style="--site: var(--f-humans);">
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
            <div class="lg-tabs" role="tablist" aria-label="Logs">
                {tab_button(tab, LogTab::War, view! { <Icon icon=icons::GLOBE_2 size=16.0 /> }.into_any(), "War log".into())}
                {tab_button(tab, LogTab::Online, view! { <Icon icon=icons::USERS size=16.0 /> }.into_any(), "Helldivers online".into())}
                <For
                    each=move || stations.get().into_iter().enumerate()
                    key=|(i, id)| (*i, *id)
                    children=move |(i, _)| {
                        tab_button(tab, LogTab::Jumps(i), view! { <DssIcon size=16.0 /> }.into_any(), format!("{} FTL jumps", station_name(i)))
                    }
                />
            </div>
            <div class="lg-panel" role="tabpanel" hidden=move || tab.get() != LogTab::War>
                <p class="lg-note caps num">{war_note}</p>
                {war_log()}
            </div>
            <div class="lg-panel" role="tabpanel" hidden=move || tab.get() != LogTab::Online>
                <p class="lg-note caps">"Each SEST day, midnight to midnight"</p>
                {activity_days(now)}
            </div>
            <For
                each=move || stations.get().into_iter().enumerate()
                key=|(i, id)| (*i, *id)
                children=move |(i, id)| {
                    view! {
                        <div class="lg-panel" role="tabpanel" hidden=move || tab.get() != LogTab::Jumps(i)>
                            <StationJumps station=id now=now />
                        </div>
                    }
                }
            />
            {related(vec!["manual:liberation", "manual:dss", "app:stations", "app:past"], "Read next")}
        </article>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(day: f64, peak: f64, low: f64) -> WarDay {
        WarDay { day, peak, peak_at: day, low, low_at: day }
    }

    fn sorted(order: DayOrder) -> Vec<f64> {
        let days = [day(3.0, 100.0, 20.0), day(2.0, 300.0, 10.0), day(1.0, 200.0, 30.0)];
        sort_days(&days, order).iter().map(|d| d.day).collect()
    }

    #[test]
    fn opens_on_the_newest_day() {
        assert_eq!(sorted(DayOrder::NEWEST_FIRST), [3.0, 2.0, 1.0]);
    }

    #[test]
    fn a_first_press_puts_the_highest_on_top_and_a_second_the_lowest() {
        let peak = DayOrder::NEWEST_FIRST.press(DayColumn::Peak);
        assert_eq!(sorted(peak), [2.0, 1.0, 3.0]);
        assert_eq!(sorted(peak.press(DayColumn::Peak)), [3.0, 1.0, 2.0]);

        let low = DayOrder::NEWEST_FIRST.press(DayColumn::Low);
        assert_eq!(sorted(low), [1.0, 3.0, 2.0]);
        assert_eq!(sorted(low.press(DayColumn::Low)), [2.0, 3.0, 1.0]);
    }

    #[test]
    fn another_column_starts_with_its_highest_on_top() {
        let lowest_peak = DayOrder::NEWEST_FIRST.press(DayColumn::Peak).press(DayColumn::Peak);
        assert_eq!(sorted(lowest_peak.press(DayColumn::Low)), [1.0, 3.0, 2.0]);
    }

    #[test]
    fn the_day_heading_brings_back_the_newest_day() {
        let peak = DayOrder::NEWEST_FIRST.press(DayColumn::Peak);
        assert_eq!(sorted(peak.press(DayColumn::Day)), [3.0, 2.0, 1.0]);
        assert_eq!(sorted(DayOrder::NEWEST_FIRST.press(DayColumn::Day)), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn equal_counts_keep_the_newest_day_on_top() {
        let days = [day(2.0, 100.0, 10.0), day(1.0, 100.0, 10.0)];
        let peak = DayOrder::NEWEST_FIRST.press(DayColumn::Peak);
        for order in [peak, peak.press(DayColumn::Peak)] {
            assert_eq!(sort_days(&days, order)[0].day, 2.0);
        }
    }
}
