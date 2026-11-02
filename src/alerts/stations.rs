//! Alerts for a space station's tactical actions: one charging fully, or going active.

use crate::alerts::{ToastKind, notify};
use crate::api::queries::queries;
use crate::state::settings;
use leptos::prelude::*;
use std::collections::HashMap;
use warroom_core::api::TacticalAction;
use warroom_core::station_actions::{ActionAlert, action_alerts};

pub fn start() {
    let q = queries();
    // Each look against the last, so nothing fires for what was already so at startup.
    let mut seen: HashMap<i64, Vec<TacticalAction>> = HashMap::new();
    Effect::new(move |_| {
        let Some(stations) = q.stations.get() else { return };
        let (on, system) = settings().untracked(|s| (s.station_alerts, s.alerts.system));
        for s in stations.iter() {
            if on && let Some(prev) = seen.get(&s.id32) {
                for alert in action_alerts(prev, &s.tactical_actions) {
                    let (title, body) = match alert {
                        ActionAlert::Active(name) => {
                            (format!("{name} is active"), "The DSS has deployed it.".to_string())
                        }
                        ActionAlert::Charged(name) => {
                            (format!("{name} is charged"), "Its readiness has reached 100%.".to_string())
                        }
                    };
                    notify(ToastKind::Campaign, &title, &body, system, None);
                }
            }
            seen.insert(s.id32, s.tactical_actions.clone());
        }
    });
}
