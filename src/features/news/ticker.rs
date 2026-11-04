use crate::api::history::history;
use crate::api::queries::queries;
use crate::js;
use crate::state::{View, ui};
use crate::ui::emblems::FactionEmblem;
use crate::ui::icons::{self, Icon};
use leptos::html;
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use warroom_core::campaign_news::campaign_events;
use warroom_core::format::fmt_age_short;
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

    // Scrolled by a transform animation the compositor runs, so the strip costs the page nothing a frame. Its
    // width changes every minute as ages tick over; the animation is then re-timed to hold its place, since
    // restarting it, or leaving its old timing, made it jump.
    let anim: StoredValue<Option<SendWrapper<JsValue>>, LocalStorage> = StoredValue::new_local(None);
    let fit = move || {
        let Some(el) = track.get_untracked() else { return };
        let half = el.scroll_width() as f64 / 2.0;
        if half <= 0.0 {
            return;
        }
        let duration = half / PX_PER_S * 1000.0;
        let current = anim.with_value(|a| a.as_ref().map(|a| (**a).clone()));
        match current {
            Some(a) => {
                let effect = js::get(&a, "effect");
                let was = js::get(&js::call(&effect, "getTiming", &[]).unwrap_or_default(), "duration").as_f64();
                let at = js::get(&a, "currentTime").as_f64().unwrap_or(0.0);
                let _ = js::call(&effect, "updateTiming", &[js::obj(&[("duration", duration.into())])]);
                // The same distance into the new, longer or shorter loop.
                if let Some(was) = was.filter(|w| *w > 0.0) {
                    let offset = (at % was) / was * (was / 1000.0 * PX_PER_S);
                    let _ = js_sys::Reflect::set(&a, &"currentTime".into(), &(offset / PX_PER_S * 1000.0).into());
                }
            }
            None => {
                let frames = js_sys::Array::of2(
                    &js::obj(&[("transform", "translateX(0)".into())]),
                    &js::obj(&[("transform", "translateX(-50%)".into())]),
                );
                let options = js::obj(&[
                    ("duration", duration.into()),
                    ("iterations", f64::INFINITY.into()),
                    ("easing", "linear".into()),
                ]);
                if let Ok(a) = js::call(&el.into(), "animate", &[frames.into(), options]) {
                    anim.set_value(Some(SendWrapper::new(a)));
                }
            }
        }
    };
    let hold = move |on: bool| {
        anim.with_value(|a| {
            if let Some(a) = a {
                let _ = js::call(a, if on { "pause" } else { "play" }, &[]);
            }
        })
    };
    // Measured a frame after the strip redraws, once its new width is laid out.
    Effect::new(move |_| {
        items.track();
        now.track();
        let f = Closure::once_into_js(fit);
        let _ = js::window().request_animation_frame(f.unchecked_ref());
    });
    on_cleanup(move || {
        if let Some(a) = anim.try_update_value(Option::take).flatten() {
            let _ = js::call(&a, "cancel", &[]);
        }
    });

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
            on:pointerenter=move |_| hold(true)
            on:pointerleave=move |_| hold(false)
            on:focusin=move |_| hold(true)
            on:focusout=move |_| hold(false)
        >
            <div node_ref=track class="ticker-track caps">
                {contents}
            </div>
        </div>
    }
    .into_any()
}
