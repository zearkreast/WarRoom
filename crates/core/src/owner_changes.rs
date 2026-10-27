//! Spotting what changed between two planet snapshots: new owners and new defenses.

use crate::api::{Faction, Planet};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub struct OwnerChange {
    pub planet: i64,
    pub planet_name: String,
    pub from: Faction,
    pub to: Faction,
    pub at: f64,
}

/// Returns every planet whose owner differs from the last owner we recorded for it.
pub fn detect_owner_changes(known: &HashMap<i64, Faction>, planets: &[Planet], at: f64) -> Vec<OwnerChange> {
    planets
        .iter()
        .filter_map(|p| {
            let was = *known.get(&p.index)?;
            (was != p.current_owner).then(|| OwnerChange {
                planet: p.index,
                planet_name: p.name.clone(),
                from: was,
                to: p.current_owner,
                at,
            })
        })
        .collect()
}

/// Planets that have a defense event that wasn't in the previous snapshot.
pub fn detect_new_defenses<'a>(prev: Option<&[Planet]>, next: &'a [Planet]) -> Vec<&'a Planet> {
    let Some(prev) = prev else { return Vec::new() };
    let had: HashSet<i64> = prev.iter().filter_map(|p| p.event.as_ref().map(|e| e.id)).collect();
    next.iter().filter(|p| p.event.as_ref().is_some_and(|e| !had.contains(&e.id))).collect()
}

/// Planets worth sampling every poll even without a campaign: contested ones. Quiet ones only need owner tracking.
pub fn is_active(p: &Planet) -> bool {
    p.event.is_some() || p.health < p.max_health
}
