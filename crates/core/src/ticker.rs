use crate::api::{Dispatch, Faction};
use crate::campaign_news::{CampaignEvent, campaign_headline};
use crate::format::dispatch_title;
use crate::history::OwnerChange;
use crate::time::parse_ms;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TickerKind {
    Dispatch,
    Liberated,
    Lost,
    Campaign,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TickerItem {
    pub key: String,
    pub at: f64,
    pub kind: TickerKind,
    pub text: String,
    /// Faction whose emblem leads the item.
    pub faction: Option<Faction>,
    /// Planet the item opens on the map.
    pub planet: Option<i64>,
}

const WINDOW_MS: f64 = 24.0 * 3_600_000.0;
const MIN_ITEMS: usize = 6;
const MAX_ITEMS: usize = 14;

/// What the status strip scrolls: the last day's dispatches, captures, losses and campaign news,
/// newest first. A quiet day is topped up with older items so the strip never runs empty.
pub fn ticker_items(
    dispatches: &[Dispatch],
    changes: &[OwnerChange],
    campaign: &[CampaignEvent],
    now: f64,
) -> Vec<TickerItem> {
    let mut all: Vec<TickerItem> = dispatches
        .iter()
        .map(|d| TickerItem {
            key: format!("d{}", d.id),
            at: parse_ms(&d.published),
            kind: TickerKind::Dispatch,
            text: dispatch_title(&d.message),
            faction: None,
            planet: None,
        })
        .chain(changes.iter().map(|c| {
            let won = c.to == Faction::Humans;
            TickerItem {
                key: format!("c{}-{}", c.planet, crate::format::js_number(c.at)),
                at: c.at,
                kind: if won { TickerKind::Liberated } else { TickerKind::Lost },
                text: format!("{} {}", c.planet_name, if won { "liberated" } else { "lost" }),
                faction: Some(if won { Faction::Humans } else { c.to }),
                planet: Some(c.planet),
            }
        }))
        .chain(campaign.iter().map(|ev| TickerItem {
            key: ev.key.clone(),
            at: ev.at,
            kind: TickerKind::Campaign,
            text: campaign_headline(ev),
            faction: Some(ev.campaign.faction),
            planet: None,
        }))
        .collect();
    all.sort_by(|a, b| b.at.total_cmp(&a.at));
    let recent: Vec<TickerItem> = all.iter().filter(|i| now - i.at <= WINDOW_MS).cloned().collect();
    let mut out = if recent.len() >= MIN_ITEMS { recent } else { all.into_iter().take(MIN_ITEMS).collect() };
    out.truncate(MAX_ITEMS);
    out
}
