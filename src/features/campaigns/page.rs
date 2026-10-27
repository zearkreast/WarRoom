//! The Active planets page: every front, filterable by enemy; stalled fronts stay hidden unless Settings shows them.

use crate::api::fronts::{Front, FrontKind, fronts, major_order_deadlines};
use crate::api::queries::queries;
use crate::features::campaigns::card::CampaignCard;
use crate::features::campaigns::mo_strip::MoStrip;
use crate::state::settings;
use crate::ui::emblems::FactionEmblem;
use crate::ui::feed_status::FeedNote;
use crate::ui::hint::Hint;
use crate::ui::primitives::{Empty, Loading};
use leptos::prelude::*;
use warroom_core::api::Faction;
use warroom_core::campaign_filter::{FrontActivity, MIN_PLAYERS, MIN_RATE, is_stalled};
use warroom_core::factions::faction_label;
use warroom_core::format::{fmt_int, js_number, js_sum};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Filter {
    All,
    Mo,
    Defense,
    Pinned,
    Enemy(Faction),
}

impl Filter {
    fn admits(self, f: &Front) -> bool {
        match self {
            Filter::All => true,
            Filter::Mo => f.in_major_order,
            Filter::Defense => f.kind == FrontKind::Defense,
            Filter::Pinned => f.pinned,
            Filter::Enemy(e) => f.enemy == e,
        }
    }
}

fn stalled(f: &Front) -> bool {
    is_stalled(&FrontActivity { players: f.players, rate: f.rate, in_major_order: f.in_major_order, pinned: f.pinned })
}

#[component]
pub fn CampaignsPage() -> impl IntoView {
    let q = queries();
    let all = fronts();
    let deadlines = major_order_deadlines();
    let show_inactive = Memo::new(move |_| settings().get(|s| s.show_inactive));
    let filter = RwSignal::new(Filter::All);
    let loading = Memo::new(move |_| q.raw.loading.get() && q.campaigns.with(Option::is_none));

    let visible = Memo::new(move |_| {
        let inactive = show_inactive.get();
        all.with(|l| l.iter().filter(|f| inactive || !stalled(f)).cloned().collect::<Vec<_>>())
    });
    let hidden = Memo::new(move |_| all.with(|l| l.len()) - visible.with(|v| v.len()));
    let shown = Memo::new(move |_| {
        let f = filter.get();
        visible.with(|v| v.iter().filter(|x| f.admits(x)).map(|x| x.campaign.id).collect::<Vec<_>>())
    });
    let options = Memo::new(move |_| {
        let count = |f: Filter| visible.with(|v| v.iter().filter(|x| f.admits(x)).count());
        let mut list = vec![
            (Filter::All, "All".to_string(), count(Filter::All)),
            (Filter::Mo, "Major Order".to_string(), count(Filter::Mo)),
            (Filter::Defense, "Defense".to_string(), count(Filter::Defense)),
        ];
        list.extend(
            Faction::ENEMIES.map(|e| (Filter::Enemy(e), faction_label(Some(e)).to_string(), count(Filter::Enemy(e)))),
        );
        list.push((Filter::Pinned, "Pinned".to_string(), count(Filter::Pinned)));
        let current = filter.get();
        list.into_iter().filter(|(id, _, n)| *n > 0 || *id == Filter::All || *id == current).collect::<Vec<_>>()
    });
    let none = Memo::new(move |_| shown.with(Vec::is_empty));
    let total_players = Signal::derive(move || q.war.with(|w| w.as_ref().map_or(0.0, |w| w.statistics.player_count)));
    let fronts_n = move || all.with(|l| l.len());
    let divers = move || all.with(|l| fmt_int(js_sum(l.iter().map(|f| f.players))));
    let defenses = move || all.with(|l| l.iter().filter(|f| f.kind == FrontKind::Defense).count());

    view! {
        <div class="page scroll starfield">
            <div class="page-inner">
                <div class="page-head">
                    <h1 class="page-title">"Active planets"</h1>
                    <div class="page-stats">
                        <div class="page-stat">
                            <b>{fronts_n}</b>
                            <span>"Fronts"</span>
                        </div>
                        <div class="page-stat">
                            <b>{defenses}</b>
                            <span>"Defenses"</span>
                        </div>
                        <div class="page-stat">
                            <b>{divers}</b>
                            <span>"Helldivers deployed"</span>
                        </div>
                    </div>
                </div>
                <FeedNote />
                <MoStrip />
                <Hint id="campaigns">
                    "Click a card for its options: pin it to the top, show it on the map, or open the planet's full record."
                </Hint>
                <div class="filter-row" role="toolbar" aria-label="Filter active planets">
                    <For each=move || options.get() key=|o| o.0 let:o>
                        {
                            let (id, label, _) = o;
                            let n = move || options.with(|l| l.iter().find(|o| o.0 == id).map_or(0, |o| o.2));
                            view! {
                                <button
                                    class=move || format!("filter-chip {}", if filter.get() == id { "on" } else { "" })
                                    aria-pressed=move || (filter.get() == id).to_string()
                                    on:click=move |_| filter.set(id)
                                >
                                    {match id {
                                        Filter::Enemy(e) => Some(view! { <FactionEmblem faction=e.as_str() size=14.0 /> }),
                                        _ => None,
                                    }}
                                    {label}
                                    " "
                                    <span class="count">{n}</span>
                                </button>
                            }
                        }
                    </For>
                </div>
                {move || {
                    if loading.get() {
                        view! { <Loading what="active planets" rows=4 /> }.into_any()
                    } else if none.get() {
                        view! { <Empty>"No active planets match this filter."</Empty> }.into_any()
                    } else {
                        view! {
                            <div class="cc-grid">
                                <For each=move || shown.get() key=|id| *id let:id>
                                    <CampaignCard
                                        id=id
                                        total_players=total_players
                                        deadline=Signal::derive(move || {
                                            let index = all.with(|l| l.iter().find(|f| f.campaign.id == id).map(|f| f.planet.index))?;
                                            deadlines.with(|d| d.get(&index).copied())
                                        })
                                    />
                                </For>
                            </div>
                        }
                        .into_any()
                    }
                }}
                <Show when=move || { hidden.get() > 0 }>
                    <p class="cc-hidden caps">
                        {move || {
                            let n = hidden.get();
                            format!(
                                "{n} stalled campaign{} hidden (under {} divers, under +{}%/h) · ",
                                if n > 1 { "s" } else { "" },
                                fmt_int(MIN_PLAYERS),
                                js_number(MIN_RATE)
                            )
                        }}
                        <button class="link" on:click=move |_| settings().set(|s| s.show_inactive = true)>
                            "Show all"
                        </button>
                    </p>
                </Show>
            </div>
        </div>
    }
}
