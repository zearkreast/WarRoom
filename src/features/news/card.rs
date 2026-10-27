//! Game text with its markup styled and planet names as links.

use crate::api::queries::queries;
use crate::state::{OptionsAnchor, ui};
use leptos::prelude::*;
use std::collections::HashMap;
use warroom_core::dispatch::{DispatchKind, dispatch_body};
use warroom_core::format::{js_trim, parse_markup};

pub fn kind_label(k: DispatchKind) -> &'static str {
    match k {
        DispatchKind::Order => "Major order",
        DispatchKind::Victory => "Victory",
        DispatchKind::Defeat => "Defeat",
        DispatchKind::Alert => "Alert",
        DispatchKind::Advisory => "Advisory",
        DispatchKind::News => "Dispatch",
    }
}

/// Game text with its markup styled and planet names turned into links to their options. Dispatches
/// lose their leading headline (cards show it as the title); `whole` keeps it, for briefings.
#[component]
pub fn MarkupText(
    #[prop(into)] message: String,
    #[prop(default = "ni-body".into(), into)] class: String,
    #[prop(optional)] whole: bool,
) -> impl IntoView {
    let planets = queries().fresh_planets;
    let spans = if whole { parse_markup(js_trim(&message)) } else { dispatch_body(&message) };
    let by_name = Memo::new(move |_| {
        planets.with(|p| {
            p.as_ref().map_or_else(HashMap::new, |list| {
                list.iter().map(|p| (p.name.to_uppercase(), p.index)).collect::<HashMap<_, _>>()
            })
        })
    });
    view! {
        <p class=class>
            {move || {
                let names = by_name.get();
                spans
                    .iter()
                    .map(|s| {
                        let text = s.text.clone();
                        match s.style {
                            None => view! { <span>{text}</span> }.into_any(),
                            Some(style) => match names.get(&js_trim(&s.text).to_uppercase()).copied() {
                                Some(planet) => {
                                    view! {
                                        <button
                                            class="link hl hl-planet"
                                            on:click=move |e| {
                                                ui().open_options(Some(OptionsAnchor::at(planet, &e)))
                                            }
                                        >
                                            {text}
                                        </button>
                                    }
                                        .into_any()
                                }
                                None => view! { <span class=format!("hl hl-{style}")>{text}</span> }.into_any(),
                            },
                        }
                    })
                    .collect_view()
            }}
        </p>
    }
}
