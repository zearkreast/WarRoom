use crate::api::queries::queries;
use crate::features::galactic_campaigns::{CampaignBody, CampaignHead, outcome};
use crate::js::{self, fmt_date, num};
use crate::ui::emblems::FactionEmblem;
use crate::ui::icons::{self, Icon};
use crate::ui::primitives::{Empty, Loading, Panel, Tile};
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use warroom_core::data::reward_label;
use warroom_core::factions::faction_color;
use warroom_core::format::js_round;
use warroom_core::snapshot::{EpisodeLive, EpisodeStatus, campaign_record, campaign_tally};

const DAY_MS: f64 = 86_400_000.0;

fn days(e: &EpisodeLive) -> Option<f64> {
    e.end.map(|end| js_round((end - e.start) / DAY_MS).max(1.0))
}

fn status_class(s: EpisodeStatus) -> &'static str {
    match s {
        EpisodeStatus::Active => "active",
        EpisodeStatus::Won => "won",
        EpisodeStatus::Lost => "lost",
    }
}

fn pick(id: i64) {
    let Some(card) = js::document().query_selector(&format!("details.pc[data-campaign=\"{id}\"]")).ok().flatten() else { return };
    let Ok(card) = card.dyn_into::<web_sys::HtmlElement>() else { return };
    let _ = card.set_attribute("open", "");
    if let Some(summary) = card.query_selector("summary").ok().flatten().and_then(|s| s.dyn_into::<web_sys::HtmlElement>().ok()) {
        let opts = web_sys::FocusOptions::new();
        opts.set_prevent_scroll(true);
        let _ = summary.focus_with_options(&opts);
    }
    let opts = web_sys::ScrollIntoViewOptions::new();
    opts.set_behavior(if js::prefers_reduced_motion() { web_sys::ScrollBehavior::Auto } else { web_sys::ScrollBehavior::Smooth });
    opts.set_block(web_sys::ScrollLogicalPosition::Start);
    card.scroll_into_view_with_scroll_into_view_options(&opts);
    // Restarts the highlight when the same block is picked twice.
    let _ = card.remove_attribute("data-flash");
    let _ = card.offset_width();
    let _ = card.set_attribute("data-flash", "");
}

/// Every campaign oldest to newest, one block each in its enemy's colour; a block opens its campaign below.
fn record_strip(past: &[EpisodeLive]) -> AnyView {
    let oldest_first: Vec<&EpisodeLive> = past.iter().rev().collect();
    let (Some(first), Some(last)) = (oldest_first.first(), past.first()) else { return ().into_any() };
    view! {
        <div class="pc-record">
            <div class="pc-record-head caps">
                <span>"Campaign record"</span>
                <span class="pc-legend" aria-hidden="true">
                    <i class="won"></i>
                    " Victory "
                    <i class="lost"></i>
                    " Failed"
                </span>
            </div>
            <ol class="pc-strip">
                {oldest_first
                    .iter()
                    .enumerate()
                    .map(|(i, e)| {
                        let id = e.id;
                        view! {
                            <li style=format!("--tone: {}; --i: {i};", faction_color(Some(e.faction)))>
                                <button
                                    class=format!("pc-seg {}", status_class(e.status))
                                    title=e.title.clone()
                                    aria-label=format!("{}: {}", e.title, outcome(e.status).0)
                                    on:click=move |_| pick(id)
                                >
                                    <FactionEmblem faction=e.faction.as_str() size=14.0 />
                                </button>
                            </li>
                        }
                    })
                    .collect_view()}
            </ol>
            <div class="pc-record-ends caps num">
                <span>{fmt_date(first.start)}</span>
                <span>{fmt_date(last.end.unwrap_or(last.start))}</span>
            </div>
        </div>
    }
    .into_any()
}

