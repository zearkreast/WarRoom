//! The planet detail overlay: sphere, header and every section.

use crate::api::fronts::{Front, FrontKind, build_front, major_order_planets, planet_rate, station_planets};
use crate::api::history::PlayerReading;
use crate::api::queries::{self, queries};
use crate::features::planet::activity::ActivitySection;
use crate::features::planet::defense::{advance_label, use_attackers, use_enemy_advance};
use crate::features::planet::globe::SuperEarthGlobe;
use crate::features::planet::record::{UNREACHABLE, use_planet_record};
use crate::features::planet::sections::{
    DispatchSection, FrontLineSection, LiberationSection, OperationsSection, PoisSection, ServiceRecordSection,
    SupplyLinesSection, WarHistorySection,
};
use crate::features::planet::tint::planet_tint;
use crate::js::{self, num};
use crate::state::{HellpadModule, settings, ui};
use crate::ui::emblems::{DssIcon, FactionEmblem};
use crate::ui::feed_status::FeedTag;
use crate::ui::icons::{self, Icon};
use crate::ui::primitives::{CtaButton, Loading, Overlay};
use leptos::prelude::*;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::f64::consts::PI;
use std::sync::Arc;
use warroom_core::api::{Faction, Planet};
use warroom_core::factions::{faction_color, faction_label};
use warroom_core::format::{fmt_int, fmt_pct, sest_clock, sest_date};
use warroom_core::progress::tracked_percent;

/// Planet sphere ringed by the liberation split: Super Earth arc vs the enemy's. In a defense, two rings like the
/// game's two bars: Super Earth's defense outside the enemy's advance.
#[component]
pub fn PlanetSphere(
    #[prop(into)] planet: Signal<Planet>,
    #[prop(into)] percent: Signal<f64>,
    #[prop(into)] enemy: Signal<String>,
    #[prop(default = 220.0)] size: f64,
) -> impl IntoView {
    let look = Memo::new(move |_| {
        planet.with(|p| {
            let t = planet_tint(p, &[]);
            (p.index, t.sky, t.ground)
        })
    });
    let advance = use_enemy_advance(planet);
    let defended = Memo::new(move |_| advance.with(Option::is_some));
    let arc = |r: f64, pct: f64| {
        let c = 2.0 * PI * r;
        format!("{} {}", num(pct.clamp(0.0, 100.0) / 100.0 * c), num(c))
    };
    move || {
        let (index, sky, ground) = look.get();
        let id = format!("sph-{index}");
        let rings = if defended.get() {
            view! {
                <circle r="105" class="sphere-track" stroke-width="8" />
                <circle
                    r="105"
                    fill="none"
                    stroke="var(--f-humans)"
                    stroke-width="8"
                    stroke-dasharray=move || arc(105.0, percent.get())
                    transform="rotate(-90)"
                    class="sphere-arc"
                />
                <circle r="94" class="sphere-track" stroke-width="6" />
                <circle
                    r="94"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="6"
                    stroke-dasharray=move || arc(94.0, advance.get().unwrap_or(0.0))
                    transform="rotate(-90)"
                    style=move || format!("color: {};", enemy.get())
                    class="sphere-arc foe"
                />
            }
            .into_any()
        } else {
            view! {
                <circle r="100" fill="none" stroke=enemy stroke-width="9" opacity="0.9" />
                <circle
                    r="100"
                    fill="none"
                    stroke="var(--f-humans)"
                    stroke-width="9"
                    stroke-dasharray=move || arc(100.0, percent.get())
                    transform="rotate(-90)"
                    class="sphere-arc"
                />
            }
            .into_any()
        };
        view! {
            <svg class="sphere" width=num(size) height=num(size) viewBox="-120 -120 240 240" aria-hidden="true">
                <defs>
                    <radialGradient id=id.clone() cx="35%" cy="30%" r="80%">
                        <stop offset="0%" stop-color=ground stop-opacity="0.95" />
                        <stop offset="55%" stop-color=ground stop-opacity="0.55" />
                        <stop offset="100%" stop-color=sky />
                    </radialGradient>
                    <clipPath id=format!("{id}-clip")>
                        <circle r="84" />
                    </clipPath>
                </defs>
                <circle r="114" class="sphere-dots" />
                {rings}
                <circle r="88" fill="var(--void)" />
                <g clip-path=format!("url(#{id}-clip)")>
                    {if index == 0 {
                        view! { <SuperEarthGlobe r=84.0 /> }.into_any()
                    } else {
                        view! {
                            <circle r="84" fill=format!("url(#{id})") />
                            {[-60.0f64, -30.0, 0.0, 30.0, 60.0]
                                .map(|y| view! { <ellipse cx="0" cy=num(y) rx=num((84.0 * 84.0 - y * y).sqrt()) ry="6" class="sphere-grid" /> })
                                .collect_view()}
                            {[-50.0f64, -20.0, 20.0, 50.0]
                                .map(|x| view! { <ellipse cx="0" cy="0" rx=num(x.abs()) ry="84" class="sphere-grid" /> })
                                .collect_view()}
                        }
                        .into_any()
                    }}
                    <circle r="84" fill="url(#sphere-shade)" />
                </g>
                <defs>
                    <radialGradient id="sphere-shade" cx="70%" cy="75%" r="75%">
                        <stop offset="40%" stop-color="#000" stop-opacity="0" />
                        <stop offset="100%" stop-color="#000" stop-opacity="0.75" />
                    </radialGradient>
                </defs>
            </svg>
        }
    }
}

