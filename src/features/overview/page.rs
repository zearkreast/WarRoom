use crate::api::queries::queries;
use crate::features::galactic_campaigns::CampaignPanel;
use crate::features::major_order::panel::MajorOrderList;
use crate::features::overview::activity::{ENEMIES, WarActivity, chart_color};
use crate::features::overview::theater::TheaterOfWar;
use crate::js::{self, num};
use crate::ui::emblems::FactionEmblem;
use crate::ui::icons::{self, Icon};
use crate::ui::neon_ring::{NeonRing, RingSegment};
use crate::ui::pointer_card::{CardRow, PointerAt, PointerCard, pointer_at};
use crate::ui::primitives::{Panel, Tile};
use leptos::prelude::*;
use std::collections::HashSet;
use warroom_core::api::Faction;
use warroom_core::faction_planets::LIST_SIZE;
use warroom_core::factions::faction_label;
use warroom_core::format::{fmt_compact, fmt_int, fmt_pct, js_sum, sest_clock, sest_date, to_fixed, war_day};
use warroom_core::snapshot::players_by_enemy;
use warroom_core::war_history::fmt_long_duration;
use wasm_bindgen::JsCast;

const TOP_PLANETS: usize = 5;
const OTHERS: &str = "Others";
const ELSEWHERE: &str = "Elsewhere";

#[derive(Clone, Debug, PartialEq)]
struct Front {
    name: String,
    players: f64,
    enemy: Faction,
}

#[derive(Clone, Debug, PartialEq)]
struct Slice {
    name: String,
    players: f64,
    enemy: Option<Faction>,
    color: String,
}

#[derive(Clone, Debug, PartialEq)]
struct Arc_ {
    key: String,
    label: String,
    value: f64,
    color: String,
}

fn enemy_value(f: Faction, e: &warroom_core::snapshot::EnemyPlayers) -> f64 {
    match f {
        Faction::Terminids => e.terminids,
        Faction::Automaton => e.automaton,
        Faction::Illuminate => e.illuminate,
        _ => 0.0,
    }
}

