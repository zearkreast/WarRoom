use crate::features::news::card::{MarkupText, kind_label};
use leptos::prelude::*;
use warroom_core::api::Dispatch;
use warroom_core::campaign_news::{CampaignThread, EventType, ThreadEntry};
use warroom_core::data::reward_label;
use warroom_core::dispatch::dispatch_kind;
use warroom_core::factions::faction_color;
use warroom_core::format::{dispatch_title, fmt_ago, sest_clock, sest_date};
use warroom_core::news_feed::body_echoes_title;
use warroom_core::snapshot::{EpisodeStatus, campaign_tally, phase_name};
use warroom_core::time::{iso_string, parse_ms};

fn result_label(s: EpisodeStatus) -> &'static str {
    match s {
        EpisodeStatus::Active => "In progress",
        EpisodeStatus::Won => "Won",
        EpisodeStatus::Lost => "Failed",
    }
}

fn status_class(s: EpisodeStatus) -> &'static str {
    match s {
        EpisodeStatus::Active => "active",
        EpisodeStatus::Won => "won",
        EpisodeStatus::Lost => "lost",
    }
}

fn clock(at: f64) -> String {
    sest_clock(at)[..5].to_string()
}

/// The frame every feed item shares: time on the rail, then a tag line, the headline and the story.
#[allow(clippy::too_many_arguments)]
fn item(
    class: String,
    at: f64,
    now: Signal<f64>,
    tag: &'static str,
    unread: bool,
    featured: bool,
    style: Option<String>,
    title: String,
    children: AnyView,
) -> AnyView {
    view! {
        <article
            class=format!("ni {class} {} {}", if unread { "unread" } else { "" }, if featured { "featured" } else { "" })
            style=style
        >
            <time class="ni-time num" datetime=iso_string(at)>{clock(at)}</time>
            <div class="ni-main">
                <div class="ni-meta caps">
                    {featured.then(|| view! { <span class="ni-latest">"Latest"</span> })}
                    <span class="ni-tag">{tag}</span>
                    {unread.then(|| view! { <span class="ni-new">"New"</span> })}
                    <span class="ni-age">{move || fmt_ago(at, now.get())}</span>
                </div>
                <h3 class="ni-title">{title}</h3>
                {children}
            </div>
        </article>
    }
    .into_any()
}

pub fn dispatch_item(d: &Dispatch, now: Signal<f64>, unread: bool, featured: bool) -> AnyView {
    let kind = dispatch_kind(&d.message);
    let body = (!body_echoes_title(&d.message)).then(|| view! { <MarkupText message=d.message.clone() class="ni-body" /> });
    item(
        format!("ni-{}", kind.as_str()),
        parse_ms(&d.published),
        now,
        kind_label(kind),
        unread,
        featured,
        None,
        dispatch_title(&d.message),
        body.into_any(),
    )
}

fn entry_label(x: &ThreadEntry) -> String {
    let Some(ev) = &x.event else {
        return x.dispatch.as_ref().map(|d| dispatch_title(&d.message)).unwrap_or_default();
    };
    let e = &ev.campaign;
    match ev.kind {
        EventType::Launched => "Campaign launched".into(),
        EventType::Ended => format!("Campaign {}", if e.status == EpisodeStatus::Won { "won" } else { "failed" }),
        EventType::Order => match e.phases.get(ev.phase).and_then(phase_name) {
            Some(name) => format!("Major Order {}: {name}", ev.phase + 1),
            None => format!("Major Order {}", ev.phase + 1),
        },
    }
}

fn entry_body(x: &ThreadEntry) -> AnyView {
    if let Some(d) = &x.dispatch {
        return if body_echoes_title(&d.message) {
            ().into_any()
        } else {
            view! { <MarkupText message=d.message.clone() class="ni-body" /> }.into_any()
        };
    }
    let Some(ev) = &x.event else { return ().into_any() };
    let e = &ev.campaign;
    match ev.kind {
        EventType::Launched => {
            let briefing = e.phases.first().map(|p| p.briefing.clone()).filter(|b| !b.is_empty());
            view! {
                {(!e.description.is_empty()).then(|| view! { <p class="ni-body">{e.description.clone()}</p> })}
                {briefing.map(|b| view! { <MarkupText message=b class="ni-body" whole=true /> })}
            }
            .into_any()
        }
        EventType::Ended => {
            let tally = campaign_tally(e);
            let outro = e.phases.last().map(|p| p.outro.clone()).filter(|o| !o.is_empty());
            view! {
                <p class="ni-body">
                    {format!(
                        "{} of {} Major Orders won. Campaign reward, {}: {}.",
                        tally.won,
                        e.phases.len(),
                        reward_label(e.reward),
                        if e.status == EpisodeStatus::Won { "earned" } else { "lost" }
                    )}
                </p>
                {outro.map(|o| view! { <MarkupText message=o class="ni-body" whole=true /> })}
            }
            .into_any()
        }
        EventType::Order => match e.phases.get(ev.phase).map(|p| p.briefing.clone()).filter(|b| !b.is_empty()) {
            Some(b) => view! { <MarkupText message=b class="ni-body" whole=true /> }.into_any(),
            None => ().into_any(),
        },
    }
}

/// A Galactic Campaign as one story: its newest entry in full, earlier ones folded underneath.
pub fn campaign_item(thread: &CampaignThread, now: Signal<f64>, unread: bool, featured: bool) -> AnyView {
    let e = &thread.campaign;
    let Some(latest) = thread.entries.last() else { return ().into_any() };
    let earlier: Vec<&ThreadEntry> = thread.entries[..thread.entries.len() - 1].iter().rev().collect();
    let tally = campaign_tally(e);
    let n = earlier.len();
    let children = view! {
        <p class="ni-sub caps num">
            <span class=format!("ni-result {}", status_class(e.status))>{result_label(e.status)}</span>
            <span>
                {format!(
                    "{} won · {} lost{} · since {}",
                    tally.won,
                    tally.lost,
                    if tally.running > 0 { format!(" · {} running", tally.running) } else { String::new() },
                    sest_date(e.start)
                )}
            </span>
        </p>
        <h4 class="ni-entry caps">{entry_label(latest)}</h4>
        {entry_body(latest)}
        {(n > 0).then(|| view! {
            <details class="ni-thread">
                <summary class="caps">{format!("{n} earlier {}", if n == 1 { "entry" } else { "entries" })}</summary>
                <ol>
                    {earlier
                        .iter()
                        .map(|x| view! {
                            <li>
                                <details>
                                    <summary>
                                        <span class="num">{format!("{} · {}", clock(x.at), sest_date(x.at))}</span>
                                        <span>{entry_label(x)}</span>
                                    </summary>
                                    {entry_body(x)}
                                </details>
                            </li>
                        })
                        .collect_view()}
                </ol>
            </details>
        })}
    }
    .into_any();
    item(
        "ni-campaign".into(),
        thread.at,
        now,
        "Campaign",
        unread,
        featured,
        Some(format!("--tone: {};", faction_color(Some(e.faction)))),
        e.title.clone(),
        children,
    )
}
