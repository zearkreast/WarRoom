//! Spotting what changed between two planet snapshots, new owners and new defenses, and searching the war log.

use crate::api::{Faction, Planet};
use crate::news_feed::matches_query;
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

/// What the war log is narrowed to; empty fields let everything through.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LogQuery {
    pub enemy: Option<Faction>,
    pub sector: Option<String>,
    /// Words that must all appear in the planet's name or sector.
    pub text: String,
}

impl LogQuery {
    pub fn is_empty(&self) -> bool {
        self.enemy.is_none() && self.sector.is_none() && self.text.trim().is_empty()
    }
}

/// The faction Super Earth took the planet from, or lost it to.
pub fn enemy_of(c: &OwnerChange) -> Faction {
    if c.to == Faction::Humans { c.from } else { c.to }
}

/// Whether a change of hands on a planet in `sector` passes the war log's filters.
pub fn log_matches(c: &OwnerChange, sector: Option<&str>, q: &LogQuery) -> bool {
    let sector = sector.unwrap_or_default();
    q.enemy.is_none_or(|e| enemy_of(c) == e)
        && q.sector.as_deref().is_none_or(|s| s == sector)
        && matches_query(&format!("{} {sector}", c.planet_name), &q.text)
}

/// Planets worth sampling every poll even without a campaign: contested ones. Quiet ones only need owner tracking.
pub fn is_active(p: &Planet) -> bool {
    p.event.is_some() || p.health < p.max_health
}
