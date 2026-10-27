mod common;
use common::*;
use warroom_core::format::{MarkupSpan, dispatch_title, fmt_duration, parse_markup, sest_date, war_day};
use warroom_core::time::utc_ms;

#[test]
fn formats_durations() {
    assert_eq!(fmt_duration(47.0 * H + 19.0 * 60_000.0), "47h 19m");
    assert_eq!(fmt_duration(5.0 * 24.0 * H), "5d 0h");
    assert_eq!(fmt_duration(-1.0), "now");
}

#[test]
fn splits_highlighted_spans() {
    let spans = parse_markup("<i=3>NEW MAJOR ORDER</i>\n\nGo to <i=1>Fury</i>.");
    let span = |text: &str, style: Option<i64>| MarkupSpan { text: text.into(), style };
    assert_eq!(
        spans,
        [span("NEW MAJOR ORDER", Some(3)), span("\n\nGo to ", None), span("Fury", Some(1)), span(".", None)]
    );
    assert_eq!(dispatch_title("<i=3>STRATEGIC ALERT</i>\n\nBody"), "STRATEGIC ALERT");
}

#[test]
fn runs_the_calendar_160_years_ahead_in_utc() {
    assert_eq!(sest_date(utc_ms(2026, 10, 23, 1, 0, 0, 0)), "23/10/2186");
}

#[test]
fn counts_war_days_from_the_start_day() {
    assert_eq!(war_day("2024-02-08T00:00:00Z", utc_ms(2024, 2, 8, 12, 0, 0, 0)), 1);
    assert_eq!(war_day("2024-02-08T00:00:00Z", utc_ms(2024, 2, 10, 12, 0, 0, 0)), 3);
}
