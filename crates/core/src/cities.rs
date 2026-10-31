//! A planet's cities: size class, state, how close an enemy-held one is to liberation, and when one fell.

use crate::api::{Faction, Planet};
use crate::data::{CityFateState, super_earth_city};
use crate::progress::Sample;
use regex_lite::Regex;
use std::sync::LazyLock;

/// Steps read either side of a jump for the front's usual pace.
const NEIGHBOURS: usize = 6;
/// Readings further apart than this blur what happened between them.
const MAX_STEP_MS: f64 = 45.0 * 60_000.0;
/// How far, in points, a jump may be from what a city deals, the usual pace taken off.
const BOOST_TOLERANCE: f64 = 0.25;
/// How long before a jump a city's own reading still counts.
const READING_REACH_MS: f64 = 60.0 * 60_000.0;

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
    /// Points of its front's progress that freeing it deals: 1.5 times its health.
    pub boost: Option<f64>,
    /// Its front is a defense, not the planet's liberation.
    pub defense: bool,
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

/// A city's liberation at a moment, from the collector's readings of it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CityProgress {
    pub region: i64,
    pub at: f64,
    pub percent: f64,
}

/// A city freed, read off the jump it left in its front's progress.
#[derive(Clone, Debug, PartialEq)]
pub struct CityLiberation {
    /// The first reading after it fell.
    pub at: f64,
    /// The front's progress at that reading.
    pub percent: f64,
    pub defending: bool,
    /// Its name, or every city it could be when same-sized ones can't be told apart.
    pub cities: Vec<String>,
    /// Size class, as in [`city_stars`].
    pub stars: usize,
    /// Points of progress it dealt.
    pub boost: f64,
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

/// The health a front counts down: a defense its event's, not the planet's.
fn front_health(p: &Planet, defending: bool) -> Option<f64> {
    let health = if defending { p.event.as_ref()?.max_health } else { p.max_health };
    nonzero(health).then_some(health)
}

fn city_boost(city_health: f64, front_health: f64) -> f64 {
    ((1.5 * city_health) / front_health) * 100.0
}

fn same_front(a: &Sample, b: &Sample) -> bool {
    a.owner == b.owner && a.defending == b.defending
}

/// Progress per ms around the step from reading `i`: the median of the steps either side, which a jump can't skew.
fn usual_pace(samples: &[Sample], i: usize) -> f64 {
    let around = i.saturating_sub(NEIGHBOURS)..(i + NEIGHBOURS + 1).min(samples.len() - 1);
    let mut paces: Vec<f64> = around
        .filter(|k| *k != i)
        .filter_map(|k| {
            let (a, b) = (&samples[k], &samples[k + 1]);
            (same_front(a, b) && b.at > a.at).then(|| (b.percent - a.percent) / (b.at - a.at))
        })
        .collect();
    paces.sort_by(f64::total_cmp);
    paces.get(paces.len() / 2).copied().unwrap_or(0.0)
}

/// Cities freed across these readings of a planet, oldest first. A jump of what a city deals names it, and
/// `fought` tells same-sized cities apart: the one that was nearest liberation when the jump came. Nothing is
/// read for a past defense, whose health left with its event.
pub fn city_liberations(p: &Planet, samples: &[Sample], fought: &[CityProgress]) -> Vec<CityLiberation> {
    let cities = cities_of(p);
    let progress_before = |region: i64, at: f64| {
        fought
            .iter()
            .filter(|c| c.region == region && c.at <= at && at - c.at <= READING_REACH_MS)
            .max_by(|a, b| a.at.total_cmp(&b.at))
            .map(|c| c.percent)
    };
    samples
        .windows(2)
        .enumerate()
        .filter_map(|(i, pair)| {
            let (a, b) = (&pair[0], &pair[1]);
            let step = b.at - a.at;
            if !same_front(a, b) || step <= 0.0 || step > MAX_STEP_MS {
                return None;
            }
            let front = front_health(p, b.defending)?;
            let gain = b.percent - a.percent - usual_pace(samples, i) * step;
            let matches: Vec<(&City, f64)> = p
                .regions
                .iter()
                .zip(&cities)
                .map(|(r, c)| (c, city_boost(r.max_health, front)))
                .filter(|(_, boost)| (boost - gain).abs() <= BOOST_TOLERANCE)
                .collect();
            let (first, boost) = *matches.first()?;
            let nearest = matches.iter().filter_map(|(c, _)| progress_before(c.id, b.at)).max_by(|x, y| x.total_cmp(y));
            let names = matches
                .iter()
                .filter(|(c, _)| nearest.is_none() || progress_before(c.id, b.at) == nearest)
                .map(|(c, _)| c.name.clone())
                .collect();
            Some(CityLiberation {
                at: b.at,
                percent: b.percent,
                defending: b.defending,
                cities: names,
                stars: first.stars,
                boost,
            })
        })
        .collect()
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
                boost: front_health(p, p.event.is_some()).map(|front| city_boost(r.max_health, front)),
                defense: p.event.is_some(),
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