#[component]
fn Distribution() -> impl IntoView {
    let q = queries();
    let now = js::use_now(1000.0);
    let total = Memo::new(move |_| q.war.with(|w| w.as_ref().map_or(0.0, |w| w.statistics.player_count)));
    let list = Memo::new(move |_| {
        let mut list: Vec<Front> = q.campaigns.with(|c| {
            c.iter()
                .flat_map(|c| c.iter())
                .map(|c| Front {
                    name: c.planet.name.clone(),
                    players: c.planet.statistics.player_count,
                    enemy: c.planet.event.as_ref().map_or(c.planet.current_owner, |e| e.faction),
                })
                .collect()
        });
        list.sort_by(|a, b| b.players.total_cmp(&a.players));
        list
    });
    let others = Memo::new(move |_| list.with(|l| l.iter().skip(TOP_PLANETS).cloned().collect::<Vec<_>>()));
    let slices = Memo::new(move |_| {
        let mut slices: Vec<Slice> = list.with(|l| {
            l.iter()
                .take(TOP_PLANETS)
                .map(|p| Slice {
                    name: p.name.clone(),
                    players: p.players,
                    enemy: Some(p.enemy),
                    color: chart_color(p.enemy).into(),
                })
                .collect()
        });
        let rest = others.with(|o| js_sum(o.iter().map(|p| p.players)));
        if rest > 0.0 {
            slices.push(Slice { name: OTHERS.into(), players: rest, enemy: None, color: "var(--faint)".into() });
        }
        slices
    });
    let fronts =
        Memo::new(move |_| q.campaigns.with(|c| players_by_enemy(c.iter().flat_map(|c| c.iter()).map(|c| &c.planet))));
    let elsewhere = Memo::new(move |_| {
        let f = fronts.get();
        (total.get() - f.terminids - f.automaton - f.illuminate).max(0.0)
    });
    let arcs = Memo::new(move |_| {
        let f = fronts.get();
        let mut arcs: Vec<Arc_> = ENEMIES
            .iter()
            .map(|e| Arc_ {
                key: e.as_str().into(),
                label: faction_label(Some(*e)).into(),
                value: enemy_value(*e, &f),
                color: chart_color(*e).into(),
            })
            .collect();
        arcs.push(Arc_ {
            key: ELSEWHERE.into(),
            label: "Elsewhere".into(),
            value: elsewhere.get(),
            color: "var(--faint)".into(),
        });
        arcs
    });

    let active_arc = RwSignal::new(None::<String>);
    let hover_row = RwSignal::new(None::<String>);
    let card = RwSignal::new(None::<(String, PointerAt)>);
    let row_slice = Memo::new(move |_| {
        hover_row.with(|h| h.as_ref().and_then(|h| slices.with(|s| s.iter().find(|p| p.name == *h).cloned())))
    });
    let arc = Memo::new(move |_| {
        let key = active_arc
            .get()
            .or_else(|| row_slice.with(|r| r.as_ref().and_then(|r| r.enemy.map(|e| e.as_str().to_string()))));
        key.and_then(|k| arcs.with(|a| a.iter().find(|a| a.key == k).cloned()))
    });
    let mouse = |e: &web_sys::PointerEvent| e.pointer_type() == "mouse";
    // A touch has no hover: a tap on a legend or front row shows it until a tap lands elsewhere.
    let away = window_event_listener(leptos::ev::pointerdown, move |e| {
        if hover_row.with_untracked(Option::is_none) && active_arc.with_untracked(Option::is_none) {
            return;
        }
        let inside = e
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
            .is_some_and(|t| t.closest(".ov-legend li, .ov-front, .nr").ok().flatten().is_some());
        if mouse(&e) || inside {
            return;
        }
        hover_row.set(None);
        active_arc.set(None);
        card.set(None);
    });
    on_cleanup(move || away.remove());
    let pct_of_online = move |n: f64| fmt_pct(n / (if total.get() != 0.0 { total.get() } else { 1.0 }) * 100.0, 1);

    let card_rows = move |kind: &str| -> Vec<CardRow> {
        if kind == OTHERS {
            return ENEMIES
                .iter()
                .map(|f| CardRow {
                    key: f.as_str().into(),
                    name: faction_label(Some(*f)).into(),
                    value: Some(others.with(|o| js_sum(o.iter().filter(|p| p.enemy == *f).map(|p| p.players)))),
                    note: None,
                    color: Some(chart_color(*f).into()),
                })
                .filter(|r| r.value.unwrap_or(0.0) > 0.0)
                .collect();
        }
        list.with(|l| {
            l.iter()
                .filter(|p| p.enemy.as_str() == kind)
                .take(LIST_SIZE)
                .map(|p| CardRow {
                    key: p.name.clone(),
                    name: p.name.clone(),
                    value: Some(p.players),
                    note: None,
                    color: None,
                })
                .collect()
        })
    };

    let stats = Memo::new(move |_| {
        q.war.with(|w| {
            let Some(s) = w.as_ref().map(|w| &w.statistics) else { return Vec::new() };
            let missions = s.missions_won + s.missions_lost;
            let kills = s.terminid_kills + s.automaton_kills + s.illuminate_kills;
            let or1 = |n: f64| if n != 0.0 { n } else { 1.0 };
            vec![
                (
                    "Missions completed",
                    format!("{} ({})", fmt_int(s.missions_won), fmt_pct(s.missions_won / or1(missions) * 100.0, 1)),
                ),
                (
                    "Missions failed",
                    format!("{} ({})", fmt_int(s.missions_lost), fmt_pct(s.missions_lost / or1(missions) * 100.0, 1)),
                ),
                ("Mission dive time", fmt_long_duration(s.mission_time / 1000.0)),
                ("Shots fired", fmt_int(s.bullets_fired)),
                ("Projectile hits", fmt_int(s.bullets_hit)),
                (
                    "Hits to shot ratio",
                    if s.bullets_fired != 0.0 {
                        format!("{} : 1", to_fixed(s.bullets_hit / s.bullets_fired, 4))
                    } else {
                        "—".into()
                    },
                ),
                (
                    "Kill to death ratio",
                    if s.deaths != 0.0 { format!("{} : 1", to_fixed(kills / s.deaths, 2)) } else { "—".into() },
                ),
                ("Dead Terminids", fmt_int(s.terminid_kills)),
                ("Dead Automatons", fmt_int(s.automaton_kills)),
                ("Dead Illuminate", fmt_int(s.illuminate_kills)),
                ("Helldivers KIA", fmt_int(s.deaths)),
                (
                    "Accidentals",
                    format!("{} ({})", fmt_int(s.friendlies), fmt_pct(s.friendlies / or1(s.deaths) * 100.0, 1)),
                ),
            ]
        })
    });

    let center = move || {
        if let Some(r) = row_slice.get() {
            return view! {
                <div class="nr-readout ov-readout" style=format!("color: {};", r.color)>
                    <span class="caps">{r.name.clone()}</span>
                    <b class="num">{pct_of_online(r.players)}</b>
                    <span class="pl num">
                        <Icon icon=icons::USERS size=11.0 />
                        {format!(" {}", fmt_int(r.players))}
                    </span>
                </div>
            }
            .into_any();
        }
        if let Some(a) = arc.get() {
            let elsewhere = a.key == ELSEWHERE;
            return view! {
                <div class="nr-readout ov-readout" style=format!("color: {};", if elsewhere { "var(--text-dim)".to_string() } else { a.color.clone() })>
                    {(!elsewhere).then(|| view! { <FactionEmblem faction=a.key.clone() size=22.0 /> })}
                    <span class="caps">{a.label.clone()}</span>
                    <b class="num">{pct_of_online(a.value)}</b>
                    <span class="pl num">
                        <Icon icon=icons::USERS size=11.0 />
                        {format!(" {}", fmt_int(a.value))}
                    </span>
                </div>
            }
            .into_any();
        }
        view! {
            <div class="ov-donut-center">
                <b class="num">{fmt_compact(total.get())}</b>
                <span class="caps">"online"</span>
            </div>
        }
        .into_any()
    };

    let segments = Signal::derive(move || {
        arcs.get()
            .into_iter()
            .map(|a| RingSegment { key: a.key, label: a.label, value: a.value, color: a.color })
            .collect::<Vec<_>>()
    });
    let ring_label = move || {
        format!(
            "Helldivers by front: {}",
            arcs.with(|a| a
                .iter()
                .map(|a| format!("{} {}", a.label, fmt_compact(a.value)))
                .collect::<Vec<_>>()
                .join(", "))
        )
    };
    let share = move |n: f64| n / (if total.get() != 0.0 { total.get() } else { 1.0 }) * 100.0;

    view! {
        <Panel title="Distribution of Democracy" icon=icons::PIE_CHART class="ov-dist">
            <div class="ov-dist-top">
                <span class="num muted">{move || format!("SEST {} · {}", sest_clock(now.get()), sest_date(now.get()))}</span>
                {move || q.war.with(|w| w.as_ref().map(|w| w.started.clone())).map(|started| view! {
                    <span class="strip-day caps num">{move || format!("Day {}", war_day(&started, now.get()))}</span>
                })}
            </div>
            <div class="ov-total">
                <Icon icon=icons::USERS size=20.0 />
                <b class="num">{move || fmt_int(total.get())}</b>
                <span class="caps">"Helldivers online"</span>
            </div>
            <div class="ov-summary">
                {move || {
                    let planets = q.fresh_planets.get();
                    let campaigns = q.campaigns.with(|c| c.as_ref().map(|c| c.len()));
                    let war = q.war.with(|w| w.as_ref().map(|w| w.impact_multiplier));
                    view! {
                        <Tile label="Planets charted" value=planets.as_ref().map_or("—".to_string(), |p| p.len().to_string()).into_any() rule="var(--cta)" />
                        <Tile
                            label="Sectors"
                            value=planets
                                .as_ref()
                                .map_or("—".to_string(), |p| p.iter().map(|p| p.sector.as_str()).collect::<HashSet<_>>().len().to_string())
                                .into_any()
                            rule="var(--cta)"
                        />
                        <Tile label="Active fronts" value=campaigns.map_or("—".to_string(), |n| n.to_string()).into_any() rule="var(--cta)" />
                        <Tile
                            label="Impact mod"
                            value=war.map_or("—".to_string(), |m| to_fixed(m, 6)).into_any()
                            color="var(--info)"
                            rule="var(--info)"
                        />
                    }
                }}
            </div>
            <Show when=move || { total.get() > 0.0 }>
                <div class="ov-donut">
                    <NeonRing
                        size=150.0
                        label=ring_label()
                        segments=segments
                        active=Signal::derive(move || arc.get().map(|a| a.key))
                        on_active=Callback::new(move |key: Option<String>| {
                            if key.is_none() {
                                card.set(None);
                            }
                            active_arc.set(key);
                        })
                        on_pointer_move=Callback::new(move |(key, e): (String, web_sys::MouseEvent)| {
                            if key == ELSEWHERE {
                                card.set(None);
                                return;
                            }
                            let svg = e
                                .current_target()
                                .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                                .and_then(|el| el.closest("svg").ok().flatten());
                            if let Some(svg) = svg {
                                card.set(Some((key, pointer_at(&e, &svg))));
                            }
                        })
                        center=std::sync::Arc::new(move || center().into_any())
                    />
                    <ul
                        class="ov-legend"
                        on:pointerleave=move |e| {
                            if !mouse(&e) {
                                return;
                            }
                            hover_row.set(None);
                            card.set(None);
                        }
                    >
                        <For each=move || slices.get() key=|p| p.name.clone() let:p>
                            {
                                let name = p.name.clone();
                                let (n1, n2, n3, n4) = (name.clone(), name.clone(), name.clone(), name.clone());
                                let players = Memo::new(move |_| slices.with(|s| s.iter().find(|x| x.name == n3).map_or(0.0, |x| x.players)));
                                view! {
                                    <li
                                        class=move || if hover_row.with(|h| h.as_deref() == Some(n1.as_str())) { "on" } else { "" }
                                        style=format!("--k: {};", p.color)
                                        on:pointerenter=move |e| {
                                            if !mouse(&e) {
                                                return;
                                            }
                                            hover_row.set(Some(n2.clone()));
                                            if n2 != OTHERS {
                                                card.set(None);
                                            }
                                        }
                                        on:pointermove=move |e: web_sys::PointerEvent| {
                                            if !mouse(&e) || name != OTHERS {
                                                return;
                                            }
                                            let ring = e
                                                .current_target()
                                                .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                                                .and_then(|el| el.closest(".ov-donut").ok().flatten())
                                                .and_then(|d| d.query_selector(".nr").ok().flatten());
                                            if let Some(ring) = ring {
                                                card.set(Some((OTHERS.into(), pointer_at(&e, &ring))));
                                            }
                                        }
                                        on:pointerup=move |e: web_sys::PointerEvent| {
                                            if mouse(&e) {
                                                return;
                                            }
                                            let again = hover_row.with_untracked(|h| h.as_deref() == Some(n4.as_str()));
                                            active_arc.set(None);
                                            hover_row.set((!again).then(|| n4.clone()));
                                            let ring = e
                                                .current_target()
                                                .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                                                .and_then(|el| el.closest(".ov-donut").ok().flatten())
                                                .and_then(|d| d.query_selector(".nr").ok().flatten());
                                            card.set(match ring {
                                                Some(ring) if !again && n4 == OTHERS => Some((OTHERS.into(), pointer_at(&e, &ring))),
                                                _ => None,
                                            });
                                        }
                                    >
                                        <span class="ov-legend-name caps">{p.name.clone()}</span>
                                        <span class="num">{move || fmt_compact(players.get())}</span>
                                        <span class="num muted">{move || fmt_pct(share(players.get()), 0)}</span>
                                    </li>
                                }
                            }
                        </For>
                    </ul>
                </div>
            </Show>
            {move || {
                let (kind, at) = card.get()?;
                let others_n = others.with(|o| o.len());
                let title = if kind == OTHERS { Some("Others".to_string()) } else { arcs.with(|a| a.iter().find(|a| a.key == kind).map(|a| a.label.clone())) }?;
                let fronts_n = (kind != OTHERS).then(|| list.with(|l| l.iter().filter(|p| p.enemy.as_str() == kind).count()));
                let faction = Faction::parse(&kind);
                Some(view! {
                    <PointerCard
                        at=at
                        tone=faction.map_or("var(--text-dim)".to_string(), |f| chart_color(f).to_string())
                        icon=faction.map(|f| view! { <FactionEmblem faction=f.as_str() size=18.0 /> }.into_any()).unwrap_or_else(|| ().into_any())
                        title=title
                        badge=format!("{} fronts", fronts_n.unwrap_or(others_n)).into_any()
                        rows=card_rows(&kind)
                        footer=fronts_n
                            .filter(|n| *n > LIST_SIZE)
                            .map(|n| format!("+{} more · busiest first", n - LIST_SIZE).into_any())
                            .unwrap_or_else(|| ().into_any())
                    />
                })
            }}
            <div class="ov-fronts">
                {ENEMIES
                    .iter()
                    .map(|f| {
                        let f = *f;
                        let value = move || enemy_value(f, &fronts.get());
                        view! {
                            <div
                                class=move || format!("ov-front {}", if arc.with(|a| a.as_ref().is_some_and(|a| a.key == f.as_str())) { "on" } else { "" })
                                style=format!("--k: {}; --t: var(--f-{});", chart_color(f), f.as_str().to_lowercase())
                                on:pointerenter=move |e| {
                                    if mouse(&e) {
                                        active_arc.set(Some(f.as_str().into()))
                                    }
                                }
                                on:pointerleave=move |e| {
                                    if mouse(&e) {
                                        active_arc.set(None)
                                    }
                                }
                                on:pointerup=move |e| {
                                    if mouse(&e) {
                                        return;
                                    }
                                    hover_row.set(None);
                                    card.set(None);
                                    let again = active_arc.with_untracked(|a| a.as_deref() == Some(f.as_str()));
                                    active_arc.set((!again).then(|| f.as_str().into()));
                                }
                            >
                                <FactionEmblem faction=f.as_str() size=18.0 />
                                <span class="caps">{faction_label(Some(f))}</span>
                                <b class="num">{move || fmt_int(value())}</b>
                                <span class="num muted">{move || fmt_pct(share(value()), 1)}</span>
                                <div class="ov-front-bar">
                                    <span style=move || format!("width: {}%;", num(share(value())))></span>
                                </div>
                            </div>
                        }
                    })
                    .collect_view()}
                <div
                    class=move || format!("ov-front {}", if arc.with(|a| a.as_ref().is_some_and(|a| a.key == ELSEWHERE)) { "on" } else { "" })
                    style="--k: var(--faint);"
                    on:pointerenter=move |e| {
                        if mouse(&e) {
                            active_arc.set(Some(ELSEWHERE.into()))
                        }
                    }
                    on:pointerleave=move |e| {
                        if mouse(&e) {
                            active_arc.set(None)
                        }
                    }
                    on:pointerup=move |e| {
                        if mouse(&e) {
                            return;
                        }
                        hover_row.set(None);
                        card.set(None);
                        let again = active_arc.with_untracked(|a| a.as_deref() == Some(ELSEWHERE));
                        active_arc.set((!again).then(|| ELSEWHERE.into()));
                    }
                >
                    <FactionEmblem faction="Humans" size=18.0 color="var(--text-dim)" />
                    <span class="caps">"Elsewhere"</span>
                    <b class="num">{move || fmt_int(elsewhere.get())}</b>
                    <span class="num muted">{move || fmt_pct(share(elsewhere.get()), 1)}</span>
                    <div class="ov-front-bar">
                        <span style=move || format!("width: {}%;", num(share(elsewhere.get())))></span>
                    </div>
                </div>
            </div>
            {move || {
                let rows = stats.get();
                (!rows.is_empty()).then(|| view! {
                    <dl class="ov-stats">
                        {rows
                            .into_iter()
                            .map(|(k, v)| view! {
                                <div>
                                    <dt>{k}</dt>
                                    <dd class="num">{v}</dd>
                                </div>
                            })
                            .collect_view()}
                    </dl>
                })
            }}
        </Panel>
    }
}

#[component]
pub fn OverviewPage() -> impl IntoView {
    view! {
        <div class="page ov-page starfield">
            <h1 class="sr-only">"High Command overview"</h1>
            <div class="ov-board">
                <div class="ov-col ov-col-left">
                    <Distribution />
                </div>
                <div class="ov-col ov-col-mid">
                    <MajorOrderList foldable=true />
                    <WarActivity />
                </div>
                <div class="ov-col ov-col-right">
                    <TheaterOfWar />
                    <CampaignPanel />
                </div>
            </div>
        </div>
    }
}
