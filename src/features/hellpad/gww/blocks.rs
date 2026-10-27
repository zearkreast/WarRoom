//! What every G.W.W. page is built from: section text, the banner and breadcrumbs, contents, pagers and related links.

use crate::features::hellpad::gww::figures::figure;
use crate::features::hellpad::gww::nav::{Rich, icon_url, use_gww_nav};
use crate::features::hellpad::gww::pictures::{ad_box, kind_label, split, target_icon};
use crate::js;
use crate::ui::icons::{self, Icon};
use leptos::html;
use leptos::prelude::*;
use warroom_core::data::{Ad, Section, Tie};
use warroom_core::gww_nav::title_of;

pub type HeadingRef = NodeRef<html::H2>;

/// The in-game Training Manual Tip box, for the game's own words.
pub fn training_tip(text: &str) -> AnyView {
    view! {
        <aside class="gww-quote">
            <Icon icon=icons::QUOTE size=16.0 />
            <p>
                <b class="caps">"Training manual tip"</b>
                <span>{text.to_string()}</span>
            </p>
        </aside>
    }
    .into_any()
}

pub(super) fn broadcast(text: &str) -> AnyView {
    view! {
        <aside class="gww-quote broadcast">
            <Icon icon=icons::RADIO size=16.0 />
            <p>
                <b class="caps">"Ship broadcast"</b>
                <span>"“"{text.to_string()}"”"</span>
            </p>
        </aside>
    }
    .into_any()
}

pub(super) fn sourced_quote(from: &'static str, text: &'static str) -> AnyView {
    view! {
        <aside class="gww-quote">
            <Icon icon=icons::QUOTE size=16.0 />
            <p>
                <b class="caps">{from}</b>
                <span>"“"{text}"”"</span>
            </p>
        </aside>
    }
    .into_any()
}

pub(super) fn field_tip(text: &'static str) -> AnyView {
    view! {
        <aside class="gww-tip">
            <Icon icon=icons::LIGHTBULB size=16.0 />
            <p>
                <b class="caps">"Field tip"</b>
                " "
                <Rich text=text />
            </p>
        </aside>
    }
    .into_any()
}

/// The term ends at the first ": ", so a link label before it mustn't contain one.
fn point(text: &'static str) -> AnyView {
    match text.find(": ") {
        Some(at) if at <= 80 => view! {
            <b>
                <Rich text=&text[..at] />
            </b>
            ": "
            <Rich text=&text[at + 2..] />
        }
        .into_any(),
        _ => view! { <Rich text=text /> }.into_any(),
    }
}

