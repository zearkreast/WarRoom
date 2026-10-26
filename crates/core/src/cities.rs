use crate::api::{Faction, Planet};
use crate::data::{CityFateState, super_earth_city};
use regex_lite::Regex;
use std::sync::LazyLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CityState {
    Liberating,
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

#[derive(Clone, Debug, PartialEq)]
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
    if p.current_owner == Faction::Humans && p.event.is_none() {
        return CityState::Secured;
    }
    if health.is_none() { CityState::Held } else { CityState::Locked }
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
            }
        })
        .collect()
}
