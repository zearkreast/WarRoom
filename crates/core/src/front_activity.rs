//! The War Activity chart's data: Helldivers per front in 15-minute buckets, merged with live readings.

use crate::api::Faction;
use crate::factions::faction_by_id;
use crate::format::js_round;
use crate::time::parse_ms;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One row of the `front_activity` database function.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FrontRow {
    pub bucket: String,
    pub owner: i64,
    /// Set for defenses, which come back per planet; None for a front's summed liberations.
    pub planet: Option<i64>,
    pub players: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrontBucket {
    pub at: f64,
    pub terminids: f64,
    pub automatons: f64,
    pub illuminate: f64,
    /// All three fronts together.
    pub players: f64,
}

pub const BUCKET_MS: f64 = 15.0 * 60_000.0;

/// Helldivers per front per bucket. A defense counts for whoever `attacker_of` names; one it can't
/// place is left out rather than guessed.
pub fn front_buckets(rows: &[FrontRow], attacker_of: impl Fn(i64) -> Option<Faction>) -> Vec<FrontBucket> {
    let mut order: Vec<u64> = Vec::new();
    let mut by_time: BTreeMap<u64, FrontBucket> = BTreeMap::new();
    for r in rows {
        let enemy = match r.planet {
            None => faction_by_id(r.owner),
            Some(p) => attacker_of(p),
        };
        let at = parse_ms(&r.bucket);
        let b = by_time.entry(at.to_bits()).or_insert_with(|| {
            order.push(at.to_bits());
            FrontBucket { at, ..Default::default() }
        });
        let slot = match enemy {
            Some(Faction::Terminids) => &mut b.terminids,
            Some(Faction::Automaton) => &mut b.automatons,
            Some(Faction::Illuminate) => &mut b.illuminate,
            _ => continue,
        };
        *slot += r.players;
        b.players += r.players;
    }
    let mut out: Vec<FrontBucket> = order.iter().map(|k| by_time[k]).collect();
    out.sort_by(|a, b| a.at.total_cmp(&b.at));
    out
}

/// The newest stretch without a missing bucket, so a collector outage doesn't draw a line across it.
pub fn latest_run(buckets: &[FrontBucket]) -> &[FrontBucket] {
    if buckets.is_empty() {
        return buckets;
    }
    let mut start = buckets.len() - 1;
    while start > 0 && buckets[start].at - buckets[start - 1].at <= BUCKET_MS {
        start -= 1;
    }
    &buckets[start..]
}

/// A galaxy-wide war reading: the online total, the multiplier and Helldivers per front.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Reading {
    pub at: f64,
    pub impact: f64,
    pub players: f64,
    pub terminids: f64,
    pub automatons: f64,
    pub illuminate: f64,
}

/// War readings averaged over `window_ms`, stamped at the window's middle. The API's player count swings
/// by thousands from one minute to the next, which turns a narrow chart's line into noise.
pub fn average_readings(readings: &[Reading], window_ms: f64) -> Vec<Reading> {
    let mut sums: BTreeMap<i64, (Reading, f64)> = BTreeMap::new();
    for r in readings {
        let at = (r.at / window_ms).floor() * window_ms;
        let s = sums.entry(at as i64).or_insert((Reading { at, ..Default::default() }, 0.0));
        s.0.impact += r.impact;
        s.0.players += r.players;
        s.0.terminids += r.terminids;
        s.0.automatons += r.automatons;
        s.0.illuminate += r.illuminate;
        s.1 += 1.0;
    }
    sums.into_values()
        .map(|(s, n)| Reading {
            at: s.at + window_ms / 2.0,
            impact: s.impact / n,
            players: js_round(s.players / n),
            terminids: js_round(s.terminids / n),
            automatons: js_round(s.automatons / n),
            illuminate: js_round(s.illuminate / n),
        })
        .collect()
}

/// One moment on the activity chart: the fronts, and the galaxy-wide reading when there is one.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ActivityPoint {
    pub at: f64,
    pub terminids: f64,
    pub automatons: f64,
    pub illuminate: f64,
    pub players: f64,
    pub online: Option<f64>,
    pub impact: Option<f64>,
}

/// Readings covering at least this long are drawn on their own.
const READINGS_ALONE_MS: f64 = 3_600_000.0;

