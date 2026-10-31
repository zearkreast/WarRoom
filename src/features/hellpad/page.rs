//! The Hellpad page: the masthead, the module tiles, and the open module.

use crate::api::queries::queries;
use crate::features::hellpad::call_in::{CallInCode, CallInWait, MarkPad, use_call_in};
use crate::features::hellpad::gww::GwwModule;
use crate::features::hellpad::past_campaigns::PastCampaignsModule;
use crate::features::hellpad::planets::PlanetsModule;
use crate::features::hellpad::stations::StationsModule;
use crate::features::hellpad::stratagems::StratagemsModule;
use crate::features::map::stratagem_art::permit_color;
use crate::js::{self, num};
use crate::state::{HellpadModule, settings, ui};
use crate::ui::emblems::{FactionEmblem, HellpadMark, IconLike, icon_view};
use crate::ui::feed_status::{FeedState, use_feed_status};
use crate::ui::icons::{self, Icon, IconData};
use crate::ui::primitives::{Countdown, LiveDot, StatusChip};
use leptos::html;
use leptos::prelude::*;
use warroom_core::data::manuals::MANUALS;
use warroom_core::factions::{faction_color, faction_label};
use warroom_core::format::{fmt_int, js_round};
use warroom_core::planet_index::control_shares;
use warroom_core::snapshot::EpisodeStatus;
use warroom_core::stratagems::{PERMITS, STRATAGEMS};
use warroom_core::time::parse_ms;
use wasm_bindgen::JsCast;

struct ModuleInfo {
    title: &'static str,
    path: &'static str,
    color: &'static str,
}

fn module_info(m: HellpadModule) -> ModuleInfo {
    match m {
        HellpadModule::Planets => ModuleInfo { title: "Planets", path: "hp://planets", color: "var(--f-humans)" },
        HellpadModule::Stations => {
            ModuleInfo { title: "Space stations", path: "hp://stations", color: "var(--impact)" }
        }
        HellpadModule::Gww => ModuleInfo { title: "G.W.W.", path: "hp://gww", color: "var(--cta)" },
        HellpadModule::PastCampaigns => {
            ModuleInfo { title: "Past campaigns", path: "hp://archive", color: "var(--warn)" }
        }
        HellpadModule::Stratagems => {
            ModuleInfo { title: "Stratagem collection", path: "hp://stratagems", color: "var(--p-offensive)" }
        }
    }
}

fn module_key(m: HellpadModule) -> &'static str {
    match m {
        HellpadModule::Planets => "planets",
        HellpadModule::Stations => "stations",
        HellpadModule::Gww => "gww",
        HellpadModule::PastCampaigns => "campaigns",
        HellpadModule::Stratagems => "stratagems",
    }
}

#[component]
fn Uplink() -> impl IntoView {
    let f = use_feed_status(15_000.0);
    move || match f.state.get() {
        FeedState::Offline => view! { <StatusChip tone="var(--bad)">"Uplink lost"</StatusChip> }.into_any(),
        FeedState::Stale => view! { <StatusChip tone="var(--warn)">"Uplink stale"</StatusChip> }.into_any(),
        FeedState::Live => view! {
            <span class="hp-uplink caps">
                <LiveDot />
                " Uplink live"
            </span>
        }
        .into_any(),
    }
}

#[component]
fn Mast() -> impl IntoView {
    let q = queries();
    let call_in = use_call_in();
    view! {
        <header class="hp-mast">
            <span class="hp-mark-box">
                <HellpadMark size=64.0 class="hp-mark" />
                <MarkPad c=call_in />
            </span>
            <div class="hp-mast-text">
                <h1 class="hp-mast-title">"Hellpad"</h1>
                <p class="caps">"Field terminal · Super Destroyer uplink"</p>
                <div class="hp-mast-chips num">
                    <Uplink />
                    {move || q.campaigns.with(|c| c.as_ref().map(|c| c.len())).map(|n| view! { <span>{format!("{n} fronts open")}</span> })}
                    {move || {
                        q.war
                            .with(|w| w.as_ref().map(|w| w.statistics.player_count))
                            .map(|n| view! { <span>{format!("{} Helldivers deployed", fmt_int(n))}</span> })
                    }}
                </div>
            </div>
            <CallInCode c=call_in />
            <CallInWait c=call_in />
        </header>
    }
}

