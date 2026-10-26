use crate::features::hellpad::gww::figures::figure;
use crate::features::hellpad::gww::live::live_panel;
use crate::features::hellpad::gww::nav::{Rich, ad_url, icon_url, logo_url, use_gww_nav};
use crate::history::history;
use crate::js::{self, fmt_date, fmt_date_time, num};
use crate::state::{OptionsAnchor, settings, ui};
use crate::ui::emblems::{FactionEmblem, IconLike, icon_view};
use crate::ui::icons::{self, Icon, IconData};
use crate::ui::primitives::{Empty, Loading, OutLink};
use leptos::html;
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use warroom_core::api::Faction;
use warroom_core::data::manuals::{CHAPTERS, MANUALS, WIKI, WIKI_LICENCE};
use warroom_core::data::sites::{ADS, CITIZENS, COMPANIES, GOV, MINISTRIES};
use warroom_core::data::{Ad, AdBy, AdShape, Company, Manual, Ministry, Section, Source, Tie};
use warroom_core::factions::{faction_color, faction_label};
use warroom_core::gww_nav::{DIRS, title_of};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

pub type HeadingRef = NodeRef<html::H2>;

/* ---------- Pictures ---------- */

/// Drawn icons for manuals the game has no picture for.
fn manual_icon_of(id: &str) -> IconLike {
    match id {
        "basics" => icons::EARTH.into(),
        "resistance" => icons::ACTIVITY.into(),
        "regions" => icons::BUILDING_2.into(),
        "supply-lines" => icons::NETWORK.into(),
        "dss" => IconLike::Dss,
        "super-destroyers" => icons::ROCKET.into(),
        "campaigns" => icons::FLAG.into(),
        "gloom" => icons::CLOUD_FOG.into(),
        "misconceptions" => icons::CIRCLE_HELP.into(),
        _ => icons::LIGHTBULB.into(),
    }
}

fn app_icon(id: &str) -> Option<IconLike> {
    Some(match id {
        "news" => icons::NEWSPAPER.into(),
        "overview" => icons::SHIELD.into(),
        "map" => icons::ORBIT.into(),
        "campaigns" => icons::LAYOUT_GRID.into(),
        "stations" => IconLike::Dss,
        "past" => icons::ARCHIVE.into(),
        "planets" => icons::GLOBE_2.into(),
        _ => return None,
    })
}

fn dir_icon(id: &str) -> Option<IconData> {
    Some(match id {
        "manuals" => icons::GRADUATION_CAP,
        "gov" => icons::LANDMARK,
        "corps" => icons::BUILDING_2,
        _ => return None,
    })
}

/// Every manual's icon as a ring around a glyph, as the game draws its campaign icons (which bring their own ring).
pub fn manual_icon(m: &Manual, size: f64) -> AnyView {
    let url = m.icon.and_then(icon_url);
    let ringed = url.is_some() && m.icon.is_some_and(|i| i.starts_with("campaign-"));
    view! {
        <span class=format!("gww-glyph {}", if ringed { "" } else { "ring" }) style=format!("font-size: {}px;", num(size)) aria-hidden="true">
            {match url {
                Some(u) => view! { <i style=format!("--glyph: url(\"{u}\");")></i> }.into_any(),
                None => icon_view(manual_icon_of(m.id), 24.0, 2.25, None),
            }}
        </span>
    }
    .into_any()
}

pub fn company_mark(c: &Company) -> AnyView {
    match c.logo.and_then(logo_url) {
        Some(u) => view! { <img class="gww-logo" src=u alt="" /> }.into_any(),
        None => view! { <span class="gww-wordmark caps" aria-hidden="true">{c.name}</span> }.into_any(),
    }
}

fn split(target: &str) -> (&str, &str) {
    match target.find(':') {
        Some(i) => (&target[..i], &target[i + 1..]),
        None => ("", target),
    }
}

