//! Galactic Campaign news: launches, Major Orders and outcomes as events, threads with their dispatches, and alerts.

use crate::api::Dispatch;
use crate::format::dispatch_title;
use crate::snapshot::{EpisodeLive, EpisodeStatus, issued_at, phase_name};
use crate::time::parse_ms;
use regex_lite::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventType {
    Launched,
    Order,
    Ended,
}

/// A moment in a Galactic Campaign worth a news item: its launch, a new Major Order, its end.
#[derive(Clone, Debug, PartialEq)]
pub struct CampaignEvent {
    pub key: String,
    pub at: f64,
    pub kind: EventType,
    pub campaign: EpisodeLive,
    /// Index of the Major Order a "launched" or "order" event announces.
    pub phase: usize,
}

/// One line of a campaign's story: one of its events, a dispatch the game sent about it, or both when
/// the dispatch is the announcement of that event.
#[derive(Clone, Debug, PartialEq)]
pub struct ThreadEntry {
    pub at: f64,
    pub key: String,
    pub event: Option<CampaignEvent>,
    pub dispatch: Option<Dispatch>,
}

/// Everything about one campaign, oldest first; it sorts into the news by its latest entry.
#[derive(Clone, Debug, PartialEq)]
pub struct CampaignThread {
    pub key: String,
    pub at: f64,
    pub campaign: EpisodeLive,
    pub entries: Vec<ThreadEntry>,
}

const HOUR: f64 = 3_600_000.0;
/// Headlines the game uses for its Major Orders, which since campaigns began are always campaign phases.
static ORDER_HEADLINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)major order|control center|campaign").unwrap());
static NEW_ORDER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)new major order").unwrap());

fn published(d: &Dispatch) -> f64 {
    parse_ms(&d.published)
}

/// Whole words only: order names are plain words ("DEPLOYMENT"), and "deployments" isn't a mention.
fn mentions(text: &str, name: &str) -> bool {
    let name = name.to_uppercase();
    let boundary = |c: Option<char>| c.is_none_or(|c| !(c.is_ascii_uppercase() || c.is_ascii_digit()));
    text.match_indices(&name)
        .any(|(i, m)| boundary(text[..i].chars().next_back()) && boundary(text[i + m.len()..].chars().next()))
}

/// The campaign a dispatch belongs to: one running when it went out that it names, else the one running
/// when an order headline went out. Names alone aren't enough: campaigns reuse them (two have a
/// "CONTAINMENT"), and an unnamed order's "MAJOR ORDER" is in every order headline.
pub fn campaign_of<'a>(d: &Dispatch, list: &'a [EpisodeLive]) -> Option<&'a EpisodeLive> {
    let text = d.message.to_uppercase();
    let at = published(d);
    let mut running: Vec<&EpisodeLive> = list
        .iter()
        .filter(|e| at >= e.start - 2.0 * HOUR && at <= e.end.unwrap_or(f64::INFINITY) + 12.0 * HOUR)
        .collect();
    let named = running.iter().find(|e| {
        mentions(&text, &e.title) || e.phases.iter().any(|p| phase_name(p).is_some_and(|n| mentions(&text, n)))
    });
    if let Some(e) = named {
        return Some(e);
    }
    if !ORDER_HEADLINE.is_match(&dispatch_title(&d.message)) {
        return None;
    }
    running.sort_by(|a, b| b.start.total_cmp(&a.start));
    running.first().copied()
}

/// The campaign's events, each with the dispatch that announced it when there is one. "New Major Order"
/// dispatches announce the orders in turn, and date an order the collector didn't see go out, so orders
/// that predate the collector still get a date. The dispatch list keeps only the newest, so once the
/// launch announcement has dropped out of it the ones left pair with the last orders.
fn events_of(e: &EpisodeLive, own: &[&Dispatch]) -> Vec<(CampaignEvent, Option<Dispatch>)> {
    let announced: Vec<&&Dispatch> = own.iter().filter(|d| NEW_ORDER.is_match(&dispatch_title(&d.message))).collect();
    let launch_told = announced.first().is_some_and(|d| published(d) <= e.start + 2.0 * HOUR);
    let offset = if launch_told { 0 } else { e.phases.len().saturating_sub(announced.len()) };
    let mut out = Vec::new();
    for (i, p) in e.phases.iter().enumerate() {
        let dispatch = i.checked_sub(offset).and_then(|k| announced.get(k)).map(|d| (**d).clone());
        let at = if i == 0 {
            Some(e.start)
        } else {
            issued_at(e.start, e.end, p.issued).or_else(|| dispatch.as_ref().map(published))
        };
        let Some(at) = at else { continue };
        let kind = if i == 0 { EventType::Launched } else { EventType::Order };
        let label = if i == 0 { "launched" } else { "order" };
        out.push((
            CampaignEvent { key: format!("{}-{label}-{i}", e.id), at, kind, campaign: e.clone(), phase: i },
            dispatch,
        ));
    }
    if let (true, Some(end)) = (e.status != EpisodeStatus::Active, e.end) {
        out.push((
            CampaignEvent {
                key: format!("{}-ended", e.id),
                at: end,
                kind: EventType::Ended,
                campaign: e.clone(),
                phase: e.phases.len().saturating_sub(1),
            },
            None,
        ));
    }
    out
}

