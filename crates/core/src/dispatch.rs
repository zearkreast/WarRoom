use crate::api::Dispatch;
use crate::format::{MarkupSpan, dispatch_title, parse_markup};
use crate::time::parse_ms;
use regex_lite::Regex;
use std::sync::LazyLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DispatchKind {
    Order,
    Victory,
    Defeat,
    Alert,
    Advisory,
    News,
}

impl DispatchKind {
    pub fn as_str(self) -> &'static str {
        match self {
            DispatchKind::Order => "order",
            DispatchKind::Victory => "victory",
            DispatchKind::Defeat => "defeat",
            DispatchKind::Alert => "alert",
            DispatchKind::Advisory => "advisory",
            DispatchKind::News => "news",
        }
    }
}

static RULES: LazyLock<Vec<(Regex, DispatchKind)>> = LazyLock::new(|| {
    [
        (r"major order (won|complete)|victory|liberated|success", DispatchKind::Victory),
        (r"major order (failed|lost)|defeat|has fallen|failed", DispatchKind::Defeat),
        (r"major order|new orders?|high command", DispatchKind::Order),
        (r"alert|attack|invasion|warning|emergency", DispatchKind::Alert),
        (r"advisory|notice|update|briefing", DispatchKind::Advisory),
    ]
    .into_iter()
    .map(|(re, k)| (Regex::new(&format!("(?i){re}")).unwrap(), k))
    .collect()
});

/// Category of a dispatch, from its headline (or first line).
pub fn dispatch_kind(message: &str) -> DispatchKind {
    let title = dispatch_title(message);
    RULES.iter().find(|(re, _)| re.is_match(&title)).map_or(DispatchKind::News, |(_, k)| *k)
}

/// The dispatch body's spans without the leading headline, so cards don't repeat the title.
pub fn dispatch_body(message: &str) -> Vec<MarkupSpan> {
    let spans = parse_markup(crate::format::js_trim(message));
    let mut rest: Vec<MarkupSpan> =
        if spans.first().is_some_and(|s| s.style.is_some()) { spans[1..].to_vec() } else { spans };
    if let Some(first) = rest.first_mut() {
        first.text = first.text.trim_start().to_string();
    }
    rest
}

pub const RECENT_DISPATCH_MS: f64 = 72.0 * 3_600_000.0;

/// Dispatches published in the last 72 hours, newest first.
pub fn recent_dispatches(list: &[Dispatch], now: f64) -> Vec<Dispatch> {
    let mut out: Vec<Dispatch> =
        list.iter().filter(|d| now - parse_ms(&d.published) <= RECENT_DISPATCH_MS).cloned().collect();
    out.sort_by(|a, b| b.id.cmp(&a.id));
    out
}

/// The newest `count` dispatches whatever their age, or the last 72 hours' for "72h".
pub fn latest_dispatches(list: &[Dispatch], count: &str, now: f64) -> Vec<Dispatch> {
    if count == "72h" {
        return recent_dispatches(list, now);
    }
    let mut out = list.to_vec();
    out.sort_by(|a, b| b.id.cmp(&a.id));
    out.truncate(count.parse().unwrap_or(0));
    out
}
