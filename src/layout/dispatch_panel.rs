use crate::config::APP_VERSION;
use crate::state::{settings, ui};
use crate::ui::icons::{self, Icon};
use crate::ui::primitives::use_escape_when;
use leptos::prelude::*;
use warroom_core::changelog::CHANGELOG;
use warroom_core::version::release_number;

/// Tracker Dispatch: what's new in the app.
#[component]
pub fn DispatchPanel() -> impl IntoView {
    let u = ui();
    Effect::new(move |_| {
        if u.dispatch_open.get() {
            settings().mark_changelog(APP_VERSION);
        }
    });
    use_escape_when(move || u.toggle_dispatch(Some(false)), move || u.dispatch_open.get());
    move || {
        u.dispatch_open.get().then(|| {
            view! {
                <aside class="dispatch-panel panel glass" aria-label="Tracker dispatch">
                    <div class="dp-head">
                        <Icon icon=icons::SCROLL_TEXT size=14.0 />
                        <span class="caps">"Tracker dispatch"</span>
                        <span class="dp-ver num">{format!("v{}", release_number(APP_VERSION))}</span>
                    </div>
                    <div class="dp-log scroll">
                        {CHANGELOG
                            .iter()
                            .map(|c| {
                                view! {
                                    <section class="dp-entry">
                                        <header>
                                            <b class="caps">"Update log"</b>
                                            <span class="num">{format!("v{} · {}", release_number(c.version), c.date)}</span>
                                        </header>
                                        <ul>{c.items.iter().map(|i| view! { <li>{*i}</li> }).collect_view()}</ul>
                                    </section>
                                }
                            })
                            .collect_view()}
                    </div>
                </aside>
            }
        })
    }
}