pub(super) fn section_block(sec: &'static Section, id: String) -> AnyView {
    view! {
        <section id=id class="gww-section">
            <h3>{sec.heading}</h3>
            {sec.text.map(|t| view! { <p><Rich text=t /></p> })}
            {sec.points.map(|p| view! { <ul class="gww-points">{p.iter().map(|p| view! { <li>{point(p)}</li> }).collect_view()}</ul> })}
            {sec.products.map(|p| view! {
                <ul class="gww-products" aria-label="Products">{p.iter().map(|p| view! { <li class="caps">{*p}</li> }).collect_view()}</ul>
            })}
            {sec.timeline.map(|t| view! {
                <ol class="gww-timeline">
                    {t.iter()
                        .map(|d| view! {
                            <li>
                                <span class="gww-timeline-when caps num">{d.when}</span>
                                <p><Rich text=d.text /></p>
                            </li>
                        })
                        .collect_view()}
                </ol>
            })}
            {sec.table.as_ref().map(|t| view! {
                <div class="gww-table-wrap">
                    <table class="gww-table num">
                        <thead>
                            <tr>{t.head.iter().map(|h| view! { <th scope="col">{*h}</th> }).collect_view()}</tr>
                        </thead>
                        <tbody>
                            {t.rows
                                .iter()
                                .map(|r| view! {
                                    <tr>
                                        {r.cells
                                            .iter()
                                            .enumerate()
                                            .map(|(i, c)| {
                                                if i == 0 {
                                                    view! {
                                                        <th scope="row">
                                                            {r.icon.and_then(icon_url).map(|u| view! { <img src=u alt="" /> })}
                                                            <Rich text=*c />
                                                        </th>
                                                    }
                                                    .into_any()
                                                } else {
                                                    view! { <td><Rich text=*c /></td> }.into_any()
                                                }
                                            })
                                            .collect_view()}
                                    </tr>
                                })
                                .collect_view()}
                        </tbody>
                    </table>
                    {t.note.map(|n| view! { <p class="gww-note">{n}</p> })}
                </div>
            })}
            {sec.kinds.map(|k| view! {
                <ul class="gww-kinds">
                    {k.iter()
                        .map(|k| view! {
                            <li>
                                <span class="gww-kind-icon">{icon_url(k.icon).map(|u| view! { <img src=u alt="" /> })}</span>
                                <div>
                                    <b class="caps">{k.name}</b>
                                    <p><Rich text=k.text /></p>
                                </div>
                            </li>
                        })
                        .collect_view()}
                </ul>
            })}
            {sec.faq.map(|f| view! {
                <div class="gww-faq">
                    {f.iter()
                        .map(|f| view! {
                            <details>
                                <summary>{f.q}</summary>
                                <p><Rich text=f.a /></p>
                            </details>
                        })
                        .collect_view()}
                </div>
            })}
            {sec.folds.map(|f| view! {
                <div class="gww-folds">
                    {f.iter()
                        .map(|f| view! {
                            <details class="gww-fold">
                                <summary>
                                    <b>{f.title}</b>
                                    <span class="caps">{f.note}</span>
                                </summary>
                                <ol class="gww-fold-list">{f.points.iter().map(|p| view! { <li>{point(p)}</li> }).collect_view()}</ol>
                            </details>
                        })
                        .collect_view()}
                </div>
            })}
            {sec.figure.map(figure)}
        </section>
    }
    .into_any()
}

pub struct Crumb {
    pub label: String,
    pub target: Option<&'static str>,
}

pub(super) fn crumb(label: impl Into<String>, target: Option<&'static str>) -> Crumb {
    Crumb { label: label.into(), target }
}

pub(super) fn banner(
    icon: Option<AnyView>,
    crumbs: Vec<Crumb>,
    title: String,
    lead: Option<AnyView>,
    heading: HeadingRef,
    corp: bool,
) -> AnyView {
    let nav = use_gww_nav();
    view! {
        <header class=format!("gww-banner {}", if corp { "corp" } else { "" })>
            {icon.map(|i| view! { <span class="gww-banner-icon">{i}</span> })}
            <div class="gww-banner-text">
                <nav class="gww-crumbs caps" aria-label="Where this page sits">
                    {crumbs
                        .into_iter()
                        .enumerate()
                        .map(|(i, c)| view! {
                            <span>
                                {(i > 0).then(|| view! { <span aria-hidden="true">" › "</span> })}
                                {match c.target {
                                    Some(t) => view! { <button class="gww-crumb" on:click=move |e| nav.follow(t, Some(&e))>{c.label}</button> }.into_any(),
                                    None => c.label.into_any(),
                                }}
                            </span>
                        })
                        .collect_view()}
                </nav>
                <h2 class="gww-title" node_ref=heading tabindex="-1">{title}</h2>
                {lead.map(|l| view! { <p class="gww-lead">{l}</p> })}
            </div>
        </header>
    }
    .into_any()
}

fn scroll_to(id: &str) {
    if let Some(el) = js::document().get_element_by_id(id) {
        let opts = web_sys::ScrollIntoViewOptions::new();
        opts.set_behavior(if js::prefers_reduced_motion() {
            web_sys::ScrollBehavior::Auto
        } else {
            web_sys::ScrollBehavior::Smooth
        });
        opts.set_block(web_sys::ScrollLogicalPosition::Start);
        el.scroll_into_view_with_scroll_into_view_options(&opts);
    }
}

