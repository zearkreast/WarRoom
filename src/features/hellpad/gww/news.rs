//! Strohmann News on the G.W.W.: the dispatches and Galactic Campaign stories laid out as a news site.

use crate::api::queries::queries;
use crate::features::galactic_campaigns::{CampaignArt, campaign_art};
use crate::features::hellpad::gww::blocks::{HeadingRef, banner, crumb, related};
use crate::features::hellpad::gww::nav::{logo_url, use_gww_nav};
use crate::features::hellpad::gww::pictures::logo_plate;
use crate::features::map::pawn_art::{hologram_in, pawn_href};
use crate::features::map::stratagem_art::icon_of;
use crate::features::news::card::{MarkupText, kind_label};
use crate::features::news::feed::{entry_body, entry_label};
use crate::features::news::items::{NewsBody, NewsItem, NewsKind, news_items};
use crate::features::planet::tint::planet_tint;
use crate::js;
use crate::ui::emblems::FactionEmblem;
use crate::ui::icons::{self, Icon};
use crate::ui::primitives::Empty;
use leptos::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use warroom_core::api::{Faction, Planet};
use warroom_core::data::sites::COMPANIES;
use warroom_core::data::stratagems::STRATAGEMS;
use warroom_core::dispatch::{DispatchKind, is_order_dispatch};
use warroom_core::factions::faction_color;
use warroom_core::format::{dispatch_title, fmt_ago, js_trim, parse_markup, sest_date};
use warroom_core::gww_nav::NEWS_TITLE;

/// Stories on the page; the full archive is the app's Strohmann News.
const SHOWN: usize = 30;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Desk {
    All,
    Orders,
    Front,
    Alerts,
    Campaigns,
}

const DESKS: [(Desk, &str); 5] = [
    (Desk::All, "Top stories"),
    (Desk::Orders, "Major Orders"),
    (Desk::Front, "Front lines"),
    (Desk::Alerts, "Alerts"),
    (Desk::Campaigns, "Campaigns"),
];

impl Desk {
    fn admits(self, i: &NewsItem, planets: &HashMap<String, Planet>) -> bool {
        let k = i.kind;
        match self {
            Desk::All => true,
            // Every Major Order runs inside a Galactic Campaign, its dispatches on the campaign's thread.
            Desk::Orders => match &i.body {
                NewsBody::Dispatch(d) => is_order_dispatch(&d.message),
                NewsBody::Thread(_) => true,
            },
            // The game words front news every way ("HEETH LOST", "MINOR TERMINID OUTBREAK"); what they share is a planet.
            Desk::Front => match &i.body {
                NewsBody::Dispatch(d) => !is_order_dispatch(&d.message) && pictured(i, planets).is_some(),
                NewsBody::Thread(_) => false,
            },
            Desk::Alerts => matches!(k, NewsKind::Dispatch(DispatchKind::Alert | DispatchKind::Advisory)),
            Desk::Campaigns => k == NewsKind::Campaign,
        }
    }
}

fn kicker(i: &NewsItem) -> &'static str {
    match i.kind {
        NewsKind::Dispatch(k) => kind_label(k),
        NewsKind::Campaign => "Campaign",
    }
}

fn headline(i: &NewsItem) -> String {
    match &i.body {
        NewsBody::Dispatch(d) => dispatch_title(&d.message),
        NewsBody::Thread(t) => t.campaign.title.clone(),
    }
}

/// The story's text after its headline, cut short for the briefs.
fn excerpt(i: &NewsItem, max: usize) -> String {
    let head = headline(i);
    let text = js_trim(i.text.strip_prefix(head.as_str()).unwrap_or(&i.text)).to_string();
    if text.chars().count() <= max {
        return text;
    }
    let cut: String = text.chars().take(max).collect();
    format!("{}…", cut.rsplit_once(' ').map_or(cut.as_str(), |(a, _)| a))
}

/// The game's highlighted names in the story's text, in order.
fn named(i: &NewsItem) -> Vec<String> {
    let mut texts: Vec<&str> = Vec::new();
    match &i.body {
        NewsBody::Dispatch(d) => texts.push(&d.message),
        NewsBody::Thread(t) => {
            texts.extend(t.entries.iter().rev().filter_map(|e| e.dispatch.as_ref()).map(|d| d.message.as_str()));
            texts.extend(t.campaign.phases.iter().rev().map(|p| p.briefing.as_str()));
        }
    }
    texts
        .into_iter()
        .flat_map(parse_markup)
        .filter(|s| s.style.is_some())
        .map(|s| js_trim(&s.text).to_uppercase())
        .collect()
}

/// Where `name` stands in `text` as whole words.
fn word_at(text: &str, name: &str) -> Option<usize> {
    let edge = |c: Option<char>| c.is_none_or(|c| !c.is_alphanumeric());
    text.match_indices(name)
        .find(|(at, _)| edge(text[..*at].chars().next_back()) && edge(text[at + name.len()..].chars().next()))
        .map(|(at, _)| at)
}

