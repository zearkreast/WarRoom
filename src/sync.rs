//! Watches the shared data for anything worth an alert. Started once at the app root.

use crate::alerts::{ToastKind, notify, request_notifications};
use crate::api::queries::queries;
use crate::config::{NEWS_PUSHED, POLL_MS};
use crate::history::history;
use crate::js;
use crate::state::settings;
use crate::tauri;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::sync::Arc;
use warroom_core::api::{Faction, Planet};
use warroom_core::campaign_news::{EventType, campaign_alerts, campaign_headline};
use warroom_core::factions::faction_label;
use warroom_core::format::dispatch_title;
use warroom_core::history::detect_new_defenses;

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
                for d in dispatches.iter().filter(|d| d.id > s.seen_dispatch).take(3) {
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
                    ToastKind::Order,
                    if launched { "New Galactic Campaign" } else { "Galactic Campaign over" },
                    &campaign_headline(ev),
                    s.alerts.system && !NEWS_PUSHED,
                    launched.then(|| ev.campaign.description.clone()),
                );
            }
        }
        settings().mark_campaign_seen(keys);
    });
}
