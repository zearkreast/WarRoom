//! The shell: the header with the war status and news ticker, the navigation rail, toasts and overlays.

use crate::alerts::{ToastKind, pops_up, toasts};
use crate::api::queries::queries;
use crate::config::APP_VERSION;
use crate::features::news::items::unread_news;
use crate::features::news::ticker::NewsTicker;
use crate::features::planet::city_detail::CityDetail;
use crate::features::planet::detail::PlanetDetail;
use crate::features::planet::options::PlanetOptions;
use crate::js;
use crate::layout::about_dialog::AboutDialog;
use crate::layout::briefing_popup::BriefingPopup;
use crate::layout::dispatch_panel::DispatchPanel;
use crate::state::{InfoDialog, View, settings, ui};
use crate::ui::emblems::{AppMark, IconLike, icon_view};
use crate::ui::feed_status::{FeedState, use_feed_status};
use crate::ui::icons::{self, Icon, IconData};
use crate::ui::primitives::{Keycap, StatusChip, use_escape_when};
use crate::update::card::UpdateCard;
use leptos::prelude::*;
use warroom_core::format::{fmt_duration, fmt_int, fmt_pct, sest_clock, sest_date, war_day};
use warroom_core::galaxy::human_control;
use warroom_core::progress::tracked_percent;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

/// Rotating one-line war status.
fn status_lines() -> Vec<AnyView> {
    let q = queries();
    let mut lines = Vec::new();
    if let Some(planets) = q.planets.get() {
        let pct = fmt_pct(human_control(&planets), 1);
        lines.push(view! { "Super Earth holds " <b class="hi">{pct}</b> " of the galaxy" }.into_any());
    }
    if let Some(campaigns) = q.campaigns.get() {
        let n = campaigns.len();
        lines.push(view! { <b class="hi">{n}</b> " active planets on the front" }.into_any());
        let mut sorted: Vec<_> = campaigns.iter().collect();
        sorted.sort_by(|a, b| b.planet.statistics.player_count.total_cmp(&a.planet.statistics.player_count));
        if let Some(top) = sorted.first() {
            let kind = if top.planet.event.is_some() { "Defense" } else { "Liberation" };
            let name = top.planet.name.clone();
            let pct = fmt_pct(tracked_percent(&top.planet), 1);
            lines.push(view! { {kind} " at " <b class="hi">{name}</b> " " <b class="hi">{pct}</b> }.into_any());
        }
    }
    if let Some(war) = q.war.get() {
        let n = fmt_int(war.statistics.player_count);
        lines.push(view! { <b class="hi">{n}</b> " Helldivers deployed" }.into_any());
    }
    lines
}

/// The war's state at a glance: link status and the rotating line, the news ticker, the clock and the war day.
#[component]
fn TopBar() -> impl IntoView {
    let q = queries();
    let u = ui();
    let f = use_feed_status(1000.0);
    let (i, set_i) = signal(0usize);
    let handle =
        set_interval_with_handle(move || set_i.update(|n| *n += 1), std::time::Duration::from_millis(8000)).ok();
    on_cleanup(move || {
        if let Some(h) = handle {
            h.clear()
        }
    });
    let line_count = Memo::new(move |_| {
        let q = queries();
        let mut n = 0;
        if q.planets.with(Option::is_some) {
            n += 1;
        }
        if let Some(c) = q.campaigns.get() {
            n += if c.is_empty() { 1 } else { 2 };
        }
        if q.war.with(Option::is_some) {
            n += 1;
        }
        n
    });
    // A new key restarts the line's fade-in.
    let key = Memo::new(move |_| match f.state.get() {
        FeedState::Stale => "stale".to_string(),
        _ => (i.get() % line_count.get().max(1)).to_string(),
    });
    view! {
        <header class="topbar">
            <button class="top-logo" on:click=move |_| u.set_view(View::Campaigns) aria-label="WarCommand: Active planets">
                <AppMark size=26.0 />
            </button>
            <div class=move || {
                format!("strip-status {}", if f.state.get() == FeedState::Live { "is-live" } else { "" })
            }>
                {move || match f.state.get() {
                    FeedState::Offline => view! { <StatusChip tone="var(--bad)">"Link lost"</StatusChip> }.into_any(),
                    FeedState::Stale => view! { <StatusChip tone="var(--warn)">"Stale"</StatusChip> }.into_any(),
                    FeedState::Live => view! { <StatusChip>"Front"</StatusChip> }.into_any(),
                }}
                {move || {
                    let k = key.get();
                    view! {
                        <span class="strip-line caps">
                            {move || {
                                let _ = &k;
                                match f.state.get() {
                                    FeedState::Offline => "Comms link down, showing last known data".into_any(),
                                    FeedState::Stale => {
                                        let gap = f.now.get() - f.last_update.get().unwrap_or(0.0);
                                        format!("No new war data for {}; the collector may be down", fmt_duration(gap)).into_any()
                                    }
                                    FeedState::Live => {
                                        let mut lines = status_lines();
                                        let n = lines.len().max(1);
                                        let at = i.get_untracked() % n;
                                        if lines.is_empty() { ().into_any() } else { lines.swap_remove(at).into_any() }
                                    }
                                }
                            }}
                        </span>
                    }
                }}
            </div>
            <NewsTicker />
            <div class="strip-clock num" title="Super Earth Standard Time (UTC)">
                <span class="caps">"SEST"</span>
                {move || format!(" {} ", sest_clock(f.now.get()))}
                <span class="strip-date">{move || sest_date(f.now.get())}</span>
            </div>
            {move || {
                q.war.get().map(|w| {
                    view! { <div class="strip-day caps num">{format!("Day {}", war_day(&w.started, f.now.get()))}</div> }
                })
            }}
            <PhoneMenu />
        </header>
    }
}

