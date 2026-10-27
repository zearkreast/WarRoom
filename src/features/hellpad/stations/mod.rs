//! The space stations module: each station's orbit, tactical actions, vote and jump log.

pub mod holo;
pub mod jumps;

use crate::api::fronts::planet_rate;
use crate::api::queries::queries;
use crate::api::stations::use_station_jumps;
use crate::features::map::pawn_art::{DssState, action_icon, dss_state};
use crate::features::planet::detail::PlanetSphere;
use crate::js::{self, fmt_date_time};
use crate::state::{HellpadModule, OptionsAnchor, View, ui};
use crate::ui::emblems::{FactionEmblem, IconLike, StationEmblem};
use crate::ui::icons::{self, Icon};
use crate::ui::primitives::{Countdown, Empty, Loading, OutLink, Panel, StatusChip};
use holo::{CssHolo, DssHolo};
use leptos::prelude::*;
use regex_lite::Regex;
use std::sync::LazyLock;
use warroom_core::api::{Faction, TacticalAction};
use warroom_core::biomes::regen_pct_per_hour;
use warroom_core::data::manuals::WIKI_LICENCE;
use warroom_core::data::stations::{PAST_STATIONS, STAR_OF_PEACE};
use warroom_core::data::{Dated, Source, StationRecord};
use warroom_core::factions::faction_color;
use warroom_core::format::{fmt_pct, js_round, to_fixed};
use warroom_core::gww_nav::{GwwPage, LOG_ADDRESS};
use warroom_core::gww_text::{Segment, parse_rich};
use warroom_core::progress::tracked_percent;
use warroom_core::snapshot::StationLive;
use warroom_core::time::parse_ms;
use warroom_core::war_history::fmt_long_duration;
use wasm_bindgen::JsCast;

static CLOSE_TWICE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(</span>)>+").unwrap());
static HIGHLIGHT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)<span[^>]*>(.*?)</span>").unwrap());

/// Splits the API's `<span data-ah="1">…</span>` highlights into plain and emphasised runs.
fn highlighted(text: &str) -> AnyView {
    // The API sometimes doubles the closing bracket ("</span>>").
    let text = CLOSE_TWICE.replace_all(text, "$1").to_string();
    let mut parts: Vec<AnyView> = Vec::new();
    let mut at = 0;
    for m in HIGHLIGHT.captures_iter(&text) {
        let whole = m.get(0).unwrap();
        parts.push(view! { <span>{text[at..whole.start()].to_string()}</span> }.into_any());
        parts.push(view! { <b class="st-hl">{m[1].to_string()}</b> }.into_any());
        at = whole.end();
    }
    parts.push(view! { <span>{text[at..].to_string()}</span> }.into_any());
    parts.into_iter().collect_view().into_any()
}

/// Text with its `[[planet:Name|label]]` links opening the planet's options, in its owner's colour.
fn planet_text(text: &str) -> AnyView {
    let planets = queries().fresh_planets;
    parse_rich(text)
        .into_iter()
        .map(|seg| match seg {
            Segment::Text(t) => t.into_any(),
            Segment::Link { kind, target, label } => {
                let p = (kind == "planet")
                    .then(|| {
                        planets.with_untracked(|p| {
                            p.as_ref()?
                                .iter()
                                .find(|x| x.name.to_lowercase() == target.to_lowercase())
                                .map(|p| (p.index, p.current_owner))
                        })
                    })
                    .flatten();
                match p {
                    None => label.into_any(),
                    Some((index, owner)) => view! {
                        <button
                            class="link st-planet-link"
                            style=format!("color: {};", faction_color(Some(owner)))
                            on:click=move |e| {
                                // A keyboard click has no pointer position; open beside the link instead.
                                let rect = e
                                    .current_target()
                                    .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                                    .map(|el| el.get_bounding_client_rect());
                                let (cx, cy) = (e.client_x() as f64, e.client_y() as f64);
                                let x = if cx != 0.0 { cx } else { rect.as_ref().map_or(0.0, |r| r.left()) };
                                let y = if cy != 0.0 { cy } else { rect.as_ref().map_or(0.0, |r| r.bottom()) };
                                ui().open_options(Some(OptionsAnchor { planet: index, x, y }))
                            }
                        >
                            {label}
                        </button>
                    }
                    .into_any(),
                }
            }
        })
        .collect_view()
        .into_any()
}

/// A bold term before the first ": ", when there is one.
fn point(text: &str) -> AnyView {
    match text.find(": ") {
        None => planet_text(text),
        Some(at) => view! {
            <b>{text[..at].to_string()}</b>
            ": "
            {planet_text(&text[at + 2..])}
        }
        .into_any(),
    }
}

