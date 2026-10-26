use crate::api::queries::queries;
use crate::state::settings;
use leptos::prelude::*;
use std::sync::{Arc, OnceLock};
use warroom_core::api::Dispatch;
use warroom_core::campaign_news::{CampaignThread, campaign_threads};
use warroom_core::dispatch::{DispatchKind, dispatch_kind};
use warroom_core::news_feed::plain_text;
use warroom_core::snapshot::phase_name;
use warroom_core::time::parse_ms;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NewsKind {
    Dispatch(DispatchKind),
    Campaign,
}

impl NewsKind {
    pub fn as_str(self) -> &'static str {
        match self {
            NewsKind::Dispatch(k) => k.as_str(),
            NewsKind::Campaign => "campaign",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum NewsBody {
    Dispatch(Dispatch),
    Thread(CampaignThread),
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewsItem {
    pub kind: NewsKind,
    pub at: f64,
    pub key: String,
    pub text: String,
    pub body: NewsBody,
}

fn thread_text(t: &CampaignThread) -> String {
    let e = &t.campaign;
    let mut parts = vec![e.title.clone(), e.description.clone()];
    for p in &e.phases {
        parts.push(phase_name(p).unwrap_or("").to_string());
        parts.push(p.briefing.clone());
    }
    for x in &t.entries {
        parts.push(x.dispatch.as_ref().map(|d| plain_text(&d.message)).unwrap_or_default());
    }
    parts.join(" ")
}

static ITEMS: OnceLock<Memo<Arc<Vec<NewsItem>>>> = OnceLock::new();

/// Dispatches and Galactic Campaign threads, newest first.
pub fn news_items() -> Memo<Arc<Vec<NewsItem>>> {
    *ITEMS.get_or_init(|| {
        let q = queries();
        crate::state::global(|| {
            Memo::new(move |_| {
                let dispatches = q.dispatches.get();
                let episodes = q.episodes.get();
                let dispatches: &[Dispatch] = dispatches.as_deref().map_or(&[], Vec::as_slice);
                // Dispatches about a campaign live on its thread, so its orders and outcomes read as one story.
                let (threads, claimed) = campaign_threads(episodes.as_deref().map_or(&[], Vec::as_slice), dispatches);
                let mut items: Vec<NewsItem> = dispatches
                    .iter()
                    .filter(|d| !claimed.contains(&d.id))
                    .map(|d| NewsItem {
                        kind: NewsKind::Dispatch(dispatch_kind(&d.message)),
                        at: parse_ms(&d.published),
                        key: format!("d{}", d.id),
                        text: plain_text(&d.message),
                        body: NewsBody::Dispatch(d.clone()),
                    })
                    .collect();
                items.extend(threads.into_iter().map(|t| NewsItem {
                    kind: NewsKind::Campaign,
                    at: t.at,
                    key: t.key.clone(),
                    text: thread_text(&t),
                    body: NewsBody::Thread(t),
                }));
                items.sort_by(|a, b| b.at.total_cmp(&a.at));
                Arc::new(items)
            })
        })
    })
}

/// News items newer than the last visit to Strohmann News; none before the first visit.
pub fn unread_news() -> Signal<usize> {
    let items = news_items();
    Signal::derive(move || {
        let seen_at = settings().get(|s| s.news_seen_at);
        if seen_at == 0.0 { 0 } else { items.with(|list| list.iter().filter(|i| i.at > seen_at).count()) }
    })
}
