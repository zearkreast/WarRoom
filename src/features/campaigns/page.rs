//! The Active planets page: every front, filterable by enemy; stalled fronts stay hidden unless Settings shows them.

use crate::api::fronts::{Front, FrontKind, fronts, major_order_deadlines};
use crate::api::queries::queries;
use crate::features::campaigns::card::CampaignCard;
use crate::features::campaigns::mo_strip::MoStrip;
use crate::js;
use crate::state::{FrontSort, OptionsAnchor, settings, ui};
use crate::ui::emblems::FactionEmblem;
use crate::ui::feed_status::FeedNote;
use crate::ui::hint::Hint;
use crate::ui::primitives::{Empty, Loading};
use leptos::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use warroom_core::api::Faction;
use warroom_core::biomes::regen_pct_per_hour;
use warroom_core::campaign_filter::{FrontActivity, MIN_PLAYERS, MIN_RATE, is_rare_campaign, is_stalled};
use warroom_core::factions::faction_label;
use warroom_core::format::{fmt_int, js_number, js_sum};
use warroom_core::most_needed::{FrontNeed, Need, front_need, typical_per_thousand};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Filter {
    All,
    Mo,
    Defense,
    Gambit,
    Siege,
    Special,
    Pinned,
    Enemy(Faction),
}

impl Filter {
    fn admits(self, f: &Front) -> bool {
        match self {
            Filter::All => true,
            Filter::Mo => f.in_major_order,
            Filter::Defense => f.kind == FrontKind::Defense,
            Filter::Gambit => f.gambit,
            Filter::Siege => f.siege,
            Filter::Special => is_rare_campaign(f.campaign.kind),
            Filter::Pinned => f.pinned,
            Filter::Enemy(e) => f.enemy == e,
        }
    }
}

const SORTS: [(FrontSort, &str); 4] = [
    (FrontSort::Priority, "Priority"),
    (FrontSort::Need, "Most needed"),
    (FrontSort::Divers, "Most Helldivers"),
    (FrontSort::Victory, "Soonest victory"),
];

fn need_input(f: &Front, mo_deadline: Option<f64>) -> FrontNeed {
    let defense = f.kind == FrontKind::Defense;
    FrontNeed {
        percent: f.percent,
        rate: f.rate,
        regen: if defense { 0.0 } else { regen_pct_per_hour(f.planet.regen_per_second, f.planet.max_health) },
        players: f.players,
        in_major_order: f.in_major_order,
        deadline: if defense { f.fail_at } else { mo_deadline },
    }
}

fn stalled(f: &Front) -> bool {
    is_stalled(&FrontActivity {
        players: f.players,
        rate: f.rate,
        in_major_order: f.in_major_order,
        pinned: f.pinned,
        strategic: f.kind == FrontKind::Defense || f.gambit || f.siege || is_rare_campaign(f.campaign.kind),
    })
}

