//! Marks for enemy resistance and the SEAF's sieges: a helmet for resistance, and the game's Heavy SEAF Presence
//! emblem with its ring closed over the top for Siege liberation.

use crate::js::num;
use crate::ui::emblems::EmblemArt;
use leptos::prelude::*;

/// Enemy resistance: a helmet, solid in the current colour.
#[component]
pub fn ResistanceMark(#[prop(default = 14.0)] size: f64) -> impl IntoView {
    view! {
        <svg class="seaf-art" width=num(size) height=num(size) viewBox="-50 -50 100 100" aria-hidden="true">
            <path d="M-36 12C-36-21-18-34 0-34C18-34 36-21 36 12Z" />
            <path d="M-45 16H45L40 25H-40Z" />
        </svg>
    }
}

/// Siege liberation: the Heavy SEAF Presence emblem, its outer ring closed over the trooper's head by two arcs
/// that close in and ease off. The arcs sit on the emblem's own outer ring, centre (0, -0.562), radii 8.763 to
/// 10.27 on its 24-unit grid.
#[component]
pub fn SiegeLiberationMark(#[prop(default = 16.0)] size: f64) -> impl IntoView {
    view! {
        <svg class="emblem seaf-art seaf-siege" width=num(size) height=num(size) viewBox="-12 -12 24 24" aria-hidden="true">
            <EmblemArt art="seaf" size=24.0 />
            <path
                class="seaf-close left"
                d="M-10.213 -1.636A10.27 10.27 0 0 1 -1.429 -10.732L-1.219 -9.240A8.763 8.763 0 0 0 -8.715 -1.478Z"
            />
            <path
                class="seaf-close right"
                d="M1.430 -10.732A10.27 10.27 0 0 1 10.214 -1.636L8.715 -1.478A8.763 8.763 0 0 0 1.220 -9.240Z"
            />
        </svg>
    }
}
