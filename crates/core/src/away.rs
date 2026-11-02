//! "While you were away": what changed in the war between the player's last look and now.

use crate::api::Faction;
use crate::owner_changes::OwnerChange;
use crate::snapshot::OrderSample;

/// Shorter breaks than this aren't worth a briefing.
pub const AWAY_MIN_MS: f64 = 3.0 * 3_600_000.0;

pub fn is_away(last_seen: f64, now: f64) -> bool {
    last_seen > 0.0 && now - last_seen >= AWAY_MIN_MS
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Captures {
    pub liberated: Vec<String>,
    pub lost: Vec<String>,
}

/// Planets that changed hands after `since`, each counted by where it ended up.
pub fn captures_since(changes: &[OwnerChange], since: f64) -> Captures {
    let mut last: Vec<&OwnerChange> = Vec::new();
    let mut after: Vec<&OwnerChange> = changes.iter().filter(|c| c.at > since).collect();
    after.sort_by(|a, b| a.at.total_cmp(&b.at));
    for c in after {
        match last.iter_mut().find(|l| l.planet == c.planet) {
            Some(l) => *l = c,
            None => last.push(c),
        }
    }
    let names = |won: bool| {
        last.iter().filter(|c| (c.to == Faction::Humans) == won).map(|c| c.planet_name.clone()).collect::<Vec<_>>()
    };
    Captures { liberated: names(true), lost: names(false) }
}

/// Each objective's progress gained since `since`, from the order's first reading at or after it; None without one.
pub fn order_moves(samples: &[OrderSample], id: i64, now: &[f64], since: f64) -> Vec<Option<f64>> {
    let first = samples.iter().filter(|s| s.id == id && s.at >= since).min_by(|a, b| a.at.total_cmp(&b.at));
    now.iter().enumerate().map(|(i, v)| first.and_then(|f| f.progress.get(i)).map(|was| v - was)).collect()
}

/// "A, B and 3 more".
pub fn name_list(names: &[String], max: usize) -> String {
    let shown: Vec<&str> = names.iter().take(max).map(String::as_str).collect();
    match names.len().saturating_sub(max) {
        0 => match shown.split_last() {
            Some((last, rest)) if !rest.is_empty() => format!("{} and {last}", rest.join(", ")),
            _ => shown.concat(),
        },
        more => format!("{} and {more} more", shown.join(", ")),
    }
}