/// A small picture for any link target.
pub fn target_icon(target: &str, size: f64) -> AnyView {
    let (kind, id) = split(target);
    let img = |src: Option<String>| {
        src.map(|s| view! { <img src=s alt="" width=num(size) height=num(size) /> }.into_any())
            .unwrap_or_else(|| ().into_any())
    };
    match kind {
        "manual" => MANUALS.iter().find(|m| m.id == id).map_or_else(|| ().into_any(), |m| manual_icon(m, size)),
        "ministry" => img(icon_url(MINISTRIES.iter().find(|m| m.id == id).map_or("", |m| m.icon))),
        "company" => match COMPANIES.iter().find(|c| c.id == id).and_then(|c| c.logo).and_then(logo_url) {
            Some(src) => img(Some(src)),
            None => view! { <Icon icon=icons::BUILDING_2 size=size stroke_width=1.5 /> }.into_any(),
        },
        "site" if id == "log" => view! { <Icon icon=icons::HISTORY size=size stroke_width=1.5 /> }.into_any(),
        "site" => img(icon_url(if id == "citizens" { CITIZENS.icon } else { GOV.icon })),
        "dir" => dir_icon(id)
            .map_or_else(|| ().into_any(), |i| view! { <Icon icon=i size=size stroke_width=1.5 /> }.into_any()),
        "app" => app_icon(id).map_or_else(|| ().into_any(), |i| icon_view(i, size, 1.5, None)),
        _ => ().into_any(),
    }
}

fn kind_label(kind: &str) -> &'static str {
    match kind {
        "manual" => "Training manual",
        "ministry" => "Ministry",
        "company" => "Corporation",
        "site" => "Super Earth",
        "dir" => "Directory",
        "app" => "In the tracker",
        _ => "",
    }
}

/// Wide enough for a rail of adverts beside the page; keep in sync with the 1100px breakpoint in gww.css.
pub const RAIL_QUERY: &str = "(min-width: 1101px)";

pub fn pick_one<T>(list: &[T]) -> Option<&T> {
    if list.is_empty() {
        return None;
    }
    list.get((js_sys::Math::random() * list.len() as f64).floor() as usize)
}

fn link_of(ad: &Ad) -> Option<&'static str> {
    match ad.by {
        AdBy::Site(s) => Some(s),
        AdBy::Brand(_) => None,
    }
}

/// A rail advert keeps its shape; a billboard fills the width at a fixed height. `here` is the link target of the
/// page it's shown on; an advert for that page doesn't link to it.
pub fn ad_box(ad: &'static Ad, here: Option<&str>, rail: bool) -> AnyView {
    let nav = use_gww_nav();
    let Some(url) = ad_url(ad.file) else { return ().into_any() };
    let art = format!("--ad: url(\"{url}\");");
    let to = link_of(ad).filter(|t| Some(*t) != here);
    let by = match ad.by {
        AdBy::Brand(b) => b.to_string(),
        AdBy::Site(s) => title_of(s),
    };
    view! {
        <figure class=format!("gww-ad {} {}", ad.shape.as_str(), if rail { "rail" } else { "billboard" })>
            {match to {
                None => view! {
                    <div class="gww-ad-open" style=art>
                        <img src=url alt=ad.alt loading="lazy" />
                    </div>
                }
                .into_any(),
                Some(to) => view! {
                    <button
                        class="gww-ad-open"
                        style=art
                        on:click=move |e| nav.follow(to, Some(&e))
                        aria-label=format!("{} Open {}.", ad.alt, title_of(to))
                    >
                        <img src=url alt="" loading="lazy" />
                    </button>
                }
                .into_any(),
            }}
            <figcaption class="caps">
                <span>"Sponsored"</span>
                <span class="gww-ad-by">{by}</span>
            </figcaption>
        </figure>
    }
    .into_any()
}

/// On wide screens a tall advert takes the rail, another advertiser's when the site has none of its own.
fn site_ads(
    files: Option<&'static [&'static str]>,
    here: String,
) -> (Signal<Option<&'static Ad>>, Signal<Vec<&'static Ad>>) {
    let wide = js::use_media(RAIL_QUERY);
    let own: Vec<&'static Ad> = ADS.iter().filter(|a| files.is_some_and(|f| f.contains(&a.file))).collect();
    let tall_others: Vec<&'static Ad> =
        ADS.iter().filter(|a| a.shape == AdShape::Tall && link_of(a) != Some(here.as_str())).collect();
    let filler = pick_one(&tall_others).copied();
    let own2 = own.clone();
    let rail = Signal::derive(move || {
        wide.get().then(|| own.iter().copied().find(|a| a.shape == AdShape::Tall).or(filler)).flatten()
    });
    let inline = Signal::derive(move || {
        let r = rail.get();
        own2.iter().copied().filter(|a| r.is_none_or(|r| !std::ptr::eq(r, *a))).collect()
    });
    (rail, inline)
}