/// One line on where the Democracy Space Station is and when it jumps next.
#[component]
fn StationReport() -> impl IntoView {
    let q = queries();
    let now = js::use_now(30_000.0);
    let report = Memo::new(move |_| {
        let s = q.stations.with(|s| s.as_ref().and_then(|s| s.first().cloned()))?;
        let p = q.fresh_planets.with(|p| {
            p.as_ref()?.iter().find(|p| p.index == s.planet_index).map(|p| (p.name.clone(), p.sector.clone()))
        })?;
        Some((p.0, p.1, parse_ms(&s.election_end)))
    });
    move || {
        let (name, sector, jump) = report.get()?;
        Some(view! {
            <button class="hp-report" on:click=move |_| ui().open_module(Some(HellpadModule::Stations))>
                <span class="hp-report-label caps">
                    <Icon icon=icons::RADIO size=14.0 />
                    " Station report"
                </span>
                <span class="hp-report-text">
                    "The DSS holds orbit over "
                    <b>{name}</b>
                    {format!(", {sector} sector. ")}
                    {move || {
                        if jump > now.get() {
                            view! {
                                "Next FTL jump in "
                                <b class="num">
                                    <Countdown to=jump />
                                </b>
                                "."
                            }
                            .into_any()
                        } else {
                            "The destination vote is closing.".into_any()
                        }
                    }}
                </span>
                <Icon icon=icons::CHEVRON_RIGHT size=16.0 />
            </button>
        })
    }
}

#[component]
fn ModuleTile(
    id: HellpadModule,
    icon: IconLike,
    #[prop(into)] text: String,
    #[prop(into)] meta: Signal<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let m = module_info(id);
    view! {
        <button
            class="hp-mod"
            data-module=module_key(id)
            style=format!("--mod: {};", m.color)
            on:click=move |_| ui().open_module(Some(id))
        >
            <span class="hp-mod-icon">{icon_view(icon, 26.0, 1.5, None)}</span>
            <span class="hp-mod-body">
                <span class="hp-mod-path num">{m.path}</span>
                <b class="hp-mod-title">{m.title}</b>
                <span class="hp-mod-text">{text}</span>
                {children.map(|c| c())}
                <span class="hp-mod-go caps">
                    {move || format!("{} ", meta.get())}
                    <Icon icon=icons::CHEVRON_RIGHT size=14.0 />
                </span>
            </span>
        </button>
    }
}

#[component]
fn LockedTile(icon: IconData, title: &'static str, path: &'static str, text: &'static str) -> impl IntoView {
    view! {
        <div class="hp-mod locked">
            <span class="hp-mod-icon">
                <Icon icon=icon size=26.0 stroke_width=1.5 />
            </span>
            <span class="hp-mod-body">
                <span class="hp-mod-path num">{format!("hp://{path}")}</span>
                <b class="hp-mod-title">{title}</b>
                <span class="hp-mod-text">{text}</span>
                <span class="hp-mod-go caps">
                    <Icon icon=icons::LOCK size=12.0 />
                    " Module building"
                    <span class="hp-segs" aria-hidden="true">
                        <i class="on"></i>
                        <i class="on"></i>
                        <i></i>
                        <i></i>
                    </span>
                </span>
            </span>
        </div>
    }
}

#[component]
fn PlanetsTile() -> impl IntoView {
    let q = queries();
    let shares = Memo::new(move |_| q.fresh_planets.with(|p| p.as_ref().map(|p| control_shares(p))));
    view! {
        <ModuleTile
            id=HellpadModule::Planets
            icon=icons::GLOBE_2.into()
            text="Every planet by sector, with search, filters and live fronts."
            meta=Signal::derive(move || {
                q.fresh_planets.with(|p| p.as_ref().map_or("Open index".to_string(), |p| format!("{} planets indexed", p.len())))
            })
        >
            {move || {
                shares.get().map(|shares| {
                    let label = format!(
                        "Planet control: {}",
                        shares.iter().map(|s| format!("{} {}%", faction_label(Some(s.faction)), num(js_round(s.pct)))).collect::<Vec<_>>().join(", ")
                    );
                    view! {
                        <span class="hp-control">
                            <span class="hp-control-bar" role="img" aria-label=label>
                                {shares
                                    .iter()
                                    .map(|s| view! { <i style=format!("flex-grow: {}; background: {};", s.planets, faction_color(Some(s.faction)))></i> })
                                    .collect_view()}
                            </span>
                            <span class="hp-control-keys num" aria-hidden="true">
                                {shares
                                    .iter()
                                    .map(|s| view! {
                                        <span style=format!("color: {};", faction_color(Some(s.faction)))>
                                            <FactionEmblem faction=s.faction.as_str() size=12.0 />
                                            {format!(" {}%", num(js_round(s.pct)))}
                                        </span>
                                    })
                                    .collect_view()}
                            </span>
                        </span>
                    }
                })
            }}
        </ModuleTile>
    }
}

