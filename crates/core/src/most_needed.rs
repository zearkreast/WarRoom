//! Ranking fronts by where more Helldivers are needed most.

use crate::progress::projected_at;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrontNeed {
    pub percent: f64,
    /// Net %/h, None while measuring.
    pub rate: Option<f64>,
    /// Enemy regeneration in %/h; 0 for a defense.
    pub regen: f64,
    pub players: f64,
    pub in_major_order: bool,
    /// A defense's own deadline, or the Major Order's for a liberation target.
    pub deadline: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NeedReason {
    MajorOrder,
    LosingGround,
    MissesDeadline,
    EachDiverCounts,
}

impl NeedReason {
    pub fn label(self) -> &'static str {
        match self {
            NeedReason::MajorOrder => "MO target",
            NeedReason::LosingGround => "Losing ground",
            NeedReason::MissesDeadline => "Misses its deadline",
            NeedReason::EachDiverCounts => "Each Helldiver counts",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Need {
    pub score: f64,
    pub reasons: Vec<NeedReason>,
}

/// %/h that 1,000 of the Helldivers deployed bring, before regen. None while measuring, and at 0%, where progress
/// can't fall any further and so hides it.
pub fn per_thousand(f: &FrontNeed) -> Option<f64> {
    let rate = f.rate?;
    (f.players > 0.0 && f.percent > 0.0).then(|| (rate + f.regen) / f.players * 1000.0)
}

/// The median of the fronts' progress per 1,000 Helldivers.
pub fn typical_per_thousand(fronts: &[FrontNeed]) -> Option<f64> {
    let mut v: Vec<f64> = fronts.iter().filter_map(per_thousand).filter(|v| *v > 0.0).collect();
    if v.is_empty() {
        return None;
    }
    v.sort_by(f64::total_cmp);
    let mid = v.len() / 2;
    Some(if v.len().is_multiple_of(2) { (v[mid - 1] + v[mid]) / 2.0 } else { v[mid] })
}

/// How much a front needs Helldivers now: each reason adds to the score, then how far a Helldiver there outdoes
/// the typical one.
pub fn front_need(f: &FrontNeed, now: f64, typical: Option<f64>) -> Need {
    let mut reasons = Vec::new();
    let mut score = 0.0;
    if f.in_major_order {
        reasons.push(NeedReason::MajorOrder);
        score += 3.0;
    }
    if f.rate.is_some_and(|r| r <= 0.0) && f.percent > 0.0 {
        reasons.push(NeedReason::LosingGround);
        score += 2.0;
    }
    let short = f.deadline.filter(|d| *d > now).and_then(|d| projected_at(f.percent, f.rate, now, d));
    if short.is_some_and(|end| end < 100.0) {
        reasons.push(NeedReason::MissesDeadline);
        score += 3.0;
    }
    let weight = per_thousand(f).zip(typical).map(|(v, t)| v / t);
    if weight.is_some_and(|w| w >= 1.5) {
        reasons.push(NeedReason::EachDiverCounts);
        score += 1.0;
    }
    Need { score: score + weight.unwrap_or(0.0).clamp(0.0, 3.0) * 0.3, reasons }
}
