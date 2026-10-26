//! G.W.W. text links. Content writes `[[kind:target|label]]` inline:
//! - `manual:`, `ministry:`, `company:`, `holiday:` an id; `site:log`, `site:gov`, `site:citizens` or
//!   `site:holidays`; `dir:` a directory id;
//! - `planet:` a planet's name;
//! - `app:` a page outside the G.W.W., one of the app pages the G.W.W. navigation knows.

use regex_lite::Regex;
use std::sync::LazyLock;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Segment {
    Text(String),
    Link { kind: String, target: String, label: String },
}

const KINDS: [&str; 8] = ["manual", "ministry", "company", "holiday", "site", "dir", "planet", "app"];
static LINK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[\[([a-z]+):([^|\]]+)\|([^\]]+)\]\]").unwrap());

/// Splits text into plain runs and links; markup with an unknown kind stays as its label.
pub fn parse_rich(text: &str) -> Vec<Segment> {
    let mut out = Vec::new();
    let mut at = 0;
    for m in LINK.captures_iter(text) {
        let whole = m.get(0).unwrap();
        if whole.start() > at {
            out.push(Segment::Text(text[at..whole.start()].to_string()));
        }
        out.push(if KINDS.contains(&&m[1]) {
            Segment::Link { kind: m[1].to_string(), target: m[2].to_string(), label: m[3].to_string() }
        } else {
            Segment::Text(m[3].to_string())
        });
        at = whole.end();
    }
    if at < text.len() {
        out.push(Segment::Text(text[at..].to_string()));
    }
    out
}

/// The text as read, links reduced to their labels.
pub fn plain_text(text: &str) -> String {
    LINK.replace_all(text, "$3").into_owned()
}

/// Every link in the text, as `kind:target`.
pub fn links_in(text: &str) -> Vec<String> {
    LINK.captures_iter(text).map(|m| format!("{}:{}", &m[1], &m[2])).collect()
}

/// The next of some yearly days, counted in whole days from today's local date (`year`, `month`, `day`);
/// 0 means today. Returns the index into `days` and the day count.
pub fn next_yearly(days: &[(u32, u32)], year: i64, month: u32, day: u32) -> Option<(usize, i64)> {
    let today = crate::time::utc_ms(year, month, day, 0, 0, 0, 0);
    let mut best: Option<(usize, i64)> = None;
    for (i, &(m, d)) in days.iter().enumerate() {
        let mut at = crate::time::utc_ms(year, m, d, 0, 0, 0, 0);
        if at < today {
            at = crate::time::utc_ms(year + 1, m, d, 0, 0, 0, 0);
        }
        let n = ((at - today) / 86_400_000.0).round() as i64;
        if best.is_none_or(|(_, b)| n < b) {
            best = Some((i, n));
        }
    }
    best
}