/// The chart's rows. Each war reading is a row with its own split by front, as fine-grained as the
/// online line. The 15-minute front buckets fill in only while the readings cover less than an hour:
/// they count the fronts from planet samples, which comes out lower than the readings' split, so
/// drawing both would put a false step where one hands over to the other. A reading without a split
/// (older rows) takes the fronts interpolated from the buckets. The bucket still filling at `now` is
/// dropped: its partial sum would draw a fall that isn't there.
pub fn merge_activity(buckets: &[FrontBucket], readings: &[Reading], now: f64) -> Vec<ActivityPoint> {
    let done: Vec<FrontBucket> = buckets.iter().filter(|b| b.at + BUCKET_MS <= now).copied().collect();
    let mut sorted = readings.to_vec();
    sorted.sort_by(|a, b| a.at.total_cmp(&b.at));
    let start = done.first().map_or(f64::INFINITY, |b| b.at).min(sorted.first().map_or(f64::INFINITY, |r| r.at));
    if !start.is_finite() {
        return Vec::new();
    }
    let inside: Vec<Reading> = sorted.into_iter().filter(|r| r.at >= start).collect();
    let first_reading = inside.first().map_or(f64::INFINITY, |r| r.at);
    let fronts = |t: f64| -> Option<FrontBucket> {
        if done.is_empty() {
            return None;
        }
        let mut i = 0;
        while i + 2 < done.len() && done[i + 1].at <= t {
            i += 1;
        }
        let a = done[i];
        let b = done[(i + 1).min(done.len() - 1)];
        let f = if b.at == a.at { 0.0 } else { ((t - a.at) / (b.at - a.at)).clamp(0.0, 1.0) };
        let mix = |x: f64, y: f64| js_round(x + (y - x) * f);
        Some(FrontBucket {
            at: t,
            terminids: mix(a.terminids, b.terminids),
            automatons: mix(a.automatons, b.automatons),
            illuminate: mix(a.illuminate, b.illuminate),
            players: mix(a.players, b.players),
        })
    };
    let alone = inside.len() > 1 && inside[inside.len() - 1].at - first_reading >= READINGS_ALONE_MS;
    let mut rows: Vec<ActivityPoint> = if alone {
        Vec::new()
    } else {
        done.iter()
            .filter(|b| b.at < first_reading)
            .map(|b| ActivityPoint {
                at: b.at,
                terminids: b.terminids,
                automatons: b.automatons,
                illuminate: b.illuminate,
                players: b.players,
                online: None,
                impact: None,
            })
            .collect()
    };
    for r in &inside {
        let split = r.terminids + r.automatons + r.illuminate;
        let own = FrontBucket {
            at: r.at,
            terminids: r.terminids,
            automatons: r.automatons,
            illuminate: r.illuminate,
            players: split,
        };
        let f = if split > 0.0 { own } else { fronts(r.at).unwrap_or(own) };
        rows.push(ActivityPoint {
            at: f.at,
            terminids: f.terminids,
            automatons: f.automatons,
            illuminate: f.illuminate,
            players: f.players,
            online: Some(r.players),
            impact: Some(r.impact),
        });
    }
    rows
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ActivitySeries {
    Terminids,
    Automatons,
    Illuminate,
    Online,
    Impact,
}

/// An activity row as drawn, with the switched-off series parked.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlotPoint {
    pub row: ActivityPoint,
    pub plot_terminids: f64,
    pub plot_automatons: f64,
    pub plot_illuminate: f64,
    pub plot_online: Option<f64>,
    pub plot_impact: Option<f64>,
}

/// The chart's rows with switched-off series parked rather than dropped, so they can morph away and back.
pub fn plot_activity(
    rows: &[ActivityPoint],
    off: &[ActivitySeries],
    has_online: bool,
    flat_impact: f64,
) -> Vec<PlotPoint> {
    let is_off = |s: ActivitySeries| off.contains(&s);
    rows.iter()
        .map(|r| {
            let t = if is_off(ActivitySeries::Terminids) { 0.0 } else { r.terminids };
            let a = if is_off(ActivitySeries::Automatons) { 0.0 } else { r.automatons };
            let i = if is_off(ActivitySeries::Illuminate) { 0.0 } else { r.illuminate };
            let online = if has_online { r.online } else { Some(r.players) };
            PlotPoint {
                row: *r,
                plot_terminids: t,
                plot_automatons: a,
                plot_illuminate: i,
                plot_online: if online.is_none() || !is_off(ActivitySeries::Online) { online } else { Some(t + a + i) },
                plot_impact: if r.impact.is_none() || !is_off(ActivitySeries::Impact) {
                    r.impact
                } else {
                    Some(flat_impact)
                },
            }
        })
        .collect()
}