/// Every Galactic Campaign that has ended, newest first; one moves here as soon as the game closes it.
#[component]
pub fn PastCampaignsModule() -> impl IntoView {
    let q = queries();
    let past = Memo::new(move |_| {
        let mut list: Vec<EpisodeLive> = q.episodes.with(|d| d.iter().flat_map(|d| d.iter()).filter(|e| e.status != EpisodeStatus::Active).cloned().collect());
        list.sort_by(|a, b| b.start.total_cmp(&a.start));
        list
    });
    let has_data = Memo::new(move |_| q.episodes.with(Option::is_some));
    let waiting = Memo::new(move |_| q.raw.loading.get() && !has_data.get());
    let right = move || {
        let p = past.get();
        (!p.is_empty()).then(|| {
            let won = campaign_record(&p).won;
            view! {
                <span class="num">
                    <span style="color: var(--good);">{format!("{won} won")}</span>
                    " · "
                    <span style="color: var(--bad);">{format!("{} failed", p.len() - won)}</span>
                </span>
            }
        })
    };
    view! {
        <Panel title="Past campaigns" icon=icons::ARCHIVE right=view! { {right} }.into_any()>
            {move || {
                if waiting.get() {
                    return view! { <Loading what="the campaign archive" rows=3 /> }.into_any();
                }
                let list = past.get();
                if list.is_empty() {
                    let text = if has_data.get() { "No campaign has ended yet." } else { "Campaign data arrives after the collector update." };
                    return view! { <Empty>{text}</Empty> }.into_any();
                }
                let record = campaign_record(&list);
                let at_war: f64 = list.iter().map(|e| days(e).unwrap_or(0.0)).sum();
                let n = list.len() as f64;
                let orders_sub = if record.orders > 0 {
                    format!("{}% of orders", num(js_round(record.orders_won as f64 / record.orders as f64 * 100.0))).into_any()
                } else {
                    ().into_any()
                };
                view! {
                    <div class="pc-archive">
                        <div class="pc-stats">
                            <Tile icon=icons::ARCHIVE label="Campaigns" value=list.len().into_any() sub=format!("{} campaign days", num(at_war)).into_any() rule="var(--cta)" />
                            <Tile
                                icon=icons::TROPHY
                                label="Victories"
                                value=record.won.into_any()
                                sub=format!("{}% win rate", num(js_round(record.won as f64 / n * 100.0))).into_any()
                                color="var(--good)"
                                rule="var(--good)"
                            />
                            <Tile
                                icon=icons::FLAG
                                label="Major Orders won"
                                value=format!("{} / {}", record.orders_won, record.orders).into_any()
                                sub=orders_sub
                                rule="var(--info)"
                            />
                            <Tile
                                icon=icons::FLAME
                                label="Win streak"
                                value=record.streak.into_any()
                                sub=(if record.streak == 1 { "Latest campaign" } else { "Campaigns in a row" }).into_any()
                                color=(record.streak > 0).then(|| "var(--cta)".to_string())
                                rule="var(--cta-deep)"
                            />
                        </div>
                        {record_strip(&list)}
                        <ol class="pc-list">
                            {list
                                .iter()
                                .enumerate()
                                .map(|(i, e)| {
                                    let t = campaign_tally(e);
                                    let d = days(e);
                                    view! {
                                        <li style=format!("--i: {i};")>
                                            <details
                                                class=format!("pc {}", status_class(e.status))
                                                style=format!("--tone: {};", faction_color(Some(e.faction)))
                                                data-campaign=e.id
                                                on:animationend=|ev| {
                                                    if ev.target() == ev.current_target()
                                                        && let Some(el) = ev.current_target().and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                                                    {
                                                        let _ = el.remove_attribute("data-flash");
                                                    }
                                                }
                                            >
                                                <summary>
                                                    <span class="pc-mark" aria-hidden="true">
                                                        <FactionEmblem faction=e.faction.as_str() size=132.0 />
                                                    </span>
                                                    <CampaignHead e=e.clone() />
                                                    <span class="pc-meta caps num">
                                                        <span class="pc-orders">
                                                            <span class="pc-pips" aria-hidden="true">
                                                                {e.phases.iter().map(|p| view! { <i class=status_class(p.status)></i> }).collect_view()}
                                                            </span>
                                                            {format!("{} of {} Major Orders won", t.won, e.phases.len())}
                                                        </span>
                                                        {d.map(|d| view! { <span>{if d == 1.0 { "1 day".to_string() } else { format!("{} days", num(d)) }}</span> })}
                                                        <span class="pc-reward">
                                                            <Icon icon=icons::GIFT size=12.0 />
                                                            <span class="pc-reward-name">{reward_label(e.reward)}</span>
                                                            {(e.status == EpisodeStatus::Lost).then(|| view! { <span class="sr-only">" (not earned)"</span> })}
                                                        </span>
                                                    </span>
                                                    <Icon icon=icons::CHEVRON_RIGHT size=16.0 class="pc-chevron" />
                                                </summary>
                                                <CampaignBody e=e.clone() />
                                            </details>
                                        </li>
                                    }
                                })
                                .collect_view()}
                        </ol>
                    </div>
                }
                .into_any()
            }}
        </Panel>
    }
}
