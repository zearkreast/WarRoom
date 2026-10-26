use crate::api::queries::queries;
use crate::history::history;
use crate::js;
use crate::state::{View, ui};
use crate::ui::emblems::FactionEmblem;
use crate::ui::icons::{self, Icon};
use leptos::html;
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use warroom_core::campaign_news::campaign_events;
use warroom_core::format::{fmt_age_short, to_fixed};
use warroom_core::ticker::{TickerItem, TickerKind, ticker_items};
use wasm_bindgen::prelude::*;

/// Scroll speed, so a long strip doesn't race and a short one doesn't crawl.
const PX_PER_S: f64 = 45.0;
const STEP_MS: u64 = 8000;

fn glyph(item: &TickerItem) -> AnyView {
    if item.kind == TickerKind::Dispatch {
        return view! { <Icon icon=icons::RADIO size=12.0 /> }.into_any();
    }
    if item.kind == TickerKind::Campaign && item.faction.is_none() {
        return view! { <Icon icon=icons::FLAG size=12.0 /> }.into_any();
    }
    let f = item.faction.map_or("Humans", |f| f.as_str());
    view! { <FactionEmblem faction=f size=12.0 /> }.into_any()
}

fn open(i: &TickerItem) {
    match i.planet {
        Some(p) => ui().show_on_map(p),
        None => ui().set_view(if i.kind == TickerKind::Campaign { View::Overview } else { View::News }),
    }
}

fn item_view(i: TickerItem, copy: bool, now: f64) -> impl IntoView {
    let target = i.clone();
    view! {
        <button
            class=format!(
                "ticker-item {}",
                match i.kind {
                    TickerKind::Dispatch => "dispatch",
                    TickerKind::Liberated => "liberated",
                    TickerKind::Lost => "lost",
                    TickerKind::Campaign => "campaign",
                },
            )
            on:click=move |_| open(&target)
            tabindex=copy.then_some("-1")
            aria-hidden=copy.then_some("true")
        >
            {glyph(&i)}
            <span class="ticker-text">{i.text.clone()}</span>
            <span class="ticker-age num">{fmt_age_short(i.at, now)}</span>
        </button>
    }
}

/// The last day's war news scrolling in the status strip; it pauses under the pointer and each item opens its source.
#[component]
pub fn NewsTicker() -> impl IntoView {
    let q = queries();
    let changes = history().changes;
    let now = js::use_now(60_000.0);
    let items = Memo::new(move |_| {
        let dispatches = q.dispatches.get();
        let episodes = q.episodes.get();
        let d = dispatches.as_deref().map_or(&[][..], Vec::as_slice);
        let changes: Vec<_> = changes.with(|c| c.iter().map(|r| r.change.clone()).collect());
        let events = campaign_events(episodes.as_deref().map_or(&[][..], Vec::as_slice), d);
        ticker_items(d, &changes, &events, now.get())
    });
    let reduced = js::prefers_reduced_motion();
    let track = NodeRef::<html::Div>::new();
    let held = Rc::new(Cell::new(false));

    if reduced {
        let (step, set_step) = signal(0usize);
        let handle =
            set_interval_with_handle(move || set_step.update(|n| *n += 1), std::time::Duration::from_millis(STEP_MS))
                .ok();
        on_cleanup(move || {
            if let Some(h) = handle {
                h.clear()
            }
        });
        return (move || {
            let list = items.get();
            if list.is_empty() {
                return view! { <div class="ticker"></div> }.into_any();
            }
            let i = list[step.get() % list.len()].clone();
            view! { <div class="ticker still" aria-live="polite">{item_view(i, false, now.get())}</div> }.into_any()
        })
        .into_any();
    }

    // Scrolled by offset rather than a CSS animation: the strip's width changes every minute as ages
    // tick over, and re-timing an animation mid-run made it jump.
    let alive = Rc::new(Cell::new(true));
    {
        let held = held.clone();
        let alive = alive.clone();
        let offset = Cell::new(0.0);
        let last = Cell::new(js::window().performance().map_or(0.0, |p| p.now()));
        let tick: Rc<RefCell<Option<Closure<dyn Fn(f64)>>>> = Rc::new(RefCell::new(None));
        let again = tick.clone();
        *tick.borrow_mut() = Some(Closure::new(move |t: f64| {
            if !alive.get() {
                again.borrow_mut().take();
                return;
            }
            let dt = ((t - last.get()) / 1000.0).min(0.1);
            last.set(t);
            if let Some(el) = track.get_untracked() {
                if !held.get() && !js::document().hidden() {
                    let lp = el.scroll_width() as f64 / 2.0;
                    offset.set(if lp > 0.0 { (offset.get() + PX_PER_S * dt) % lp } else { 0.0 });
                    let _ = web_sys::HtmlElement::style(&el)
                        .set_property("transform", &format!("translateX({}px)", to_fixed(-offset.get(), 1)));
                }
            }
            if let Some(f) = again.borrow().as_ref() {
                let _ = js::window().request_animation_frame(f.as_ref().unchecked_ref());
            }
        }));
        if let Some(f) = tick.borrow().as_ref() {
            let _ = js::window().request_animation_frame(f.as_ref().unchecked_ref());
        }
    }
    let alive = SendWrapper::new(alive);
    on_cleanup(move || alive.set(false));

    let (h1, h2, h3, h4) = (
        SendWrapper::new(held.clone()),
        SendWrapper::new(held.clone()),
        SendWrapper::new(held.clone()),
        SendWrapper::new(held),
    );
    let contents = move || {
        let list = items.get();
        let t = now.get();
        view! {
            {list.iter().cloned().map(|i| item_view(i, false, t)).collect_view()}
            {list.iter().cloned().map(|i| item_view(i, true, t)).collect_view()}
        }
    };
    view! {
        <div
            class="ticker"
            aria-label=move || (!items.with(Vec::is_empty)).then_some("Latest war news")
            on:pointerenter=move |_| h1.set(true)
            on:pointerleave=move |_| h2.set(false)
            on:focusin=move |_| h3.set(true)
            on:focusout=move |_| h4.set(false)
        >
            <div node_ref=track class="ticker-track caps">
                {contents}
            </div>
        </div>
    }
    .into_any()
}
