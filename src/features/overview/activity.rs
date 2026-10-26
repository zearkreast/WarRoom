use crate::api::queries::queries;
use crate::api::war_samples::{use_front_activity, use_war_samples};
use crate::js::{self, fmt_date_time, fmt_time, num};
use crate::ui::chart::{Chart, Curve, Frame, Grid, Overlay, Plot, Series, Tooltip, XAxis, YAxis, hour_ticks};
use crate::ui::emblems::FactionEmblem;
use crate::ui::icons::{self, Icon};
use crate::ui::primitives::{Empty, Loading, Panel};
use leptos::prelude::*;
use std::collections::HashSet;
use std::sync::Arc;
use warroom_core::api::Faction;
use warroom_core::factions::faction_label;
use warroom_core::format::{fmt_int, js_round, to_fixed};
use warroom_core::front_activity::{
    ActivityPoint, ActivitySeries, PlotPoint, Reading, average_readings, merge_activity, plot_activity,
};

/// Chart fills per faction (dark-band steps of the faction hues).
pub fn chart_color(f: Faction) -> &'static str {
    match f {
        Faction::Humans => "var(--c-humans)",
        Faction::Terminids => "var(--c-terminids)",
        Faction::Automaton => "var(--c-automaton)",
        Faction::Illuminate => "var(--c-illuminate)",
        Faction::Unknown => "var(--text-dim)",
    }
}

pub const ENEMIES: [Faction; 3] = [Faction::Terminids, Faction::Illuminate, Faction::Automaton];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum SeriesKey {
    Front(Faction),
    Online,
    Impact,
}

/// Bottom to top of the stack.
const STACK: [Faction; 3] = [Faction::Terminids, Faction::Automaton, Faction::Illuminate];

fn front_series(f: Faction) -> ActivitySeries {
    match f {
        Faction::Terminids => ActivitySeries::Terminids,
        Faction::Automaton => ActivitySeries::Automatons,
        _ => ActivitySeries::Illuminate,
    }
}

fn front_value(r: &ActivityPoint, f: Faction) -> f64 {
    match f {
        Faction::Terminids => r.terminids,
        Faction::Automaton => r.automatons,
        _ => r.illuminate,
    }
}

fn plot_value(p: &PlotPoint, f: Faction) -> f64 {
    match f {
        Faction::Terminids => p.plot_terminids,
        Faction::Automaton => p.plot_automatons,
        _ => p.plot_illuminate,
    }
}

// Series colours are the panel's own tokens (warActivity.css).
fn front_color(f: Faction) -> &'static str {
    match f {
        Faction::Terminids => "var(--wa-terminids)",
        Faction::Automaton => "var(--wa-automaton)",
        _ => "var(--wa-illuminate)",
    }
}

const ONLINE: &str = "var(--wa-online)";
const IMPACT: &str = "var(--wa-impact)";
const MUTED: &str = "var(--wa-muted)";
const FULL_DAY_MS: f64 = 23.5 * 3_600_000.0;
const STEP: f64 = 25_000.0;
const READING_WINDOW_MS: f64 = 2.0 * 60_000.0;
// Keep in sync with --wa-morph in warActivity.css.
const MORPH_MS: f64 = 700.0;
// Keep in sync with --ease in theme.css, which the CSS fades use.
const MORPH_EASE: (f64, f64, f64, f64) = (0.2, 0.85, 0.25, 1.0);

fn fmt_k(v: f64) -> String {
    if v == 0.0 { "0".into() } else { format!("{}k", num(js_round(v / 1000.0))) }
}

/// Five evenly spaced ticks around the multiplier's range, which moves in the fourth decimal.
fn impact_scale(values: &[f64]) -> ((f64, f64), Vec<f64>) {
    let lo = values.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let pad = ((hi - lo) * 0.12).max(0.0004);
    let from = (lo - pad).max(0.0);
    let to = hi + pad;
    ((from, to), (0..5).map(|i| from + (to - from) * i as f64 / 4.0).collect())
}

/// The cubic-bezier timing Recharts animates with, solved for x by Newton's method as it does.
fn bezier(t: f64, (x1, y1, x2, y2): (f64, f64, f64, f64)) -> f64 {
    let curve = |a: f64, b: f64, t: f64| {
        let mt = 1.0 - t;
        3.0 * a * mt * mt * t + 3.0 * b * mt * t * t + t * t * t
    };
    let der = |a: f64, b: f64, t: f64| {
        let mt = 1.0 - t;
        3.0 * a * mt * mt + 6.0 * (b - a) * mt * t + 3.0 * (1.0 - b) * t * t
    };
    let t = t.min(1.0);
    let mut x = t;
    for _ in 0..8 {
        let eval = curve(x1, x2, x) - t;
        let d = der(x1, x2, x);
        if (eval - t).abs() < 1e-4 || d < 1e-4 {
            return curve(y1, y2, x);
        }
        x = (x - eval / d).clamp(0.0, 1.0);
    }
    curve(y1, y2, x)
}

