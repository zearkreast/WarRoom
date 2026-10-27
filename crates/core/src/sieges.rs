use crate::api::{Faction, Planet};
use std::collections::{HashMap, HashSet};

/// Supply-line neighbours by planet index, in the order they're met; a link listed on either end counts both ways.
pub fn neighbours(planets: &[Planet]) -> HashMap<i64, Vec<i64>> {
    let mut links: HashMap<i64, Vec<i64>> = planets.iter().map(|p| (p.index, Vec::new())).collect();
    let mut add = |a: i64, b: i64| {
        if let Some(list) = links.get_mut(&a)
            && !list.contains(&b)
        {
            list.push(b);
        }
    };
    for p in planets {
        for &w in &p.waypoints {
            add(p.index, w);
            add(w, p.index);
        }
    }
    links
}

#[derive(Clone, Debug, PartialEq)]
pub struct Circumvallation {
    /// Enemy planets that liberating the front would leave with only Super Earth planets around them.
    pub targets: Vec<Planet>,
    /// Of the targets, those that are fronts themselves and so could be liberated directly instead.
    pub direct: Vec<Planet>,
    /// Whether liberating the front costs less than liberating the direct targets.
    pub worth: bool,
}

fn enemy_held(p: &Planet) -> bool {
    p.current_owner != Faction::Humans
}

fn regen_per_hour(p: &Planet) -> f64 {
    p.regen_per_second * 3600.0
}

/// Hours to empty a planet's health at `pace` health an hour, its own resistance taken off. The pace is twice
/// the strongest resistance among the planets compared, so the comparison turns on health and resistance alone.
fn hours_at(p: &Planet, pace: f64) -> f64 {
    let net = pace - regen_per_hour(p);
    if pace > 0.0 && net > 0.0 { p.health / net } else { p.health }
}

/// What liberating `front` would do for a SEAF siege: the enemy planets it would leave surrounded, a siege needing
/// a planet on its own. None when it surrounds none. `fronts` are the planets with a campaign now.
pub fn circumvallation(
    front: &Planet,
    planets: &[Planet],
    links: &HashMap<i64, Vec<i64>>,
    fronts: &HashSet<i64>,
) -> Option<Circumvallation> {
    if !enemy_held(front) || front.event.is_some() {
        return None;
    }
    let by_index: HashMap<i64, &Planet> = planets.iter().map(|p| (p.index, p)).collect();
    let around = |i: i64| links.get(&i).map_or(&[][..], Vec::as_slice);
    let targets: Vec<Planet> = around(front.index)
        .iter()
        .filter_map(|i| {
            let x = by_index.get(i)?;
            if x.disabled || !enemy_held(x) {
                return None;
            }
            let surrounded = around(*i)
                .iter()
                .filter(|n| **n != front.index)
                .all(|n| by_index.get(n).is_some_and(|p| p.current_owner == Faction::Humans));
            surrounded.then(|| (*x).clone())
        })
        .collect();
    if targets.is_empty() {
        return None;
    }
    let direct: Vec<Planet> = targets.iter().filter(|x| fronts.contains(&x.index)).cloned().collect();
    let pace = 2.0 * std::iter::once(front).chain(&targets).map(regen_per_hour).fold(f64::NEG_INFINITY, f64::max);
    let worth = direct.is_empty() || hours_at(front, pace) <= direct.iter().fold(0.0, |n, x| n + hours_at(x, pace));
    Some(Circumvallation { targets, direct, worth })
}
