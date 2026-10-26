use crate::js::{self, num};
use crate::tauri;
use crate::ui::emblems::{FactionEmblem, IconLike, icon_view};
use crate::ui::icons::{self, Icon, IconData};
use leptos::prelude::*;
use leptos::task::spawn_local;
use send_wrapper::SendWrapper;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use warroom_core::api::Faction;
use warroom_core::factions::{faction_color, faction_label};
use warroom_core::format::{fmt_duration, js_round};

/// Panel with a hatch-striped header strip: icon box, title, and an optional right slot.
#[component]
pub fn Panel(
    #[prop(optional, into)] title: Option<TextProp>,
    #[prop(optional, into)] icon: Option<IconLike>,
    #[prop(optional)] right: Option<AnyView>,
    /// Tints the header stripe and icon box. Defaults to the CTA yellow.
    #[prop(optional, into)]
    tone: Option<String>,
    #[prop(optional)] glass: bool,
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] body_class: Option<String>,
    children: Children,
) -> impl IntoView {
    let style = tone.map(|t| format!("--tone: {t};"));
    let class = format!("panel {} {}", if glass { "glass" } else { "" }, class.unwrap_or_default());
    view! {
        <section class=class style=style>
            {title.map(|title| {
                view! {
                    <header class="panel-head">
                        {icon.map(|i| view! { <span class="panel-icon">{icon_view(i, 14.0, 2.0, None)}</span> })}
                        <h2 class="panel-title">{move || title.get()}</h2>
                        {right.map(|r| view! { <div class="panel-right">{r}</div> })}
                    </header>
                }
            })}
            <div class=body_class.unwrap_or_else(|| "panel-body".into())>{children()}</div>
        </section>
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MeterSize {
    S,
    #[default]
    M,
    L,
}

/// Progress meter: tick-gapped fill with a hatched remainder. `rest` colours the remainder
/// (usually the enemy faction), `marker` draws a projected end position.
#[component]
pub fn Meter(
    #[prop(into)] value: Signal<f64>,
    #[prop(into)] color: Signal<String>,
    #[prop(optional, into)] rest: Option<Signal<String>>,
    #[prop(optional, into)] marker: Option<Signal<Option<f64>>>,
    #[prop(optional)] size: MeterSize,
    /// Chevron fill used for the big liberation bars.
    #[prop(optional)]
    chevron: bool,
    #[prop(optional, into)] label: Option<String>,
) -> impl IntoView {
    let v = move || value.get().clamp(0.0, 100.0);
    let size = match size {
        MeterSize::S => "s",
        MeterSize::M => "m",
        MeterSize::L => "l",
    };
    let class = format!("meter meter-{size} {}", if chevron { "chevron" } else { "" });
    let style = move || {
        format!("--fill: {}; --rest: {};", color.get(), rest.map_or_else(|| "var(--line-strong)".into(), |r| r.get()))
    };
    let mark = move || marker.and_then(|m| m.get()).filter(|m| *m > 0.0 && *m < 100.0);
    view! {
        <div
            class=class
            style=style
            role="progressbar"
            aria-valuenow=move || num(js_round(v()))
            aria-valuemin="0"
            aria-valuemax="100"
            aria-label=label
        >
            <div class="meter-fill" style=move || format!("width: {}%;", num(v()))></div>
            <div class="meter-rest" style=move || format!("left: {}%;", num(v()))></div>
            {move || mark().map(|m| view! { <div class="meter-marker" style=format!("left: {}%;", num(m))></div> })}
        </div>
    }
}

#[component]
pub fn Keycap(#[prop(into)] k: String) -> impl IntoView {
    view! { <kbd class="keycap">{k}</kbd> }
}

#[component]
pub fn HudButton(
    icon: IconLike,
    #[prop(into)] label: String,
    #[prop(optional)] on_click: Option<Callback<()>>,
    #[prop(optional, into)] active: Option<Signal<bool>>,
    #[prop(optional, into)] key_hint: Option<String>,
    #[prop(default = 18.0)] size: f64,
) -> impl IntoView {
    let title = match &key_hint {
        Some(k) => format!("{label} ({k})"),
        None => label.clone(),
    };
    view! {
        <button
            class=move || format!("hud-btn icon {}", if active.is_some_and(|a| a.get()) { "on" } else { "" })
            on:click=move |_| {
                if let Some(f) = on_click {
                    f.run(())
                }
            }
            aria-label=label
            title=title
        >
            {icon_view(icon, size, 1.75, None)}
        </button>
    }
}

#[component]
pub fn CtaButton(
    #[prop(optional)] icon: Option<IconData>,
    #[prop(optional)] on_click: Option<Callback<()>>,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            class="cta-btn"
            on:click=move |_| {
                if let Some(f) = on_click {
                    f.run(())
                }
            }
        >
            {icon.map(|i| view! { <Icon icon=i size=15.0 stroke_width=2.0 /> })}
            {children()}
        </button>
    }
}

/// Button with a keycap prefix: [ESC] CLOSE.
#[component]
pub fn KeyButton(
    #[prop(into)] k: String,
    #[prop(optional)] on_click: Option<Callback<()>>,
    #[prop(optional)] badge: Option<AnyView>,
    #[prop(optional, into)] active: Option<Signal<bool>>,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            class=move || format!("key-btn {}", if active.is_some_and(|a| a.get()) { "on" } else { "" })
            on:click=move |_| {
                if let Some(f) = on_click {
                    f.run(())
                }
            }
        >
            <Keycap k=k />
            <span>{children()}</span>
            {badge.map(|b| view! { <span class="key-badge num">{b}</span> })}
        </button>
    }
}