fn timeline(events: &'static [Dated]) -> impl IntoView {
    let items = events
        .iter()
        .map(|e| {
            view! {
                <li>
                    <span class="st-timeline-when caps num">{e.when}</span>
                    <p>{planet_text(e.text)}</p>
                </li>
            }
        })
        .collect_view();
    view! { <ol class="st-timeline">{items}</ol> }
}

fn sources(list: &'static [Source]) -> impl IntoView {
    let links = list
        .iter()
        .enumerate()
        .map(|(i, s)| {
            view! {
                {(i > 0).then_some(", ")}
                <OutLink href=s.url>{s.title}</OutLink>
            }
        })
        .collect_view();
    view! {
        <p class="st-source">
            "Adapted from the Helldivers Wiki: "
            {links}
            ", under "
            <OutLink href=WIKI_LICENCE.url>{WIKI_LICENCE.title}</OutLink>
            "."
        </p>
    }
}

#[component]
fn StarOfPeace() -> impl IntoView {
    view! {
        <Panel
            title="Star of Peace"
            icon=icons::SPARKLE
            tone="var(--warn)"
            right=view! { <StatusChip tone="var(--warn)">{STAR_OF_PEACE.status}</StatusChip> }.into_any()
        >
            <div class="st-sop">
                <p class="st-lead">{planet_text(STAR_OF_PEACE.summary)}</p>
                <ul class="st-points">{STAR_OF_PEACE.points.iter().map(|p| view! { <li>{point(p)}</li> }).collect_view()}</ul>
                <h3 class="st-sub caps">"Operation Free Space and after"</h3>
                {timeline(STAR_OF_PEACE.timeline)}
                {sources(STAR_OF_PEACE.sources)}
            </div>
        </Panel>
    }
}

fn dss_state_label(s: DssState) -> &'static str {
    match s {
        DssState::Active => "Action deployed",
        DssState::Cooldown => "Cooling down",
        DssState::Offline => "Offline",
        DssState::Default => "Gathering",
    }
}

fn is_active(a: &TacticalAction, now: f64) -> bool {
    a.status == 2 && parse_ms(&a.status_expire) > now
}

fn is_cooling(a: &TacticalAction, now: f64) -> bool {
    a.status == 3 && parse_ms(&a.status_expire) > now
}

/// Mode (its CSS class), label and tone.
fn action_state(a: &TacticalAction, now: f64) -> (&'static str, &'static str, &'static str) {
    if is_active(a, now) {
        return ("active", "Active", "var(--good)");
    }
    if is_cooling(a, now) {
        return ("cooling", "Cooldown", "var(--warn)");
    }
    if a.costs.first().is_some_and(|c| c.current_value >= c.target_value) {
        return ("ready", "Ready", "var(--good)");
    }
    ("gathering", "Gathering", "var(--f-humans)")
}

fn action_icon_view(name: &str, size: f64) -> AnyView {
    match action_icon(name) {
        None => view! { <Icon icon=icons::ZAP size=size /> }.into_any(),
        Some(href) => {
            view! { <img class="st-action-icon" src=href width=js::num(size) height=js::num(size) alt="" /> }.into_any()
        }
    }
}

fn action_bar(pct: f64, mode: &'static str, label: String) -> impl IntoView {
    let v = pct.clamp(0.0, 100.0);
    view! {
        <div
            class=format!("st-bar {mode}")
            role="progressbar"
            aria-valuenow=js_round(v).to_string()
            aria-valuemin="0"
            aria-valuemax="100"
            aria-label=label
        >
            <div class="st-bar-fill" style=format!("width: {}%;", js::num(v))></div>
            {(mode == "cooling").then(|| view! { <span class="st-bar-tag caps">"Standby"</span> })}
        </div>
    }
}