fn status_line(p: &Planet, front: Option<&Front>, from: &[Planet]) -> String {
    let owner = faction_label(Some(p.current_owner));
    match front {
        Some(f) if f.kind == FrontKind::Defense => {
            let foe = faction_label(Some(f.enemy));
            if from.is_empty() {
                format!("Defense campaign · {foe} attacking")
            } else {
                let names: Vec<&str> = from.iter().map(|a| a.name.as_str()).collect();
                format!("Defense campaign · {foe} attacking from {}", names.join(", "))
            }
        }
        Some(_) => format!("Liberation campaign · {owner} control"),
        None if p.current_owner == Faction::Humans => "Secured · Super Earth control".into(),
        None => format!("{owner} control"),
    }
}

#[component]
fn Hero(p: Signal<Planet>, front: Memo<Option<Front>>) -> impl IntoView {
    let stations = station_planets();
    let mo = major_order_planets();
    let now = js::use_now(1000.0);
    let attackers = use_attackers(p);
    let advance = use_enemy_advance(p);
    let enemy_faction = Memo::new(move |_| {
        front
            .with(|f| f.as_ref().map(|f| f.enemy))
            .or_else(|| p.with(|p| (p.current_owner != Faction::Humans).then_some(p.current_owner)))
    });
    let percent = Signal::derive(move || {
        if enemy_faction.get().is_none() {
            return 100.0;
        }
        front.with(|f| f.as_ref().map(|f| f.percent)).unwrap_or_else(|| p.with(tracked_percent))
    });
    let enemy = Signal::derive(move || faction_color(enemy_faction.get()).to_string());
    let style = move || {
        let tone = faction_color(front.with(|f| f.as_ref().map(|f| f.enemy)).or(Some(p.with(|p| p.current_owner))));
        let t = p.with(|p| planet_tint(p, &[]));
        format!(
            "--tone: {tone}; --sky: {}; --ground: {};{}",
            t.sky,
            t.ground,
            t.art.map(|a| format!(" --art: {a};")).unwrap_or_default()
        )
    };
    let index = move || p.with(|p| p.index);
    let sector = move || p.with(|p| p.sector.clone());
    let name = move || p.with(|p| p.name.clone());
    view! {
        <section class="pd-hero brackets" style=style>
            <div class="pd-hero-main">
                <nav class="pd-crumbs caps" aria-label="Breadcrumb">
                    <button class="link" on:click=move |_| ui().open_module(Some(HellpadModule::Planets))>
                        "Planets"
                    </button>
                    <span>"/"</span>
                    <span>{move || format!("{} sector", sector())}</span>
                </nav>
                <div class="pd-name-row">
                    {move || {
                        let f = front.with(|f| f.as_ref().map(|f| f.enemy)).unwrap_or_else(|| p.with(|p| p.current_owner));
                        view! { <FactionEmblem faction=f.as_str() size=46.0 /> }
                    }}
                    <h1 class="pd-name">{name}</h1>
                </div>
                <div class="pd-status caps">
                    {move || attackers.with(|from| p.with(|p| status_line(p, front.get().as_ref(), from)))}
                </div>
                <div class="pd-chips">
                    <span class="chip">
                        <Icon icon=icons::COMPASS size=13.0 />
                        {move || format!(" {} sector", sector())}
                    </span>
                    {move || {
                        let biome = p.with(|p| p.biome.name.clone());
                        (!biome.is_empty()).then(|| view! {
                            <span class="chip">
                                <Icon icon=icons::MOUNTAIN size=13.0 />
                                {format!(" {biome}")}
                            </span>
                        })
                    }}
                    <span class="chip num">
                        <Icon icon=icons::USERS size=13.0 />
                        {move || format!(" {} ", fmt_int(p.with(|p| p.statistics.player_count)))}
                        <span class="muted">"deployed"</span>
                    </span>
                    <Show when=move || stations.with(|s| s.contains(&index()))>
                        <button class="chip chip-dss" on:click=move |_| ui().open_module(Some(HellpadModule::Stations))>
                            <DssIcon size=13.0 />
                            " Democracy Space Station"
                        </button>
                    </Show>
                    <Show when=move || mo.with(|m| m.contains(&index()))>
                        <span class="chip chip-mo">
                            <Icon icon=icons::TARGET size=13.0 />
                            " Major Order target"
                        </span>
                    </Show>
                </div>
                <CtaButton icon=icons::MAP_PIN on_click=Callback::new(move |_| ui().show_on_map(index()))>
                    "Show on map"
                </CtaButton>
            </div>
            <div class="pd-hero-sphere">
                <PlanetSphere planet=p percent=percent enemy=enemy />
                {move || {
                    let foe = advance.get()?;
                    Some(view! {
                        <div class="pd-legend pd-rings num">
                            <span style="--k: var(--f-humans);">{format!("Defended {}", fmt_pct(percent.get(), 1))}</span>
                            <span style=format!("--k: {};", enemy.get())>
                                {format!("{} {}", enemy_faction.get().map_or("Enemy advance".to_string(), advance_label), fmt_pct(foe, 0))}
                            </span>
                        </div>
                    })
                }}
            </div>
            <footer class="pd-hero-foot caps">
                <span>{move || format!("Satcom / {} / {}", sector(), name())}</span>
                <span class="pd-live">
                    <FeedTag />
                    <span class="num">{move || format!("SEST {} {}", sest_date(now.get()), sest_clock(now.get()))}</span>
                </span>
            </footer>
        </section>
    }
}

