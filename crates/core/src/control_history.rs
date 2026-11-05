//! How many planets each side held, day by day, worked back from today's owners through every change of hands.

use crate::api::Faction;
use crate::owner_changes::OwnerChange;
use std::collections::HashMap;

pub const DAY_MS: f64 = 86_400_000.0;

#[derive(Clone, Debug, PartialEq)]
pub struct DayControl {
    /// The day's end (the SEST midnight after it, or now for today), epoch ms.
    pub at: f64,
    /// Planets held then, in `Faction::ALL` order.
    pub held: [usize; 4],
}

/// Planets held at the end of each of the last `days` days, oldest first. A change on a planet missing from
/// `owners` (who holds each planet now) is skipped.
pub fn control_by_day(
    owners: &HashMap<i64, Faction>,
    changes: &[OwnerChange],
    days: usize,
    now: f64,
) -> Vec<DayControl> {
    let mut owners = owners.clone();
    let mut back: Vec<&OwnerChange> = changes.iter().collect();
    back.sort_by(|a, b| b.at.total_cmp(&a.at));
    let mut back = back.into_iter().peekable();
    let today_end = now - now.rem_euclid(DAY_MS) + DAY_MS;
    let mut out: Vec<DayControl> = (0..days)
        .map(|k| {
            let at = (today_end - k as f64 * DAY_MS).min(now);
            while let Some(c) = back.next_if(|c| c.at > at) {
                if let Some(o) = owners.get_mut(&c.planet) {
                    *o = c.from;
                }
            }
            DayControl { at, held: Faction::ALL.map(|f| owners.values().filter(|o| **o == f).count()) }
        })
        .collect();
    out.reverse();
    out
}

/// Liberated from and lost to each enemy since `since`, in `Faction::ENEMIES` order.
pub fn front_tally(changes: &[OwnerChange], since: f64) -> [(usize, usize); 3] {
    Faction::ENEMIES.map(|e| {
        let recent = changes.iter().filter(|c| c.at > since);
        let won = recent.clone().filter(|c| c.to == Faction::Humans && c.from == e).count();
        let lost = recent.filter(|c| c.from == Faction::Humans && c.to == e).count();
        (won, lost)
    })
}