fn with_rail(
    rail: Signal<Option<&'static Ad>>,
    here: String,
    page: impl Fn() -> AnyView + Send + Sync + 'static,
) -> AnyView {
    let shown = Memo::new(move |_| rail.get().map(|a| a.file));
    (move || match shown.get().and_then(|f| ADS.iter().find(|a| a.file == f)) {
        None => page(),
        Some(ad) => view! {
            <div class="gww-railed">
                {page()}
                <aside class="gww-rail" aria-label="Sponsored">
                    {ad_box(ad, Some(&here), true)}
                </aside>
            </div>
        }
        .into_any(),
    })
    .into_any()
}

/* ---------- Text ---------- */

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

fn broadcast(text: &str) -> AnyView {
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

fn field_tip(text: &'static str) -> AnyView {
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

fn section_block(sec: &'static Section, id: String) -> AnyView {
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

/* ---------- Page frame ---------- */

pub struct Crumb {
    pub label: String,
    pub target: Option<&'static str>,
}

fn crumb(label: impl Into<String>, target: Option<&'static str>) -> Crumb {
    Crumb { label: label.into(), target }
}

fn banner(
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

fn toc(sections: &'static [Section], prefix: &str) -> AnyView {
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

fn ties(ties: &'static [Tie], title: &'static str) -> AnyView {
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

fn source_line(sources: &[&'static Source], quoted: Option<&'static str>) -> AnyView {
    let body = if sources.is_empty() {
        "Written for this tracker.".into_any()
    } else {
        view! {
            "Adapted from the "{WIKI.home.title}":"" "
            {sources
                .iter()
                .copied()
                .enumerate()
                .map(|(i, s)| view! {
                    <span>
                        {(i > 0).then_some(", ")}
                        <OutLink href=s.url>{s.title}</OutLink>
                    </span>
                })
                .collect_view()}
            ", under "
            <OutLink href=WIKI_LICENCE.url>{WIKI_LICENCE.title}</OutLink>
            ", which this page shares."
        }
        .into_any()
    };
    view! {
        <footer class="gww-source">
            {body}
            {quoted.map(|q| view! { " "{q} })}
        </footer>
    }
    .into_any()
}

fn pager(list: Vec<String>, at: usize, noun: &'static str) -> AnyView {
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

fn ads_inline(ads: Signal<Vec<&'static Ad>>, here: String) -> AnyView {
    (move || {
        let list = ads.get();
        (!list.is_empty()).then(|| {
            view! { <div class="gww-ads-inline">{list.into_iter().map(|a| ad_box(a, Some(&here), false)).collect_view()}</div> }
        })
    })
    .into_any()
}

/* ---------- Tiles ---------- */

#[allow(clippy::too_many_arguments)]
pub fn site_tile(
    target: String,
    address: &'static str,
    title: String,
    text: String,
    icon: AnyView,
    color: Option<String>,
    done: Signal<bool>,
    out: bool,
) -> AnyView {
    let nav = use_gww_nav();
    view! {
        <button
            class=move || format!("gww-site {}", if done.get() { "done" } else { "" })
            style=color.map(|c| format!("--site: {c};"))
            on:click=move |e| nav.follow(&target, Some(&e))
        >
            <span class="gww-site-icon">{icon}</span>
            <span class="gww-site-body">
                <span class="gww-address num">
                    {address}
                    {out.then(|| view! {
                        <Icon icon=icons::ARROW_UP_RIGHT size=11.0 />
                        <span class="sr-only">" (leaves the G.W.W.)"</span>
                    })}
                </span>
                <b class="gww-site-title">{title}</b>
                <span class="gww-site-text">{text}</span>
            </span>
            <Show when=move || done.get()>
                <span class="gww-site-done" title="Read">
                    <Icon icon=icons::CHECK size=12.0 label="Read" />
                </span>
            </Show>
        </button>
    }
    .into_any()
}

pub fn company_tile(c: &'static Company) -> AnyView {
    let nav = use_gww_nav();
    let target = format!("company:{}", c.id);
    view! {
        <button class="gww-site corp" style=format!("--site: {};", c.color) on:click=move |e| nav.follow(&target, Some(&e))>
            <span class="gww-corp-logo">{company_mark(c)}</span>
            <span class="gww-site-body">
                <span class="gww-address num">{c.address}</span>
                <b class="gww-site-title">{c.name}</b>
                <span class="gww-site-text">{c.slogan.unwrap_or(c.summary)}</span>
            </span>
        </button>
    }
    .into_any()
}

pub fn manual_target(m: &Manual) -> String {
    format!("manual:{}", m.id)
}

pub fn manual_tile(m: &'static Manual) -> AnyView {
    let id = m.id;
    let read = Signal::derive(move || settings().get(|s| s.gww_read.iter().any(|r| r == id)));
    site_tile(
        manual_target(m),
        m.address,
        m.title.into(),
        m.summary.into(),
        manual_icon(m, 30.0),
        Some("var(--cta)".into()),
        read,
        false,
    )
}

pub fn ministry_tile(m: &'static Ministry) -> AnyView {
    site_tile(
        format!("ministry:{}", m.id),
        m.address,
        m.name.into(),
        m.summary.into(),
        view! { <img src=icon_url(m.icon) alt="" /> }.into_any(),
        Some(m.color.into()),
        Signal::stored(false),
        false,
    )
}

fn humanity() -> &'static str {
    MINISTRIES.iter().find(|m| m.id == "humanity").map_or("", |m| m.color)
}

pub fn citizens_tile() -> AnyView {
    site_tile(
        "site:citizens".into(),
        CITIZENS.address,
        CITIZENS.title.into(),
        CITIZENS.summary.into(),
        view! { <img src=icon_url(CITIZENS.icon) alt="" /> }.into_any(),
        Some(humanity().into()),
        Signal::stored(false),
        false,
    )
}

pub fn gov_tile() -> AnyView {
    site_tile(
        "site:gov".into(),
        GOV.address,
        GOV.title.into(),
        GOV.summary.into(),
        view! { <img src=icon_url(GOV.icon) alt="" /> }.into_any(),
        Some("var(--f-humans)".into()),
        Signal::stored(false),
        false,
    )
}

pub fn log_tile(log_text: Signal<String>) -> AnyView {
    (move || {
        site_tile(
            "site:log".into(),
            "se://hc/war-log",
            "Galactic war log".into(),
            log_text.get(),
            view! { <Icon icon=icons::HISTORY size=26.0 stroke_width=1.5 /> }.into_any(),
            Some("var(--f-humans)".into()),
            Signal::stored(false),
            false,
        )
    })
    .into_any()
}

pub fn news_tile() -> AnyView {
    site_tile(
        "app:news".into(),
        "se://strohmann.news",
        "Strohmann News".into(),
        "Dispatches and campaign news, in the tracker's news page.".into(),
        view! { <Icon icon=icons::NEWSPAPER size=26.0 stroke_width=1.5 /> }.into_any(),
        Some("#f074d8".into()),
        Signal::stored(false),
        true,
    )
}

#[component]
pub fn CourseProgress(#[prop(optional)] compact: bool) -> impl IntoView {
    let nav = use_gww_nav();
    let read = Memo::new(move |_| settings().get(|s| s.gww_read.clone()));
    let is_read = move |id: &str| read.with(|r| r.iter().any(|x| x == id));
    let done = Memo::new(move |_| MANUALS.iter().filter(|m| is_read(m.id)).count());
    let next = Memo::new(move |_| MANUALS.iter().position(|m| !is_read(m.id)));
    view! {
        <section class=format!("gww-course {}", if compact { "compact" } else { "" }) aria-label="Recruit's course">
            <div class="gww-course-head">
                <Icon icon=icons::GRADUATION_CAP size=20.0 />
                <div>
                    <b class="caps">"Recruit's course"</b>
                    <span class="num">{move || done.get()}" of "{MANUALS.len()}" training manuals read"</span>
                </div>
                {move || match next.get() {
                    Some(i) => {
                        let m = &MANUALS[i];
                        let target = manual_target(m);
                        view! {
                            <button class="gww-course-next" on:click=move |e| nav.follow(&target, Some(&e))>
                                <span class="caps">{if done.get() == 0 { "Start with" } else { "Continue with" }}</span>
                                <b>{m.title}</b>
                                <Icon icon=icons::ARROW_RIGHT size=14.0 />
                            </button>
                        }
                        .into_any()
                    }
                    None => view! { <span class="gww-course-done caps">"Course complete. Democracy thanks you."</span> }.into_any(),
                }}
            </div>
            <ol class="gww-course-bar" aria-label="Manuals, in reading order">
                {MANUALS
                    .iter()
                    .map(|m| {
                        let target = manual_target(m);
                        view! {
                            <li class=move || if is_read(m.id) { "on" } else { "" }>
                                <button
                                    title=m.title
                                    aria-label=move || format!("{}{}", m.title, if is_read(m.id) { ", read" } else { "" })
                                    on:click=move |e| nav.follow(&target, Some(&e))
                                ></button>
                            </li>
                        }
                    })
                    .collect_view()}
            </ol>
            {(!compact).then(|| view! {
                <Show when=move || { done.get() > 0 }>
                    <button class="link gww-course-reset caps" on:click=move |_| settings().clear_read()>
                        "Start the course over"
                    </button>
                </Show>
            })}
        </section>
    }
}

/* ---------- Pages ---------- */

pub fn manual_page(m: &'static Manual, heading: HeadingRef) -> AnyView {
    let end = NodeRef::<html::Span>::new();
    let at = MANUALS.iter().position(|x| x.id == m.id).unwrap_or(0);
    let chapter = CHAPTERS.iter().position(|c| c.id == m.chapter).unwrap_or(0);
    let prefix = format!("gww-{}", m.id);
    // A manual counts as read once its end has been on screen.
    Effect::new(move |_| {
        let Some(el) = end.get() else { return };
        let id = m.id;
        let cb = Closure::<dyn Fn(js_sys::Array)>::new(move |entries: js_sys::Array| {
            if entries
                .iter()
                .any(|e| e.dyn_ref::<web_sys::IntersectionObserverEntry>().is_some_and(|e| e.is_intersecting()))
            {
                settings().mark_read(id);
            }
        });
        if let Ok(io) = web_sys::IntersectionObserver::new(cb.as_ref().unchecked_ref()) {
            io.observe(&el);
            let io = SendWrapper::new(io);
            let cb = SendWrapper::new(cb);
            on_cleanup(move || {
                io.disconnect();
                drop(cb);
            });
        }
    });
    let sources: Vec<&'static Source> = m.sources.map(|s| s.iter().collect()).unwrap_or_default();
    view! {
        <article class="gww-page">
            {banner(
                Some(manual_icon(m, 42.0)),
                vec![
                    crumb("Training manuals", Some("dir:manuals")),
                    crumb(format!("Chapter {}: {}", chapter + 1, CHAPTERS[chapter].title), None),
                ],
                m.title.into(),
                Some(m.summary.into_any()),
                heading,
                false,
            )}
            {toc(m.sections, &prefix)}
            {live_panel(m.id)}
            {m.sections.iter().enumerate().map(|(i, sec)| section_block(sec, format!("{prefix}-{i}"))).collect_view()}
            {m.quotes.map(|q| q.iter().map(|q| training_tip(q)).collect_view())}
            {m.tips.iter().map(|t| field_tip(t)).collect_view()}
            {related(m.related.to_vec(), "Read next")}
            <span node_ref=end class="gww-end" aria-hidden="true"></span>
            {source_line(&sources, m.quotes.filter(|q| !q.is_empty()).map(|_| "Training manual tips are quoted from the game."))}
            {pager(MANUALS.iter().map(manual_target).collect(), at, "manual")}
        </article>
    }
    .into_any()
}

fn slogan_or_summary(slogan: Option<&'static str>, summary: &'static str) -> AnyView {
    match slogan {
        Some(s) => view! { <span class="gww-slogan">"“"{s}"”"</span> }.into_any(),
        None => summary.into_any(),
    }
}

pub fn ministry_page(m: &'static Ministry, heading: HeadingRef) -> AnyView {
    let prefix = format!("gww-{}", m.id);
    let here = format!("ministry:{}", m.id);
    let (rail, inline) = site_ads(m.ads, here.clone());
    let at = MINISTRIES.iter().position(|x| x.id == m.id).unwrap_or(0);
    let here2 = here.clone();
    let page = move || {
        view! {
            <article class="gww-page" style=format!("--site: {};", m.color)>
                {banner(
                    icon_url(m.icon).map(|u| view! { <img src=u alt="" width="72" height="72" /> }.into_any()),
                    vec![crumb("Super Earth", Some("dir:gov")), crumb("Ministries", None)],
                    m.name.into(),
                    Some(slogan_or_summary(m.slogan, m.summary)),
                    heading,
                    false,
                )}
                {training_tip(m.motto)}
                {toc(m.sections, &prefix)}
                {m.sections.iter().enumerate().map(|(i, sec)| section_block(sec, format!("{prefix}-{i}"))).collect_view()}
                {m.ties.map(|t| ties(t, "Ties"))}
                {ads_inline(inline, here2.clone())}
                {related(m.related.map(|r| r.to_vec()).unwrap_or_default(), "Read next")}
                {source_line(&[&m.source, &WIKI.government], Some("Its tip and poster line are quoted from the game."))}
                {pager(MINISTRIES.iter().map(|x| format!("ministry:{}", x.id)).collect(), at, "ministry")}
            </article>
        }
        .into_any()
    };
    with_rail(rail, here, page)
}

pub fn company_page(c: &'static Company, heading: HeadingRef) -> AnyView {
    let prefix = format!("gww-{}", c.id);
    let here = format!("company:{}", c.id);
    let (rail, inline) = site_ads(c.ads, here.clone());
    let at = COMPANIES.iter().position(|x| x.id == c.id).unwrap_or(0);
    let here2 = here.clone();
    let page = move || {
        let showcase = c.showcase.as_ref().and_then(|s| logo_url(s.image).map(|u| (u, s.alt)));
        view! {
            <article class="gww-page" style=format!("--site: {};", c.color)>
                {banner(
                    Some(company_mark(c)),
                    vec![crumb("Corporations", Some("dir:corps")), crumb(c.trade, None)],
                    c.name.into(),
                    Some(slogan_or_summary(c.slogan, c.summary)),
                    heading,
                    true,
                )}
                {c.broadcast.map(broadcast)}
                {toc(c.sections, &prefix)}
                {c.sections.iter().enumerate().map(|(i, sec)| section_block(sec, format!("{prefix}-{i}"))).collect_view()}
                {showcase.map(|(u, alt)| view! {
                    <figure class="gww-showcase">
                        <img src=u alt=alt loading="lazy" />
                    </figure>
                })}
                {c.quote.map(training_tip)}
                {c.ties.map(|t| ties(t, "Ties"))}
                {ads_inline(inline, here2.clone())}
                {related(c.related.map(|r| r.to_vec()).unwrap_or_default(), "Read next")}
                {source_line(&[&c.source], Some("Slogans and broadcast lines are quoted from the game."))}
                {pager(COMPANIES.iter().map(|x| format!("company:{}", x.id)).collect(), at, "corporation")}
            </article>
        }
        .into_any()
    };
    with_rail(rail, here, page)
}

/// Super Earth's own sites: Managed Democracy, and the Citizenship Classification Structure under it.
pub fn gov_page(site: &str, heading: HeadingRef) -> AnyView {
    let gov = site == "gov";
    let prefix = format!("gww-{site}");
    let (title, icon, summary, sections, related_list, sources) = if gov {
        (GOV.title, GOV.icon, GOV.summary, GOV.sections, GOV.related, GOV.sources)
    } else {
        (CITIZENS.title, CITIZENS.icon, CITIZENS.summary, CITIZENS.sections, CITIZENS.related, CITIZENS.sources)
    };
    let mut crumbs = vec![crumb("Super Earth", Some("dir:gov"))];
    if !gov {
        crumbs.push(crumb(GOV.title, Some("site:gov")));
    }
    let sources: Vec<&'static Source> = sources.iter().collect();
    view! {
        <article class="gww-page" style=format!("--site: {};", if gov { "var(--f-humans)" } else { humanity() })>
            {banner(
                Some(view! { <img src=icon_url(icon) alt="" width="72" height="72" /> }.into_any()),
                crumbs,
                title.into(),
                Some(summary.into_any()),
                heading,
                false,
            )}
            {(!gov).then(|| CITIZENS.broadcasts.iter().map(|b| broadcast(b)).collect_view())}
            {toc(sections, &prefix)}
            {sections.iter().enumerate().map(|(i, sec)| section_block(sec, format!("{prefix}-{i}"))).collect_view()}
            {gov.then(|| GOV.quotes.iter().map(|q| training_tip(q)).collect_view())}
            {related(related_list.to_vec(), "Read next")}
            {source_line(
                &sources,
                Some(if gov { "Training manual tips are quoted from the game." } else { "Ship broadcasts are quoted from the game." }),
            )}
        </article>
    }
    .into_any()
}

pub fn missing_page(address: String, heading: HeadingRef) -> AnyView {
    view! {
        <article class="gww-page" style="--site: var(--bad);">
            {banner(
                Some(view! { <Icon icon=icons::TRIANGLE_ALERT size=40.0 stroke_width=1.5 /> }.into_any()),
                vec![crumb("G.W.W.", Some("home")), crumb("Error 404", None)],
                "Site not found".into(),
                Some(view! {
                    "No site answers at "
                    <span class="num">{address}</span>
                    ". If one ever did, the Ministry of Truth has seen to it."
                }
                .into_any()),
                heading,
                false,
            )}
            {related(vec!["dir:manuals", "dir:gov", "dir:corps"], "Try instead")}
        </article>
    }
    .into_any()
}

#[derive(Clone, Copy, PartialEq)]
enum LogFilter {
    All,
    Won,
    Lost,
}

pub fn war_log_page(heading: HeadingRef) -> AnyView {
    let h = history();
    let show = RwSignal::new(LogFilter::All);
    let changes = h.changes;
    let ready = h.ready;
    let lead = move || {
        if ready.get() {
            let (lib, total) = changes.with(|c| (c.iter().filter(|c| c.change.to == Faction::Humans).count(), c.len()));
            view! {
                <span class="num">
                    <span style="color: var(--f-humans);">{lib}" liberated"</span>
                    " ·"
                    " "
                    <span style="color: var(--bad);">{total - lib}" lost"</span>
                    " since tracking began"
                </span>
            }
            .into_any()
        } else {
            "Every planet that changed hands since tracking began.".into_any()
        }
    };
    let body = move || {
        if !ready.get() {
            return view! { <Loading what="the war archive" rows=3 /> }.into_any();
        }
        let s = show.get();
        let list: Vec<_> = changes.with(|c| {
            c.iter()
                .filter(|c| s == LogFilter::All || (s == LogFilter::Won) == (c.change.to == Faction::Humans))
                .cloned()
                .collect()
        });
        if list.is_empty() {
            let text = match s {
                LogFilter::All => "No planet has changed hands since tracking began.".to_string(),
                LogFilter::Won => "No planet has been liberated since tracking began.".to_string(),
                LogFilter::Lost => "No planet has been lost since tracking began.".to_string(),
            };
            return view! { <Empty>{text}</Empty> }.into_any();
        }
        let mut days: Vec<(String, Vec<_>)> = Vec::new();
        for c in list {
            let day = fmt_date(c.change.at);
            match days.last_mut() {
                Some((d, items)) if *d == day => items.push(c),
                _ => days.push((day, vec![c])),
            }
        }
        days.into_iter()
            .map(|(day, items)| view! {
                <section class="gww-log-day">
                    <h3 class="caps num">{day}</h3>
                    <ol class="wl">
                        {items
                            .into_iter()
                            .map(|c| {
                                let ch = c.change;
                                let won = ch.to == Faction::Humans;
                                let color = faction_color(Some(if won { Faction::Humans } else { ch.to }));
                                let planet = ch.planet;
                                view! {
                                    <li class="wl-row">
                                        <span class="wl-mark" style=format!("--k: {color};")>
                                            <FactionEmblem faction=if won { "Humans" } else { ch.to.as_str() } size=14.0 />
                                        </span>
                                        <div>
                                            <div class="wl-title caps">
                                                <button
                                                    class="link"
                                                    style="color: var(--text-hi);"
                                                    on:click=move |e| ui().open_options(Some(OptionsAnchor { planet, x: e.client_x() as f64, y: e.client_y() as f64 }))
                                                >
                                                    {ch.planet_name.clone()}
                                                </button>
                                                " "
                                                <span style=format!("color: {color};")>
                                                    {if won { format!("liberated from {}", faction_label(Some(ch.from))) } else { format!("fell to {}", faction_label(Some(ch.to))) }}
                                                </span>
                                            </div>
                                            <div class="wl-sub">{fmt_date_time(ch.at)}</div>
                                        </div>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ol>
                </section>
            })
            .collect_view()
            .into_any()
    };
    view! {
        <article class="gww-page" style="--site: var(--f-humans);">
            {banner(
                Some(view! { <Icon icon=icons::HISTORY size=40.0 stroke_width=1.5 /> }.into_any()),
                vec![crumb("Super Earth", Some("dir:gov"))],
                "Galactic war log".into(),
                Some(lead.into_any()),
                heading,
                false,
            )}
            <div class="gww-filter" role="group" aria-label="Show">
                {[(LogFilter::All, "Everything"), (LogFilter::Won, "Liberated"), (LogFilter::Lost, "Lost")]
                    .map(|(k, label)| view! {
                        <button
                            class=move || format!("gww-chip {}", if show.get() == k { "on" } else { "" })
                            aria-pressed=move || (show.get() == k).to_string()
                            on:click=move |_| show.set(k)
                        >
                            {label}
                        </button>
                    })
                    .collect_view()}
            </div>
            {body}
            {related(vec!["manual:liberation", "manual:defense", "app:past"], "Read next")}
        </article>
    }
    .into_any()
}

pub fn dir_page(id: &str, heading: HeadingRef, log_text: Signal<String>) -> AnyView {
    let lead = match id {
        "manuals" => "How the Second Galactic War works, in reading order.",
        "gov" => "The Federation's own sites: its government, its ministries, its war log and its news.",
        _ => "The mega-corporations that arm, feed, heal and bank the Federation.",
    };
    let title = DIRS.iter().find(|(k, _)| *k == id).map_or("", |(_, t)| *t);
    let body = match id {
        "manuals" => view! {
            <CourseProgress />
            {CHAPTERS
                .iter()
                .enumerate()
                .map(|(i, ch)| view! {
                    <section class="gww-chapter">
                        <h3>
                            <span class="caps num">"Chapter "{i + 1}</span>
                            " "{ch.title}
                        </h3>
                        <p class="gww-chapter-sub">{ch.summary}</p>
                        <div class="gww-sites">{MANUALS.iter().filter(|m| m.chapter == ch.id).map(manual_tile).collect_view()}</div>
                    </section>
                })
                .collect_view()}
        }
        .into_any(),
        "gov" => view! {
            <div class="gww-sites">
                {gov_tile()}
                {citizens_tile()}
                {log_tile(log_text)}
                {news_tile()}
                {MINISTRIES.iter().map(ministry_tile).collect_view()}
            </div>
        }
        .into_any(),
        _ => view! { <div class="gww-sites">{COMPANIES.iter().map(company_tile).collect_view()}</div> }.into_any(),
    };
    let here: &'static str = match id {
        "manuals" => "dir:manuals",
        "gov" => "dir:gov",
        _ => "dir:corps",
    };
    view! {
        <article class="gww-page wide">
            {banner(
                dir_icon(id).map(|i| view! { <Icon icon=i size=40.0 stroke_width=1.5 /> }.into_any()),
                vec![crumb("G.W.W.", Some("home"))],
                title.into(),
                Some(lead.into_any()),
                heading,
                false,
            )}
            {body}
            {related(["dir:manuals", "dir:gov", "dir:corps"].into_iter().filter(|t| *t != here).collect(), "Elsewhere on the G.W.W.")}
        </article>
    }
    .into_any()
}