fn action_card(a: &TacticalAction, now: f64) -> AnyView {
    let cost = a.costs.first();
    let active = is_active(a, now);
    let cooling = is_cooling(a, now);
    // A deployed action has spent its contributions; the game shows its bar full until it ends.
    let pct = if active {
        100.0
    } else {
        cost.filter(|c| c.target_value != 0.0).map_or(0.0, |c| c.current_value / c.target_value * 100.0)
    };
    let remaining = cost
        .filter(|c| c.delta_per_second > 0.0 && c.current_value < c.target_value)
        .map(|c| (c.target_value - c.current_value) / c.delta_per_second);
    let (mode, label, tone) = action_state(a, now);
    let expire = parse_ms(&a.status_expire);
    let progress = cost.map(|_| {
        let text = if active {
            view! { "Active · ends in " <Countdown to=expire /> }.into_any()
        } else if cooling {
            view! { "Ready again in " <Countdown to=expire /> }.into_any()
        } else if let Some(r) = remaining {
            format!(
                "Estimated ready in {}",
                if r < 3600.0 { "under an hour".to_string() } else { fmt_long_duration(r) }
            )
            .into_any()
        } else {
            "Holding".into_any()
        };
        view! {
            <div class="st-progress">
                {action_bar(pct, mode, format!("{} readiness", a.name))}
                <div class="st-progress-row num">
                    <span>{text}</span>
                    <b>{fmt_pct(pct, 3)}</b>
                </div>
            </div>
        }
    });
    view! {
        <article class=format!("st-action {mode}") style=format!("--tone: {tone};")>
            <header>
                {action_icon_view(&a.name, 34.0)}
                <h3>{a.name.clone()}</h3>
                <span class="status-chip" style=format!("--tone: {tone};")>{label}</span>
            </header>
            <p class="st-strategic">{highlighted(&a.strategic_description)}</p>
            <p class="st-desc">{a.description.clone()}</p>
            {progress}
        </article>
    }
    .into_any()
}

#[component]
fn JumpHistoryLink(station: i64) -> impl IntoView {
    let jumps = use_station_jumps(move || station).data;
    let open = move |_| {
        let u = ui();
        u.view.set(View::Hellpad);
        u.module.set(Some(HellpadModule::Gww));
        u.set_gww(vec![GwwPage::Home, GwwPage::Site { id: "log".into() }]);
        u.options.set(None);
        u.detail.set(None);
    };
    let summary = move || {
        jumps.with(|j| {
            let j = j.as_deref().map_or(&[][..], |j| j.as_slice());
            match j.first() {
                None => "No jumps logged yet".to_string(),
                Some(last) => format!(
                    "{} logged · last {}",
                    if j.len() == 1 { "1 jump".to_string() } else { format!("{} jumps", j.len()) },
                    fmt_date_time(last.at)
                ),
            }
        })
    };
    view! {
        <button class="st-jumplink" on:click=open>
            <Icon icon=icons::HISTORY size=18.0 />
            <span class="st-jumplink-text">
                <b class="caps">"FTL jump history"</b>
                <span class="num">{summary}</span>
            </span>
            <span class="st-jumplink-to caps num">
                {LOG_ADDRESS}
                " "
                <Icon icon=icons::ARROW_RIGHT size=14.0 />
            </span>
        </button>
    }
}

