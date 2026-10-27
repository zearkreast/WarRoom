use crate::js::num;
use leptos::prelude::*;

/// Idle hologram for a ring's centre: a wireframe globe with turning meridians.
#[component]
pub fn HoloGlobe(#[prop(default = 120.0)] size: f64) -> impl IntoView {
    view! {
        <svg class="holo-globe" width=num(size) height=num(size) viewBox="-60 -60 120 120" aria-hidden="true">
            <defs>
                <radialGradient id="holo-fill" cx="40%" cy="35%" r="70%">
                    <stop offset="0%" stop-color="#9fd4ff" stop-opacity="0.35" />
                    <stop offset="70%" stop-color="#46a4ff" stop-opacity="0.12" />
                    <stop offset="100%" stop-color="#46a4ff" stop-opacity="0.02" />
                </radialGradient>
            </defs>
            <circle r="48" fill="url(#holo-fill)" class="holo-rim" />
            {[-32.0f64, -16.0, 0.0, 16.0, 32.0]
                .into_iter()
                .map(|y| {
                    view! { <ellipse cy=num(y) rx=num((48.0f64 * 48.0 - y * y).sqrt()) ry="4" class="holo-line" /> }
                })
                .collect_view()}
            {(0..6)
                .map(|i| {
                    view! {
                        <ellipse rx="48" ry="48" class="holo-meridian" style=format!("animation-delay: {}s;", -i) />
                    }
                })
                .collect_view()}
            <ellipse rx="58" ry="9" class="holo-orbit" />
        </svg>
    }
}
