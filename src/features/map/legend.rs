use crate::ui::emblems::EmblemArt;
use crate::ui::icons::{self, Icon};
use leptos::html;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

struct Entry {
    mark: fn() -> AnyView,
    name: &'static str,
    text: &'static str,
}

/// A small picture of each mark, drawn the way the map draws it, on a 24-unit grid centred on (0, 0).
fn swatch(children: AnyView) -> AnyView {
    view! {
        <svg class="ml-swatch" viewBox="-12 -12 24 24" width="26" height="26" aria-hidden="true">
            {children}
        </svg>
    }
    .into_any()
}

const ENTRIES: &[Entry] = &[
    Entry {
        mark: || {
            swatch(
                view! {
                    <circle cx="-5" r="2.4" fill="var(--f-humans)"></circle>
                    <circle cx="1" cy="-4" r="2" fill="var(--f-terminids)"></circle>
                    <circle cx="1" cy="4" r="2" fill="var(--f-automaton)"></circle>
                    <circle cx="7" r="2" fill="var(--f-illuminate)"></circle>
                }
                .into_any(),
            )
        },
        name: "Planets",
        text: "Blue for Super Earth, otherwise the colour of the faction holding it.",
    },
    Entry {
        mark: || {
            swatch(
                view! {
                    <circle r="7" fill="none" stroke="var(--f-automaton)" stroke-width="1.4"></circle>
                    <circle r="2.6" fill="var(--f-automaton)"></circle>
                }
                .into_any(),
            )
        },
        name: "Active front",
        text: "A pulsing ring: Helldivers are fighting for the planet now.",
    },
    Entry {
        mark: || {
            swatch(
                view! {
                    <circle r="6" fill="none" stroke="var(--cta)" stroke-width="1.4" stroke-dasharray="2.4 2"></circle>
                    <circle r="2" fill="var(--f-terminids)"></circle>
                }
                .into_any(),
            )
        },
        name: "Major Order target",
        text: "A dashed yellow ring round an objective of the current order.",
    },
    Entry {
        mark: || {
            swatch(
                view! {
                    <line
                        x1="-9"
                        y1="6"
                        x2="9"
                        y2="-6"
                        stroke="var(--f-illuminate)"
                        stroke-width="1.5"
                        stroke-dasharray="3 3"
                    ></line>
                }
                .into_any(),
            )
        },
        name: "Attack line",
        text: "From an enemy world to the planet it is attacking.",
    },
    Entry {
        mark: || {
            swatch(
                view! {
                    <path d="M-9 5 -2 -3 4 2 9 -6" fill="none" stroke="rgba(210, 228, 255, 0.5)" stroke-width="1"></path>
                    <circle cx="-2" cy="-3" r="1.4" fill="var(--f-humans)"></circle>
                    <circle cx="4" cy="2" r="1.4" fill="var(--f-humans)"></circle>
                }
                .into_any(),
            )
        },
        name: "Supply lane",
        text: "Planets can only be attacked or liberated along these lines.",
    },
    Entry {
        mark: || {
            swatch(
                view! {
                    <path
                        d="M-9 -7 7 -9 9 6 -6 8Z"
                        fill="color-mix(in srgb, var(--f-automaton) 14%, transparent)"
                        stroke="var(--f-automaton)"
                        stroke-width="1.2"
                    ></path>
                }
                .into_any(),
            )
        },
        name: "Enemy territory",
        text: "Regions outline each enemy's ground; Sectors hatch the sectors it holds.",
    },
    Entry {
        mark: || {
            swatch(
                view! {
                    <circle r="9" fill="var(--f-terminids)" opacity="0.25"></circle>
                    <circle cx="-3" cy="2" r="5" fill="var(--f-terminids)" opacity="0.3"></circle>
                }
                .into_any(),
            )
        },
        name: "The Gloom",
        text: "An amber cloud over the planets in or at the edge of the Gloom.",
    },
    Entry {
        mark: || {
            swatch(
                view! {
                    <rect x="-8" y="-8" width="16" height="16" fill="var(--surface-0)" stroke="var(--f-automaton)"></rect>
                    <EmblemArt art="incineration-corps" size=12.0 color="var(--f-automaton)" />
                }
                .into_any(),
            )
        },
        name: "Sub-faction",
        text: "An enemy unit on the planet, such as the Incineration Corps, in its emblem. Zoom in to see it.",
    },
    Entry {
        mark: || {
            swatch(
                view! {
                    <ellipse cy="6" rx="7" ry="2.4" fill="none" stroke="var(--f-humans)"></ellipse>
                    <path d="M-4 5 -2 -7 2 -7 4 5Z" fill="var(--f-humans)" opacity="0.35"></path>
                }
                .into_any(),
            )
        },
        name: "Holograms",
        text: "SEAF forces, enemy units and the Space Station stand beside a planet once zoomed in.",
    },
];

/// Closes a panel beside the map's rail on Escape or a press anywhere outside it; the rail's own button
/// toggles the panel, so a press there is left to its click.
pub fn close_outside(panel: NodeRef<html::Div>, on_close: Callback<()>) {
    let down = window_event_listener(leptos::ev::pointerdown, move |e| {
        let Some(t) = e.target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) else { return };
        let inside = panel.get_untracked().is_some_and(|p| p.contains(Some(&t)));
        if !inside && t.closest(".map-rail").ok().flatten().is_none() {
            on_close.run(());
        }
    });
    let key = window_event_listener(leptos::ev::keydown, move |e| {
        if e.key() == "Escape" {
            on_close.run(());
        }
    });
    on_cleanup(move || {
        down.remove();
        key.remove();
    });
}

/// What the marks on the map mean; opens beside the rail like the map settings.
#[component]
pub fn MapLegend(on_close: Callback<()>) -> impl IntoView {
    let panel = NodeRef::<html::Div>::new();
    close_outside(panel, on_close);
    view! {
        <div node_ref=panel class="map-settings map-legend panel glass" role="dialog" aria-label="Map legend">
            <div class="ms-head caps">
                <Icon icon=icons::BOOK_OPEN size=14.0 />
                " Legend"
                <button class="ml-close" on:click=move |_| on_close.run(()) aria-label="Close legend">
                    <Icon icon=icons::X size=14.0 />
                </button>
            </div>
            <dl class="ml-list">
                {ENTRIES
                    .iter()
                    .map(|e| {
                        view! {
                            <div>
                                <dt>{(e.mark)()}</dt>
                                <dd>
                                    <b class="caps">{e.name}</b>
                                    <span>{e.text}</span>
                                </dd>
                            </div>
                        }
                    })
                    .collect_view()}
            </dl>
        </div>
    }
}