#[component]
fn StationPanel(station: Memo<Option<StationLive>>) -> impl IntoView {
    let q = queries();
    let now = js::use_now(30_000.0);
    let id = station.with_untracked(|s| s.as_ref().map_or(0, |s| s.id32));
    // Redrawn when an action starts or ends, not on every clock tick.
    let phases = Memo::new(move |_| {
        let n = now.get();
        station.with(|s| {
            s.as_ref()
                .map(|s| s.tactical_actions.iter().map(|a| (is_active(a, n), is_cooling(a, n))).collect::<Vec<_>>())
        })
    });
    move || {
        phases.with(|_| ());
        let s = station.get()?;
        let now_v = now.get_untracked();
        let p = q.fresh_planets.with(|p| p.as_ref()?.iter().find(|x| x.index == s.planet_index).cloned())?;
        let mut effects =
            q.effects.with(|e| e.as_ref().and_then(|e| e.stations.get(&s.id32).cloned()).unwrap_or_default());
        effects.sort_by_key(|e| action_icon(&e.name).is_some());
        let front =
            q.campaigns.with(|c| c.as_ref().is_some_and(|c| c.iter().any(|c| c.planet.index == s.planet_index)));
        let rate = planet_rate(Some(s.planet_index));
        let enemy =
            p.event.as_ref().map(|e| e.faction).or((p.current_owner != Faction::Humans).then_some(p.current_owner));
        let pct = if enemy.is_some() { tracked_percent(&p) } else { 100.0 };
        let jump = parse_ms(&s.election_end);
        let live: Vec<TacticalAction> = s.tactical_actions.iter().filter(|a| is_active(a, now_v)).cloned().collect();
        let state = dss_state(&s.tactical_actions, now_v);
        let index = p.index;
        let face = enemy.unwrap_or(Faction::Humans);
        let sphere_planet = p.clone();
        let station_readout = if live.is_empty() {
            dss_state_label(state).into_any()
        } else {
            let names = live.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(", ");
            view! {
                <span class="st-holo-icons" aria-label=names>
                    {live
                        .iter()
                        .map(|a| view! { <span title=a.name.clone()>{action_icon_view(&a.name, 22.0)}</span> })
                        .collect_view()}
                </span>
            }
            .into_any()
        };
        let jump_view = move || {
            if jump > now.get() { view! { <Countdown to=jump /> }.into_any() } else { "Vote closing".into_any() }
        };
        Some(view! {
            <div class="st-layout">
                <div class="st">
                    <section class="st-hero brackets" style=format!("--tone: {};", faction_color(Some(face)))>
                        <div class="st-hero-text">
                            <button class="st-planet" on:click=move |_| ui().open_detail(Some(index))>
                                <FactionEmblem faction=enemy.unwrap_or(p.current_owner).as_str() size=30.0 />
                                <span>
                                    <b>{p.name.clone()}</b>
                                    <span class="caps">{format!("{} sector", p.sector)}</span>
                                </span>
                            </button>
                            <div class="st-stats">
                                <span class="num">
                                    <FactionEmblem faction="Humans" size=14.0 />
                                    {format!(
                                        " {} {}",
                                        fmt_pct(pct, 2),
                                        if front { if p.event.is_some() { "defended" } else { "liberated" } } else { "held" }
                                    )}
                                </span>
                                {front.then(|| view! {
                                    <span class="num" style=format!("color: {};", if rate.is_some_and(|r| r > 0.0) { "var(--good)" } else { "var(--text-dim)" })>
                                        {rate.map_or("Rate measuring…".to_string(), |r| format!("{}{}% / h", if r > 0.0 { "+" } else { "" }, to_fixed(r, 2)))}
                                    </span>
                                })}
                                {enemy.map(|e| view! {
                                    <span class="num" style=format!("color: {};", faction_color(Some(e)))>
                                        <FactionEmblem faction=e.as_str() size=14.0 />
                                        {format!(" -{}% / h regen", to_fixed(regen_pct_per_hour(p.regen_per_second, p.max_health), 2))}
                                    </span>
                                })}
                            </div>
                            <div class="st-jump">
                                <Icon icon=icons::TIMER size=16.0 />
                                <span class="caps">"Next FTL jump"</span>
                                <b class="num">{jump_view}</b>
                            </div>
                            {(!live.is_empty()).then(|| view! {
                                <ul class="st-live" aria-label="Active tactical actions">
                                    {live
                                        .iter()
                                        .map(|a| view! {
                                            <li title=a.name.clone()>
                                                {action_icon_view(&a.name, 22.0)}
                                                <span class="st-live-name">
                                                    <b class="caps">{a.name.clone()}</b>
                                                    <span class="caps">{format!("Over {}", p.name)}</span>
                                                </span>
                                                <span class="num">
                                                    "ends in "
                                                    <Countdown to=parse_ms(&a.status_expire) />
                                                </span>
                                            </li>
                                        })
                                        .collect_view()}
                                </ul>
                            })}
                            {(!effects.is_empty()).then(|| view! {
                                <ul class="st-effects" aria-label="Station effects">
                                    {effects
                                        .iter()
                                        .map(|e| view! {
                                            <li class="caps" title=e.description.clone()>
                                                {action_icon(&e.name).map(|_| action_icon_view(&e.name, 14.0))}
                                                {e.name.clone()}
                                            </li>
                                        })
                                        .collect_view()}
                                </ul>
                            })}
                        </div>
                        <PlanetSphere
                            planet=Signal::stored(sphere_planet)
                            percent=pct
                            enemy=faction_color(Some(face)).to_string()
                            size=200.0
                        />
                    </section>
                    <div class="st-actions">{s.tactical_actions.iter().map(|a| action_card(a, now_v)).collect_view()}</div>
                    <JumpHistoryLink station=id />
                    <StarOfPeace />
                </div>
                <DssHolo
                    state=state
                    readout=vec![
                        ("Orbit".to_string(), p.name.clone().into_any()),
                        ("Station".to_string(), station_readout),
                        ("Next jump".to_string(), (move || jump_view()).into_any()),
                    ]
                />
            </div>
        })
    }
}

