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
use warroom_core::galaxy::LANDMARKS;
use warroom_core::gww_nav::GwwPage;
use warroom_core::planet_index::control_shares;
use warroom_core::snapshot::EpisodeStatus;
use warroom_core::stratagems::{PERMITS, STRATAGEMS};
use warroom_core::time::parse_ms;
use wasm_bindgen::JsCast;

struct ModuleInfo {
    title: &'static str,
    path: &'static str,
    color: &'static str,
    icon: IconLike,
    blurb: &'static str,
}

fn module_info(m: HellpadModule) -> ModuleInfo {
    match m {
        HellpadModule::Planets => ModuleInfo {
            title: "Planets",
            path: "hp://planets",
            color: "var(--f-humans)",
            icon: IconLike::Lucide(icons::GLOBE_2),
            blurb: "Every planet by sector, with search, filters and live fronts.",
        },
        HellpadModule::Stations => ModuleInfo {
            title: "Space stations",
            path: "hp://stations",
            color: "var(--impact)",
            icon: IconLike::Dss,
            blurb: "Where the DSS is, its next FTL jump, its tactical actions and the Automaton station it destroyed.",
        },
        HellpadModule::Gww => ModuleInfo {
            title: "G.W.W.",
            path: "hp://gww",
            color: "var(--cta)",
            icon: IconLike::Lucide(icons::GLOBE),
            blurb: "Galactic Wide Web: Super Earth's sites and ministries, training manuals tied to the live war, and the corporations.",
        },
        HellpadModule::PastCampaigns => ModuleInfo {
            title: "Past campaigns",
            path: "hp://campaigns",
            color: "var(--warn)",
            icon: IconLike::Lucide(icons::ARCHIVE),
            blurb: "Every Galactic Campaign that has ended: its Major Orders, outcome and reward.",
        },
        HellpadModule::Stratagems => ModuleInfo {
            title: "Stratagem collection",
            path: "hp://stratagems",
            color: "var(--p-offensive)",
            icon: IconLike::Lucide(icons::ZAP),
            blurb: "Every stratagem with its code, cooldown and where it is unlocked, and the drill to learn the codes.",
        },
    }
}

/// What a module holds, in a few words: on its tile and in its header. None until its data has arrived.
fn module_meta(m: HellpadModule) -> Signal<Option<String>> {
    let q = queries();
    match m {
        // Counted as the index lists them, landmarks included.
        HellpadModule::Planets => Signal::derive(move || {
            q.fresh_planets.with(|p| p.as_ref().map(|p| format!("{} planets indexed", p.len() + LANDMARKS.len())))
        }),
        HellpadModule::Stations => Signal::derive(move || {
            q.stations.with(|s| {
                s.as_ref()
                    .map(|s| if s.is_empty() { "No station in orbit".into() } else { format!("{} in orbit", s.len()) })
            })
        }),
        HellpadModule::Gww => {
            let read = Memo::new(move |_| {
                settings().get(|s| MANUALS.iter().filter(|m| s.gww_read.iter().any(|x| x == m.id)).count())
            });
            Signal::derive(move || {
                Some(match read.get() {
                    0 => format!("{} training manuals", MANUALS.len()),
                    n => format!("{n} of {} manuals read", MANUALS.len()),
                })
            })
        }
        HellpadModule::PastCampaigns => Signal::derive(move || {
            q.episodes.with(|e| {
                let n = e.as_ref()?.iter().filter(|e| e.status != EpisodeStatus::Active).count();
                Some(if n == 1 { "1 campaign".into() } else { format!("{n} campaigns") })
            })
        }),
        HellpadModule::Stratagems => Signal::stored(Some(format!("{} stratagems", STRATAGEMS.len()))),
    }
}

/// Where an open module stands after its `hp://` address: a filter, a tab or a search, as `/tab` or `?q=…`.
#[derive(Clone, Copy)]
pub struct ModuleAt(RwSignal<String>);

/// Keeps the open module's address in step with `at`.
pub fn use_module_at(at: impl Fn() -> String + Send + Sync + 'static) {
    let Some(ModuleAt(s)) = use_context::<ModuleAt>() else { return };
    Effect::new(move |_| s.set(at()));
    on_cleanup(move || {
        let _ = s.try_set(String::new());
    });
}

