use crate::state::settings;
use crate::ui::icons::{self, Icon};
use leptos::prelude::*;

/// One-time hint for a screen; dismissed hints stay hidden (Settings › Reset unread shows them again), and
/// Settings › Hints hides them all.
#[component]
pub fn Hint(#[prop(into)] id: String, children: ChildrenFn) -> impl IntoView {
    let key = id.clone();
    let seen = Memo::new(move |_| settings().get(|s| !s.show_hints || s.hints_seen.contains(&key)));
    move || {
        let id = id.clone();
        (!seen.get()).then(|| {
            view! {
                <div class="hint" role="note">
                    <Icon icon=icons::LIGHTBULB size=15.0 />
                    <div class="hint-text">{children()}</div>
                    <button class="toast-x" aria-label="Dismiss hint" on:click=move |_| settings().dismiss_hint(&id)>
                        <Icon icon=icons::X size=15.0 />
                    </button>
                </div>
            }
        })
    }
}
