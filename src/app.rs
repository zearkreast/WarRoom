//! The root component: the shell around whichever page is open.

use crate::features::campaigns::page::CampaignsPage;
use crate::features::hellpad::page::HellpadPage;
use crate::features::map::page::MapPage;
use crate::features::news::page::NewsPage;
use crate::features::overview::page::OverviewPage;
use crate::features::settings::page::SettingsPage;
use crate::js;
use crate::layout::shell::Shell;
use crate::state::{View, ui};
use leptos::prelude::*;
use std::time::Duration;

/// How long after a page or Hellpad module opens its sections still rise in (see `.arriving` in ui.css).
const ARRIVAL_MS: u64 = 700;

/// Marks the document while a page or module is arriving, so what a search, sort or poll redraws later doesn't
/// rise in again.
fn mark_arrivals(view: Memo<View>) {
    let module = ui().module;
    let turn = StoredValue::new(0u32);
    let set = |on: bool| {
        if let Some(root) = js::document().document_element() {
            let _ = if on { root.class_list().add_1("arriving") } else { root.class_list().remove_1("arriving") };
        }
    };
    Effect::new(move |_| {
        view.track();
        module.track();
        set(true);
        let n = turn.get_value().wrapping_add(1);
        turn.set_value(n);
        set_timeout(
            move || {
                if turn.get_value() == n {
                    set(false);
                }
            },
            Duration::from_millis(ARRIVAL_MS),
        );
    });
}

#[component]
pub fn App() -> impl IntoView {
    // A memo, so setting the same view again (opening a Hellpad module) doesn't remount the page.
    let view = Memo::new(move |_| ui().view.get());
    mark_arrivals(view);
    view! {
        <Shell>
            {move || match view.get() {
                View::Campaigns => view! { <CampaignsPage /> }.into_any(),
                View::Map => view! { <MapPage /> }.into_any(),
                View::Hellpad => view! { <HellpadPage /> }.into_any(),
                View::Overview => view! { <OverviewPage /> }.into_any(),
                View::News => view! { <NewsPage /> }.into_any(),
                View::Settings => view! { <SettingsPage /> }.into_any(),
            }}
        </Shell>
    }
}
