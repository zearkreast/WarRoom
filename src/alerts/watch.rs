//! Watches the shared data for anything worth an alert. Started once at the app root.

use crate::alerts::{ToastKind, notify, request_notifications};
use crate::api::fronts::{Front, fronts};
use crate::api::history::history;
use crate::api::queries::queries;
use crate::config::{NEWS_PUSHED, POLL_MS};
use crate::features::major_order::outlooks::outlooks;
use crate::js::{self, get};
use crate::state::settings;
use crate::tauri;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::HashMap;
use std::sync::Arc;
use warroom_core::api::{Faction, Planet};
use warroom_core::campaign_news::{EventType, campaign_alerts, campaign_headline};
use warroom_core::cities::{CityState, cities_of};
use warroom_core::factions::faction_label;
use warroom_core::format::{dispatch_title, fmt_duration, fmt_int, to_fixed};
use warroom_core::major_order::{DeadlineOutlook, describe_assignment, order_verdict};
use warroom_core::owner_changes::detect_new_defenses;
use warroom_core::time::parse_ms;
use warroom_core::watchlist::{FrontAlert, FrontLook, OrderAlert, OrderLook, front_alerts, order_alerts};

fn front_look(f: &Front, at: f64) -> FrontLook {
    let cities = cities_of(&f.planet);
    FrontLook {
        at,
        rate: f.rate,
        victory_at: f.victory_at.filter(|v| v.is_finite()),
        waiting: cities.iter().filter(|c| c.waiting).map(|c| c.id).collect(),
        open: cities.iter().filter(|c| c.state == CityState::Liberating).map(|c| (c.id, c.name.clone())).collect(),
    }
}

fn front_alert(f: &Front, alert: FrontAlert, at: f64) -> (ToastKind, String, String) {
    let name = &f.planet.name;
    match alert {
        FrontAlert::VictorySoon(v) => (
            ToastKind::Capture,
            format!("{name}: victory within the hour"),
            format!("At today's pace {name} is liberated in {}.", fmt_duration(v - at)),
        ),
        FrontAlert::LosingGround(r) => (
            ToastKind::Defense,
            format!("{name} is losing ground"),
            format!("{}% an hour with {} Helldivers deployed.", to_fixed(r, 2), fmt_int(f.players)),
        ),
        FrontAlert::CityOpens(city) => (
            ToastKind::Campaign,
            format!("{city} opens on {name}"),
            format!("Helldivers can now fight to free {city}."),
        ),
    }
}