/// On phones the header holds what the rail's foot holds on wider screens.
#[component]
fn PhoneMenu() -> impl IntoView {
    let u = ui();
    let open = RwSignal::new(false);
    use_escape_when(move || open.set(false), move || open.get());
    let seen = Memo::new(move |_| settings().get(|s| s.seen_changelog.clone()));
    let pick = move |f: fn()| {
        move |_| {
            open.set(false);
            f();
        }
    };
    view! {
        <div class="top-phone">
            <button
                class=move || format!("top-icon {}", if u.view.get() == View::Settings { "on" } else { "" })
                aria-label="Settings"
                on:click=move |_| u.set_view(View::Settings)
            >
                <Icon icon=icons::SETTINGS size=20.0 stroke_width=1.75 />
            </button>
            <button
                class=move || format!("top-icon {}", if open.get() { "on" } else { "" })
                aria-label="More"
                aria-expanded=move || open.get().to_string()
                on:click=move |_| open.update(|o| *o = !*o)
            >
                <Icon icon=icons::ELLIPSIS size=20.0 stroke_width=2.0 />
                {move || (seen.get() != APP_VERSION).then(|| view! { <span class="unread" aria-label="Unread"></span> })}
            </button>
            <Show when=move || open.get()>
                <div class="more-scrim" on:click=move |_| open.set(false)></div>
                <div class="more-menu panel glass" role="menu">
                    <button role="menuitem" on:click=pick(|| ui().toggle_dispatch(Some(true)))>
                        <Icon icon=icons::SCROLL_TEXT size=18.0 />
                        "Tracker Dispatch"
                    </button>
                    <button role="menuitem" on:click=pick(|| ui().show_dialog(Some(InfoDialog::About)))>
                        <Icon icon=icons::INFO size=18.0 />
                        "About"
                    </button>
                    <button role="menuitem" on:click=pick(|| ui().show_dialog(Some(InfoDialog::Legend)))>
                        <Icon icon=icons::BOOK_OPEN size=18.0 />
                        "Icons legend"
                    </button>
                </div>
            </Show>
        </div>
    }
}

struct Tab {
    view: View,
    label: &'static str,
    short: &'static str,
    icon: IconLike,
    key: &'static str,
}

const TABS: [Tab; 5] = [
    Tab { view: View::Map, label: "Galaxy map", short: "Map", icon: IconLike::Lucide(icons::ORBIT), key: "1" },
    Tab {
        view: View::Campaigns,
        label: "Active planets",
        short: "Planets",
        icon: IconLike::Lucide(icons::LAYOUT_GRID),
        key: "2",
    },
    Tab { view: View::Hellpad, label: "Hellpad", short: "Hellpad", icon: IconLike::Hellpad, key: "3" },
    Tab {
        view: View::Overview,
        label: "High Command",
        short: "Command",
        icon: IconLike::Lucide(icons::SHIELD_HALF),
        key: "4",
    },
    Tab {
        view: View::News,
        label: "Strohmann News",
        short: "News",
        icon: IconLike::Lucide(icons::NEWSPAPER),
        key: "5",
    },
];