#[component]
fn PastStation(s: &'static StationRecord) -> impl IntoView {
    let tone = faction_color(Some(s.faction));
    view! {
        <div class="st-layout">
            <div class="st">
                <section class="st-hero brackets st-past" style=format!("--tone: {tone};")>
                    <div class="st-hero-text">
                        <div class="st-planet">
                            <StationEmblem station=s.art size=34.0 color=tone />
                            <span>
                                <b>{s.name}</b>
                                <span class="caps">{s.status}</span>
                            </span>
                        </div>
                        <p class="st-aka">{format!("Also called the {}.", s.aka.join("; the "))}</p>
                        <p class="st-note">{planet_text(s.note)}</p>
                    </div>
                </section>
                <div class="st-cols">
                    <Panel title=s.operation icon=icons::HISTORY tone=tone>
                        {timeline(s.timeline)}
                    </Panel>
                    <Panel title="Dossier" icon=icons::FILE_TEXT tone=tone>
                        <div class="st-dossier">
                            {s
                                .sections
                                .iter()
                                .map(|sec| view! {
                                    <section>
                                        <h3 class="st-sub caps">{sec.heading}</h3>
                                        <p>{planet_text(sec.text)}</p>
                                    </section>
                                })
                                .collect_view()}
                            {sources(s.sources)}
                        </div>
                    </Panel>
                </div>
            </div>
            {(s.art == "css").then(|| view! {
                <CssHolo readout=s.readout.iter().map(|(k, v)| (k.to_string(), (*v).into_any())).collect() />
            })}
        </div>
    }
}

/// Every station the war has had: the live ones from the API, then the ones only history remembers.
#[component]
pub fn StationsModule() -> impl IntoView {
    let q = queries();
    let chosen = RwSignal::new(None::<String>);
    let live_ids = Memo::new(move |_| {
        q.stations.with(|s| s.as_ref().map(|s| s.iter().map(|s| s.id32).collect::<Vec<_>>()).unwrap_or_default())
    });
    let options = Memo::new(move |_| {
        let mut out: Vec<(String, String, String, Faction, bool)> = live_ids
            .get()
            .iter()
            .enumerate()
            .map(|(i, id)| {
                (
                    id.to_string(),
                    if i == 0 { "DSS".into() } else { format!("Station {}", i + 1) },
                    "Online".into(),
                    Faction::Humans,
                    false,
                )
            })
            .collect();
        out.extend(
            PAST_STATIONS
                .iter()
                .map(|s| (s.id.to_string(), s.short.to_string(), s.status.to_string(), s.faction, true)),
        );
        out
    });
    let current = Memo::new(move |_| {
        let c = chosen.get();
        options.with(|o| o.iter().find(|x| Some(&x.0) == c.as_ref()).or(o.first()).map(|x| x.0.clone()))
    });
    let waiting = Memo::new(move |_| q.raw.loading.get() && q.stations.with(Option::is_none));
    view! {
        <Show when=move || !waiting.get() fallback=|| view! { <Loading what="station telemetry" rows=3 /> }>
            <div class="st-list">
                <div class="st-picker" role="tablist" aria-label="Space stations">
                    <For each=move || options.get() key=|o| o.0.clone() let:o>
                        {
                            let key = o.0.clone();
                            let k2 = key.clone();
                            let on = move || current.get().as_deref() == Some(key.as_str());
                            let on2 = on.clone();
                            view! {
                                <button
                                    role="tab"
                                    aria-selected=move || on().to_string()
                                    class=move || format!("st-pick {}", if on2() { "on" } else { "" })
                                    style=format!("--tone: {};", faction_color(Some(o.3)))
                                    on:click=move |_| chosen.set(Some(k2.clone()))
                                >
                                    <StationEmblem station=if o.4 { "css" } else { "dss" } size=18.0 />
                                    <b class="caps">{o.1.clone()}</b>
                                    <span class="status-chip" style=format!("--tone: {};", if o.4 { "var(--bad)" } else { "var(--good)" })>
                                        {o.2.clone()}
                                    </span>
                                </button>
                            }
                        }
                    </For>
                </div>
                <Show when=move || live_ids.with(Vec::is_empty)>
                    <Panel title="Democracy Space Station" icon=IconLike::Dss>
                        <Empty>"No station telemetry yet. It arrives with the next collector update."</Empty>
                    </Panel>
                </Show>
                {move || {
                    let key = current.get()?;
                    if let Some(past) = PAST_STATIONS.iter().find(|s| s.id == key) {
                        return Some(view! { <PastStation s=past /> }.into_any());
                    }
                    let id: i64 = key.parse().ok()?;
                    let station = Memo::new(move |_| q.stations.with(|s| s.as_ref()?.iter().find(|s| s.id32 == id).cloned()));
                    Some(view! { <StationPanel station=station /> }.into_any())
                }}
            </div>
        </Show>
    }
}
