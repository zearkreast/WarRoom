mod common;
use common::*;
use warroom_core::api::Faction;
use warroom_core::format::fmt_age_short;
use warroom_core::owner_changes::OwnerChange;
use warroom_core::ticker::{TickerKind, ticker_items};

#[test]
fn ticker_shows_the_last_day_newest_first_with_captures_and_losses() {
    let now = ms("2026-10-23T12:00:00Z");
    let d = |id: i64, hours_ago: f64| dispatch(id, now - hours_ago * H, &format!("<i=3>NEWS {id}</i>\n\nBody"));
    let change = OwnerChange {
        planet: 7,
        planet_name: "Fury".into(),
        from: Faction::Automaton,
        to: Faction::Humans,
        at: now - 1_800_000.0,
    };
    let items = ticker_items(&(1..=6).map(|i| d(i, i as f64)).collect::<Vec<_>>(), &[change], &[], now);
    assert_eq!(
        (items[0].kind, items[0].text.as_str(), items[0].planet),
        (TickerKind::Liberated, "Fury liberated", Some(7))
    );
    assert_eq!(items.len(), 7);
}

#[test]
fn ticker_tops_a_quiet_day_up_with_older_news() {
    let now = ms("2026-10-23T12:00:00Z");
    let d = |id: i64, hours_ago: f64| dispatch(id, now - hours_ago * H, &format!("<i=3>NEWS {id}</i>\n\nBody"));
    let items = ticker_items(&[d(1, 30.0), d(2, 50.0), d(3, 2.0)], &[], &[], now);
    assert_eq!(items.iter().map(|i| i.text.as_str()).collect::<Vec<_>>(), ["NEWS 3", "NEWS 1", "NEWS 2"]);
    assert_eq!(fmt_age_short(now - 90.0 * 60_000.0, now), "1h");
}
