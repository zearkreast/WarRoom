//! The Strohmann News page: the latest dispatch, then the feed by day with search.

use crate::alerts::{ToastKind, toasts};
use crate::api::queries::queries;
use crate::api::strohmann::{older_dispatches, search_dispatches};
use crate::features::news::card::kind_label;
use crate::features::news::feed::{campaign_item, dispatch_item};
use crate::features::news::items::{NewsBody, NewsItem, NewsKind, dispatch_news_item, news_items};
use crate::js;
use crate::state::settings;
use crate::ui::emblems::SuperEarthSeal;
use crate::ui::hint::Hint;
use crate::ui::icons::{self, Icon};
use crate::ui::primitives::{CtaButton, Empty, Loading};
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use warroom_core::api::Dispatch;
use warroom_core::format::{dispatch_title, fmt_ago};
use warroom_core::news_feed::{group_by_day, matches_query};
use warroom_core::time::parse_ms;

const PAGE: usize = 20;
/// Dispatches read from the archive at a time.
const ARCHIVE_PAGE: usize = 50;

/// The newest dispatch, to read again as the game shows it in case its popup was missed.
#[component]
fn LastMessage(d: Memo<Option<Dispatch>>, now: Signal<f64>) -> impl IntoView {
    move || {
        let d = d.get()?;
        let title = dispatch_title(&d.message);
        let published = parse_ms(&d.published);
        let (body, message) = (title.clone(), d.message.clone());
        Some(view! {
            <section class="news-last" aria-label="Last message from Super Earth">
                <span class="news-last-seal">
                    <SuperEarthSeal size=40.0 />
                </span>
                <div class="news-last-text">
                    <span class="caps">{move || format!("Last message from Super Earth · {}", fmt_ago(published, now.get()))}</span>
                    <b>{title}</b>
                </div>
                <CtaButton
                    icon=icons::MAIL_OPEN
                    on_click=Callback::new(move |_| {
                        toasts().show(ToastKind::Dispatch, "Message from Super Earth".into(), body.clone(), Some(message.clone()))
                    })
                >
                    "Read message"
                </CtaButton>
            </section>
        })
    }
}

fn feed_item(i: &NewsItem, now: Signal<f64>, unread: bool, featured: bool) -> AnyView {
    match &i.body {
        NewsBody::Dispatch(d) => dispatch_item(d, now, unread, featured),
        NewsBody::Thread(t) => campaign_item(t, now, unread, featured),
    }
}

