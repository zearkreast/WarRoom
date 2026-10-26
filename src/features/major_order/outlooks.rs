use crate::api::queries::queries;
use leptos::prelude::*;
use warroom_core::api::{Assignment, Faction};
use warroom_core::major_order::{DeadlineOutlook, TaskOutlook, TaskView, order_task_rates, task_outlook, task_percent};
use warroom_core::progress::tracked_percent;
use warroom_core::time::parse_ms;

/// Each task's outlook at the current pace: counts from the order's own readings, planets from their campaign.
/// Reads the snapshots it needs, so calling it inside a reactive closure keeps it current.
pub fn outlooks(a: &Assignment, tasks: &[TaskView], now: f64) -> Vec<TaskOutlook> {
    let q = queries();
    let samples = q.order_samples.get();
    let campaigns = q.campaigns.get();
    let deadline = parse_ms(&a.expiration);
    let counts = order_task_rates(samples.as_deref().map(|s| s.as_slice()), a.id, a.tasks.len());
    tasks
        .iter()
        .enumerate()
        .map(|(i, t)| {
            if !t.is_planet {
                let rate = counts[i].filter(|_| t.goal != 0.0).map(|c| c / t.goal * 100.0);
                return task_outlook(task_percent(t), rate, now, deadline);
            }
            let p = t.planet.as_ref().expect("planet task");
            if t.done || (p.current_owner == Faction::Humans && p.event.is_none()) {
                return TaskOutlook { outlook: DeadlineOutlook::Done, end: Some(100.0), eta: None, rate: None };
            }
            let front = campaigns.as_ref().and_then(|c| c.iter().find(|c| c.planet.index == p.index));
            if campaigns.is_some() && front.is_none() {
                return TaskOutlook { outlook: DeadlineOutlook::NoFront, end: None, eta: None, rate: None };
            }
            task_outlook(tracked_percent(p), front.and_then(|f| f.rate), now, deadline)
        })
        .collect()
}
