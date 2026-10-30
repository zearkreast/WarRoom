//! Defenses: the enemy's advance against the clock, where the attack comes from, what holding needs, and the
//! gambit of liberating the attacking planet before the deadline.

use crate::api::{Faction, Planet, PlanetEvent};
use crate::progress::{defense_percent, liberation_percent, projected_completion};
use crate::time::parse_ms;
use std::collections::HashMap;

const HOUR: f64 = 3_600_000.0;

/// The attacker's bar: the share of the event's time gone, 0-100. The planet falls if it reaches 100% first.
pub fn enemy_advance(e: &PlanetEvent, now: f64) -> Option<f64> {
    let (start, end) = (parse_ms(&e.start_time), parse_ms(&e.end_time));
    (end > start).then(|| ((now - start) / (end - start) * 100.0).clamp(0.0, 100.0))
}

/// Epoch ms the attack began and its deadline, when both are known.
pub fn event_span(e: &PlanetEvent) -> Option<(f64, f64)> {
    let (start, end) = (parse_ms(&e.start_time), parse_ms(&e.end_time));
    (end > start).then_some((start, end))
}

/// Enemy planets attacking `target` along their supply lines; those of the attacking faction when any are.
pub fn attackers<'a>(target: &Planet, planets: &'a [Planet]) -> Vec<&'a Planet> {
    let all: Vec<&Planet> =
        planets.iter().filter(|p| p.current_owner != Faction::Humans && p.attacking.contains(&target.index)).collect();
    match target.event.as_ref().map(|e| e.faction) {
        Some(f) if all.iter().any(|p| p.current_owner == f) => {
            all.into_iter().filter(|p| p.current_owner == f).collect()
        }
        _ => all,
    }
}

/// %/h a defense needs from now on to reach 100% by its deadline; None once the deadline has passed.
pub fn needed_rate(percent: f64, now: f64, deadline: f64) -> Option<f64> {
    let hours = (deadline - now) / HOUR;
    (hours > 0.0).then(|| (100.0 - percent).max(0.0) / hours)
}

/// Roughly how many more Helldivers the needed rate takes, at today's progress per Helldiver; 0 when the
/// measured rate is enough. None while the rate is unknown or not gaining.
pub fn divers_short(rate: Option<f64>, needed: f64, players: f64) -> Option<f64> {
    let rate = rate.filter(|r| *r > 0.0)?;
    if !(players > 0.0) {
        return None;
    }
    if rate >= needed {
        return Some(0.0);
    }
    Some(((needed - rate) / (rate / players)).ceil())
}

/// An attacking planet in a gambit: how far its liberation is and when it would be done at its rate.
#[derive(Clone, Debug, PartialEq)]
pub struct GambitTarget {
    pub planet: Planet,
    pub percent: f64,
    /// Measured %/h; None while measuring.
    pub rate: Option<f64>,
    /// Epoch ms it would be liberated at that rate.
    pub done_at: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GambitOutlook {
    /// The attackers fall before the deadline, and sooner than the defense would be won.
    Ahead,
    /// The attackers fall before the deadline, but holding the planet directly is quicker.
    Slower,
    /// Not before the deadline at the current pace.
    OutOfReach,
    /// An attacker's rate is still being measured.
    Measuring,
}

/// Liberating every planet the attack comes from ends a defense: what that would take, against holding it.
#[derive(Clone, Debug, PartialEq)]
pub struct Gambit {
    pub defended: Planet,
    pub attackers: Vec<GambitTarget>,
    /// The defense's deadline, epoch ms.
    pub deadline: f64,
    /// When the last attacker would be liberated at the current rates.
    pub done_at: Option<f64>,
    /// When the defense would be won outright at its rate, if before the deadline.
    pub held_at: Option<f64>,
    pub outlook: GambitOutlook,
}

/// The gambit open to the defense of `defended`: None unless it is under attack and every planet attacking it is
/// a liberation front. `rates` holds every front's measured %/h by planet index (None while measuring).
pub fn gambit(defended: &Planet, planets: &[Planet], rates: &HashMap<i64, Option<f64>>, now: f64) -> Option<Gambit> {
    let event = defended.event.as_ref()?;
    let deadline = parse_ms(&event.end_time);
    if !deadline.is_finite() {
        return None;
    }
    let from = attackers(defended, planets);
    if from.is_empty() || from.iter().any(|p| !rates.contains_key(&p.index) || p.event.is_some()) {
        return None;
    }
    let targets: Vec<GambitTarget> = from
        .iter()
        .map(|p| {
            let percent = liberation_percent(p.health, p.max_health);
            let rate = rates.get(&p.index).copied().flatten();
            GambitTarget { planet: (*p).clone(), percent, rate, done_at: projected_completion(percent, rate, now) }
        })
        .collect();
    let done_at = targets.iter().try_fold(f64::NEG_INFINITY, |m, t| t.done_at.map(|d| m.max(d)));
    let held_at = defense_percent(defended)
        .and_then(|pct| projected_completion(pct, rates.get(&defended.index).copied().flatten(), now))
        .filter(|h| *h < deadline);
    let outlook = match done_at {
        Some(d) if d < deadline && held_at.is_some_and(|h| h <= d) => GambitOutlook::Slower,
        Some(d) if d < deadline => GambitOutlook::Ahead,
        Some(_) => GambitOutlook::OutOfReach,
        None if targets.iter().any(|t| t.rate.is_some_and(|r| r <= 0.0)) => GambitOutlook::OutOfReach,
        None => GambitOutlook::Measuring,
    };
    Some(Gambit { defended: defended.clone(), attackers: targets, deadline, done_at, held_at, outlook })
}

/// The gambits liberating `front` takes part in: one per defense it is attacking.
pub fn gambits_from(front: &Planet, planets: &[Planet], rates: &HashMap<i64, Option<f64>>, now: f64) -> Vec<Gambit> {
    planets
        .iter()
        .filter(|p| p.event.is_some() && front.attacking.contains(&p.index))
        .filter_map(|p| gambit(p, planets, rates, now))
        .filter(|g| g.attackers.iter().any(|a| a.planet.index == front.index))
        .collect()
}
