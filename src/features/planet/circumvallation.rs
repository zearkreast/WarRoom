use crate::api::queries::queries;
use crate::js::num;
use leptos::prelude::*;
use std::collections::HashSet;
use std::sync::Arc;
use warroom_core::api::Planet;
use warroom_core::factions::faction_color;
use warroom_core::sieges::{Circumvallation, circumvallation, neighbours};

/// The siege a front would open, if liberating it would leave enemy planets surrounded.
pub fn use_circumvallation(p: Signal<Planet>) -> Memo<Option<Circumvallation>> {
    let q = queries();
    let links = Memo::new(move |_| q.fresh_planets.with(|ps| ps.as_ref().map(|ps| Arc::new(neighbours(ps)))));
    Memo::new(move |_| {
        let planets = q.fresh_planets.get()?;
        let fronts: HashSet<i64> =
            q.campaigns.with(|c| c.as_ref().map(|c| c.iter().map(|c| c.planet.index).collect()))?;
        let links = links.get()?;
        let index = p.with(|p| p.index);
        if !fronts.contains(&index) {
            return None;
        }
        let front = planets.iter().find(|x| x.index == index).cloned().unwrap_or_else(|| p.get());
        circumvallation(&front, &planets, &links, &fronts)
    })
}

/// A siege marker drawn for the app: arrows closing a ring on one enemy world, or on two.
#[component]
pub fn SiegeMark(#[prop(optional)] double: bool, #[prop(default = 30.0)] size: f64) -> impl IntoView {
    view! {
        <svg class="siege-mark" width=num(size) height=num(size) viewBox="-12 -12 24 24" aria-hidden="true">
            <circle r="7.4" class="siege-ring" />
            {[0, 90, 180, 270]
                .into_iter()
                .map(|a| {
                    view! {
                        <path d="M-2.6 -11.3L0 -8.7L2.6 -11.3" transform=format!("rotate({a})") class="siege-arrow" />
                    }
                })
                .collect_view()}
            {if double {
                view! {
                    <circle cx="-2.7" r="2.2" class="siege-core" />
                    <circle cx="2.7" r="2.2" class="siege-core" />
                }
                    .into_any()
            } else {
                view! { <circle r="3.1" class="siege-core" /> }.into_any()
            }}
        </svg>
    }
}

fn name(p: &Planet) -> AnyView {
    view! { <b style=format!("color: {};", faction_color(Some(p.current_owner)))>{p.name.clone()}</b> }.into_any()
}

fn names(list: &[Planet]) -> AnyView {
    list.iter()
        .enumerate()
        .map(|(i, p)| {
            view! {
                {(i > 0).then(|| if i == list.len() - 1 { " and " } else { ", " })}
                {name(p)}
            }
        })
        .collect_view()
        .into_any()
}

/// Blue when the ring is worth closing, bright for two sieges, grey when liberating the targets directly is quicker.
pub fn circumvallation_note(front: &Planet, c: &Circumvallation, class: &str) -> AnyView {
    let double = c.targets.len() > 1;
    let title = if double { "Double circumvallation" } else { "Circumvallation" };
    view! {
        <section
            class=format!(
                "siege {} {} {class}",
                if double { "double" } else { "" },
                if c.worth { "" } else { "direct" },
            )
            aria-label=title
        >
            <SiegeMark double=double />
            <div class="siege-body">
                <span class="siege-kind caps">"Siege objective"</span>
                <b class="siege-title caps">{title}</b>
                {(!c.worth).then(|| view! {
                    <p class="siege-notice">
                        <b class="caps">"Notice:"</b>
                        " liberating "
                        {names(&c.direct)}
                        " directly is quicker than closing the ring."
                    </p>
                })}
                <p>
                    "Liberating "
                    {name(front)}
                    " leaves "
                    {double.then_some("both ")}
                    {names(&c.targets)}
                    " with only Super Earth planets around "
                    {if double { "them" } else { "it" }}
                    ". Held, the ring lets the SEAF lay "
                    {if double { "two sieges" } else { "siege" }}
                    ", turning "
                    {if double { "their" } else { "its" }}
                    " resistance into slow liberation."
                </p>
            </div>
        </section>
    }
    .into_any()
}
