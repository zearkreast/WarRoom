//! The G.W.W.'s pages: manuals, ministries, corporations, the government site, holidays, directories, and the page for an unknown address.

use crate::features::hellpad::gww::blocks::{
    Crumb, HeadingRef, ads_inline, banner, broadcast, crumb, field_tip, pager, related, section_block, sourced_quote,
    ties, toc, training_tip,
};
use crate::features::hellpad::gww::live::live_panel;
use crate::features::hellpad::gww::nav::{icon_url, logo_url, use_gww_nav};
use crate::features::hellpad::gww::pictures::{company_mark, dir_icon, manual_icon, site_ads, with_rail};
use crate::features::hellpad::gww::tiles::{
    CourseProgress, company_tile, gov_tile, humanity, log_tile, manual_target, manual_tile, ministry_tile, news_tile,
};
use crate::js;
use crate::state::settings;
use crate::ui::icons::{self, Icon};
use leptos::html;
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use warroom_core::data::manuals::{CHAPTERS, MANUALS};
use warroom_core::data::sites::{CITIZENS, COMPANIES, GOV, HOLIDAYS, HOLIDAYS_SITE, MINISTRIES};
use warroom_core::data::{Company, Holiday, Manual, Ministry};
use warroom_core::gww_nav::DIRS;
use warroom_core::gww_text::{next_yearly, plain_text};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

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
                    // The name is beside the banner's picture, so a company without a logo shows no picture there.
                    c.logo.and_then(logo_url).map(|_| company_mark(c)),
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
                {pager(COMPANIES.iter().map(|x| format!("company:{}", x.id)).collect(), at, "corporation")}
            </article>
        }
        .into_any()
    };
    with_rail(rail, here, page)
}

const GOV_TABS: [(&str, &str); 3] =
    [("gov", "Managed Democracy"), ("citizens", "Citizen classes"), ("holidays", "Holidays")];

/// Managed Democracy's own menu: its sub-sites are reached from here, not from the G.W.W. home.
fn gov_nav(at: &str) -> AnyView {
    let nav = use_gww_nav();
    view! {
        <nav class="gww-sitenav" aria-label="Managed Democracy">
            <span class="gww-sitenav-brand caps">
                <img src=icon_url(GOV.icon) alt="" width="18" height="18" />
                "se://gov"
            </span>
            {GOV_TABS
                .iter()
                .map(|&(id, label)| {
                    let on = id == at;
                    let target = format!("site:{id}");
                    view! {
                        <button
                            class=format!("gww-sitenav-link caps {}", if on { "on" } else { "" })
                            aria-current=on.then_some("page")
                            on:click=move |e| nav.follow(&target, Some(&e))
                        >
                            {label}
                        </button>
                    }
                })
                .collect_view()}
        </nav>
    }
    .into_any()
}

fn gov_crumbs(site: &str) -> Vec<Crumb> {
    let mut crumbs = vec![crumb("Super Earth", Some("dir:gov"))];
    if site != "gov" {
        crumbs.push(crumb(GOV.title, Some("site:gov")));
    }
    crumbs
}