#[component]
fn Body(index: i64) -> impl IntoView {
    let q = queries();
    let planet = Memo::new(move |_| queries::planet(Some(index)));
    let mo = major_order_planets();
    let stations = station_planets();
    let record = use_planet_record(index);
    let front = Memo::new(move |_| {
        let rate = planet_rate(Some(index));
        q.campaigns.with(|c| {
            let c = c.as_ref()?.iter().find(|c| c.planet.index == index)?;
            Some(build_front(&c.campaign, rate, &mo.get(), js::now(), &HashSet::new(), &stations.get()))
        })
    });
    let ready = Memo::new(move |_| planet.with(Option::is_some));
    let has_front = Memo::new(move |_| front.with(Option::is_some));
    view! {
        <Show when=move || ready.get() fallback=|| view! { <Loading what="planet record" /> }>
            {move || {
                let p = Signal::derive(move || planet.get().unwrap_or_default());
                view! {
                    <div class="pd">
                        <Hero p=p front=front />
                        <Show
                            when=move || has_front.get()
                            fallback=move || {
                                view! {
                                    <ActivitySection
                                        title="Helldiver activity · last 24 h".to_string()
                                        now=Signal::derive(move || p.with(|p| p.statistics.player_count))
                                        points=Signal::derive(move || {
                                            record.with(|r| {
                                                let mut points: Vec<PlayerReading> = r
                                                    .samples
                                                    .iter()
                                                    .map(|s| PlayerReading { at: s.at, players: s.players })
                                                    .chain(r.idle.iter().copied())
                                                    .collect();
                                                points.sort_by(|a, b| a.at.partial_cmp(&b.at).unwrap_or(Ordering::Equal));
                                                Arc::new(points)
                                            })
                                        })
                                        loading=Signal::derive(move || record.with(|r| r.loading))
                                        empty=Signal::derive(move || {
                                            if record.with(|r| r.error) {
                                                return UNREACHABLE.to_string();
                                            }
                                            format!(
                                                "No Helldivers recorded on {} in the last 24 hours. A reading is kept whenever any are there.",
                                                p.with(|p| p.name.clone())
                                            )
                                        })
                                    />
                                }
                            }
                        >
                            <LiberationSection p=p front=front />
                            <FrontLineSection p=p front=front record=record />
                        </Show>
                        <ServiceRecordSection p=p />
                        <WarHistorySection p=p record=record />
                        <div class="pd-cols">
                            <DispatchSection
                                p=p
                                front=front
                                in_major_order=Signal::derive(move || mo.with(|m| m.contains(&index)))
                                station=Signal::derive(move || stations.with(|s| s.contains(&index)))
                            />
                            <OperationsSection p=p />
                        </div>
                        <PoisSection p=p />
                        <SupplyLinesSection p=p />
                    </div>
                }
            }}
        </Show>
    }
}

#[component]
pub fn PlanetDetail() -> impl IntoView {
    let detail = ui().detail;
    move || {
        let index = detail.get()?;
        let pinned = Memo::new(move |_| settings().get(|s| s.pinned.contains(&index)));
        Some(view! {
            <Overlay
                on_close=Callback::new(|_| ui().open_detail(None))
                label="Planet details"
                footer=view! {
                    <button
                        class=move || format!("hud-btn pd-pin {}", if pinned.get() { "on" } else { "" })
                        on:click=move |_| settings().toggle_pin(index)
                    >
                        {move || {
                            let on = pinned.get();
                            view! {
                                <Icon icon=if on { icons::PIN_OFF } else { icons::PIN } size=15.0 />
                                {if on { " Unpin" } else { " Pin to top" }}
                            }
                        }}
                    </button>
                }
                .into_any()
            >
                <Body index=index />
            </Overlay>
        })
    }
}
