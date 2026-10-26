use crate::alerts::{ToastKind, toasts};
use crate::api::queries::queries;
use crate::config::APP_VERSION;
use crate::features::news::items::unread_news;
use crate::features::news::ticker::NewsTicker;
use crate::features::planet::detail::PlanetDetail;
use crate::features::planet::options::PlanetOptions;
use crate::js;
use crate::layout::about_dialog::AboutDialog;
use crate::layout::briefing_popup::BriefingPopup;
use crate::layout::dispatch_panel::DispatchPanel;
use crate::state::{InfoDialog, Popups, View, settings, ui};
use crate::ui::emblems::{AppMark, IconLike, icon_view};
use crate::ui::feed::{FeedState, use_feed_status};
use crate::ui::icons::{self, Icon, IconData};
use crate::ui::primitives::{Keycap, StatusChip};
use crate::update::card::UpdateCard;
use leptos::prelude::*;
use warroom_core::format::{fmt_duration, fmt_int, fmt_pct, sest_clock, sest_date, war_day};
use warroom_core::galaxy::human_control;
use warroom_core::progress::liberation_percent;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

#[component]
pub fn Logo() -> impl IntoView {
    view! {
        <div class="logo">
            <AppMark />
            <div>
                <div class="logo-name">"WarCommand"</div>
                <div class="logo-sub">"Galactic War Tracker"</div>
            </div>
        </div>
    }
}

#[component]
fn TopButton(
    view: View,
    icon: IconData,
    small: &'static str,
    big: &'static str,
    #[prop(optional)] class: &'static str,
    #[prop(optional, into)] badge: Option<Signal<usize>>,
) -> impl IntoView {
    let u = ui();
    let on = move || u.view.get() == view;
    view! {
        <button
            class=move || format!("top-btn {} {class}", if on() { "on" } else { "" })
            on:click=move |_| u.set_view(if on() { View::Campaigns } else { view })
            aria-pressed=move || on().to_string()
        >
            <span class="top-btn-icon">
                <Icon icon=icon size=18.0 stroke_width=1.75 />
            </span>
            <span class="top-btn-text">
                <span class="top-btn-small">{small}</span>
                <span class="top-btn-big">{big}</span>
            </span>
            {move || {
                let n = badge.map_or(0, |b| b.get());
                (n > 0).then(|| view! {
                    <span class="top-btn-badge num" aria-label=format!("{n} new")>
                        {n}
                    </span>
                })
            }}
        </button>
    }
}

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
            let pct = fmt_pct(liberation_percent(top.planet.health, top.planet.max_health), 1);
            lines.push(view! { {kind} " at " <b class="hi">{name}</b> " " <b class="hi">{pct}</b> }.into_any());
        }
    }
    if let Some(war) = q.war.get() {
        let n = fmt_int(war.statistics.player_count);
        lines.push(view! { <b class="hi">{n}</b> " Helldivers deployed" }.into_any());
    }
    lines
}

#[component]
fn StatusStrip() -> impl IntoView {
    let q = queries();
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
        <div class="strip">
            <div class="strip-status">
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
        </div>
    }
}

const TABS: [(View, &str, IconLike, &str); 3] = [
    (View::Map, "Galaxy map", IconLike::Lucide(icons::ORBIT), "1"),
    (View::Campaigns, "Active planets", IconLike::Lucide(icons::LAYOUT_GRID), "2"),
    (View::Hellpad, "Hellpad", IconLike::Hellpad, "3"),
];

#[component]
fn Dock() -> impl IntoView {
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
        if let Some(t) = TABS.iter().find(|t| t.3 == e.key()) {
            u.set_view(t.0);
        }
    });
    let _ = js::window().add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref());
    let on_key = send_wrapper::SendWrapper::new(on_key);
    on_cleanup(move || {
        let f = on_key.take();
        let _ = js::window().remove_event_listener_with_callback("keydown", f.as_ref().unchecked_ref());
    });
    view! {
        <nav class="dock" aria-label="Main">
            <div class="dock-side">
                <button
                    class=move || format!("hud-btn icon {}", if u.view.get() == View::Settings { "on" } else { "" })
                    aria-label="Settings"
                    title="Settings"
                    on:click=move |_| {
                        u.set_view(if u.view.get_untracked() == View::Settings { View::Campaigns } else { View::Settings })
                    }
                >
                    <Icon icon=icons::SETTINGS size=18.0 stroke_width=1.75 />
                </button>
                <button
                    class="hud-btn icon"
                    aria-label="About"
                    title="About"
                    on:click=move |_| u.show_dialog(Some(InfoDialog::About))
                >
                    <Icon icon=icons::INFO size=18.0 stroke_width=1.75 />
                </button>
                <button
                    class="hud-btn icon"
                    aria-label="Icons legend"
                    title="Icons legend"
                    on:click=move |_| u.show_dialog(Some(InfoDialog::Legend))
                >
                    <Icon icon=icons::BOOK_OPEN size=18.0 stroke_width=1.75 />
                </button>
            </div>
            <div class="dock-tabs" role="tablist">
                {TABS
                    .iter()
                    .map(|&(view, label, icon, key)| {
                        view! {
                            <button
                                role="tab"
                                aria-selected=move || (u.view.get() == view).to_string()
                                class=move || format!("dock-tab {}", if u.view.get() == view { "on" } else { "" })
                                on:click=move |_| u.set_view(view)
                                title=format!("{label} ({key})")
                            >
                                {icon_view(icon, 22.0, 1.6, None)}
                                <span class="dock-label">{label}</span>
                                <span class="dock-key">
                                    <Keycap k=key />
                                </span>
                            </button>
                        }
                    })
                    .collect_view()}
            </div>
            <div class="dock-side right">
                <button
                    class=move || format!("dock-dispatch {}", if u.dispatch_open.get() { "on" } else { "" })
                    on:click=move |_| u.toggle_dispatch(None)
                    aria-expanded=move || u.dispatch_open.get().to_string()
                >
                    <span>"Tracker"</span>
                    <b>"Dispatch"</b>
                    {move || (seen.get() != APP_VERSION).then(|| view! { <span class="unread" aria-label="Unread"></span> })}
                </button>
            </div>
        </nav>
    }
}

fn toast_look(k: ToastKind) -> (&'static str, IconData) {
    match k {
        ToastKind::Order => ("var(--cta)", icons::TARGET),
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
        let list = t.toasts.get();
        match popups.get() {
            Popups::None => Vec::new(),
            Popups::Priority => list.into_iter().filter(|t| t.kind != ToastKind::Dispatch).collect(),
            Popups::All => list,
        }
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
            <header class="topbar">
                <Logo />
                <div class="top-actions">
                    <TopButton view=View::News icon=icons::NEWSPAPER small="Strohmann" big="News" badge=unread_news />
                    <TopButton view=View::Overview icon=icons::SHIELD_HALF small="High Command" big="Overview" />
                    // The dock has no room for Settings on phones.
                    <TopButton view=View::Settings icon=icons::SETTINGS small="Tracker" big="Settings" class="narrow-only" />
                </div>
            </header>
            <StatusStrip />
            <main class="main">{children()}</main>
            <Dock />
            <DispatchPanel />
            <AboutDialog />
            <PlanetOptions />
            <PlanetDetail />
            <Toasts />
            <BriefingPopup />
        </div>
    }
}