/// A module's address tail: an optional segment, then the search if there is one.
pub fn address_tail(segment: Option<&str>, search: &str) -> String {
    let q = search.split_whitespace().collect::<Vec<_>>().join("+").to_lowercase();
    format!(
        "{}{}",
        segment.map(|s| format!("/{s}")).unwrap_or_default(),
        if q.is_empty() { String::new() } else { format!("?q={q}") }
    )
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
fn ModuleTile(id: HellpadModule, #[prop(optional)] children: Option<Children>) -> impl IntoView {
    let m = module_info(id);
    let meta = module_meta(id);
    view! {
        <button
            class="hp-mod"
            data-module=module_key(id)
            style=format!("--mod: {};", m.color)
            on:click=move |_| {
                // Coming in from the Hellpad starts the G.W.W. at its home page, not where it was left.
                if id == HellpadModule::Gww {
                    ui().set_gww(vec![GwwPage::Home]);
                }
                ui().open_module(Some(id))
            }
        >
            <span class="hp-mod-icon">{icon_view(m.icon, 26.0, 1.5, None)}</span>
            <span class="hp-mod-body">
                <span class="hp-mod-path num">{m.path}</span>
                <b class="hp-mod-title">{m.title}</b>
                <span class="hp-mod-text">{m.blurb}</span>
                {children.map(|c| c())}
                <span class="hp-mod-go caps">
                    {move || format!("{} ", meta.get().unwrap_or_else(|| "Open".into()))}
                    <Icon icon=icons::CHEVRON_RIGHT size=14.0 />
                </span>
            </span>
        </button>
    }
}

/// A module the Hellpad doesn't have yet, with what it waits on.
#[component]
fn PlannedTile(
    icon: IconData,
    title: &'static str,
    path: &'static str,
    text: &'static str,
    #[prop(default = "Planned")] status: &'static str,
) -> impl IntoView {
    view! {
        <div class="hp-mod planned">
            <span class="hp-mod-icon">
                <Icon icon=icon size=20.0 stroke_width=1.5 />
            </span>
            <span class="hp-mod-body">
                <span class="hp-mod-path num">
                    {format!("hp://{path}")}
                    <span class="hp-mod-status caps">
                        <Icon icon=icons::LOCK size=12.0 />
                        {format!(" {status}")}
                    </span>
                </span>
                <b class="hp-mod-title">{title}</b>
                <span class="hp-mod-text">{text}</span>
            </span>
        </div>
    }
}

#[component]
fn PlanetsTile() -> impl IntoView {
    let q = queries();
    let shares = Memo::new(move |_| q.fresh_planets.with(|p| p.as_ref().map(|p| control_shares(p))));
    view! {
        <ModuleTile id=HellpadModule::Planets>
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
    let place = Memo::new(move |_| {
        let index = q.stations.with(|s| s.as_ref()?.first().map(|s| s.planet_index))?;
        q.fresh_planets.with(|p| p.as_ref()?.iter().find(|p| p.index == index).map(|p| p.name.clone()))
    });
    view! {
        <ModuleTile id=HellpadModule::Stations>
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
    view! {
        <ModuleTile id=HellpadModule::Gww>
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
        <ModuleTile id=HellpadModule::PastCampaigns>
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
        <ModuleTile id=HellpadModule::Stratagems>
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
fn Group(
    title: &'static str,
    count: usize,
    #[prop(default = "hp-grid")] class: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <section class="hp-group">
            <h2 class="hp-group-title caps">
                {title}
                " "
                <span class="num">{count}</span>
            </h2>
            <div class=class>{children()}</div>
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
            <Group title="Planned" count=5 class="hp-grid planned">
                <PlannedTile
                    icon=icons::SKULL
                    title="Enemy encyclopedia"
                    path="enemies"
                    text="Every enemy by faction: its armour, weak spots and what brings it down."
                />
                <PlannedTile
                    icon=icons::CROSSHAIR
                    title="Weaponry"
                    path="weapons"
                    text="Primaries, secondaries, throwables and support weapons, with their stats."
                />
                <PlannedTile
                    icon=icons::LIBRARY
                    title="Ship library"
                    path="library"
                    text="The Super Destroyer's library, once the game opens it."
                    status="Awaits the game"
                />
                <PlannedTile icon=icons::ROCKET title="Warbonds" path="warbonds" text="Warbond pages, items and medal costs." />
                <PlannedTile icon=icons::GIFT title="Boosters" path="boosters" text="Booster effects and where to find them." />
            </Group>
        </div>
    }
}

/// The head of an open module: the way back to the hub and its address, then what the module is.
#[component]
fn ModuleHead(module: HellpadModule) -> impl IntoView {
    let m = module_info(module);
    let meta = module_meta(module);
    let at = use_context::<ModuleAt>();
    view! {
        <header class="hp-head" style=format!("--mod: {};", m.color)>
            <nav class="hp-bar" aria-label="Breadcrumb">
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
                <span class="hp-bar-addr num" aria-hidden="true">
                    {m.path}
                    <span class="hp-bar-at">{move || at.map(|a| a.0.get()).unwrap_or_default()}</span>
                </span>
            </nav>
            <div class="hp-head-main">
                <span class="hp-head-icon" aria-hidden="true">{icon_view(m.icon, 28.0, 1.5, None)}</span>
                <div class="hp-head-text">
                    <h1 class="hp-head-title">{m.title}</h1>
                    <p class="hp-head-blurb">{m.blurb}</p>
                </div>
                <span class="hp-head-meta caps num">{move || meta.get().unwrap_or_default()}</span>
            </div>
        </header>
    }
}

#[component]
pub fn HellpadPage() -> impl IntoView {
    let module = ui().module;
    let root = NodeRef::<html::Div>::new();
    provide_context(ModuleAt(RwSignal::new(String::new())));
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
                {move || match module.get() {
                    None => view! { <Hub /> }.into_any(),
                    Some(m) => view! {
                        <ModuleHead module=m />
                        <div class="hp-module">
                            {match m {
                                HellpadModule::Planets => view! { <PlanetsModule /> }.into_any(),
                                HellpadModule::Stations => view! { <StationsModule /> }.into_any(),
                                HellpadModule::Gww => view! { <GwwModule /> }.into_any(),
                                HellpadModule::PastCampaigns => view! { <PastCampaignsModule /> }.into_any(),
                                HellpadModule::Stratagems => view! { <StratagemsModule /> }.into_any(),
                            }}
                        </div>
                    }
                    .into_any(),
                }}
            </div>
        </div>
    }
}