/// The destinations: a rail down the left on wide screens, a tab bar along the foot on phones.
#[component]
fn Nav(#[prop(into)] unread_news: Signal<usize>) -> impl IntoView {
    let u = ui();
    let seen = Memo::new(move |_| settings().get(|s| s.seen_changelog.clone()));
    let on_key = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
        let in_field = e
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
            .and_then(|el| el.closest("input, textarea, select").ok().flatten())
            .is_some();
        if e.ctrl_key() || e.meta_key() || e.alt_key() || in_field {
            return;
        }
        if js::document().query_selector("dialog[open], .overlay").ok().flatten().is_some() {
            return;
        }
        if let Some(t) = TABS.iter().find(|t| t.key == e.key()) {
            u.set_view(t.view);
        }
    });
    let _ = js::window().add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref());
    let on_key = send_wrapper::SendWrapper::new(on_key);
    on_cleanup(move || {
        let f = on_key.take();
        let _ = js::window().remove_event_listener_with_callback("keydown", f.as_ref().unchecked_ref());
    });
    let foot_btn = move |view: Option<View>, icon: IconData, label: &'static str, on_click: fn()| {
        view! {
            <button
                class=move || {
                    format!("nav-btn {}", if view.is_some_and(|v| u.view.get() == v) { "on" } else { "" })
                }
                aria-label=label
                title=label
                on:click=move |_| on_click()
            >
                <Icon icon=icon size=20.0 stroke_width=1.75 />
            </button>
        }
    };
    view! {
        <nav class="nav" aria-label="Main">
            <button class="nav-logo" on:click=move |_| u.set_view(View::Campaigns) aria-label="WarCommand: Active planets">
                <AppMark size=30.0 />
            </button>
            <div class="nav-tabs">
                {TABS
                    .iter()
                    .map(|t| {
                        let (view, label, short, icon, key) = (t.view, t.label, t.short, t.icon, t.key);
                        let on = move || u.view.get() == view;
                        view! {
                            <button
                                class=move || format!("nav-tab {}", if on() { "on" } else { "" })
                                aria-current=move || on().then_some("page")
                                on:click=move |_| u.set_view(view)
                                title=format!("{label} ({key})")
                            >
                                <span class="nav-icon">{icon_view(icon, 24.0, 1.6, None)}</span>
                                <span class="nav-label">{label}</span>
                                <span class="nav-short">{short}</span>
                                <span class="nav-key">
                                    <Keycap k=key />
                                </span>
                                {(view == View::News)
                                    .then(|| {
                                        move || {
                                            let n = unread_news.get();
                                            (n > 0)
                                                .then(|| {
                                                    view! {
                                                        <span class="nav-badge num" aria-label=format!("{n} new")>
                                                            {n}
                                                        </span>
                                                    }
                                                })
                                        }
                                    })}
                            </button>
                        }
                    })
                    .collect_view()}
            </div>
            <div class="nav-foot">
                {foot_btn(Some(View::Settings), icons::SETTINGS, "Settings", || ui().set_view(View::Settings))}
                {foot_btn(None, icons::INFO, "About", || ui().show_dialog(Some(InfoDialog::About)))}
                {foot_btn(None, icons::BOOK_OPEN, "Icons legend", || ui().show_dialog(Some(InfoDialog::Legend)))}
                <button
                    class=move || format!("nav-btn {}", if u.dispatch_open.get() { "on" } else { "" })
                    aria-label="Tracker Dispatch"
                    title="Tracker Dispatch"
                    aria-expanded=move || u.dispatch_open.get().to_string()
                    on:click=move |_| u.toggle_dispatch(None)
                >
                    <Icon icon=icons::SCROLL_TEXT size=20.0 stroke_width=1.75 />
                    {move || (seen.get() != APP_VERSION).then(|| view! { <span class="unread" aria-label="Unread"></span> })}
                </button>
            </div>
        </nav>
    }
}

fn toast_look(k: ToastKind) -> (&'static str, IconData) {
    match k {
        ToastKind::Order => ("var(--cta)", icons::TARGET),
        ToastKind::Campaign => ("var(--info)", icons::FLAG),
        ToastKind::Dispatch => ("var(--info)", icons::RADIO),
        ToastKind::Capture => ("var(--good)", icons::FLAG),
        ToastKind::Loss => ("var(--bad)", icons::FLAG_OFF),
        ToastKind::Defense => ("var(--warn)", icons::SHIELD_ALERT),
    }
}

#[component]
fn Toasts() -> impl IntoView {
    let t = toasts();
    let popups = Memo::new(move |_| settings().get(|s| s.popups));
    let shown = Memo::new(move |_| {
        let popups = popups.get();
        t.toasts.get().into_iter().filter(|t| pops_up(popups, t.kind)).collect::<Vec<_>>()
    });
    view! {
        <div class="toasts" aria-live="polite">
            <UpdateCard />
            <For each=move || shown.get() key=|t| t.id let:toast>
                {
                    let (tone, icon) = toast_look(toast.kind);
                    let id = toast.id;
                    view! {
                        <div class="toast" style=format!("--tone: {tone};")>
                            <button class="toast-open" on:click=move |_| t.open(id) title="Open briefing">
                                <span class="toast-icon">
                                    <Icon icon=icon size=16.0 />
                                </span>
                                <span class="toast-body">
                                    <b>{toast.title.clone()}</b>
                                    {format!(" {}", toast.body)}
                                </span>
                            </button>
                            <button class="toast-x" aria-label="Dismiss" on:click=move |_| t.dismiss(id)>
                                <Icon icon=icons::X size=16.0 />
                            </button>
                        </div>
                    }
                }
            </For>
        </div>
    }
}

#[component]
pub fn Shell(children: ChildrenFn) -> impl IntoView {
    let u = ui();
    let unread = unread_news();
    // On the page itself the items carry their own marks.
    let unread_news = Signal::derive(move || if u.view.get() == View::News { 0 } else { unread.get() });
    crate::update::start_update_checks();
    view! {
        <div class="shell">
            <TopBar />
            <Nav unread_news=unread_news />
            <main class="main">{children()}</main>
            <DispatchPanel />
            <AboutDialog />
            <PlanetOptions />
            <PlanetDetail />
            <CityDetail />
            <Toasts />
            <BriefingPopup />
        </div>
    }
}