#[component]
pub fn NewsPage() -> impl IntoView {
    let q = queries();
    let items = news_items();
    let now = Signal::from(js::use_now(60_000.0));
    let shown = RwSignal::new(PAGE);
    let filter = RwSignal::new(None::<NewsKind>);
    let query = RwSignal::new(String::new());
    // What counted as new when the page opened; it stays marked for this visit while the store moves on.
    let seen_at = settings().untracked(|s| s.news_seen_at);
    Effect::new(move |_| {
        if let Some(newest) = items.with(|l| l.first().map(|i| i.at)) {
            settings().mark_news_seen(newest);
        }
    });
    let loading = Memo::new(move |_| q.raw.loading.get() && q.dispatches.with(Option::is_none));
    // Dispatches read from the archive and those a search found there; `more_archived` turns false at its end.
    let archive = RwSignal::new(Vec::<Dispatch>::new());
    let found = RwSignal::new(Vec::<Dispatch>::new());
    let more_archived = RwSignal::new(true);
    let reading = RwSignal::new(false);
    let all = Memo::new(move |_| {
        let held: HashSet<i64> = q.dispatches.with(|d| d.iter().flat_map(|l| l.iter()).map(|d| d.id).collect());
        let mut extra: Vec<NewsItem> = Vec::new();
        let mut seen = held;
        for d in archive.get().iter().chain(found.get().iter()) {
            if seen.insert(d.id) {
                extra.push(dispatch_news_item(d));
            }
        }
        if extra.is_empty() {
            return items.get();
        }
        let mut list: Vec<NewsItem> = items.with(|l| l.to_vec());
        list.extend(extra);
        list.sort_by(|a, b| b.at.total_cmp(&a.at));
        Arc::new(list)
    });
    let searches = StoredValue::new(0_u64);
    Effect::new(move |_| {
        let words: Vec<String> = query.with(|q| q.split_whitespace().map(str::to_string).collect());
        let run = searches.get_value() + 1;
        searches.set_value(run);
        if words.iter().map(String::len).sum::<usize>() < 3 {
            found.set(Vec::new());
            return;
        }
        spawn_local(async move {
            // Waits for typing to pause before asking the archive.
            js::sleep(400.0).await;
            if searches.get_value() != run {
                return;
            }
            if let Ok(Some(list)) = search_dispatches(&words, 100).await
                && searches.get_value() == run
            {
                found.set(list);
            }
        });
    });
    let read_older = move || {
        let oldest = all.with(|l| {
            l.iter().filter_map(|i| if let NewsBody::Dispatch(d) = &i.body { Some(d.id) } else { None }).min()
        });
        let Some(oldest) = oldest else { return };
        reading.set(true);
        spawn_local(async move {
            match older_dispatches(oldest, ARCHIVE_PAGE).await {
                Ok(Some(list)) => {
                    more_archived.set(list.len() == ARCHIVE_PAGE);
                    archive.update(|a| a.extend(list));
                    shown.update(|n| *n += PAGE);
                }
                Ok(None) => more_archived.set(false),
                Err(_) => {}
            }
            reading.set(false);
        });
    };
    let kinds = Memo::new(move |_| {
        let mut out: Vec<(NewsKind, usize)> = Vec::new();
        all.with(|l| {
            for i in l.iter() {
                match out.iter_mut().find(|(k, _)| *k == i.kind) {
                    Some((_, n)) => *n += 1,
                    None => out.push((i.kind, 1)),
                }
            }
        });
        out
    });
    let list = Memo::new(move |_| {
        let f = filter.get();
        query.with(|q| {
            all.with(|l| {
                l.iter()
                    .filter(|i| f.is_none_or(|k| i.kind == k) && matches_query(&i.text, q))
                    .cloned()
                    .collect::<Vec<_>>()
            })
        })
    });
    let is_unread = move |i: &NewsItem| seen_at > 0.0 && i.at > seen_at;
    let unread = Memo::new(move |_| items.with(|l| l.iter().filter(|i| is_unread(i)).count()));
    let last = Memo::new(move |_| {
        q.dispatches.with(|d| {
            d.as_ref().and_then(|d| {
                d.iter()
                    .fold(None::<&Dispatch>, |a, d| if a.is_none_or(|a| d.id > a.id) { Some(d) } else { a })
                    .cloned()
            })
        })
    });
    let page = Memo::new(move |_| {
        let n = shown.get();
        list.with(|l| Arc::new(l.iter().take(n).cloned().collect::<Vec<_>>()))
    });
    let by_key = Memo::new(move |_| {
        page.with(|p| Arc::new(p.iter().map(|i| (i.key.clone(), i.clone())).collect::<HashMap<_, _>>()))
    });
    let featured = Memo::new(move |_| page.with(|p| p.first().map(|i| i.key.clone())));
    let days = Memo::new(move |_| {
        let now = now.get();
        page.with(|p| {
            let rest: Vec<(String, f64)> = p.iter().skip(1).map(|i| (i.key.clone(), i.at)).collect();
            group_by_day(rest, |x| x.1, now)
                .into_iter()
                .map(|g| (g.key.to_bits(), g.label, g.items.into_iter().map(|x| x.0).collect::<Vec<_>>()))
                .collect::<Vec<_>>()
        })
    });
    let render = move |key: String, featured: bool| {
        let item = Memo::new(move |_| by_key.with(|m| m.get(&key).cloned()));
        move || item.with(|i| i.as_ref().map(|i| feed_item(i, now, is_unread(i), featured)))
    };
    let kind_keys = move || kinds.get().into_iter().map(|(k, _)| k).collect::<Vec<_>>();
    let state = Memo::new(move |_| {
        if loading.get() {
            0
        } else if featured.with(Option::is_none) {
            1
        } else {
            2
        }
    });

    view! {
        <div class="page scroll starfield">
            <div class="page-inner news-page">
                <div class="page-head">
                    <div>
                        <h1 class="page-title">"Strohmann News"</h1>
                        <p class="page-sub">
                            "Dispatches and Galactic Campaign news from Super Earth, newest first. Planet names open their options."
                        </p>
                    </div>
                </div>
                <LastMessage d=last now=now />
                <Hint id="news">
                    "Planet names in a dispatch are links: click one to pin it, find it on the map or open its record."
                </Hint>
                <div class="news-tools">
                    <label class="news-search">
                        <Icon icon=icons::SEARCH size=15.0 />
                        <input
                            type="search"
                            placeholder="Search the news"
                            prop:value=move || query.get()
                            on:input=move |e| {
                                query.set(event_target_value(&e));
                                shown.set(PAGE);
                            }
                            aria-label="Search the news"
                        />
                        <Show when=move || query.with(|q| !q.is_empty())>
                            <button class="news-search-x" aria-label="Clear search" on:click=move |_| query.set(String::new())>
                                <Icon icon=icons::X size=15.0 />
                            </button>
                        </Show>
                    </label>
                    <div class="filter-row" role="toolbar" aria-label="Filter news">
                        <button
                            class=move || format!("filter-chip {}", if filter.get().is_none() { "on" } else { "" })
                            aria-pressed=move || filter.get().is_none().to_string()
                            on:click=move |_| filter.set(None)
                        >
                            "All "
                            <span class="count">{move || all.with(|l| l.len())}</span>
                        </button>
                        <For each=kind_keys key=|k| k.as_str() let:k>
                            <button
                                class=move || format!("filter-chip {}", if filter.get() == Some(k) { "on" } else { "" })
                                aria-pressed=move || (filter.get() == Some(k)).to_string()
                                on:click=move |_| {
                                    filter.set(Some(k));
                                    shown.set(PAGE);
                                }
                            >
                                {match k {
                                    NewsKind::Campaign => "Campaign",
                                    NewsKind::Dispatch(d) => kind_label(d),
                                }}
                                " "
                                <span class="count">{move || kinds.with(|l| l.iter().find(|(x, _)| *x == k).map_or(0, |x| x.1))}</span>
                            </button>
                        </For>
                    </div>
                </div>
                <Show when=move || { unread.get() > 0 }>
                    <p class="news-unread caps">{move || format!("{} new since your last visit", unread.get())}</p>
                </Show>
                {move || match state.get() {
                    0 => view! { <Loading what="dispatches" rows=4 /> }.into_any(),
                    1 => {
                        let any = items.with(|l| !l.is_empty());
                        view! { <Empty>{if any { "No news matches that search." } else { "No dispatches received yet." }}</Empty> }.into_any()
                    }
                    _ => view! {
                        {move || featured.get().map(|k| render(k, true))}
                        <For each=move || days.get() key=|g| (g.0, g.1.clone(), g.2.len(), g.2.first().cloned()) let:g>
                            <section class="news-day" aria-label=g.1.clone()>
                                <h2 class="news-day-head caps">
                                    {g.1.clone()}
                                    " "
                                    <span class="num">{g.2.len()}</span>
                                </h2>
                                <div class="news-rail">
                                    <For each=move || g.2.clone() key=|k| k.clone() let:k>
                                        {render(k, false)}
                                    </For>
                                </div>
                            </section>
                        </For>
                    }
                    .into_any(),
                }}
                <Show when=move || { shown.get() < list.with(|l| l.len()) || (more_archived.get() && query.with(String::is_empty)) }>
                    <button
                        class="hud-btn news-more"
                        disabled=move || reading.get()
                        on:click=move |_| {
                            if shown.get_untracked() < list.with_untracked(|l| l.len()) {
                                shown.update(|n| *n += PAGE);
                            } else {
                                read_older();
                            }
                        }
                    >
                        "Load older news"
                    </button>
                </Show>
            </div>
        </div>
    }
}
