//! The icons legend: faction colours and every mark and badge the map, cards and lists use.

use crate::ui::emblems::{DssIcon, FactionEmblem};
use crate::ui::icons::{self, Icon};
use crate::ui::primitives::Panel;
use leptos::prelude::*;
use warroom_core::api::Faction;
use warroom_core::factions::{faction_color, faction_label};

struct Mark {
    icon: fn() -> AnyView,
    name: &'static str,
    text: &'static str,
}

const MARKS: &[Mark] = &[
    Mark {
        icon: || {
            view! {
                <svg width="28" height="28" viewBox="-14 -14 28 28" aria-hidden="true">
                    <circle r="9" fill="none" stroke="var(--f-automaton)" stroke-width="1.5"></circle>
                    <circle r="3" fill="var(--f-automaton)" stroke="#fff" stroke-width="0.8"></circle>
                </svg>
            }
            .into_any()
        },
        name: "Active planet",
        text: "A pulsing ring on the map: Helldivers are fighting for this planet right now.",
    },
    Mark {
        icon: || {
            view! {
                <svg width="28" height="28" viewBox="-14 -14 28 28" aria-hidden="true">
                    <circle r="8" fill="none" stroke="var(--cta)" stroke-width="1.5" stroke-dasharray="3 2"></circle>
                    <circle r="3" fill="var(--f-humans)" stroke="#fff" stroke-width="0.8"></circle>
                </svg>
            }
            .into_any()
        },
        name: "Major Order target",
        text: "A dashed yellow ring on the map, and a target badge on cards and planet rows.",
    },
    Mark {
        icon: || {
            view! {
                <svg width="28" height="28" viewBox="0 0 28 28" aria-hidden="true">
                    <line x1="2" y1="22" x2="26" y2="6" stroke="var(--f-terminids)" stroke-width="1.5" stroke-dasharray="4 3"></line>
                </svg>
            }
            .into_any()
        },
        name: "Attack line",
        text: "A dashed line from an enemy world to the planet it is attacking.",
    },
    Mark {
        icon: || {
            view! {
                <svg width="28" height="28" viewBox="0 0 28 28" aria-hidden="true">
                    <path d="M3 20 C 6 6, 18 4, 25 10 S 20 26, 3 20Z" fill="none" stroke="var(--f-illuminate)" stroke-width="2"></path>
                </svg>
            }
            .into_any()
        },
        name: "Front contour",
        text: "Regions view: the outline of one enemy's contiguous territory. Super Earth space is left open.",
    },
    Mark {
        icon: || view! { <Icon icon=icons::LANDMARK size=20.0 color="var(--cta)" /> }.into_any(),
        name: "Point of interest",
        text: "A pawn beside a planet once zoomed in: a city, a TCS+ tower network, a base or another site the game marks.",
    },
    Mark {
        icon: || view! { <Icon icon=icons::SHIELD size=20.0 color="var(--f-humans)" /> }.into_any(),
        name: "SEAF and sub-faction pawns",
        text: "Heavy SEAF presence in blue; an enemy unit such as the Incineration Corps in that enemy's colour.",
    },
    Mark {
        icon: || view! { <Icon icon=icons::CLOUD_FOG size=20.0 color="var(--f-terminids)" /> }.into_any(),
        name: "The Gloom",
        text: "An amber cloud over planets in or on the edge of the Gloom, in every overlay.",
    },
    Mark {
        icon: || {
            view! {
                <svg width="28" height="28" viewBox="-14 -14 28 28" aria-hidden="true">
                    <circle r="7" fill="#c8481c"></circle>
                    <path d="M-5,-2.5L-1,-0.6L1.4,-4.2M-1,-0.6L0.6,3.6L4.6,2.2" fill="none" stroke="#2a0904" stroke-width="1"></path>
                </svg>
            }
            .into_any()
        },
        name: "Scorched and fractured worlds",
        text: "Planets razed in the war are drawn burnt; those torn apart by the Meridian singularity, as debris.",
    },
    Mark {
        icon: || view! { <Icon icon=icons::TARGET size=20.0 color="var(--cta)" /> }.into_any(),
        name: "Major Order badge",
        text: "The planet is an objective of the current Major Order.",
    },
    Mark {
        icon: || view! { <DssIcon size=20.0 color="var(--f-humans)" /> }.into_any(),
        name: "Democracy Space Station",
        text: "The DSS is in orbit; see Hellpad › Space Stations for its tactical actions.",
    },
    Mark {
        icon: || view! { <Icon icon=icons::SHIELD size=20.0 color="var(--text-hi)" /> }.into_any(),
        name: "Defense",
        text: "The planet belongs to Super Earth and is under attack until the event timer runs out.",
    },
    Mark {
        icon: || view! { <Icon icon=icons::RADIO size=20.0 color="var(--info)" /> }.into_any(),
        name: "Dispatch",
        text: "A message from Super Earth. Planet names inside it are links.",
    },
    Mark {
        icon: || view! { <Icon icon=icons::DATABASE size=20.0 color="var(--text-dim)" /> }.into_any(),
        name: "Stale data",
        text: "No new data from the collector for ten minutes; numbers are the last known values.",
    },
];

#[component]
pub fn Legend() -> impl IntoView {
    let factions = [Faction::Humans, Faction::Terminids, Faction::Automaton, Faction::Illuminate];
    view! {
        <div class="legend-page">
            <Panel title="Factions" icon=icons::BOOK_OPEN>
                <ul class="legend-list factions">
                    {factions
                        .map(|f| view! {
                            <li>
                                <FactionEmblem faction=f.as_str() size=26.0 />
                                <b class="caps" style=format!("color: {};", faction_color(Some(f)))>{faction_label(Some(f))}</b>
                            </li>
                        })
                        .collect_view()}
                </ul>
            </Panel>
            <Panel title="Marks and badges" icon=icons::BOOK_OPEN>
                <ul class="legend-list">
                    {MARKS
                        .iter()
                        .map(|m| view! {
                            <li>
                                <span class="legend-icon">{(m.icon)()}</span>
                                <div>
                                    <b class="caps">{m.name}</b>
                                    <p>{m.text}</p>
                                </div>
                            </li>
                        })
                        .collect_view()}
                </ul>
            </Panel>
        </div>
    }
}