#[component]
fn StationsTile() -> impl IntoView {
    let q = queries();
    let count = Memo::new(move |_| q.stations.with(|s| s.as_ref().map_or(0, |s| s.len())));
    let place = Memo::new(move |_| {
        let index = q.stations.with(|s| s.as_ref()?.first().map(|s| s.planet_index))?;
        q.fresh_planets.with(|p| p.as_ref()?.iter().find(|p| p.index == index).map(|p| p.name.clone()))
    });
    view! {
        <ModuleTile
            id=HellpadModule::Stations
            icon=IconLike::Dss
            text="Where the DSS is, its next FTL jump, its tactical actions and the Automaton station it destroyed."
            meta=Signal::derive(move || if count.get() > 0 { format!("{} in orbit", count.get()) } else { "Open".into() })
        >
            {move || place.get().map(|w| view! {
                <span class="hp-mod-live caps">
                    <LiveDot />
                    {format!(" DSS over {w}")}
                </span>
            })}
        </ModuleTile>
    }
}

#[component]
fn GwwTile() -> impl IntoView {
    let read = Memo::new(move |_| settings().get(|s| s.gww_read.clone()));
    let done = Memo::new(move |_| read.with(|r| MANUALS.iter().filter(|m| r.iter().any(|x| x == m.id)).count()));
    view! {
        <ModuleTile
            id=HellpadModule::Gww
            icon=icons::GLOBE.into()
            text="Galactic Wide Web: Super Earth's sites and ministries, training manuals tied to the live war, and the corporations."
            meta=Signal::derive(move || {
                if done.get() > 0 { format!("{} of {} manuals read", done.get(), MANUALS.len()) } else { "Open".into() }
            })
        >
            <span class="hp-course" aria-hidden="true">
                {MANUALS
                    .iter()
                    .map(|m| view! { <i class=move || if read.with(|r| r.iter().any(|x| x == m.id)) { "on" } else { "" }></i> })
                    .collect_view()}
            </span>
        </ModuleTile>
    }
}

#[component]
fn PastTile() -> impl IntoView {
    let q = queries();
    let ended = Memo::new(move |_| {
        q.episodes.with(|e| {
            e.as_ref().map(|e| e.iter().filter(|e| e.status != EpisodeStatus::Active).cloned().collect::<Vec<_>>())
        })
    });
    let last = Memo::new(move |_| {
        ended.with(|e| {
            let e = e.as_ref()?;
            let first = e.first()?;
            Some(e.iter().fold(first, |a, b| if b.end.unwrap_or(0.0) > a.end.unwrap_or(0.0) { b } else { a }).clone())
        })
    });
    view! {
        <ModuleTile
            id=HellpadModule::PastCampaigns
            icon=icons::ARCHIVE.into()
            text="Every Galactic Campaign that has ended: its Major Orders, outcome and reward."
            meta=Signal::derive(move || {
                ended.with(|e| e.as_ref().filter(|e| !e.is_empty()).map_or("Open".to_string(), |e| format!("{} campaigns", e.len())))
            })
        >
            {move || last.get().map(|l| {
                let won = l.status == EpisodeStatus::Won;
                view! {
                    <span class="hp-mod-live caps">
                        {format!("Last: {}", l.title)}
                        <StatusChip tone=if won { "var(--good)" } else { "var(--bad)" }>{if won { "won" } else { "lost" }}</StatusChip>
                    </span>
                }
            })}
        </ModuleTile>
    }
}

#[component]
fn StratagemsTile() -> impl IntoView {
    let label = format!(
        "Stratagems by permit: {}",
        PERMITS.map(|p| format!("{} {}", p.label(), STRATAGEMS.iter().filter(|s| s.permit() == p).count())).join(", ")
    );
    view! {
        <ModuleTile
            id=HellpadModule::Stratagems
            icon=icons::ZAP.into()
            text="Every stratagem with its code, cooldown and where it is unlocked, and the drill to learn the codes."
            meta=Signal::stored(format!("{} stratagems", STRATAGEMS.len()))
        >
            <span class="hp-control">
                <span class="hp-control-bar" role="img" aria-label=label>
                    {PERMITS
                        .map(|p| {
                            let n = STRATAGEMS.iter().filter(|s| s.permit() == p).count();
                            view! { <i style=format!("flex-grow: {n}; background: {};", permit_color(p))></i> }
                        })
                        .collect_view()}
                </span>
            </span>
        </ModuleTile>
    }
}

