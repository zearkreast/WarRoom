use crate::ui::holo_globe::HoloGlobe;
use leptos::prelude::*;

/// How long the hold lasts before what it stands in front of opens.
pub const PLEASE_WAIT_MS: f64 = 1600.0;

/// The game's "Please wait democratically" hold before something opens.
#[component]
pub fn PleaseWait() -> impl IntoView {
    view! {
        <div class="bp-wait" role="status">
            <HoloGlobe size=84.0 />
            <span class="caps">"Please wait democratically"</span>
        </div>
    }
}
