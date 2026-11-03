use crate::api::history::history;
use crate::api::queries::queries;
use crate::js;
use crate::state::{View, ui};
use crate::ui::emblems::FactionEmblem;
use crate::ui::icons::{self, Icon};
use leptos::prelude::*;
use warroom_core::campaign_news::campaign_events;
use warroom_core::format::fmt_age_short;
use warroom_core::ticker::{TickerItem, TickerKind, ticker_items};

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

fn item_view(i: TickerItem, now: f64) -> impl IntoView {
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
        >
            {glyph(&i)}
            <span class="ticker-text">{i.text.clone()}</span>
            <span class="ticker-age num">{fmt_age_short(i.at, now)}</span>
        </button>
    }
}

/// The last day's war news in the status strip, one item at a time; it holds under the pointer and each item
/// opens its source.
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
    let held = RwSignal::new(false);
    let (step, set_step) = signal(0usize);
    let handle = set_interval_with_handle(
        move || {
            if !held.get_untracked() {
                set_step.update(|n| *n += 1);
            }
        },
        std::time::Duration::from_millis(STEP_MS),
    )
    .ok();
    on_cleanup(move || {
        if let Some(h) = handle {
            h.clear()
        }
    });
    view! {
        <div
            class="ticker still caps"
            aria-label=move || (!items.with(Vec::is_empty)).then_some("Latest war news")
            on:pointerenter=move |_| held.set(true)
            on:pointerleave=move |_| held.set(false)
            on:focusin=move |_| held.set(true)
            on:focusout=move |_| held.set(false)
        >
            {move || {
                let list = items.get();
                (!list.is_empty()).then(|| item_view(list[step.get() % list.len()].clone(), now.get()))
            }}
        </div>
    }
}