fn mix(a: &[PlotPoint], b: &[PlotPoint], k: f64) -> Vec<PlotPoint> {
    if a.len() != b.len() {
        return b.to_vec();
    }
    let lerp = |x: f64, y: f64| x + (y - x) * k;
    let opt = |x: Option<f64>, y: Option<f64>| match (x, y) {
        (Some(x), Some(y)) => Some(lerp(x, y)),
        _ => y,
    };
    a.iter()
        .zip(b)
        .map(|(p, q)| PlotPoint {
            row: q.row,
            plot_terminids: lerp(p.plot_terminids, q.plot_terminids),
            plot_automatons: lerp(p.plot_automatons, q.plot_automatons),
            plot_illuminate: lerp(p.plot_illuminate, q.plot_illuminate),
            plot_online: opt(p.plot_online, q.plot_online),
            plot_impact: opt(p.plot_impact, q.plot_impact),
        })
        .collect()
}

fn extreme(
    points: &[ActivityPoint],
    value: impl Fn(&ActivityPoint) -> f64,
    pick: impl Fn(f64, f64) -> bool,
) -> Option<ActivityPoint> {
    points.iter().fold(None, |m: Option<ActivityPoint>, p| match m {
        None => Some(*p),
        Some(m) if pick(value(p), value(&m)) => Some(*p),
        m => m,
    })
}

#[component]
fn Stat(
    #[prop(into)] label: String,
    #[prop(into)] value: Signal<String>,
    #[prop(optional, into)] when: MaybeProp<String>,
    #[prop(into)] tone: String,
    #[prop(optional)] toned: bool,
) -> impl IntoView {
    let style = toned.then(|| format!("color: {tone};"));
    view! {
        <div>
            <span class="pd-key" style=format!("--k: {tone};")>{label}</span>
            <b class="num" style=style>{value}</b>
            {move || when.get().map(|w| view! { <span class="ov-when num">{w}</span> })}
        </div>
    }
}