pub fn start() {
    // Not on the web, where browsers block or bury a permission prompt nobody clicked for.
    if tauri::is_tauri() && settings().untracked(|s| s.alerts.system) {
        request_notifications();
    }

    // Captures and losses, from the collector's ownership log.
    spawn_local(async {
        loop {
            let fresh = history().refresh().await.unwrap_or_default();
            let alerts = settings().untracked(|s| s.alerts.clone());
            if alerts.captures {
                for c in fresh.iter().rev().map(|r| &r.change) {
                    let won = c.to == Faction::Humans;
                    let (title, body) = if won {
                        (
                            format!("{} liberated", c.planet_name),
                            format!("Super Earth took {} from the {}.", c.planet_name, faction_label(Some(c.from))),
                        )
                    } else {
                        (
                            format!("{} lost", c.planet_name),
                            format!("The {} took {}.", faction_label(Some(c.to)), c.planet_name),
                        )
                    };
                    notify(if won { ToastKind::Capture } else { ToastKind::Loss }, &title, &body, alerts.system, None);
                }
            }
            js::sleep(POLL_MS).await;
        }
    });

    let q = queries();

    // New defense campaigns.
    let mut last_defenses: Option<Vec<Planet>> = None;
    Effect::new(move |_| {
        let Some(campaigns) = q.campaigns.get() else { return };
        let list: Vec<Planet> = campaigns.iter().map(|c| c.planet.clone()).collect();
        let fresh: Vec<Planet> = detect_new_defenses(last_defenses.as_deref(), &list).into_iter().cloned().collect();
        last_defenses = Some(list);
        let alerts = settings().untracked(|s| s.alerts.clone());
        if !alerts.defenses {
            return;
        }
        for p in fresh {
            let enemy = faction_label(p.event.as_ref().map(|e| e.faction));
            notify(
                ToastKind::Defense,
                &format!("Defend {}", p.name),
                &format!("The {enemy} are attacking {}.", p.name),
                alerts.system,
                None,
            );
        }
    });

    // New Major Orders and dispatches. The first load only seeds what's been seen.
    // Strohmann News alerts: keep their wording in step with the collector's pushes (collector/src/tasks.rs).
    Effect::new(move |_| {
        let (Some(assignments), Some(dispatches)) = (q.assignments.get(), q.dispatches.get()) else { return };
        let s = settings().untracked(|s| s.clone());
        let system = s.alerts.system && !NEWS_PUSHED;
        let first_run = s.seen_assignments.is_empty() && s.seen_dispatch == 0;
        let latest_dispatch = dispatches.iter().map(|d| d.id).fold(0, i64::max);
        if !first_run {
            if s.alerts.orders {
                for a in assignments.iter().filter(|a| !s.seen_assignments.contains(&a.id)) {
                    let briefing = a
                        .briefing
                        .clone()
                        .or_else(|| a.title.clone())
                        .unwrap_or_else(|| "High Command has issued new orders.".into());
                    notify(ToastKind::Order, "New Major Order", &briefing, system, Some(briefing.clone()));
                }
            }
            if s.alerts.dispatches {
                let fresh: Vec<_> = dispatches.iter().filter(|d| d.id > s.seen_dispatch).take(3).collect();
                for d in fresh.into_iter().rev() {
                    notify(
                        ToastKind::Dispatch,
                        "New dispatch",
                        &dispatch_title(&d.message),
                        system,
                        Some(d.message.clone()),
                    );
                }
            }
        }
        let mut seen = s.seen_assignments.clone();
        for a in assignments.iter() {
            if !seen.contains(&a.id) {
                seen.push(a.id);
            }
        }
        let keep = seen.len().saturating_sub(50);
        settings().mark_seen(seen.split_off(keep), s.seen_dispatch.max(latest_dispatch));
    });

    // Galactic Campaign launches and outcomes, as Strohmann News tells them.
    Effect::new(move |_| {
        let Some(episodes): Option<Arc<_>> = q.episodes.get() else { return };
        let s = settings().untracked(|s| s.clone());
        let (fresh, keys) = campaign_alerts(&episodes, &s.seen_campaign_events);
        if s.alerts.orders {
            for ev in &fresh {
                let launched = ev.kind == EventType::Launched;
                notify(
                    ToastKind::Campaign,
                    if launched { "New Galactic Campaign" } else { "Galactic Campaign over" },
                    &campaign_headline(ev),
                    s.alerts.system && !NEWS_PUSHED,
                    launched.then(|| ev.campaign.description.clone()),
                );
            }
        }
        settings().mark_campaign_seen(keys);
    });

    // Pinned fronts, each look against the last, so nothing fires for what was already so at startup.
    let all = fronts();
    let mut seen_fronts: HashMap<i64, FrontLook> = HashMap::new();
    Effect::new(move |_| {
        let at = q.last_update.get().unwrap_or_else(js::now);
        let pinned = settings().get(|s| s.pinned.clone());
        let looks: HashMap<i64, FrontLook> = all.with(|l| {
            l.iter().filter(|f| pinned.contains(&f.planet.index)).map(|f| (f.planet.index, front_look(f, at))).collect()
        });
        let alerts = settings().untracked(|s| s.alerts.clone());
        if alerts.pinned {
            all.with_untracked(|l| {
                for f in l.iter() {
                    let (Some(prev), Some(next)) = (seen_fronts.get(&f.planet.index), looks.get(&f.planet.index))
                    else {
                        continue;
                    };
                    for alert in front_alerts(prev, next) {
                        let (kind, title, body) = front_alert(f, alert, at);
                        notify(kind, &title, &body, alerts.system, None);
                    }
                }
            });
        }
        seen_fronts = looks;
    });

    // The Major Order's progress, compared the same way.
    let clock = js::use_now(60_000.0);
    let mut seen_orders: HashMap<i64, OrderLook> = HashMap::new();
    Effect::new(move |_| {
        let at = clock.get();
        let Some(assignments) = q.assignments.get() else { return };
        let planets = q.fresh_planets.get();
        let alerts = settings().untracked(|s| s.alerts.clone());
        let mut looks = HashMap::new();
        for a in assignments.iter() {
            let tasks = describe_assignment(a, planets.as_deref().map(|p| p.as_slice()));
            let outlook = outlooks(a, &tasks, at);
            let next = OrderLook {
                at,
                ends: parse_ms(&a.expiration),
                losing: outlook.iter().any(|o| matches!(o.outlook, DeadlineOutlook::Behind | DeadlineOutlook::NoFront)),
                done: tasks.iter().map(|t| t.done).collect(),
            };
            if alerts.progress
                && let Some(prev) = seen_orders.get(&a.id)
            {
                for alert in order_alerts(prev, &next) {
                    let (title, body) = match alert {
                        OrderAlert::Losing => (
                            "Major Order now losing".to_string(),
                            order_verdict(&tasks, &outlook).map_or(String::new(), |(text, _)| text),
                        ),
                        OrderAlert::EndsIn(h) => (
                            format!("Major Order ends in {h} hour{}", if h == 1 { "" } else { "s" }),
                            format!(
                                "{} of {} objectives complete.",
                                next.done.iter().filter(|d| **d).count(),
                                tasks.len()
                            ),
                        ),
                        OrderAlert::ObjectiveDone(i) => (
                            "Objective complete".to_string(),
                            tasks.get(i).map_or(String::new(), |t| {
                                format!("{} is done.", warroom_core::major_order::task_name(t))
                            }),
                        ),
                    };
                    notify(ToastKind::Order, &title, &body, alerts.system, None);
                }
            }
            looks.insert(a.id, next);
        }
        seen_orders = looks;
    });

    // Android shows the collector's pushes itself, so these settings switch which topics it gets.
    if NEWS_PUSHED {
        let topics = Memo::new(move |_| {
            settings().get(|s| (s.alerts.system && s.alerts.orders, s.alerts.system && s.alerts.dispatches))
        });
        Effect::new(move |_| {
            let (orders, dispatches) = topics.get();
            let _ = js::call(&get(&js::window(), "WarRoomPush"), "setTopics", &[orders.into(), dispatches.into()]);
        });
    }
}