pub(super) fn toc(sections: &'static [Section], prefix: &str) -> AnyView {
    if sections.len() < 3 {
        return ().into_any();
    }
    view! {
        <nav class="gww-toc" aria-label="On this page">
            <span class="caps">"On this page"</span>
            {sections
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    let id = format!("{prefix}-{i}");
                    view! { <button class="gww-toc-link" on:click=move |_| scroll_to(&id)>{s.heading}</button> }
                })
                .collect_view()}
        </nav>
    }
    .into_any()
}

pub fn related(targets: Vec<&'static str>, title: &'static str) -> AnyView {
    let nav = use_gww_nav();
    if targets.is_empty() {
        return ().into_any();
    }
    view! {
        <nav class="gww-related" aria-label=title>
            <h3 class="caps">{title}</h3>
            <div class="gww-related-list">
                {targets
                    .into_iter()
                    .map(|t| view! {
                        <button class="gww-related-link" on:click=move |e| nav.follow(t, Some(&e))>
                            <span class="gww-related-icon">{target_icon(t, 20.0)}</span>
                            <span>
                                <span class="caps">{kind_label(split(t).0)}</span>
                                <b>{title_of(t)}</b>
                            </span>
                            <Icon icon=if t.starts_with("app:") { icons::ARROW_UP_RIGHT } else { icons::ARROW_RIGHT } size=14.0 />
                        </button>
                    })
                    .collect_view()}
            </div>
        </nav>
    }
    .into_any()
}

pub(super) fn ties(ties: &'static [Tie], title: &'static str) -> AnyView {
    let nav = use_gww_nav();
    view! {
        <section class="gww-section">
            <h3>{title}</h3>
            <div class="gww-ties">
                {ties
                    .iter()
                    .map(|t| view! {
                        <button class="gww-tie" on:click=move |e| nav.follow(t.to, Some(&e))>
                            <span class="gww-related-icon">{target_icon(t.to, 28.0)}</span>
                            <span>
                                <b>{title_of(t.to)}</b>
                                <span>{t.how}</span>
                            </span>
                        </button>
                    })
                    .collect_view()}
            </div>
        </section>
    }
    .into_any()
}

pub(super) fn pager(list: Vec<String>, at: usize, noun: &'static str) -> AnyView {
    let nav = use_gww_nav();
    let prev = at.checked_sub(1).and_then(|i| list.get(i).cloned());
    let next = list.get(at + 1).cloned();
    view! {
        <nav class="gww-pager" aria-label=format!("Other {noun}s")>
            {match prev {
                Some(p) => {
                    let title = title_of(&p);
                    view! {
                        <button class="gww-pager-link" on:click=move |e| nav.follow(&p, Some(&e))>
                            <Icon icon=icons::ARROW_LEFT size=14.0 />
                            <span>
                                <span class="caps">"Previous "{noun}</span>
                                " "{title}
                            </span>
                        </button>
                    }
                    .into_any()
                }
                None => view! { <span></span> }.into_any(),
            }}
            {next.map(|n| {
                let title = title_of(&n);
                view! {
                    <button class="gww-pager-link next" on:click=move |e| nav.follow(&n, Some(&e))>
                        <span>
                            <span class="caps">"Next "{noun}</span>
                            " "{title}
                        </span>
                        <Icon icon=icons::ARROW_RIGHT size=14.0 />
                    </button>
                }
            })}
        </nav>
    }
    .into_any()
}

pub(super) fn ads_inline(ads: Signal<Vec<&'static Ad>>, here: String) -> AnyView {
    (move || {
        let list = ads.get();
        (!list.is_empty()).then(|| {
            view! { <div class="gww-ads-inline">{list.into_iter().map(|a| ad_box(a, Some(&here), false)).collect_view()}</div> }
        })
    })
    .into_any()
}
