//! A planet's fight as the game shows it: Super Earth's bar and, in a defense, the enemy's advance against the
//! clock, where the attack comes from, and the gambit of liberating the attacker first.

use crate::api::fronts::planet_rate;
use crate::api::queries::queries;
use crate::features::planet::cities::CityBar;
use crate::js::{self, num};
use crate::state::ui;
use leptos::prelude::*;
use std::collections::HashMap;
use warroom_core::api::{Faction, Planet};
use warroom_core::defense::{Gambit, GambitOutlook, attackers, enemy_advance, gambit, gambits_from};
use warroom_core::factions::{faction_color, faction_label};
use warroom_core::format::{fmt_duration, fmt_pct, fmt_rate, fmt_time_left, js_round};
use warroom_core::progress::projected_at;
use warroom_core::time::parse_ms;

/// The enemy's advance in a defense, 0-100, kept current; None without one.
pub fn use_enemy_advance(p: Signal<Planet>) -> Memo<Option<f64>> {
    let clock = js::use_now(30_000.0);
    Memo::new(move |_| {
        let now = clock.get();
        p.with(|p| p.event.as_ref().and_then(|e| enemy_advance(e, now)))
    })
}

/// The enemy planets a defense is attacked from.
pub fn use_attackers(p: Signal<Planet>) -> Memo<Vec<Planet>> {
    let q = queries();
    Memo::new(move |_| {
        q.fresh_planets.with(|all| {
            let Some(all) = all else { return Vec::new() };
            p.with(|p| if p.event.is_none() { Vec::new() } else { attackers(p, all).into_iter().cloned().collect() })
        })
    })
}

/// The gambits a planet takes part in: the one open to its defense, or those its liberation would win.
pub fn use_gambits(p: Signal<Planet>) -> Memo<Vec<Gambit>> {
    let q = queries();
    let clock = js::use_now(60_000.0);
    Memo::new(move |_| {
        let now = clock.get();
        let Some(all) = q.fresh_planets.get() else { return Vec::new() };
        let rates: HashMap<i64, Option<f64>> =
            q.campaigns.with(|c| c.iter().flat_map(|l| l.iter()).map(|c| (c.planet.index, c.rate)).collect());
        let index = p.with(|p| p.index);
        let Some(me) = all.iter().find(|x| x.index == index) else { return Vec::new() };
        if me.event.is_some() {
            gambit(me, &all, &rates, now).into_iter().collect()
        } else if rates.contains_key(&index) {
            gambits_from(me, &all, &rates, now)
        } else {
            Vec::new()
        }
    })
}

/// "Automatons advance".
pub fn advance_label(f: Faction) -> String {
    format!("{} advance", faction_label(Some(f)))
}

/// The planet's progress as the map's cards show it. A liberation is one bar with a mark where each city opens;
/// a defense is two, as in the game: Super Earth's defense right over the enemy's advance.
#[component]
pub fn LiberationBar(
    #[prop(into)] p: Signal<Planet>,
    #[prop(into)] pct: Signal<f64>,
    #[prop(into)] enemy: Signal<Faction>,
) -> impl IntoView {
    let advance = use_enemy_advance(p);
    let defended = Memo::new(move |_| p.with(|p| p.event.is_some()));
    let clock = js::use_now(30_000.0);
    let deadline = Memo::new(move |_| p.with(|p| p.event.as_ref().map(|e| parse_ms(&e.end_time))));
    let projected = Memo::new(move |_| {
        let end = deadline.get()?;
        let index = p.with(|p| p.index);
        projected_at(pct.get(), planet_rate(Some(index)), clock.get(), end).filter(|v| *v > 0.0 && *v < 100.0)
    });
    let track = move |value: Signal<f64>, label: Signal<String>, class: &'static str| {
        view! {
            <div
                class=format!("hc-bar-track {class}")
                role="progressbar"
                aria-valuenow=move || num(js_round(value.get()))
                aria-valuemin="0"
                aria-valuemax="100"
                aria-label=move || label.get()
            >
                <div class="hc-bar-fill" style=move || format!("width: {}%;", num(value.get().clamp(0.0, 100.0)))></div>
            </div>
        }
    };
    view! {
        <div
            class=move || format!("hc-bar {}", if defended.get() { "fight" } else { "" })
            style=move || format!("--rest: {};", faction_color(Some(enemy.get())))
        >
            {move || {
                if !defended.get() {
                    return view! { <CityBar planet=p>{track(pct, Signal::stored("Liberated".to_string()), "")}</CityBar> }
                        .into_any();
                }
                let foe = Signal::derive(move || advance.get().unwrap_or(0.0));
                view! {
                    <div class="fight-head caps">
                        <span>"Defended"</span>
                        <b class="num">{move || fmt_pct(pct.get(), 2)}</b>
                    </div>
                    <div class="fight-se">
                        {track(pct, Signal::stored("Defended".to_string()), "")}
                        {move || {
                            projected
                                .get()
                                .map(|m| {
                                    view! {
                                        <span
                                            class="fight-marker"
                                            style=format!("left: {}%;", num(m))
                                            title=format!("{} by the deadline at this pace", fmt_pct(m, 1))
                                        ></span>
                                    }
                                })
                        }}
                    </div>
                    {track(foe, Signal::derive(move || advance_label(enemy.get())), "foe")}
                    <div class="fight-head foe caps">
                        <span>{move || advance_label(enemy.get())}</span>
                        <span class="fight-left num">
                            {move || deadline.get().map(|d| format!("{} left", fmt_time_left(d - clock.get())))}
                        </span>
                        <b class="num">{move || fmt_pct(foe.get(), 0)}</b>
                    </div>
                }
                .into_any()
            }}
        </div>
    }
}