/// Campaign threads, newest activity first, and the ids of the dispatches they took in.
pub fn campaign_threads(list: &[EpisodeLive], dispatches: &[Dispatch]) -> (Vec<CampaignThread>, HashSet<i64>) {
    let mut own: HashMap<i64, Vec<&Dispatch>> = HashMap::new();
    let mut claimed = HashSet::new();
    for d in dispatches {
        let Some(e) = campaign_of(d, list) else { continue };
        own.entry(e.id).or_default().push(d);
        claimed.insert(d.id);
    }
    let mut threads: Vec<CampaignThread> = list
        .iter()
        .map(|e| {
            let mut mine = own.get(&e.id).cloned().unwrap_or_default();
            mine.sort_by(|a, b| published(a).total_cmp(&published(b)));
            let events = events_of(e, &mine);
            let told: HashSet<i64> = events.iter().filter_map(|(_, d)| d.as_ref().map(|d| d.id)).collect();
            let mut entries: Vec<ThreadEntry> = events
                .into_iter()
                .map(|(ev, dispatch)| ThreadEntry { at: ev.at, key: ev.key.clone(), event: Some(ev), dispatch })
                .chain(mine.iter().filter(|d| !told.contains(&d.id)).map(|d| ThreadEntry {
                    at: published(d),
                    key: format!("d{}", d.id),
                    event: None,
                    dispatch: Some((*d).clone()),
                }))
                .collect();
            entries.sort_by(|a, b| a.at.total_cmp(&b.at));
            let at = entries.last().map_or(f64::NAN, |x| x.at);
            CampaignThread { key: format!("c{}", e.id), at, campaign: e.clone(), entries }
        })
        .collect();
    threads.sort_by(|a, b| b.at.total_cmp(&a.at));
    (threads, claimed)
}

/// Every dated campaign event, newest first. Orders with no known or announced date are left out.
pub fn campaign_events(list: &[EpisodeLive], dispatches: &[Dispatch]) -> Vec<CampaignEvent> {
    let (threads, _) = campaign_threads(list, dispatches);
    let mut out: Vec<CampaignEvent> =
        threads.into_iter().flat_map(|t| t.entries.into_iter().filter_map(|x| x.event)).collect();
    out.sort_by(|a, b| b.at.total_cmp(&a.at));
    out
}

/// Campaign launches and outcomes not alerted on yet, oldest first, and the keys to remember as seen. New
/// orders are left to the Major Order alert, which comes sooner. With nothing seen yet, nothing is fresh.
pub fn campaign_alerts(list: &[EpisodeLive], seen: &[String]) -> (Vec<CampaignEvent>, Vec<String>) {
    let events: Vec<CampaignEvent> =
        campaign_events(list, &[]).into_iter().filter(|ev| ev.kind != EventType::Order).collect();
    let keys = events.iter().map(|ev| ev.key.clone()).collect();
    let fresh = if seen.is_empty() {
        Vec::new()
    } else {
        events.into_iter().filter(|ev| !seen.contains(&ev.key)).rev().collect()
    };
    (fresh, keys)
}

/// One-line headline for a campaign event, in the game's words.
pub fn campaign_headline(ev: &CampaignEvent) -> String {
    let e = &ev.campaign;
    match ev.kind {
        EventType::Launched => format!("New campaign: {}", e.title),
        EventType::Order => {
            let name = e.phases.get(ev.phase).and_then(phase_name);
            format!("{}: Major Order {}{}", e.title, ev.phase + 1, name.map(|n| format!(", {n}")).unwrap_or_default())
        }
        EventType::Ended => format!("{} {}", e.title, if e.status == EpisodeStatus::Won { "won" } else { "failed" }),
    }
}
