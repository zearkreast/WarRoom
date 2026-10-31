//! A planet's cities: size class, state and how close an enemy-held one is to liberation.

use crate::api::{Faction, Planet};
use crate::data::{CityFateState, super_earth_city};
use regex_lite::Regex;
use std::sync::LazyLock;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CityState {
    Liberating,
    #[default]
    Locked,
    Held,
    Secured,
    Destroyed,
    Rebuilt,
    Intact,
}

impl CityState {
    pub fn as_str(self) -> &'static str {
        match self {
            CityState::Liberating => "liberating",
            CityState::Locked => "locked",
            CityState::Held => "held",
            CityState::Secured => "secured",
            CityState::Destroyed => "destroyed",
            CityState::Rebuilt => "rebuilt",
            CityState::Intact => "intact",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct City {
    pub id: i64,
    pub name: String,
    /// Its name before it was rebuilt.
    pub formerly: Option<String>,
    pub size: String,
    pub stars: usize,
    pub description: String,
    pub note: Option<&'static str>,
    pub state: CityState,
    /// Liberated share, while the enemy's hold on it is being counted down.
    pub percent: Option<f64>,
    pub players: f64,
    /// Planet liberation, in points, that freeing it deals: 1.5 times its health.
    pub boost: Option<f64>,
    /// Enemy regeneration in %/h of the city's health.
    pub resistance: Option<f64>,
    /// Net progress in %/h, measured by the collector.
    pub rate: Option<f64>,
    /// The planet's liberation at which it opens; 0 for a city open from the start.
    pub opens_at: Option<f64>,
    /// Closed on an enemy-held planet, waiting on the planet's liberation to reach `opens_at` (above 0).
    pub waiting: bool,
    /// Who holds it: Super Earth once it's liberated (or the planet is), else the planet's owner.
    pub holder: Faction,
}

/// Hours until a city is liberated at its current net rate, or None if it isn't gaining.
pub fn city_hours_left(percent: Option<f64>, rate: Option<f64>) -> Option<f64> {
    let (p, r) = (percent?, rate?);
    (r > 0.0).then(|| (100.0 - p) / r)
}

/// The game marks a region's size class with one to four stars over its icon.
pub fn city_stars(size: &str) -> usize {
    match size {
        "Settlement" => 1,
        "Town" => 2,
        "City" => 3,
        "MegaCity" => 4,
        _ => 0,
    }
}

static CAMEL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([a-z])([A-Z])").unwrap());

/// "MegaCity" → "Mega City".
pub fn size_label(size: &str) -> String {
    CAMEL.replace_all(size, "$1 $2").into_owned()
}

fn state_of(p: &Planet, health: Option<f64>, open: bool) -> CityState {
    if open {
        return CityState::Liberating;
    }
    // Cities stay Super Earth's while their planet is defended.
    if p.current_owner == Faction::Humans {
        return CityState::Secured;
    }
    if health.is_none() { CityState::Held } else { CityState::Locked }
}

/// Liberation % at which a region opens. A factor of 0 marks a special site outside the opening order: no threshold.
fn opening(factor: Option<f64>) -> Option<f64> {
    factor.filter(|f| *f > 0.0 && *f <= 1.0).map(|f| (1.0 - f) * 100.0)
}

fn nonzero(v: f64) -> bool {
    v != 0.0 && !v.is_nan()
}

/// A planet's cities as the player sees them, with what is known of each one's fate.
pub fn cities_of(p: &Planet) -> Vec<City> {
    p.regions
        .iter()
        .map(|r| {
            let fate = if p.index == 0 { super_earth_city(r.id) } else { None };
            let percent =
                r.health.filter(|_| nonzero(r.max_health)).map(|h| (100.0 - (h / r.max_health) * 100.0).max(0.0));
            let live = state_of(p, r.health, r.is_available);
            let fate_name = fate.and_then(|f| f.name);
            let state = if live == CityState::Liberating {
                live
            } else {
                match fate.map(|f| f.state) {
                    Some(CityFateState::Destroyed) => CityState::Destroyed,
                    Some(CityFateState::Rebuilt) => CityState::Rebuilt,
                    Some(CityFateState::Intact) => CityState::Intact,
                    None => live,
                }
            };
            let opens_at = opening(r.availability_factor);
            City {
                id: r.id,
                name: fate_name.map_or_else(|| r.name.clone(), String::from),
                formerly: fate_name.map(|_| r.name.clone()),
                size: r.size.clone(),
                stars: city_stars(&r.size),
                // The API sends the string "null" for regions without one.
                description: if r.description == "null" { String::new() } else { r.description.clone() },
                note: fate.map(|f| f.note),
                state,
                percent,
                players: r.players,
                boost: nonzero(p.max_health).then(|| ((1.5 * r.max_health) / p.max_health) * 100.0),
                resistance: r
                    .regen_per_second
                    .filter(|_| nonzero(r.max_health))
                    .map(|regen| ((regen * 3600.0) / r.max_health) * 100.0),
                rate: r.rate,
                opens_at,
                waiting: matches!(state, CityState::Locked | CityState::Held) && opens_at.is_some_and(|at| at > 0.0),
                holder: if percent.is_some_and(|p| p >= 100.0) { Faction::Humans } else { p.current_owner },
            }
        })
        .collect()
}

/// A city's mark on its planet's liberation bar.
#[derive(Clone, Debug, PartialEq)]
pub struct BarMark {
    pub city: City,
    /// Liberation % it opens at: where its stem meets the bar.
    pub at: f64,
    /// Row of its icon over the bar, 0 nearest; marks closer than the gap stack instead of shifting aside.
    pub row: usize,
}

/// Where each city opens along its planet's liberation bar, left to right. None while the planet is defended:
/// the thresholds count liberation, not defense.
pub fn bar_marks(p: &Planet, gap: f64) -> Vec<BarMark> {
    if p.event.is_some() || p.current_owner == Faction::Humans {
        return Vec::new();
    }
    let mut marks: Vec<BarMark> =
        cities_of(p).into_iter().filter_map(|c| Some(BarMark { at: c.opens_at?, city: c, row: 0 })).collect();
    marks.sort_by(|a, b| a.at.total_cmp(&b.at).then(a.city.id.cmp(&b.city.id)));
    let mut last: Vec<f64> = Vec::new();
    for m in &mut marks {
        m.row = last.iter().position(|x| m.at - x >= gap).unwrap_or(last.len());
        match last.get_mut(m.row) {
            Some(x) => *x = m.at,
            None => last.push(m.at),
        }
    }
    marks
}
