use crate::api::queries::queries;
use crate::features::news::card::MarkupText;
use crate::js::fmt_date;
use crate::ui::emblems::FactionEmblem;
use crate::ui::icons;
use crate::ui::primitives::{Empty, Panel, StatusChip};
use leptos::prelude::*;
use warroom_core::data::campaign_reward;
use warroom_core::factions::faction_color;
use warroom_core::snapshot::{EpisodeLive, EpisodeStatus, campaign_tally};

pub fn outcome(s: EpisodeStatus) -> (&'static str, &'static str) {
    match s {
        EpisodeStatus::Active => ("In progress", "var(--info)"),
        EpisodeStatus::Won => ("Victory", "var(--good)"),
        EpisodeStatus::Lost => ("Failed", "var(--bad)"),
    }
}

fn status_class(s: EpisodeStatus) -> &'static str {
    match s {
        EpisodeStatus::Active => "active",
        EpisodeStatus::Won => "won",
        EpisodeStatus::Lost => "lost",
    }
}

/// A Galactic Campaign runs three Major Orders; the game names each one only when it is issued.
const CAMPAIGN_ORDERS: usize = 3;

fn reward_rule(s: EpisodeStatus) -> &'static str {
    match s {
        EpisodeStatus::Active => "Earned by winning most of the campaign's Major Orders.",
        EpisodeStatus::Won => "Earned: most Major Orders were won.",
        EpisodeStatus::Lost => "Lost: most Major Orders failed.",
    }
}

#[component]
pub fn OutcomeChip(status: EpisodeStatus) -> impl IntoView {
    let (label, tone) = outcome(status);
    view! { <StatusChip tone=tone>{label}</StatusChip> }
}

pub fn campaign_dates(e: &EpisodeLive) -> String {
    match e.end {
        None => format!("Since {}", fmt_date(e.start)),
        Some(end) => format!("{} – {}", fmt_date(e.start), fmt_date(end)),
    }
}

/// Emblem, name, dates and outcome: the campaign's title row.
#[component]
pub fn CampaignHead(e: EpisodeLive) -> impl IntoView {
    view! {
        <div class="cmp-head" style=format!("--tone: {};", faction_color(Some(e.faction)))>
            <FactionEmblem faction=e.faction.as_str() size=22.0 />
            <span class="cmp-name">
                <b class="cmp-title">{e.title.clone()}</b>
                <span class="cmp-dates caps num">{campaign_dates(&e)}</span>
            </span>
            <OutcomeChip status=e.status />
        </div>
    }
}

/// Description, reward, tally and each Major Order with its briefing, as the game's Control Center lists them.
#[component]
pub fn CampaignBody(e: EpisodeLive) -> impl IntoView {
    let tally = campaign_tally(&e);
    let reward = e.reward.and_then(campaign_reward);
    let to_come = if e.status == EpisodeStatus::Active { CAMPAIGN_ORDERS.saturating_sub(e.phases.len()) } else { 0 };
    let n = e.phases.len();
    view! {
        <div class="cmp-body">
            {(!e.description.is_empty()).then(|| view! { <p class="cmp-desc">{e.description.clone()}</p> })}
            <div class=format!("cmp-reward {}", status_class(e.status))>
                <span class="cmp-label caps">"Campaign reward"</span>
                <b class="cmp-reward-name">{reward.as_ref().map_or("Not announced", |r| r.name.unwrap_or(r.kind))}</b>
                {reward.filter(|r| r.name.is_some()).map(|r| view! { <span class="cmp-reward-kind caps">{r.kind}</span> })}
                <span class="cmp-rule">{reward_rule(e.status)}</span>
            </div>
            <div class="cmp-tally caps num">
                "Major Orders "
                <b>{tally.won}</b>
                " won · "
                <b>{tally.lost}</b>
                " lost"
                {(tally.running > 0).then(|| view! { " · " <b>{tally.running}</b> " running" })}
                {(to_come > 0).then(|| view! { " · " <b>{to_come}</b> " to come" })}
            </div>
            <ol class="cmp-orders">
                {e
                    .phases
                    .iter()
                    .enumerate()
                    .map(|(i, p)| view! {
                        <li>
                            <details>
                                <summary>
                                    <span class="cmp-mo caps num">{format!("MO {}", i + 1)}</span>
                                    <b class="cmp-mo-title">{p.title.clone()}</b>
                                    {p.medals.map(|m| view! { <span class="cmp-medals caps num">{format!("{} medals", crate::js::num(m))}</span> })}
                                    <OutcomeChip status=p.status />
                                </summary>
                                {p.issued.map(|t| view! { <span class="cmp-issued caps num">{format!("Issued {}", fmt_date(t))}</span> })}
                                {(!p.briefing.is_empty()).then(|| view! { <MarkupText message=p.briefing.clone() class="cmp-text" whole=true /> })}
                                {(!p.outro.is_empty()).then(|| view! { <MarkupText message=p.outro.clone() class="cmp-text cmp-outro" whole=true /> })}
                            </details>
                        </li>
                    })
                    .collect_view()}
                {(0..to_come)
                    .map(|i| view! {
                        <li class="cmp-slot">
                            <span class="cmp-mo caps num">{format!("MO {}", n + i + 1)}</span>
                            <span class="cmp-slot-text caps">"Awaiting orders from High Command"</span>
                        </li>
                    })
                    .collect_view()}
            </ol>
        </div>
    }
}

/// The running Galactic Campaign; between campaigns, the last one and how it ended.
#[component]
pub fn CampaignPanel() -> impl IntoView {
    let q = queries();
    let latest = Memo::new(move |_| {
        q.episodes.with(|d| {
            d.as_ref().and_then(|d| {
                d.iter()
                    .fold(None::<&EpisodeLive>, |m, e| if m.is_none_or(|m| e.start > m.start) { Some(e) } else { m })
                    .cloned()
            })
        })
    });
    move || {
        if q.raw.loading.get() && q.episodes.with(Option::is_none) {
            return None;
        }
        let latest = latest.get();
        let title = if latest.as_ref().is_some_and(|e| e.status != EpisodeStatus::Active) {
            "Last campaign"
        } else {
            "Galactic campaign"
        };
        Some(view! {
            <Panel class="cmp-panel" icon=icons::FLAG title=title>
                {match latest {
                    Some(e) => view! {
                        <div class="cmp">
                            <CampaignHead e=e.clone() />
                            <CampaignBody e=e />
                        </div>
                    }
                    .into_any(),
                    None => view! { <Empty>"Campaign data arrives after the collector update."</Empty> }.into_any(),
                }}
            </Panel>
        })
    }
}
