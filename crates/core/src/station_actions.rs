//! A space station's tactical actions over time, from the collector's log of their status changes, and the
//! moments worth an alert.

use crate::api::TacticalAction;

/// The API's status for an action in effect.
pub const ACTIVE: i64 = 2;
/// The API's status for an action gathering its contributions.
pub const PREPARING: i64 = 1;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActionChange {
    pub action: i64,
    /// Epoch ms.
    pub at: f64,
    pub status: i64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ActionRecord {
    pub last_active: Option<f64>,
    /// Times it went active since the record's start.
    pub times: usize,
}

pub fn action_record(changes: &[ActionChange], action: i64) -> ActionRecord {
    let went: Vec<f64> = changes.iter().filter(|c| c.action == action && c.status == ACTIVE).map(|c| c.at).collect();
    ActionRecord { last_active: went.iter().copied().reduce(f64::max), times: went.len() }
}

/// Every contribution it needs is in.
pub fn is_charged(a: &TacticalAction) -> bool {
    a.status == PREPARING
        && !a.costs.is_empty()
        && a.costs.iter().all(|c| c.target_value > 0.0 && c.current_value >= c.target_value)
}

#[derive(Clone, Debug, PartialEq)]
pub enum ActionAlert {
    Active(String),
    Charged(String),
}

/// What changed between two looks at one station's actions.
pub fn action_alerts(prev: &[TacticalAction], next: &[TacticalAction]) -> Vec<ActionAlert> {
    next.iter()
        .filter_map(|a| {
            let was = prev.iter().find(|p| p.id32 == a.id32)?;
            if a.status == ACTIVE && was.status != ACTIVE {
                Some(ActionAlert::Active(a.name.clone()))
            } else if is_charged(a) && !is_charged(was) {
                Some(ActionAlert::Charged(a.name.clone()))
            } else {
                None
            }
        })
        .collect()
}
