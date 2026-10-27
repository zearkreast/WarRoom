//! The map's button rail and its settings popover.

use crate::features::map::legend::close_outside;
use crate::features::map::options::{overlay_options, planet_options, quality_options};
use crate::state::{View, settings, ui};
use crate::ui::icons::{self, Icon, IconData};
use crate::ui::primitives::Stepper;
use leptos::html;
use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Side {
    Settings,
    Legend,
}

#[component]
pub(super) fn MapSettings(on_close: Callback<()>) -> impl IntoView {
    let panel = NodeRef::<html::Div>::new();
    close_outside(panel, on_close);
    let s = settings();
    view! {
        <div node_ref=panel class="map-settings panel glass" role="dialog" aria-label="Map settings">
            <div class="ms-head caps">
                <Icon icon=icons::SETTINGS size=14.0 />
                " Map settings"
            </div>
            <div class="ms-row">
                <span class="caps">"Overlay"</span>
                <Stepper
                    label="Map overlay"
                    options=overlay_options()
                    value=Signal::derive(move || s.get(|s| s.map_overlay))
                    on_change=move |v| s.set(|s| s.map_overlay = v)
                />
            </div>
            <div class="ms-row">
                <span class="caps">"Planets"</span>
                <Stepper
                    label="Planet style"
                    options=planet_options()
                    value=Signal::derive(move || s.get(|s| s.map_planets))
                    on_change=move |v| s.set(|s| s.map_planets = v)
                />
            </div>
            <div class="ms-row">
                <span class="caps">"Quality"</span>
                <Stepper
                    label="Render quality"
                    options=quality_options()
                    value=Signal::derive(move || s.get(|s| s.map_quality))
                    on_change=move |v| s.set(|s| s.map_quality = v)
                />
            </div>
            <button class="link ms-more caps" on:click=move |_| ui().set_view(View::Settings)>
                "All settings "
                <Icon icon=icons::ARROW_UP_RIGHT size=12.0 />
            </button>
        </div>
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn rail_button(
    icon: IconData,
    label: &'static str,
    title: Option<&'static str>,
    key_hint: Option<&'static str>,
    active: Option<Signal<bool>>,
    class: &'static str,
    icon_style: Option<String>,
    node_ref: NodeRef<html::Button>,
    on_click: impl Fn() + 'static,
) -> impl IntoView {
    let title = title.unwrap_or(label);
    let full = match key_hint {
        Some(k) => format!("{title} ({k})"),
        None => title.to_string(),
    };
    view! {
        <button
            node_ref=node_ref
            class=move || format!("rail-btn {} {class}", if active.is_some_and(|a| a.get()) { "on" } else { "" })
            on:click=move |_| on_click()
            aria-label=title
            aria-pressed=move || active.map(|a| a.get().to_string())
            title=full
        >
            <Icon icon=icon size=18.0 stroke_width=1.75 style=icon_style />
            <span class="caps">{label}</span>
        </button>
    }
}
