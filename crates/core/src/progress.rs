use crate::api::{Faction, Planet};

/// A point-in-time reading of a planet, as stored by the history recorder.
#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    pub planet: i64,
    /// Epoch ms.
    pub at: f64,
    pub owner: Faction,
    /// Liberation % (0-100) for a liberation campaign, defense % during a defense event.
    pub percent: f64,
    pub players: f64,
    pub defending: bool,
}

pub fn liberation_percent(health: f64, max_health: f64) -> f64 {
    if max_health == 0.0 || max_health.is_nan() {
        return 0.0;
    }
    clamp(100.0 - (health / max_health) * 100.0, 0.0, 100.0)
}

pub fn defense_percent(p: &Planet) -> Option<f64> {
    let e = p.event.as_ref()?;
    if e.max_health == 0.0 || e.max_health.is_nan() {
        return None;
    }
    Some(clamp(100.0 - (e.health / e.max_health) * 100.0, 0.0, 100.0))
}

/// The percent the UI tracks for a planet: defense progress if under attack, else liberation.
pub fn tracked_percent(p: &Planet) -> f64 {
    defense_percent(p).unwrap_or_else(|| liberation_percent(p.health, p.max_health))
}

pub fn to_sample(p: &Planet, at: f64) -> Sample {
    Sample {
        planet: p.index,
        at,
        owner: p.current_owner,
        percent: tracked_percent(p),
        players: p.statistics.player_count,
        defending: p.event.is_some(),
    }
}

const MIN_WINDOW_MS: f64 = 10.0 * 60_000.0;
const RATE_WINDOW_MS: f64 = 60.0 * 60_000.0;

/// Least-squares slope of `value` per hour. The game updates progress in coarse steps, so a slope over
/// every reading is far steadier than comparing the first and last ones.
pub fn slope_per_hour(points: &[(f64, f64)]) -> Option<f64> {
    let n = points.len();
    if n < 2 {
        return None;
    }
    let t0 = points[0].0;
    let mean_t = crate::format::js_sum(points.iter().map(|p| p.0 - t0)) / n as f64;
    let mean_v = crate::format::js_sum(points.iter().map(|p| p.1)) / n as f64;
    let (mut num, mut den) = (0.0, 0.0);
    for &(at, value) in points {
        let dt = at - t0 - mean_t;
        num += dt * (value - mean_v);
        den += dt * dt;
    }
    (den != 0.0).then(|| (num / den) * 3_600_000.0)
}

/// Percent-per-hour trend over the samples from the last hour. Returns None until there's 10 minutes
/// of comparable data (same owner and defense state, so a capture doesn't produce a spike).
pub fn rate_per_hour(samples: &[Sample]) -> Option<f64> {
    if samples.len() < 2 {
        return None;
    }
    let mut sorted: Vec<&Sample> = samples.iter().collect();
    sorted.sort_by(|a, b| a.at.total_cmp(&b.at));
    let last = *sorted.last()?;
    let window: Vec<&Sample> = sorted
        .into_iter()
        .filter(|s| s.at >= last.at - RATE_WINDOW_MS && s.owner == last.owner && s.defending == last.defending)
        .collect();
    if last.at - window[0].at < MIN_WINDOW_MS {
        return None;
    }
    slope_per_hour(&window.iter().map(|s| (s.at, s.percent)).collect::<Vec<_>>())
}

/// Epoch ms when the planet reaches 100% at the current rate, or None if it isn't progressing.
pub fn projected_completion(percent: f64, rate: Option<f64>, now: f64) -> Option<f64> {
    let rate = rate.filter(|r| *r > 0.0)?;
    Some(now + ((100.0 - percent) / rate) * 3_600_000.0)
}

/// Epoch ms when a liberation drops to 0% at the current (negative) rate.
pub fn projected_loss(percent: f64, rate: Option<f64>, now: f64) -> Option<f64> {
    let rate = rate.filter(|r| *r < 0.0)?;
    if percent <= 0.0 {
        return None;
    }
    Some(now + (percent / -rate) * 3_600_000.0)
}

/// Percent the planet will be at by `deadline` if the current rate holds (can exceed 100).
pub fn projected_at(percent: f64, rate: Option<f64>, now: f64, deadline: f64) -> Option<f64> {
    Some(percent + rate? * ((deadline - now) / 3_600_000.0))
}

fn clamp(n: f64, lo: f64, hi: f64) -> f64 {
    n.clamp(lo, hi)
}