#[component]
pub fn StatusChip(#[prop(default = "var(--good)".into(), into)] tone: String, children: Children) -> impl IntoView {
    view! {
        <span class="status-chip" style=format!("--tone: {tone};")>
            {children()}
        </span>
    }
}

#[component]
pub fn LiveDot(#[prop(default = true)] ok: bool) -> impl IntoView {
    view! { <span class=format!("live-dot {}", if ok { "" } else { "off" }) aria-hidden="true"></span> }
}

/// ◀ VALUE ▶ selector with a segmented underline showing the position.
#[component]
pub fn Stepper<T>(
    options: Vec<(T, &'static str)>,
    #[prop(into)] value: Signal<T>,
    on_change: impl Fn(T) + Clone + 'static,
    #[prop(into)] label: String,
) -> impl IntoView
where
    T: Clone + PartialEq + Send + Sync + 'static,
{
    let options = std::sync::Arc::new(options);
    let index = {
        let options = options.clone();
        move || options.iter().position(|(v, _)| *v == value.get()).unwrap_or(0)
    };
    let go = {
        let options = options.clone();
        let index = index.clone();
        move |d: isize| {
            let n = options.len() as isize;
            let i = (index() as isize + d + n) % n;
            on_change(options[i as usize].0.clone());
        }
    };
    let (go_prev, go_next) = (go.clone(), go);
    let text = {
        let options = options.clone();
        let index = index.clone();
        move || options.get(index()).map(|o| o.1).unwrap_or_default()
    };
    let segs = move || {
        let i = index();
        (0..options.len()).map(|k| view! { <span class=if k == i { "on" } else { "" }></span> }).collect_view()
    };
    view! {
        <div class="stepper" role="group" aria-label=label.clone()>
            <button class="stepper-arrow" on:click=move |_| go_prev(-1) aria-label=format!("Previous {label}")>
                <Icon icon=icons::CHEVRON_LEFT size=18.0 />
            </button>
            <div class="stepper-value">
                <span class="caps">{text}</span>
                <span class="stepper-segs">{segs}</span>
            </div>
            <button class="stepper-arrow" on:click=move |_| go_next(1) aria-label=format!("Next {label}")>
                <Icon icon=icons::CHEVRON_RIGHT size=18.0 />
            </button>
        </div>
    }
}

/// Labelled value tile.
#[component]
pub fn Tile(
    #[prop(into)] label: String,
    value: AnyView,
    #[prop(optional)] sub: Option<AnyView>,
    #[prop(optional)] icon: Option<IconData>,
    #[prop(optional, into)] color: Option<String>,
    #[prop(optional, into)] rule: Option<String>,
) -> impl IntoView {
    view! {
        <div class="tile" style=rule.map(|r| format!("--rule: {r};"))>
            <div class="tile-label">{icon.map(|i| view! { <Icon icon=i size=12.0 stroke_width=2.0 /> })} {label}</div>
            <div class="tile-value num" style=color.map(|c| format!("color: {c};"))>
                {value}
            </div>
            {sub.map(|s| view! { <div class="tile-sub">{s}</div> })}
        </div>
    }
}

#[component]
pub fn FactionTag(faction: Faction) -> impl IntoView {
    view! {
        <span class="faction-tag" style=format!("color: {};", faction_color(Some(faction)))>
            <FactionEmblem faction=faction.as_str() size=12.0 />
            {faction_label(Some(faction))}
        </span>
    }
}

thread_local! {
    static LAYERS: RefCell<Vec<(u64, Rc<dyn Fn()>)>> = const { RefCell::new(Vec::new()) };
    static NEXT_LAYER: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static LISTENER: RefCell<Option<Closure<dyn Fn(web_sys::KeyboardEvent)>>> = const { RefCell::new(None) };
}

fn ensure_listener() {
    LISTENER.with_borrow_mut(|l| {
        if l.is_some() {
            return;
        }
        let f = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(|e: web_sys::KeyboardEvent| {
            // A modal <dialog> on top closes itself.
            let modal_open = js::document().query_selector("dialog[open]").ok().flatten().is_some();
            if e.key() == "Escape" && !modal_open {
                let top = LAYERS.with_borrow(|layers| layers.last().map(|(_, f)| f.clone()));
                if let Some(close) = top {
                    close();
                }
            }
        });
        let _ = js::window().add_event_listener_with_callback("keydown", f.as_ref().unchecked_ref());
        *l = Some(f);
    });
}

/// Escape closes only the layer opened last, so one press is one step back through the history. Layers stack
/// in the order they open; keep that the order they are drawn in. The layer lives while the caller does.
pub fn use_escape(close: impl Fn() + 'static) {
    let id = NEXT_LAYER.with(|n| {
        let id = n.get();
        n.set(id + 1);
        id
    });
    LAYERS.with_borrow_mut(|l| l.push((id, Rc::new(close))));
    ensure_listener();
    on_cleanup(move || LAYERS.with_borrow_mut(|l| l.retain(|(i, _)| *i != id)));
}

/// `use_escape` for a layer that is open only while `open()` holds.
pub fn use_escape_when(close: impl Fn() + Clone + 'static, open: impl Fn() -> bool + 'static) {
    let current = Rc::new(RefCell::new(None::<u64>));
    let c = current.clone();
    let effect = Effect::new(move |_| {
        let is_open = open();
        let mut cur = c.borrow_mut();
        match (is_open, *cur) {
            (true, None) => {
                let id = NEXT_LAYER.with(|n| {
                    let id = n.get();
                    n.set(id + 1);
                    id
                });
                let close = close.clone();
                LAYERS.with_borrow_mut(|l| l.push((id, Rc::new(close))));
                ensure_listener();
                *cur = Some(id);
            }
            (false, Some(id)) => {
                LAYERS.with_borrow_mut(|l| l.retain(|(i, _)| *i != id));
                *cur = None;
            }
            _ => {}
        }
    });
    let current = SendWrapper::new(current);
    on_cleanup(move || {
        drop(effect);
        if let Some(id) = current.take().borrow_mut().take() {
            LAYERS.with_borrow_mut(|l| l.retain(|(i, _)| *i != id));
        }
    });
}

/// Full-viewport overlay on grid paper, closed with ESC or the corner button.
#[component]
pub fn Overlay(
    on_close: Callback<()>,
    #[prop(into)] label: String,
    #[prop(optional)] footer: Option<AnyView>,
    children: Children,
) -> impl IntoView {
    use_escape(move || on_close.run(()));
    view! {
        <div class="overlay" role="dialog" aria-modal="true" aria-label=label>
            <div class="overlay-scroll scroll">{children()}</div>
            <div class="overlay-foot">
                <KeyButton k="ESC" on_click=on_close>
                    "Close"
                </KeyButton>
                {footer}
            </div>
        </div>
    }
}

#[component]
pub fn Countdown(#[prop(into)] to: Signal<f64>) -> impl IntoView {
    let now = js::use_now(30_000.0);
    view! { <span class="num">{move || fmt_duration(to.get() - now.get())}</span> }
}

/// Skeleton rows while a snapshot loads.
#[component]
pub fn Loading(#[prop(into)] what: String, #[prop(default = 3)] rows: usize) -> impl IntoView {
    view! {
        <div class="loading" aria-busy="true" aria-label=format!("Loading {what}")>
            {(0..rows).map(|_| view! { <div class="skeleton"></div> }).collect_view()}
            <div class="loading-text caps">{format!("Receiving {what}…")}</div>
        </div>
    }
}

#[component]
pub fn Empty(children: Children) -> impl IntoView {
    view! { <div class="empty caps">{children()}</div> }
}

/// A link out of the app: a new tab on the web, the system browser in the desktop app.
#[component]
pub fn OutLink(#[prop(into)] href: String, children: Children) -> impl IntoView {
    let target = href.clone();
    view! {
        <a
            href=href
            target="_blank"
            rel="noreferrer"
            on:click=move |e| {
                if !tauri::is_tauri() {
                    return;
                }
                e.prevent_default();
                let url = target.clone();
                spawn_local(async move {
                    let _ = tauri::open_url(&url).await;
                });
            }
        >
            {children()}
        </a>
    }
}