#[component]
pub fn CampaignsPage() -> impl IntoView {
    let q = queries();
    let all = fronts();
    let deadlines = major_order_deadlines();
    let show_inactive = Memo::new(move |_| settings().get(|s| s.show_inactive));
    let filter = RwSignal::new(Filter::All);
    let sort = Memo::new(move |_| settings().get(|s| s.front_sort));
    let clock = js::use_now(60_000.0);
    let loading = Memo::new(move |_| q.raw.loading.get() && q.campaigns.with(Option::is_none));

    let visible = Memo::new(move |_| {
        let inactive = show_inactive.get();
        all.with(|l| l.iter().filter(|f| inactive || !stalled(f)).cloned().collect::<Vec<_>>())
    });
    let hidden = Memo::new(move |_| all.with(|l| l.len()) - visible.with(|v| v.len()));
    let needs = Memo::new(move |_| {
        let now = clock.get();
        let inputs: Vec<(i64, FrontNeed)> = visible.with(|v| {
            deadlines
                .with(|d| v.iter().map(|f| (f.campaign.id, need_input(f, d.get(&f.planet.index).copied()))).collect())
        });
        let typical = typical_per_thousand(&inputs.iter().map(|(_, n)| *n).collect::<Vec<_>>());
        Arc::new(inputs.into_iter().map(|(id, n)| (id, front_need(&n, now, typical))).collect::<HashMap<i64, Need>>())
    });
    let shown = Memo::new(move |_| {
        let f = filter.get();
        visible.with(|v| {
            let mut list: Vec<&Front> = v.iter().filter(|x| f.admits(x)).collect();
            match sort.get() {
                FrontSort::Priority => {}
                FrontSort::Need => needs.with(|n| {
                    let score = |f: &Front| n.get(&f.campaign.id).map_or(0.0, |n| n.score);
                    list.sort_by(|a, b| score(b).total_cmp(&score(a)).then(b.players.total_cmp(&a.players)));
                }),
                FrontSort::Divers => list.sort_by(|a, b| b.players.total_cmp(&a.players)),
                FrontSort::Victory => list.sort_by(|a, b| {
                    let at = |f: &Front| f.victory_at.filter(|v| v.is_finite() && *v > 0.0).unwrap_or(f64::INFINITY);
                    at(a).total_cmp(&at(b)).then(b.players.total_cmp(&a.players))
                }),
            }
            // Stable, so watched planets lead whichever order was picked.
            list.sort_by_key(|f| !f.pinned);
            list.iter().map(|x| x.campaign.id).collect::<Vec<_>>()
        })
    });
    let options = Memo::new(move |_| {
        let count = |f: Filter| visible.with(|v| v.iter().filter(|x| f.admits(x)).count());
        let mut list = vec![
            (Filter::All, "All".to_string(), count(Filter::All)),
            (Filter::Mo, "Major Order".to_string(), count(Filter::Mo)),
            (Filter::Defense, "Defense".to_string(), count(Filter::Defense)),
            (Filter::Gambit, "Gambit".to_string(), count(Filter::Gambit)),
            (Filter::Siege, "Circumvallation".to_string(), count(Filter::Siege)),
            (Filter::Special, "Special".to_string(), count(Filter::Special)),
        ];
        list.extend(
            Faction::ENEMIES.map(|e| (Filter::Enemy(e), faction_label(Some(e)).to_string(), count(Filter::Enemy(e)))),
        );
        list.push((Filter::Pinned, "Watchlist".to_string(), count(Filter::Pinned)));
        let current = filter.get();
        list.into_iter().filter(|(id, _, n)| *n > 0 || *id == Filter::All || *id == current).collect::<Vec<_>>()
    });
    let none = Memo::new(move |_| shown.with(Vec::is_empty));
    // Watched planets with no campaign have no card, so they get a row of their own.
    let idle = Memo::new(move |_| {
        let pins = settings().get(|s| s.pinned.clone());
        let active: HashSet<i64> = all.with(|l| l.iter().map(|f| f.planet.index).collect());
        q.fresh_planets.with(|ps| {
            ps.iter()
                .flat_map(|l| l.iter())
                .filter(|p| pins.contains(&p.index) && !active.contains(&p.index))
                .map(|p| (p.index, p.name.clone(), p.current_owner))
                .collect::<Vec<_>>()
        })
    });
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
                    "Click a card for its options: add it to your watchlist, show it on the map, or open the planet's full record."
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
                <div class="filter-row cc-sort" role="toolbar" aria-label="Sort active planets">
                    <span class="cc-sort-label caps">"Sort"</span>
                    {SORTS
                        .map(|(k, label)| view! {
                            <button
                                class=move || format!("filter-chip {}", if sort.get() == k { "on" } else { "" })
                                aria-pressed=move || (sort.get() == k).to_string()
                                on:click=move |_| settings().set(|s| s.front_sort = k)
                            >
                                {label}
                            </button>
                        })
                        .collect_view()}
                </div>
                <Show when=move || idle.with(|l| !l.is_empty())>
                    <div class="filter-row cc-sort" role="group" aria-label="Watched planets with no campaign">
                        <span class="cc-sort-label caps">"Watching, no campaign"</span>
                        <For each=move || idle.get() key=|p| p.0 let:p>
                            {
                                let (index, name, owner) = p;
                                view! {
                                    <button
                                        class="filter-chip"
                                        on:click=move |e| ui().open_options(Some(OptionsAnchor::at(index, &e)))
                                    >
                                        <FactionEmblem faction=owner.as_str() size=14.0 />
                                        {name}
                                    </button>
                                }
                            }
                        </For>
                    </div>
                </Show>
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
                                        reasons=Signal::derive(move || {
                                            if sort.get() != FrontSort::Need {
                                                return Vec::new();
                                            }
                                            needs.with(|n| n.get(&id).map(|n| n.reasons.clone()).unwrap_or_default())
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
