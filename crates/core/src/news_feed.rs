//! Strohmann News helpers: plain text, day groups and search.

use crate::dispatch::dispatch_body;
use crate::format::{dispatch_title, parse_markup, sest_date};

/// So texts that differ only in case and punctuation compare equal.
fn bare(s: &str) -> String {
    s.to_lowercase().chars().filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit()).collect()
}

/// A dispatch's text without the game's markup tags.
pub fn plain_text(message: &str) -> String {
    parse_markup(message).into_iter().map(|s| s.text).collect()
}

/// True when a dispatch's body only repeats its headline, as the game's attack alerts do.
pub fn body_echoes_title(message: &str) -> bool {
    let body = bare(&dispatch_body(message).into_iter().map(|s| s.text).collect::<String>());
    body.is_empty() || bare(&dispatch_title(message)).contains(&body)
}

const DAY: f64 = 86_400_000.0;
/// Days run on Super Earth Standard Time, which is UTC.
fn day_start(ms: f64) -> f64 {
    (ms / DAY).floor() * DAY
}

pub fn day_label(at: f64, now: f64) -> String {
    let days = (day_start(now) - day_start(at)) / DAY;
    if days == 0.0 {
        return "Today".into();
    }
    if days == 1.0 {
        return "Yesterday".into();
    }
    // No weekday: the in-game calendar runs 160 years ahead, and today's weekday would be wrong for it.
    sest_date(at)
}

pub struct DayGroup<T> {
    pub key: f64,
    pub label: String,
    pub items: Vec<T>,
}

/// Runs of items on the same day, in the order given; sort by time first or a day can appear twice.
pub fn group_by_day<T>(items: Vec<T>, at: impl Fn(&T) -> f64, now: f64) -> Vec<DayGroup<T>> {
    let mut groups: Vec<DayGroup<T>> = Vec::new();
    for item in items {
        let t = at(&item);
        let key = day_start(t);
        match groups.last_mut() {
            Some(last) if last.key == key => last.items.push(item),
            _ => groups.push(DayGroup { key, label: day_label(t, now), items: vec![item] }),
        }
    }
    groups
}

/// Whether every word of the query appears somewhere in the text, in any order and case.
pub fn matches_query(text: &str, query: &str) -> bool {
    let hay = text.to_lowercase();
    query.to_lowercase().split_whitespace().all(|w| hay.contains(w))
}