/// The first planet a story names, for its picture: in its highlights, which may name several ("FRONTERIA,
/// CRIMSICA and…"), then in its headline, which the game doesn't always highlight.
fn pictured<'a>(i: &NewsItem, planets: &'a HashMap<String, Planet>) -> Option<&'a Planet> {
    let first_in = |text: &str| {
        planets
            .iter()
            .filter_map(|(key, p)| word_at(text, key).map(|at| (at, std::cmp::Reverse(key.len()), p)))
            .min_by_key(|(at, len, _)| (*at, *len))
            .map(|(_, _, p)| p)
    };
    named(i).iter().find_map(|n| first_in(n)).or_else(|| first_in(&headline(i).to_uppercase()))
}

/// The icon of the stratagem a story is about, the longest name it holds winning ("Orbital Laser" over "Laser").
/// Only a story that says "Stratagem" counts, so "reinforce" in passing isn't the Reinforce stratagem.
fn stratagem_of(i: &NewsItem) -> Option<String> {
    let text = i.text.to_uppercase();
    if !text.contains("STRATAGEM") {
        return None;
    }
    STRATAGEMS
        .iter()
        .filter(|s| word_at(&text, &s.short_name().to_uppercase()).is_some())
        .max_by_key(|s| s.short_name().len())
        .and_then(icon_of)
}

/// The enemy a story is about: its campaign's, or the first one its text names.
fn enemy_of(i: &NewsItem) -> Option<Faction> {
    if let NewsBody::Thread(t) = &i.body
        && t.campaign.faction != Faction::Humans
    {
        return Some(t.campaign.faction);
    }
    let text = i.text.to_lowercase();
    [("terminid", Faction::Terminids), ("automaton", Faction::Automaton), ("illuminate", Faction::Illuminate)]
        .into_iter()
        .filter_map(|(word, f)| text.find(word).map(|at| (at, f)))
        .min_by_key(|(at, _)| *at)
        .map(|(_, f)| f)
}

/// A story's picture: its campaign's own, else the planet it names with the stratagem it brings, else the
/// stratagem, hologram or enemy it is about.
fn picture(i: &NewsItem, planets: &HashMap<String, Planet>, class: &'static str) -> AnyView {
    let enemy = enemy_of(i);
    let tone = enemy.map_or("#f074d8", |f| faction_color(Some(f)));
    let art = match &i.body {
        NewsBody::Thread(t) => Some(view! { <CampaignArt names=campaign_art(&t.campaign) class="sn-art" /> }),
        NewsBody::Dispatch(_) => None,
    };
    let stratagem = stratagem_of(i);
    if let Some(p) = pictured(i, planets) {
        let land = planet_tint(p, &[]).art.unwrap_or_default();
        let owner = faction_color(Some(p.current_owner));
        return view! {
            <figure class=format!("sn-pic {class}") style=format!("--art: {land}; --k: {owner};")>
                {art}
                {stratagem.map(|u| view! { <img class="sn-badge" src=u alt="" /> })}
                <figcaption class="caps">{p.name.clone()}</figcaption>
            </figure>
        }
        .into_any();
    }
    let hologram = hologram_in(&i.text).and_then(pawn_href);
    view! {
        <figure class=format!("sn-pic plain {class}") style=format!("--k: {tone};") aria-hidden="true">
            {match (stratagem, hologram, enemy) {
                (Some(u), _, _) => view! { <img class="sn-glyph" src=u alt="" /> }.into_any(),
                (None, Some(u), _) => view! { <img class="sn-holo" src=u alt="" /> }.into_any(),
                (None, None, Some(f)) => view! { <FactionEmblem faction=f.as_str() size=96.0 color=tone.to_string() /> }.into_any(),
                (None, None, None) => logo_url("strohmann-news").map(|u| view! { <img src=u alt="" /> }).into_any(),
            }}
            {art}
        </figure>
    }
    .into_any()
}

fn story_body(i: &NewsItem) -> AnyView {
    match &i.body {
        NewsBody::Dispatch(d) => view! { <MarkupText message=d.message.clone() class="sn-text" /> }.into_any(),
        NewsBody::Thread(t) => match t.entries.last() {
            Some(latest) => view! {
                <p class="sn-entry caps">{entry_label(latest)}</p>
                {entry_body(latest)}
            }
            .into_any(),
            None => view! { <p class="sn-text">{t.campaign.description.clone()}</p> }.into_any(),
        },
    }
}

fn byline(i: &NewsItem, now: ReadSignal<f64>) -> AnyView {
    let at = i.at;
    view! {
        <p class="sn-byline caps num">
            <span class="sn-kicker">{kicker(i)}</span>
            <span>{move || fmt_ago(at, now.get())}</span>
            <span>{format!("SEST {}", sest_date(at))}</span>
        </p>
    }
    .into_any()
}

