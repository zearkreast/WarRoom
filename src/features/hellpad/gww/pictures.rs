//! Pictures for G.W.W. pages: manual, company and link icons, and the adverts in a page's rail.

use crate::features::hellpad::gww::nav::{ad_url, icon_url, logo_url, use_gww_nav};
use crate::js::{self, num};
use crate::ui::emblems::{IconLike, icon_view};
use crate::ui::icons::{self, Icon, IconData};
use leptos::prelude::*;
use warroom_core::data::manuals::MANUALS;
use warroom_core::data::sites::{ADS, CITIZENS, COMPANIES, GOV, MINISTRIES};
use warroom_core::data::{Ad, AdBy, AdShape, Company, Manual};
use warroom_core::gww_nav::title_of;

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

pub(super) fn dir_icon(id: &str) -> Option<IconData> {
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

pub(super) fn split(target: &str) -> (&str, &str) {
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
        k if k == "holiday" || (k == "site" && id == "holidays") => {
            view! { <Icon icon=icons::CALENDAR_DAYS size=size stroke_width=1.5 /> }.into_any()
        }
        "site" if id == "log" => view! { <Icon icon=icons::HISTORY size=size stroke_width=1.5 /> }.into_any(),
        "site" => img(icon_url(if id == "citizens" { CITIZENS.icon } else { GOV.icon })),
        "dir" => dir_icon(id)
            .map_or_else(|| ().into_any(), |i| view! { <Icon icon=i size=size stroke_width=1.5 /> }.into_any()),
        "app" => app_icon(id).map_or_else(|| ().into_any(), |i| icon_view(i, size, 1.5, None)),
        _ => ().into_any(),
    }
}

pub(super) fn kind_label(kind: &str) -> &'static str {
    match kind {
        "manual" => "Training manual",
        "ministry" => "Ministry",
        "company" => "Corporation",
        "holiday" => "Holiday",
        "site" => "Super Earth",
        "dir" => "Directory",
        "app" => "Off the G.W.W.",
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
pub(super) fn site_ads(
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

pub(super) fn with_rail(
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
