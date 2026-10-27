//! The Hellpad planet index: search, filters, sector groups and each faction's share of the galaxy.

use crate::api::{Faction, Planet};
use crate::collate::locale_cmp;
use crate::galaxy::{group_by, majority};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanetFilter {
    All,
    Active,
    Pinned,
    Owner(Faction),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectorSort {
    Az,
    Players,
    Active,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SectorGroup<'a> {
    pub name: String,
    pub planets: Vec<&'a Planet>,
    pub active: usize,
    pub players: f64,
    /// Faction holding most of the sector's planets.
    pub owner: Faction,
    pub hottest: bool,
}

/// Search across planet, sector, biome and hazard names; every word must match somewhere.
pub fn matches_search(p: &Planet, query: &str) -> bool {
    let q = query.to_lowercase();
    let words: Vec<&str> = q.split_whitespace().collect();
    if words.is_empty() {
        return true;
    }
    let mut hay = vec![p.name.as_str(), p.sector.as_str(), p.biome.name.as_str()];
    hay.extend(p.hazards.iter().map(|h| h.name.as_str()));
    let hay = hay.join(" ").to_lowercase();
    words.iter().all(|w| hay.contains(w))
}

pub fn matches_filter(p: &Planet, filter: PlanetFilter, active: &HashSet<i64>, pinned: &HashSet<i64>) -> bool {
    match filter {
        PlanetFilter::All => true,
        PlanetFilter::Active => active.contains(&p.index),
        PlanetFilter::Pinned => pinned.contains(&p.index),
        PlanetFilter::Owner(f) => p.current_owner == f,
    }
}

/// Groups planets by sector for the planet index. Inside a sector, fronts come first, then
/// the busiest planets. The sector with the most Helldivers on active fronts is the hottest.
pub fn group_by_sector<'a>(planets: &'a [Planet], active: &HashSet<i64>, sort: SectorSort) -> Vec<SectorGroup<'a>> {
    let mut groups: Vec<SectorGroup> = group_by(planets, |p| p.sector.clone())
        .into_iter()
        .map(|(name, list)| {
            let mut sorted = list.clone();
            sorted.sort_by(|a, b| {
                active
                    .contains(&b.index)
                    .cmp(&active.contains(&a.index))
                    .then_with(|| b.statistics.player_count.total_cmp(&a.statistics.player_count))
                    .then_with(|| locale_cmp(&a.name, &b.name))
            });
            SectorGroup {
                name,
                active: list.iter().filter(|p| active.contains(&p.index)).count(),
                players: crate::format::js_sum(list.iter().map(|p| p.statistics.player_count)),
                owner: majority(list.iter().map(|p| p.current_owner)).unwrap_or_default(),
                planets: sorted,
                hottest: false,
            }
        })
        .collect();

    let mut hot: Option<usize> = None;
    for (i, g) in groups.iter().enumerate() {
        if g.active > 0 && hot.is_none_or(|h| g.players > groups[h].players) {
            hot = Some(i);
        }
    }
    if let Some(h) = hot {
        groups[h].hottest = true;
    }

    let by_name = |a: &SectorGroup, b: &SectorGroup| locale_cmp(&a.name, &b.name);
    groups.sort_by(|a, b| match sort {
        SectorSort::Az => by_name(a, b),
        SectorSort::Players => b.players.total_cmp(&a.players).then_with(|| by_name(a, b)),
        SectorSort::Active => {
            b.active.cmp(&a.active).then_with(|| b.players.total_cmp(&a.players)).then_with(|| by_name(a, b))
        }
    });
    groups
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlShare {
    pub faction: Faction,
    pub planets: usize,
    pub pct: f64,
    pub players: f64,
}

/// Share of planets per faction, 0-100, in a fixed faction order.
pub fn control_shares(planets: &[Planet]) -> Vec<ControlShare> {
    let total = if planets.is_empty() { 1.0 } else { planets.len() as f64 };
    Faction::ALL
        .into_iter()
        .map(|f| {
            let owned: Vec<&Planet> = planets.iter().filter(|p| p.current_owner == f).collect();
            ControlShare {
                faction: f,
                planets: owned.len(),
                pct: owned.len() as f64 / total * 100.0,
                players: crate::format::js_sum(owned.iter().map(|p| p.statistics.player_count)),
            }
        })
        .collect()
}