pub fn news_page(heading: HeadingRef) -> AnyView {
    let nav = use_gww_nav();
    let q = queries();
    let items = news_items();
    let desk = RwSignal::new(Desk::All);
    let now = js::use_now(60_000.0);
    let planets = Memo::new(move |_| {
        Arc::new(q.fresh_planets.with(|p| {
            p.iter().flat_map(|l| l.iter()).map(|p| (p.name.to_uppercase(), p.clone())).collect::<HashMap<_, _>>()
        }))
    });
    let shown = Memo::new(move |_| {
        let d = desk.get();
        let planets = planets.get();
        items.with(|l| l.iter().filter(|i| d.admits(i, &planets)).take(SHOWN).cloned().collect::<Vec<_>>())
    });
    let company = COMPANIES.iter().find(|c| c.id == "strohmann-media");
    let paper = move || {
        let list = shown.get();
        let planets = planets.get();
        if list.is_empty() {
            let loading = q.dispatches.with(Option::is_none);
            return view! {
                <Empty>{if loading { "The newsroom is receiving the latest dispatches…" } else { "No story on this desk yet." }}</Empty>
            }
            .into_any();
        }
        let lead = &list[0];
        let seconds: Vec<&NewsItem> = list.iter().skip(1).take(2).collect();
        let briefs: Vec<&NewsItem> = list.iter().skip(3).collect();
        view! {
            <article class="sn-lead">
                {picture(lead, &planets, "big")}
                <div class="sn-lead-text">
                    {byline(lead, now)}
                    <h3 class="sn-headline">{headline(lead)}</h3>
                    {story_body(lead)}
                </div>
            </article>
            {(!seconds.is_empty()).then(|| view! {
                <div class="sn-seconds">
                    {seconds
                        .into_iter()
                        .map(|i| view! {
                            <article class="sn-second">
                                {picture(i, &planets, "mid")}
                                {byline(i, now)}
                                <h4 class="sn-headline">{headline(i)}</h4>
                                <p class="sn-text">{excerpt(i, 220)}</p>
                            </article>
                        })
                        .collect_view()}
                </div>
            })}
            {(!briefs.is_empty()).then(|| view! {
                <section class="sn-briefs" aria-label="More stories">
                    <h3 class="sn-desk-title caps">"In brief"</h3>
                    <ol>
                        {briefs
                            .into_iter()
                            .map(|i| view! {
                                <li>
                                    <details class="sn-brief">
                                        <summary>
                                            {picture(i, &planets, "thumb")}
                                            <span class="sn-brief-text">
                                                {byline(i, now)}
                                                <b class="sn-brief-head">{headline(i)}</b>
                                                <span class="sn-brief-cut">{excerpt(i, 140)}</span>
                                            </span>
                                        </summary>
                                        <div class="sn-brief-body">{story_body(i)}</div>
                                    </details>
                                </li>
                            })
                            .collect_view()}
                    </ol>
                </section>
            })}
        }
        .into_any()
    };
    view! {
        <article class="gww-page sn" style="--site: #f074d8;">
            {banner(
                logo_url("strohmann-news").map(logo_plate),
                vec![crumb("Corporations", Some("dir:corps")), crumb("Strohmann Media", Some("company:strohmann-media"))],
                NEWS_TITLE.into(),
                company.and_then(|c| c.slogan).map(|s| s.into_any()),
                heading,
                true,
            )}
            <div class="sn-dateline caps num">
                <span class="sn-live">"Live"</span>
                <span>{move || format!("SEST {}", sest_date(now.get()))}</span>
                <span>{move || format!("{} stories", items.with(|l| l.len()))}</span>
            </div>
            <nav class="sn-desks" aria-label="Desks">
                {DESKS
                    .into_iter()
                    .map(|(d, label)| view! {
                        <button
                            class=move || format!("sn-desk caps {}", if desk.get() == d { "on" } else { "" })
                            aria-pressed=move || (desk.get() == d).to_string()
                            on:click=move |_| desk.set(d)
                        >
                            {label}
                        </button>
                    })
                    .collect_view()}
            </nav>
            {paper}
            <div class="sn-more">
                <button class="sn-archive" on:click=move |e| nav.follow("app:news", Some(&e))>
                    <Icon icon=icons::NEWSPAPER size=16.0 />
                    <span>
                        <b>"Every dispatch, by day"</b>
                        <span class="caps">"Search the archive in Strohmann News, off the G.W.W."</span>
                    </span>
                    <Icon icon=icons::ARROW_UP_RIGHT size=14.0 />
                </button>
            </div>
            {related(vec!["company:strohmann-media", "ministry:truth", "site:log"], "Read next")}
        </article>
    }
    .into_any()
}