/// Super Earth's own sites: Managed Democracy, and the Citizenship Classification Structure under it.
pub fn gov_page(site: &str, heading: HeadingRef) -> AnyView {
    let gov = site == "gov";
    let prefix = format!("gww-{site}");
    let (title, icon, summary, sections, related_list) = if gov {
        (GOV.title, GOV.icon, GOV.summary, GOV.sections, GOV.related)
    } else {
        (CITIZENS.title, CITIZENS.icon, CITIZENS.summary, CITIZENS.sections, CITIZENS.related)
    };
    view! {
        <article class="gww-page" style=format!("--site: {};", if gov { "var(--f-humans)" } else { humanity() })>
            {gov_nav(site)}
            {banner(
                Some(view! { <img src=icon_url(icon) alt="" width="72" height="72" /> }.into_any()),
                gov_crumbs(site),
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
        </article>
    }
    .into_any()
}

const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

fn date_badge(h: &Holiday) -> AnyView {
    view! {
        <span class="gww-date" aria-hidden="true">
            <span class="caps">{MONTHS[h.month as usize - 1]}</span>
            <b class="num">{h.day}</b>
        </span>
    }
    .into_any()
}

fn countdown(days: i64) -> String {
    match days {
        0 => "Today".into(),
        1 => "Tomorrow".into(),
        n => format!("In {n} days"),
    }
}

fn days_until(h: &Holiday, now: f64) -> i64 {
    let (y, m, d) = js::local_today(now);
    next_yearly(&[(h.month, h.day)], y, m, d).map_or(0, |(_, n)| n)
}

fn holiday_target(h: &Holiday) -> String {
    format!("holiday:{}", h.id)
}

pub fn holidays_page(heading: HeadingRef) -> AnyView {
    let nav = use_gww_nav();
    let now = js::use_now(60.0 * 60_000.0);
    let cards = move || {
        let now = now.get();
        HOLIDAYS
            .iter()
            .map(|h| {
                let days = days_until(h, now);
                let target = holiday_target(h);
                view! {
                    <li>
                        <button
                            class=format!("gww-holiday-card {}", if days == 0 { "today" } else { "" })
                            style=format!("--site: {};", h.color)
                            on:click=move |e| nav.follow(&target, Some(&e))
                        >
                            {date_badge(h)}
                            <span class="gww-holiday-body">
                                <span class="gww-address num">{h.address}</span>
                                <b class="gww-site-title">{h.name}</b>
                                <span class="caps gww-holiday-when">{h.when}</span>
                                <span class="gww-site-text">{h.summary}</span>
                                {h.sections.first().and_then(|s| s.text).map(|t| view! {
                                    <span class="gww-holiday-story">{plain_text(t)}</span>
                                })}
                            </span>
                            <span class=format!("gww-holiday-in caps num {}", if days <= 1 { "soon" } else { "" })>
                                {countdown(days)}
                            </span>
                        </button>
                    </li>
                }
            })
            .collect_view()
    };
    view! {
        <article class="gww-page" style="--site: var(--cta);">
            {gov_nav("holidays")}
            {banner(
                Some(view! { <Icon icon=icons::CALENDAR_DAYS size=40.0 stroke_width=1.5 /> }.into_any()),
                gov_crumbs("holidays"),
                HOLIDAYS_SITE.title.into(),
                Some(HOLIDAYS_SITE.summary.into_any()),
                heading,
                false,
            )}
            <ol class="gww-holidays">{cards}</ol>
            {related(HOLIDAYS_SITE.related.to_vec(), "Read next")}
        </article>
    }
    .into_any()
}

pub fn holiday_page(h: &'static Holiday, heading: HeadingRef) -> AnyView {
    let prefix = format!("gww-{}", h.id);
    let now = js::use_now(60.0 * 60_000.0);
    let days = Memo::new(move |_| days_until(h, now.get()));
    let mut crumbs = gov_crumbs("holidays");
    crumbs.push(crumb("Holidays", Some("site:holidays")));
    let at = HOLIDAYS.iter().position(|x| x.id == h.id).unwrap_or(0);
    view! {
        <article class="gww-page" style=format!("--site: {};", h.color)>
            {gov_nav("holidays")}
            {banner(
                Some(date_badge(h)),
                crumbs,
                h.name.into(),
                Some(view! {
                    {h.summary}
                    " "
                    <span class=move || format!("gww-holiday-in caps num {}", if days.get() <= 1 { "soon" } else { "" })>
                        {h.when}
                        " · "
                        {move || countdown(days.get())}
                    </span>
                }
                .into_any()),
                heading,
                false,
            )}
            {h.quotes.iter().take(1).map(|q| sourced_quote(q.from, q.text)).collect_view()}
            {toc(h.sections, &prefix)}
            {h.sections.iter().enumerate().map(|(i, sec)| section_block(sec, format!("{prefix}-{i}"))).collect_view()}
            {h.quotes.iter().skip(1).map(|q| sourced_quote(q.from, q.text)).collect_view()}
            {related(h.related.to_vec(), "Read next")}
            {pager(HOLIDAYS.iter().map(holiday_target).collect(), at, "holiday")}
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

pub fn dir_page(id: &str, heading: HeadingRef, log_text: Signal<String>) -> AnyView {
    let lead = match id {
        "manuals" => "How the Second Galactic War works, in reading order.",
        "gov" => "The Federation's own sites: its government, its ministries, its logs and its news.",
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