/// Galaxy-wide telemetry for the last 24 hours, or since the collector's latest gap: Helldivers per
/// front stacked under the online total, with the impact multiplier on its own right-hand axis.
#[component]
pub fn WarActivity() -> impl IntoView {
    let q = queries();
    let (data, loading) = use_front_activity();
    let readings = use_war_samples().data;
    let hidden = RwSignal::new(HashSet::<SeriesKey>::new());
    // Chips toggled at least once, so their switch animation never plays on first paint.
    let touched = RwSignal::new(HashSet::<SeriesKey>::new());
    let morph_next = StoredValue::new(false);

    let rows = Memo::new(move |_| {
        let (Some(data), Some(last)) = (data.get(), q.last_update.get()) else { return Arc::new(Vec::new()) };
        let readings: Vec<Reading> = readings
            .with(|r| {
                r.as_ref().map(|r| {
                    r.iter()
                        .map(|s| Reading {
                            at: s.at,
                            impact: s.impact,
                            players: s.players,
                            terminids: s.terminids,
                            automatons: s.automatons,
                            illuminate: s.illuminate,
                        })
                        .collect()
                })
            })
            .unwrap_or_default();
        Arc::new(merge_activity(&data, &average_readings(&readings, READING_WINDOW_MS), last))
    });
    let has_online = Memo::new(move |_| rows.with(|r| r.iter().filter(|r| r.online.is_some()).count() > 1));
    // The online total once the collector has recorded it; until then, everyone on the fronts.
    let total = move |p: &ActivityPoint, has_online: bool| if has_online { p.online.unwrap_or(0.0) } else { p.players };
    let pool = Memo::new(move |_| {
        let h = has_online.get();
        rows.with(|r| if h { r.iter().filter(|r| r.online.is_some()).copied().collect::<Vec<_>>() } else { r.to_vec() })
    });
    let peak = Memo::new(move |_| {
        let h = has_online.get();
        pool.with(|p| extreme(p, |x| total(x, h), |a, b| a > b))
    });
    let low = Memo::new(move |_| {
        let h = has_online.get();
        pool.with(|p| extreme(p, |x| total(x, h), |a, b| a < b))
    });
    let last = Memo::new(move |_| pool.with(|p| p.last().copied()));
    let impact_axis = Memo::new(move |_| {
        let impacts: Vec<f64> = rows.with(|r| r.iter().filter_map(|r| r.impact).collect());
        (impacts.len() > 1).then(|| impact_scale(&impacts))
    });
    let last_impact = Memo::new(move |_| rows.with(|r| r.iter().rev().find_map(|r| r.impact)));
    let top = Memo::new(move |_| {
        let h = has_online.get();
        STEP.max((peak.get().map_or(0.0, |p| total(&p, h)) / STEP).ceil() * STEP)
    });
    let domain = Memo::new(move |_| rows.with(|r| (r.first().map_or(0.0, |r| r.at), r.last().map_or(0.0, |r| r.at))));
    let span = move || {
        let (a, b) = domain.get();
        if rows.with(|r| r.len()) < 2 || b - a >= FULL_DAY_MS {
            "last 24 h".to_string()
        } else {
            format!("since {}", fmt_time(a))
        }
    };
    let flat_impact = Memo::new(move |_| impact_axis.get().map_or(0.0, |((a, b), _)| (a + b) / 2.0));
    let target = Memo::new(move |_| {
        let off: Vec<ActivitySeries> = hidden.with(|h| {
            h.iter()
                .map(|s| match s {
                    SeriesKey::Front(f) => front_series(*f),
                    SeriesKey::Online => ActivitySeries::Online,
                    SeriesKey::Impact => ActivitySeries::Impact,
                })
                .collect()
        });
        Arc::new(rows.with(|r| plot_activity(r, &off, has_online.get(), flat_impact.get())))
    });

    // Toggling a series morphs the lines to their new place; data refreshes redraw at once.
    let shown = RwSignal::new(Arc::new(Vec::<PlotPoint>::new()));
    let anim: StoredValue<Option<send_wrapper::SendWrapper<std::rc::Rc<std::cell::Cell<bool>>>>> =
        StoredValue::new(None);
    Effect::new(move |_| {
        let to = target.get();
        if let Some(stop) = anim.get_value() {
            stop.set(true);
        }
        let from = shown.get_untracked();
        if !morph_next.get_value() || js::prefers_reduced_motion() || from.len() != to.len() {
            shown.set(to);
            return;
        }
        morph_next.set_value(false);
        let start = js::performance_now();
        let stop = js::animation_loop(move |t| {
            let k = ((t - start) / MORPH_MS).clamp(0.0, 1.0);
            shown.set(Arc::new(mix(&from, &to, bezier(k, MORPH_EASE))));
            k < 1.0
        });
        anim.set_value(Some(send_wrapper::SendWrapper::new(stop)));
    });

    let off = move |s: SeriesKey| if hidden.with(|h| h.contains(&s)) { " is-off" } else { "" };
    let toggle = move |s: SeriesKey| {
        morph_next.set_value(true);
        hidden.update(|h| {
            if !h.remove(&s) {
                h.insert(s);
            }
        });
        touched.update(|t| {
            t.insert(s);
        });
    };

    let plot = Signal::derive(move || {
        let data = shown.get();
        let (d0, d1) = domain.get();
        let top = top.get();
        let mut y = vec![YAxis {
            width: 44.0,
            line: false,
            tick_fill: MUTED.into(),
            ..YAxis::new((0.0, top), (0..=(top / STEP) as usize).map(|i| i as f64 * STEP).collect(), Arc::new(fmt_k))
        }];
        if let Some((dom, ticks)) = impact_axis.get() {
            y.push(YAxis {
                width: 56.0,
                right: true,
                line: false,
                tick_fill: MUTED.into(),
                tick_class: Some(format!("wa-axis-tick{}", off(SeriesKey::Impact))),
                ..YAxis::new(dom, ticks, Arc::new(|v| to_fixed(v, 4)))
            });
        }
        let xs: Vec<f64> = data.iter().map(|p| p.row.at).collect();
        let mut base = vec![0.0; data.len()];
        let mut series = Vec::new();
        for f in STACK {
            let values: Vec<f64> = data.iter().map(|p| plot_value(p, f)).collect();
            let tops: Vec<f64> = base.iter().zip(&values).map(|(b, v)| b + v).collect();
            series.push(Series {
                base: Some(base.clone()),
                stroke_opacity: Some(0.7),
                fill_opacity: 1.0,
                class: format!("wa-series wa-front wa-front-{}{}", f.as_str(), off(SeriesKey::Front(f))),
                curve: Curve::Monotone,
                active_dot: false,
                ..Series::area(
                    faction_label(Some(f)),
                    tops.iter().map(|v| Some(*v)).collect(),
                    front_color(f),
                    1.0,
                    &format!("url(#wa-front-{})", f.as_str()),
                )
            });
            base = tops;
        }
        series.push(Series {
            class: format!("wa-series wa-online{}", off(SeriesKey::Online)),
            curve: Curve::Monotone,
            active_dot: false,
            connect_nulls: true,
            ..Series::line("Helldivers online", data.iter().map(|p| p.plot_online).collect(), ONLINE, 2.5)
        });
        if impact_axis.get().is_some() {
            series.push(Series {
                class: format!("wa-series wa-impact{}", off(SeriesKey::Impact)),
                axis: 1,
                curve: Curve::Monotone,
                active_dot: false,
                connect_nulls: true,
                ..Series::line("Galactic impact mod", data.iter().map(|p| p.plot_impact).collect(), IMPACT, 2.0)
            });
        }
        Arc::new(Plot {
            xs,
            x: XAxis { line: false, tick_fill: MUTED.into(), ..XAxis::time((d0, d1), hour_ticks(d0, d1, 4.0)) },
            y,
            grid: Some(Grid { stroke: MUTED.into(), opacity: Some(0.15), edges_only: true }),
            series,
            ref_lines: Vec::new(),
        })
    });

    let tooltip: Tooltip = Arc::new(move |plot: &Plot, i: usize| {
        let row = shown.with_untracked(|d| d.get(i).map(|p| p.row))?;
        let hidden = hidden.get_untracked();
        let mut lines: Vec<(String, String, &'static str)> = Vec::new();
        if let Some(o) = row.online.filter(|_| !hidden.contains(&SeriesKey::Online)) {
            lines.push(("Helldivers online".into(), fmt_int(o), ONLINE));
        }
        for f in STACK.iter().rev() {
            if !hidden.contains(&SeriesKey::Front(*f)) {
                lines.push((
                    format!("Fighting {}", faction_label(Some(*f))),
                    fmt_int(front_value(&row, *f)),
                    front_color(*f),
                ));
            }
        }
        if let Some(v) = row.impact.filter(|_| !hidden.contains(&SeriesKey::Impact)) {
            lines.push(("Galactic impact mod".into(), to_fixed(v, 6), IMPACT));
        }
        Some(
            view! {
                <div class="wa-tip">
                    <p class="wa-tip-when num">{fmt_date_time(plot.xs[i])}</p>
                    {lines
                        .into_iter()
                        .map(|(name, value, color)| view! {
                            <p>
                                <i style=format!("background: {color};")></i>
                                <span>{name}</span>
                                <b class="num">{value}</b>
                            </p>
                        })
                        .collect_view()}
                </div>
            }
            .into_any(),
        )
    });

    let overlay: Overlay = Arc::new(move |f: &Frame| {
        let h = has_online.get_untracked();
        let online_off = off(SeriesKey::Online);
        let dots = [peak.get_untracked(), low.get_untracked()]
            .into_iter()
            .flatten()
            .map(|p| {
                view! {
                    <g class=format!("recharts-layer recharts-reference-dot wa-dot{online_off}")>
                        <circle
                            r="4"
                            fill=ONLINE
                            stroke="var(--wa-surface)"
                            stroke-width="2"
                            cx=num(f.x(p.at))
                            cy=num(f.y(0, total(&p, h)))
                            class="recharts-dot recharts-reference-dot-dot"
                        ></circle>
                    </g>
                }
            })
            .collect_view();
        let live = last.get_untracked().map(|p| {
            let (cx, cy) = (num(f.x(p.at)), num(f.y(0, total(&p, h))));
            view! {
                <g class=format!("recharts-layer recharts-reference-dot wa-live-dot{online_off}")>
                    <g aria-hidden="true" class="wa-live">
                        <circle cx=cx.clone() cy=cy.clone() r="5" fill="none" stroke=ONLINE stroke-width="1.5" class="wa-ping" />
                        <circle cx=cx cy=cy r="5" fill="none" stroke=ONLINE stroke-width="1.5" />
                    </g>
                </g>
            }
        });
        view! { {dots} {live} }.into_any()
    });

    let chips: Vec<(SeriesKey, String)> = std::iter::once((SeriesKey::Online, "Helldivers online".to_string()))
        .chain(
            [Faction::Terminids, Faction::Automaton, Faction::Illuminate]
                .map(|f| (SeriesKey::Front(f), format!("Fighting {}", faction_label(Some(f))))),
        )
        .chain(std::iter::once((SeriesKey::Impact, "Galactic impact mod".to_string())))
        .collect();
    let state = Memo::new(move |_| {
        if (loading.get() && data.with(Option::is_none)) || q.last_update.with(Option::is_none) {
            0
        } else if rows.with(|r| r.len()) < 2 {
            1
        } else {
            2
        }
    });
    let h = move || has_online.get();

    view! {
        <Panel class="ov-activity wa" title=Signal::derive(move || format!("Galactic war activity · {}", span())) icon=icons::ACTIVITY>
            <div class="ov-heads">
                <Stat
                    label="Galactic impact mod"
                    value=Signal::derive(move || {
                        format!("×{}", to_fixed(last_impact.get().or_else(|| q.war.with(|w| w.as_ref().map(|w| w.impact_multiplier))).unwrap_or(0.0), 6))
                    })
                    tone=IMPACT
                    toned=true
                />
                <Stat
                    label="Peak Helldivers"
                    value=Signal::derive(move || peak.get().map_or("—".into(), |p| fmt_int(total(&p, h()))))
                    when=Signal::derive(move || peak.get().map(|p| fmt_time(p.at)))
                    tone=ONLINE
                />
                <Stat
                    label="Lowest Helldivers"
                    value=Signal::derive(move || low.get().map_or("—".into(), |p| fmt_int(total(&p, h()))))
                    when=Signal::derive(move || low.get().map(|p| fmt_time(p.at)))
                    tone="var(--text-dim)"
                />
            </div>
            {move || match state.get() {
                0 => view! { <Loading what="war telemetry" rows=2 /> }.into_any(),
                1 => view! { <Empty>"No front readings in the last 24 hours; the collector may be down."</Empty> }.into_any(),
                _ => {
                    let chips = chips.clone();
                    view! {
                        <div class="wa-plot">
                            <span class="wa-corner tl" aria-hidden="true"></span>
                            <span class="wa-corner tr" aria-hidden="true"></span>
                            <span class="wa-corner bl" aria-hidden="true"></span>
                            <span class="wa-corner br" aria-hidden="true"></span>
                            <Chart
                                plot=plot
                                height=280.0
                                margin=(20.0, 0.0, 0.0, 0.0)
                                tooltip=tooltip.clone()
                                overlay=overlay.clone()
                                cursor=MUTED
                                cursor_opacity=0.5
                                defs=ViewFn::from(|| {
                                    STACK
                                        .map(|f| view! {
                                            <linearGradient id=format!("wa-front-{}", f.as_str()) x1="0" y1="0" x2="0" y2="1">
                                                <stop offset="0%" stop-color=front_color(f) stop-opacity="0.45" />
                                                <stop offset="100%" stop-color=front_color(f) stop-opacity="0.12" />
                                            </linearGradient>
                                        })
                                        .collect_view()
                                })
                            />
                        </div>
                        <Show when=move || impact_axis.with(Option::is_none)>
                            <p class="wa-note">"The impact multiplier's history is recorded from the next collector update."</p>
                        </Show>
                        <div class="ov-chips" role="group" aria-label="Show series">
                            {chips
                                .into_iter()
                                .map(|(s, label)| {
                                    let tone = match s {
                                        SeriesKey::Online => ONLINE,
                                        SeriesKey::Impact => IMPACT,
                                        SeriesKey::Front(f) => front_color(f),
                                    };
                                    let on = move || !hidden.with(|h| h.contains(&s));
                                    view! {
                                        <button
                                            class=move || format!("filter-chip {}", if on() { "on" } else { "" })
                                            aria-pressed=move || on().to_string()
                                            data-flash=move || touched.with(|t| t.contains(&s)).then(|| if on() { "on" } else { "off" })
                                            on:click=move |_| toggle(s)
                                            style=format!("--k: {tone};")
                                        >
                                            {match s {
                                                SeriesKey::Online => view! { <FactionEmblem faction="Humans" size=14.0 color=ONLINE /> }.into_any(),
                                                SeriesKey::Impact => view! { <Icon icon=icons::ACTIVITY size=14.0 color=IMPACT /> }.into_any(),
                                                SeriesKey::Front(f) => view! { <FactionEmblem faction=f.as_str() size=14.0 color=front_color(f) /> }.into_any(),
                                            }}
                                            {label}
                                        </button>
                                    }
                                })
                                .collect_view()}
                        </div>
                    }
                    .into_any()
                }
            }}
        </Panel>
    }
}
