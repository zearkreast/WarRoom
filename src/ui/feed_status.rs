//! How fresh the war data is, and the chips and notes that say so.

use crate::api::queries::queries;
use crate::js::{self, fmt_date_time};
use crate::ui::primitives::{LiveDot, StatusChip};
use leptos::prelude::*;
use warroom_core::format::fmt_ago;

/// The collector writes campaigns every minute; quiet for this long means it has stopped.
pub const STALE_MS: f64 = 10.0 * 60_000.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeedState {
    Live,
    Stale,
    Offline,
}

impl FeedState {
    pub fn as_str(self) -> &'static str {
        match self {
            FeedState::Live => "live",
            FeedState::Stale => "stale",
            FeedState::Offline => "offline",
        }
    }
}

#[derive(Clone, Copy)]
pub struct FeedStatus {
    pub state: Signal<FeedState>,
    pub last_update: Memo<Option<f64>>,
    pub now: ReadSignal<f64>,
}

/// How fresh the war data is: shared by the status strip, the planet detail and page notes.
pub fn use_feed_status(tick_ms: f64) -> FeedStatus {
    let q = queries();
    let now = js::use_now(tick_ms);
    let last_update = q.last_update;
    let state = Signal::derive(move || {
        if q.raw.error.get() {
            FeedState::Offline
        } else if last_update.get().is_some_and(|l| now.get() - l > STALE_MS) {
            FeedState::Stale
        } else {
            FeedState::Live
        }
    });
    FeedStatus { state, last_update, now }
}

/// Live dot with the data's age, or the STALE / LINK LOST chip; never claims "live" for old data.
#[component]
pub fn FeedTag() -> impl IntoView {
    let f = use_feed_status(15_000.0);
    move || match f.state.get() {
        FeedState::Offline => view! { <StatusChip tone="var(--bad)">"Link lost"</StatusChip> }.into_any(),
        FeedState::Stale => {
            let ago = fmt_ago(f.last_update.get().unwrap_or(0.0), f.now.get());
            view! {
                <StatusChip tone="var(--warn)">"Stale"</StatusChip>
                <span class="num">{format!("Data from {ago}")}</span>
            }
            .into_any()
        }
        FeedState::Live => {
            let ago = f.last_update.get().map(|l| fmt_ago(l, f.now.get()));
            view! {
                <LiveDot />
                " Live"
                {ago.map(|a| view! { <span class="num">{format!("· updated {a}")}</span> })}
            }
            .into_any()
        }
    }
}

/// One line above a page's data when it is stale or offline; nothing while live.
#[component]
pub fn FeedNote() -> impl IntoView {
    let f = use_feed_status(60_000.0);
    move || {
        let state = f.state.get();
        (state != FeedState::Live).then(|| {
            let text = if state == FeedState::Offline {
                "Comms link down: showing the last data received.".to_string()
            } else {
                let at = f.last_update.get().unwrap_or(0.0);
                format!(
                    "Showing data from {} ({}): projections may be off.",
                    fmt_date_time(at),
                    fmt_ago(at, f.now.get())
                )
            };
            view! {
                <p class=format!("feed-note caps {}", state.as_str()) role="status">
                    {text}
                </p>
            }
        })
    }
}
