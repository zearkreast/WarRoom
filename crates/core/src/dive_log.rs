//! The player's own dives, kept only on their device, and how each campaign they joined turned out.

use crate::api::Faction;
use crate::owner_changes::OwnerChange;
use serde::{Deserialize, Serialize};

/// Dives kept at most; the oldest go first.
pub const DIVE_LIMIT: usize = 500;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Dive {
    pub planet: i64,
    pub name: String,
    /// Epoch ms.
    pub at: f64,
    /// The attack's event id for a defense; None for a liberation.
    pub event: Option<i64>,
    pub enemy: Faction,
    pub major_order: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DiveOutcome {
    Ongoing,
    /// When the planet was taken.
    Liberated(f64),
    Held,
    /// When the planet fell.
    Lost(f64),
    /// The liberation ended without the planet being taken.
    Withdrawn,
}

impl DiveOutcome {
    pub fn label(self) -> &'static str {
        match self {
            DiveOutcome::Ongoing => "Ongoing",
            DiveOutcome::Liberated(_) => "Liberated",
            DiveOutcome::Held => "Defense held",
            DiveOutcome::Lost(_) => "Lost",
            DiveOutcome::Withdrawn => "Not liberated",
        }
    }
}

/// What a planet is fighting now: None without a campaign, else the defense's event id (None for a liberation).
pub type Fight = Option<Option<i64>>;

/// How a dive turned out, from the planet's changes of hands since and the campaign it has now.
pub fn dive_outcome(d: &Dive, changes: &[OwnerChange], now: Fight) -> DiveOutcome {
    let mut after: Vec<&OwnerChange> = changes.iter().filter(|c| c.planet == d.planet && c.at > d.at).collect();
    after.sort_by(|a, b| a.at.total_cmp(&b.at));
    match d.event {
        None => match after.iter().find(|c| c.to == Faction::Humans) {
            Some(c) => DiveOutcome::Liberated(c.at),
            None if now == Some(None) => DiveOutcome::Ongoing,
            None => DiveOutcome::Withdrawn,
        },
        Some(id) => match after.iter().find(|c| c.from == Faction::Humans) {
            Some(c) => DiveOutcome::Lost(c.at),
            None if now == Some(Some(id)) => DiveOutcome::Ongoing,
            None => DiveOutcome::Held,
        },
    }
}

/// Dives on one planet that ended the same way, which is one campaign joined several times.
#[derive(Clone, Debug, PartialEq)]
pub struct DiveGroup {
    pub planet: i64,
    pub name: String,
    pub enemy: Faction,
    pub defense: bool,
    pub count: usize,
    pub first: f64,
    pub last: f64,
    pub major_order: bool,
    pub outcome: DiveOutcome,
}

/// The dives as campaigns joined, latest first.
pub fn group_dives(dives: &[(Dive, DiveOutcome)]) -> Vec<DiveGroup> {
    let mut groups: Vec<DiveGroup> = Vec::new();
    for (d, outcome) in dives {
        match groups
            .iter_mut()
            .find(|g| g.planet == d.planet && g.defense == d.event.is_some() && g.outcome == *outcome)
        {
            Some(g) => {
                g.count += 1;
                g.first = g.first.min(d.at);
                g.last = g.last.max(d.at);
                g.major_order |= d.major_order;
            }
            None => groups.push(DiveGroup {
                planet: d.planet,
                name: d.name.clone(),
                enemy: d.enemy,
                defense: d.event.is_some(),
                count: 1,
                first: d.at,
                last: d.at,
                major_order: d.major_order,
                outcome: *outcome,
            }),
        }
    }
    groups.sort_by(|a, b| b.last.total_cmp(&a.last));
    groups
}
