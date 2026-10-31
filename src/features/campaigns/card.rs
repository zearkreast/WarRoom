//! A front's card: progress, pace, ETA or deadline, Helldivers and badges.

use crate::api::fronts::{Front, FrontKind, fronts};
use crate::features::planet::defense::{GambitMark, advance_label, use_attackers, use_enemy_advance, use_gambits};
use crate::features::planet::tint::planet_tint;
use crate::js;
use crate::state::{OptionsAnchor, ui};
use crate::ui::emblems::{DssIcon, FactionEmblem};
use crate::ui::hazard_icons::hazard_icon;
use crate::ui::icons::{self, Icon};
use crate::ui::primitives::{Countdown, Meter, MeterSize};
use leptos::prelude::*;
use warroom_core::api::{Faction, Named, Planet};
use warroom_core::biomes::regen_pct_per_hour;
use warroom_core::defense::{GambitOutlook, divers_short, event_span, needed_rate};
use warroom_core::factions::{faction_color, faction_label};
use warroom_core::format::{fmt_duration, fmt_int, fmt_pct, fmt_time_left, to_fixed};

/// What a card shows that rarely changes; the card is rebuilt only when it does.
#[derive(Clone, PartialEq)]
struct Look {
    index: i64,
    name: String,
    sector: String,
    biome: String,
    hazards: Vec<Named>,
    kind: FrontKind,
    enemy: Faction,
    style: String,
}

#[derive(Clone, Copy, PartialEq)]
struct EtaView {
    good: bool,
    label: &'static str,
    to: f64,
    late: bool,
}

/// "Victory in …" / "Falls in …" above the card, if the rate says anything.
fn eta(f: &Front, deadline: Option<f64>) -> Option<EtaView> {
    f.rate?;
    if f.kind == FrontKind::Defense {
        let wins = matches!((f.victory_at, f.fail_at), (Some(v), Some(d)) if v < d);
        let to = if wins { f.victory_at } else { f.fail_at };
        return Some(EtaView {
            good: wins,
            label: if wins { "Defended in" } else { "Falls in" },
            to: to.unwrap_or(0.0),
            late: false,
        });
    }
    if let Some(v) = f.victory_at.filter(|v| *v != 0.0 && !v.is_nan()) {
        return Some(EtaView { good: true, label: "Victory in", to: v, late: deadline.is_some_and(|d| v > d) });
    }
    if let Some(d) = f.fail_at.filter(|d| *d != 0.0 && !d.is_nan()) {
        return Some(EtaView { good: false, label: "Withdrawn in", to: d, late: false });
    }
    None
}

/// The enemy's side of a defense: its advance, and the ms since the attack began and until its deadline.
#[derive(Clone, Copy, PartialEq)]
struct Advance {
    enemy: Faction,
    percent: f64,
    since: f64,
    left: f64,
}

#[derive(Clone, Copy, PartialEq)]
struct OutlookView {
    holding: bool,
    percent: f64,
    advance: Option<Advance>,
    rate: Option<f64>,
}

fn advance_view(a: Advance) -> impl IntoView {
    view! {
        <span class="cc-advance" title=format!("Under attack for {}", fmt_duration(a.since))>
            <span class="caps">{advance_label(a.enemy)}</span>
            <b class="num caps">{format!("{} · {} left", fmt_pct(a.percent, 0), fmt_time_left(a.left))}</b>
        </span>
    }
}

fn outlook_view(o: OutlookView) -> AnyView {
    if o.holding {
        return view! {
            <div class="cc-outlook holding caps">
                <Icon icon=icons::USERS size=16.0 />
                " Holding for reinforcement"
            </div>
        }
        .into_any();
    }
    view! {
        <div class="cc-outlook">
            {match o.rate {
                None => view! { <span class="cc-rate measuring caps">"Measuring rate…"</span> }.into_any(),
                Some(r) => {
                    view! {
                        <span class=format!("cc-rate {}", if r > 0.0 { "up" } else { "down" })>
                            <Icon icon=if r < 0.0 { icons::TRENDING_DOWN } else { icons::TRENDING_UP } size=18.0 />
                            <b class="num">{format!("{}{}%", if r > 0.0 { "+" } else { "" }, to_fixed(r, 2))}</b>
                            <span class="caps">"per hour"</span>
                        </span>
                    }
                    .into_any()
                }
            }}
            {o.advance.map(advance_view)}
        </div>
    }
    .into_any()
}