fn planet_link(p: &Planet) -> AnyView {
    let index = p.index;
    view! {
        <button
            class="link gambit-planet caps"
            style=format!("color: {};", faction_color(Some(p.current_owner)))
            on:click=move |_| ui().open_detail(Some(index))
        >
            {p.name.clone()}
        </button>
    }
    .into_any()
}

fn planet_links(list: &[&Planet]) -> AnyView {
    list.iter()
        .enumerate()
        .map(|(i, p)| {
            view! {
                {(i > 0).then(|| if i == list.len() - 1 { " and " } else { ", " })}
                {planet_link(p)}
            }
        })
        .collect_view()
        .into_any()
}

/// A gambit drawn small: the attacking world in its sights, its line cut short of the shield.
#[component]
pub fn GambitMark(#[prop(default = 30.0)] size: f64) -> impl IntoView {
    view! {
        <svg class="gambit-mark" width=num(size) height=num(size) viewBox="-12 -12 24 24" aria-hidden="true">
            <circle cx="-5" r="5.6" class="gambit-ring" />
            <circle cx="-5" r="2.3" class="gambit-foe" />
            <path d="M1.2 0H5.4M3.6 -1.8L5.4 0L3.6 1.8" class="gambit-arrow" />
            <path d="M0.4 3.4L3.4 -3.4" class="gambit-cut" />
            <path d="M9 -5.4L11.8 -4.2V-0.6C11.8 1.8 10.6 3.4 9 4.2C7.4 3.4 6.2 1.8 6.2 -0.6V-4.2Z" class="gambit-shield" />
        </svg>
    }
}

fn outlook_tag(o: GambitOutlook) -> (&'static str, &'static str) {
    match o {
        GambitOutlook::Ahead => ("In reach", "ahead"),
        GambitOutlook::Slower => ("Holding is quicker", "slower"),
        GambitOutlook::OutOfReach => ("Out of reach", "out"),
        GambitOutlook::Measuring => ("Measuring", "measuring"),
    }
}

/// The gambit advisory: which planets to liberate to end a defense, and whether that beats the deadline and
/// holding the planet outright. `here` is the planet the note is shown on, defended or attacking.
pub fn gambit_note(g: &Gambit, here: i64, now: f64, class: &str) -> AnyView {
    let (tag, tone) = outlook_tag(g.outlook);
    let from: Vec<&Planet> = g.attackers.iter().map(|a| &a.planet).collect();
    let defended = &g.defended;
    let lead = if here == defended.index {
        view! {
            "Liberating "
            {planet_links(&from)}
            " ends this defense: the attack comes from "
            {if from.len() > 1 { "them" } else { "it" }}
            "."
        }
        .into_any()
    } else {
        view! {
            "Liberating "
            {planet_links(&from)}
            " ends the defense of "
            {planet_link(defended)}
            ", which "
            {if from.len() > 1 { "they attack" } else { "it attacks" }}
            "."
        }
        .into_any()
    };
    let left = g.deadline - now;
    let verdict = match g.outlook {
        GambitOutlook::Ahead => g.done_at.map(|d| {
            format!(
                "At the current pace that takes {}, {} before the deadline.",
                fmt_duration(d - now),
                fmt_duration(g.deadline - d)
            )
        }),
        GambitOutlook::Slower => {
            g.held_at.map(|h| format!("Holding outright takes {}, sooner than the gambit.", fmt_duration(h - now)))
        }
        GambitOutlook::OutOfReach => Some(match g.done_at {
            Some(d) => format!(
                "At the current pace that takes {}; the defense ends in {}. Hold the planet directly.",
                fmt_duration(d - now),
                fmt_time_left(left)
            ),
            None => {
                format!("Not gaining ground; the defense ends in {}. Hold the planet directly.", fmt_time_left(left))
            }
        }),
        GambitOutlook::Measuring => Some("The attacker's pace is still being measured.".to_string()),
    };
    view! {
        <section class=format!("gambit {tone} {class}") aria-label="Gambit">
            <GambitMark />
            <div class="gambit-body">
                <span class="gambit-kind caps">"Defense objective"</span>
                <div class="gambit-head">
                    <b class="gambit-title caps">"Gambit"</b>
                    <span class="gambit-tag caps">{tag}</span>
                </div>
                <p>{lead}</p>
                {verdict.map(|v| view! { <p class="gambit-verdict">{v}</p> })}
                <ul class="gambit-fronts">
                    {g
                        .attackers
                        .iter()
                        .map(|a| {
                            view! {
                                <li class="num caps">
                                    <span style=format!("--k: {};", faction_color(Some(a.planet.current_owner)))>
                                        {a.planet.name.clone()}
                                    </span>
                                    {format!(" {} liberated · {}", fmt_pct(a.percent, 1), fmt_rate(a.rate))}
                                </li>
                            }
                        })
                        .collect_view()}
                </ul>
            </div>
        </section>
    }
    .into_any()
}

/// Every gambit the planet takes part in, as advisories; nothing when there's none.
#[component]
pub fn GambitNotes(#[prop(into)] p: Signal<Planet>, #[prop(into)] class: String) -> impl IntoView {
    let list = use_gambits(p);
    let clock = js::use_now(60_000.0);
    move || {
        let here = p.with(|p| p.index);
        let now = clock.get();
        list.with(|l| l.iter().map(|g| gambit_note(g, here, now, &class)).collect_view())
    }
}