#[component]
fn Group(title: &'static str, count: usize, children: Children) -> impl IntoView {
    view! {
        <section class="hp-group">
            <h2 class="hp-group-title caps">
                {title}
                " "
                <span class="num">{count}</span>
            </h2>
            <div class="hp-grid">{children()}</div>
        </section>
    }
}

#[component]
fn Hub() -> impl IntoView {
    view! {
        <div class="hp-hub">
            <Mast />
            <StationReport />
            <Group title="Field modules" count=5>
                <PlanetsTile />
                <StationsTile />
                <GwwTile />
                <PastTile />
                <StratagemsTile />
            </Group>
            <Group title="In development" count=5>
                <LockedTile
                    icon=icons::SKULL
                    title="Enemy encyclopedia"
                    path="enemies"
                    text="Every enemy by faction: its armour, weak spots and what brings it down."
                />
                <LockedTile
                    icon=icons::CROSSHAIR
                    title="Weaponry"
                    path="weapons"
                    text="Primaries, secondaries, throwables and support weapons, with their stats."
                />
                <LockedTile
                    icon=icons::LIBRARY
                    title="Ship library"
                    path="library"
                    text="For the Super Destroyer library Arrowhead has promised, once it is in the game."
                />
                <LockedTile icon=icons::ROCKET title="Warbonds" path="warbonds" text="Warbond pages, items and medal costs." />
                <LockedTile icon=icons::GIFT title="Boosters" path="boosters" text="Booster effects and where to find them." />
            </Group>
        </div>
    }
}

/// The strip over an open module: back to the hub, and where the module sits.
#[component]
fn ModuleBar(module: HellpadModule) -> impl IntoView {
    let m = module_info(module);
    view! {
        <nav class="hp-bar" aria-label="Breadcrumb" style=format!("--mod: {};", m.color)>
            <button class="hp-bar-back" aria-label="Back to the Hellpad" title="Hellpad" on:click=move |_| ui().open_module(None)>
                <Icon icon=icons::ARROW_LEFT size=16.0 />
            </button>
            <span class="hp-bar-path caps">
                <button class="link" on:click=move |_| ui().open_module(None)>
                    "Hellpad"
                </button>
                <Icon icon=icons::CHEVRON_RIGHT size=14.0 />
                <span aria-current="page">{m.title}</span>
            </span>
            <span class="hp-bar-addr num" aria-hidden="true">{m.path}</span>
        </nav>
    }
}

#[component]
pub fn HellpadPage() -> impl IntoView {
    let module = ui().module;
    let root = NodeRef::<html::Div>::new();
    // Into a module, focus its strip's Back; out of one, the tile it was opened from.
    Effect::new(move |was: Option<Option<HellpadModule>>| {
        let now = module.get();
        if let Some(left) = was
            && left != now
        {
            let to = match (now, left) {
                (Some(_), _) => ".hp-bar-back".to_string(),
                (None, Some(l)) => format!(".hp-mod[data-module=\"{}\"]", module_key(l)),
                (None, None) => return now,
            };
            if let Some(el) = root
                .get()
                .and_then(|r| r.query_selector(&to).ok().flatten())
                .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
            {
                let opts = web_sys::FocusOptions::new();
                opts.set_prevent_scroll(true);
                let _ = el.focus_with_options(&opts);
            }
        }
        now
    });
    view! {
        <div class="page scroll starfield">
            <div class="page-inner" node_ref=root>
                {move || {
                    module.get().map(|m| view! { <ModuleBar module=m /> })
                }}
                {move || match module.get() {
                    None => view! { <Hub /> }.into_any(),
                    Some(HellpadModule::Planets) => view! { <PlanetsModule /> }.into_any(),
                    Some(HellpadModule::Stations) => view! { <StationsModule /> }.into_any(),
                    Some(HellpadModule::Gww) => view! { <GwwModule /> }.into_any(),
                    Some(HellpadModule::PastCampaigns) => view! { <PastCampaignsModule /> }.into_any(),
                    Some(HellpadModule::Stratagems) => view! { <StratagemsModule /> }.into_any(),
                }}
            </div>
        </div>
    }
}