/// " · from USTOTU": the planets a defense is attacked from, in their owner's colour.
fn attacked_from(list: &[Planet]) -> Option<AnyView> {
    (!list.is_empty()).then(|| {
        let names = list
            .iter()
            .enumerate()
            .map(|(i, a)| {
                view! {
                    {(i > 0).then_some(if i == list.len() - 1 { " and " } else { ", " })}
                    <b style=format!("color: {};", faction_color(Some(a.current_owner)))>{a.name.clone()}</b>
                }
            })
            .collect_view();
        view! { " · from " {names} }.into_any()
    })
}

/// The game's own campaign-type glyph, tinted with the text colour.
fn kind_icon(kind: FrontKind) -> impl IntoView {
    let name = if kind == FrontKind::Defense { "campaign-defense" } else { "campaign-liberation" };
    let url = crate::assets::url(&format!("gww/icons/{name}.svg")).unwrap_or_default();
    view! { <span class="cc-kind-icon" style=format!("--icon: url({url});") aria-hidden="true"></span> }
}

#[component]
pub fn CampaignCard(
    id: i64,
    #[prop(into)] total_players: Signal<f64>,
    #[prop(into)] deadline: Signal<Option<f64>>,
) -> impl IntoView {
    let all = fronts();
    let front = Memo::new(move |_| all.with(|l| l.iter().find(|f| f.campaign.id == id).cloned()));
    let look = Memo::new(move |_| {
        front.with(|f| {
            let f = f.as_ref()?;
            let p = &f.planet;
            let tint = planet_tint(p, &[]);
            let style = format!(
                "--enemy: {}; --sky: {}; --ground: {};{}",
                faction_color(Some(f.enemy)),
                tint.sky,
                tint.ground,
                tint.art.map(|a| format!(" --art: {a};")).unwrap_or_default()
            );
            Some(Look {
                index: p.index,
                name: p.name.clone(),
                sector: p.sector.clone(),
                biome: p.biome.name.clone(),
                hazards: p.hazards.iter().take(4).cloned().collect(),
                kind: f.kind,
                enemy: f.enemy,
                style,
            })
        })
    });
    let get = move |pick: fn(&Front) -> f64| move || front.with(|f| f.as_ref().map_or(0.0, pick));
    let percent = Signal::derive(get(|f| f.percent));
    let players = Signal::derive(get(|f| f.players));
    let regen = Signal::derive(move || {
        front.with(|f| {
            f.as_ref()
                .filter(|f| f.kind == FrontKind::Liberation)
                .map(|f| regen_pct_per_hour(f.planet.regen_per_second, f.planet.max_health))
        })
    });
    let projected = Signal::derive(move || front.with(|f| f.as_ref().and_then(|f| f.projected)));
    let rate = Memo::new(move |_| front.with(|f| f.as_ref().and_then(|f| f.rate)));
    let flag = move |pick: fn(&Front) -> bool| Memo::new(move |_| front.with(|f| f.as_ref().is_some_and(pick)));
    let (in_mo, pinned, station) = (flag(|f| f.in_major_order), flag(|f| f.pinned), flag(|f| f.station));
    let eta = Memo::new(move |_| front.with(|f| f.as_ref().and_then(|f| eta(f, deadline.get()))));
    let planet = Memo::new(move |_| front.with(|f| f.as_ref().map(|f| f.planet.clone()).unwrap_or_default()));
    let advance = use_enemy_advance(planet.into());
    let attackers = use_attackers(planet.into());
    let gambits = use_gambits(planet.into());
    let clock = js::use_now(30_000.0);
    let outlook = Memo::new(move |_| {
        let now = clock.get();
        let advance = advance.get();
        front.with(|f| {
            f.as_ref().map(|f| OutlookView {
                holding: f.rate.is_none() && f.kind == FrontKind::Liberation && f.percent == 0.0,
                percent: f.percent,
                advance: advance.zip(f.planet.event.as_ref().and_then(event_span)).map(|(percent, (start, end))| {
                    Advance { enemy: f.enemy, percent, since: now - start, left: end - now }
                }),
                rate: f.rate,
            })
        })
    });
    let needed = Memo::new(move |_| {
        let now = clock.get();
        front.with(|f| {
            let f = f.as_ref().filter(|f| f.kind == FrontKind::Defense)?;
            let need = needed_rate(f.percent, now, f.fail_at?)?;
            Some((need, divers_short(f.rate, need, f.players)))
        })
    });
    let needed_tone = move || {
        let k = match (needed.get(), rate.get()) {
            (Some((n, _)), Some(r)) if r >= n => "var(--good)",
            (Some(_), Some(_)) => "var(--bad)",
            _ => "var(--text-hi)",
        };
        format!("--k: {k};")
    };
    let needed_title = move || {
        let base = "Progress per hour needed to win the defense before its deadline";
        match needed.get() {
            Some((_, Some(n))) if n <= 0.0 => format!("{base}; the measured pace is enough"),
            Some((_, Some(n))) => {
                format!("{base}; roughly {} more Helldivers at today's progress per Helldiver (estimate)", fmt_int(n))
            }
            _ => base.to_string(),
        }
    };
    let gambit = Memo::new(move |_| {
        gambits.with(|list| {
            list.iter().find(|g| g.outlook == GambitOutlook::Ahead).map(|g| {
                let from: Vec<&str> = g.attackers.iter().map(|a| a.planet.name.as_str()).collect();
                format!(
                    "Gambit: liberating {} ends the defense of {} before its deadline",
                    from.join(" and "),
                    g.defended.name
                )
            })
        })
    });
    let share = Signal::derive(move || {
        let total = total_players.get();
        if total != 0.0 && !total.is_nan() { players.get() / total * 100.0 } else { 0.0 }
    });
    move || {
        let l = look.get()?;
        let enemy = faction_color(Some(l.enemy)).to_string();
        let foe = enemy.clone();
        let kind = l.kind.as_str();
        let index = l.index;
        let aria = format!("{}, {kind} against {}. Open options", l.name, faction_label(Some(l.enemy)));
        Some(view! {
            <article class="cc" style=l.style.clone()>
                <button
                    class=move || format!("cc-card {}", if in_mo.get() { "is-mo" } else { "" })
                    on:click=move |e| {
                        ui().open_options(Some(OptionsAnchor::at(index, &e)))
                    }
                    aria-label=aria
                >
                    <div class="cc-in">
                        <div class="cc-kicker caps">
                            <span class="cc-type">{kind_icon(l.kind)} {kind.to_string()}</span>
                            {move || {
                                eta.get().map(|e| {
                                    view! {
                                        <span class=format!("cc-eta {}", if e.good { "is-good" } else { "is-bad" })>
                                            {format!("{} ", e.label)}
                                            <b><Countdown to=e.to /></b>
                                        </span>
                                    }
                                })
                            }}
                        </div>
                        <header class="cc-head">
                            <FactionEmblem faction=l.enemy.as_str() size=28.0 />
                            <div class="cc-title">
                                <h3 class="cc-name">{l.name.clone()}</h3>
                                <div class="cc-sector caps">
                                    {format!("{} sector", l.sector)}
                                    {move || attackers.with(|list| attacked_from(list))}
                                </div>
                            </div>
                            <div class="cc-pct">
                                <b class="num">{move || outlook.get().map(|o| fmt_pct(o.percent, 2)).unwrap_or_default()}</b>
                                <span class="caps">{if l.kind == FrontKind::Defense { "defended" } else { "liberated" }}</span>
                            </div>
                        </header>
                        <div class="cc-art">
                            <div class="cc-badges">
                                <Show when=move || in_mo.get()>
                                    <span class="cc-badge is-mo" title="Major Order target">
                                        <Icon icon=icons::TARGET size=14.0 />
                                        "MO"
                                    </span>
                                </Show>
                                <Show when=move || station.get()>
                                    <span class="cc-badge is-dss" title="Democracy Space Station in orbit">
                                        <DssIcon size=14.0 />
                                        "DSS"
                                    </span>
                                </Show>
                                <Show when=move || pinned.get()>
                                    <span class="cc-badge is-pin" title="Pinned">
                                        <Icon icon=icons::PIN size=14.0 label="Pinned" />
                                    </span>
                                </Show>
                                {move || {
                                    gambit
                                        .get()
                                        .map(|t| {
                                            view! {
                                                <span class="cc-badge is-gambit" title=t>
                                                    <GambitMark size=16.0 />
                                                    "Gambit"
                                                </span>
                                            }
                                        })
                                }}
                                {move || {
                                    eta.get().filter(|e| e.late).map(|_| view! { <span class="cc-badge is-late">"After MO deadline"</span> })
                                }}
                            </div>
                            <span class="cc-biome caps">{l.biome.clone()}</span>
                            <div class="cc-hazards">
                                {l
                                    .hazards
                                    .iter()
                                    .map(|h| {
                                        view! {
                                            <span class="cc-hazard" title=format!("{}: {}", h.name, h.description)>
                                                <Icon icon=hazard_icon(&h.name) size=14.0 />
                                            </span>
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        </div>
                        <div class="cc-meter">
                            <Meter
                                value=percent
                                color="var(--f-humans)".to_string()
                                rest=enemy.clone()
                                marker=projected
                                size=MeterSize::L
                                chevron=true
                                label=format!("{} progress", l.name)
                            />
                            <Show when=move || advance.with(Option::is_some)>
                                <div class="cc-foe">
                                    <Meter
                                        value=Signal::derive(move || advance.get().unwrap_or(0.0))
                                        color=foe.clone()
                                        rest="transparent".to_string()
                                        size=MeterSize::S
                                        label=advance_label(l.enemy)
                                    />
                                </div>
                            </Show>
                        </div>
                        {move || outlook.get().map(outlook_view)}
                        <footer class="cc-foot">
                            <span class="cc-share" title="Share of all Helldivers online">
                                <span class="cc-foot-label caps">"Share"</span>
                                <b class="num">{move || format!("{}%", to_fixed(share.get(), 0))}</b>
                            </span>
                            <span title="Helldivers deployed">
                                <span class="cc-foot-label caps">"Divers"</span>
                                <b class="num">{move || fmt_int(players.get())}</b>
                            </span>
                            <span title="Super Earth progress per hour, before enemy regeneration" style="--k: var(--f-humans);">
                                <span class="cc-foot-label caps">"Impact"</span>
                                <b class="num">
                                    {move || {
                                        rate.get().map_or("—".to_string(), |r| format!("{}%", to_fixed(r + regen.get().unwrap_or(0.0), 2)))
                                    }}
                                </b>
                            </span>
                            {if l.kind == FrontKind::Defense {
                                view! {
                                    <span title=needed_title style=needed_tone>
                                        <span class="cc-foot-label caps">"Needed"</span>
                                        <b class="num">
                                            {move || needed.get().map_or("—".to_string(), |(n, _)| format!("{}%", to_fixed(n, 2)))}
                                        </b>
                                    </span>
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <span title="Enemy regeneration per hour" style=format!("--k: {enemy};")>
                                        <span class="cc-foot-label caps">"Regen"</span>
                                        <b class="num">
                                            {move || regen.get().map_or("—".to_string(), |r| format!("{}%", to_fixed(r, 2)))}
                                        </b>
                                    </span>
                                }
                                    .into_any()
                            }}
                        </footer>
                    </div>
                </button>
            </article>
        })
    }
}
