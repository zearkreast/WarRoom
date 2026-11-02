//! The last day of fighting, from the hourly readings of the game's all-time mission and kill counters.

use serde::{Deserialize, Serialize};

const HOUR: f64 = 3_600_000.0;
/// Less than this between the readings compared and the trend isn't shown.
const MIN_SPAN_HOURS: f64 = 12.0;

/// The galaxy's readings use this in place of a planet index.
pub const GALAXY: i64 = -1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CombatReading {
    /// Epoch ms.
    pub at: f64,
    pub missions_won: f64,
    pub missions_lost: f64,
    pub terminid_kills: f64,
    pub automaton_kills: f64,
    pub illuminate_kills: f64,
    pub deaths: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CombatTrend {
    /// Hours between the readings compared, about 24.
    pub hours: f64,
    pub won: f64,
    pub lost: f64,
    /// Kills of the Terminids, Automatons and Illuminate, in `Faction::ENEMIES` order.
    pub kills: [f64; 3],
    pub deaths: f64,
}

impl CombatTrend {
    pub fn success_rate(&self) -> Option<f64> {
        let missions = self.won + self.lost;
        (missions > 0.0).then(|| self.won / missions * 100.0)
    }
}

/// The newest reading against the oldest within a day of it; None with less than 12 hours between them.
/// A counter that went down (the game reset it) reads as None too.
pub fn last_day(readings: &[CombatReading]) -> Option<CombatTrend> {
    let newest = readings.iter().max_by(|a, b| a.at.total_cmp(&b.at))?;
    let oldest = readings.iter().filter(|r| r.at >= newest.at - 24.0 * HOUR).min_by(|a, b| a.at.total_cmp(&b.at))?;
    let hours = (newest.at - oldest.at) / HOUR;
    if hours < MIN_SPAN_HOURS {
        return None;
    }
    let d = |f: fn(&CombatReading) -> f64| f(newest) - f(oldest);
    let t = CombatTrend {
        hours,
        won: d(|r| r.missions_won),
        lost: d(|r| r.missions_lost),
        kills: [d(|r| r.terminid_kills), d(|r| r.automaton_kills), d(|r| r.illuminate_kills)],
        deaths: d(|r| r.deaths),
    };
    let all = [t.won, t.lost, t.deaths, t.kills[0], t.kills[1], t.kills[2]];
    all.iter().all(|v| *v >= 0.0).then_some(t)
}
