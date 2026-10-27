mod common;
use common::*;
use warroom_core::api::Dispatch;
use warroom_core::dispatch::{DispatchKind, dispatch_body, dispatch_kind, latest_dispatches, recent_dispatches};
use warroom_core::time::iso_string;

#[test]
fn keeps_dispatches_from_the_last_72_hours_newest_first() {
    let now = ms("2026-10-23T12:00:00Z");
    let at = |h: f64, id: i64| Dispatch { id, kind: 0, message: String::new(), published: iso_string(now - h * H) };
    let ids: Vec<i64> = recent_dispatches(&[at(80.0, 1), at(70.0, 2), at(1.0, 3)], now).iter().map(|d| d.id).collect();
    assert_eq!(ids, [3, 2]);
}

#[test]
fn categorises_dispatches_by_headline() {
    assert_eq!(dispatch_kind("<i=3>NEW MAJOR ORDER</i>\nLiberate X."), DispatchKind::Order);
    assert_eq!(dispatch_kind("<i=3>MAJOR ORDER WON</i>\nWell done."), DispatchKind::Victory);
    assert_eq!(dispatch_kind("<i=3>STRATEGIC ALERT</i>\nBots on Y."), DispatchKind::Alert);
    assert_eq!(dispatch_kind("<i=3>CONTROL CENTER ADVISORY</i>\nNote."), DispatchKind::Advisory);
    assert_eq!(dispatch_kind("Something happened."), DispatchKind::News);
}

#[test]
fn drops_the_headline_from_the_body() {
    let body = dispatch_body("<i=3>NEW MAJOR ORDER</i>\nLiberate <i=1>FURY</i>.");
    assert_eq!(body.iter().map(|s| s.text.as_str()).collect::<String>(), "Liberate FURY.");
}

#[test]
fn lists_the_newest_few_whatever_their_age_or_the_last_72_hours() {
    let now = ms("2026-10-24T12:00:00Z");
    let d = |id: i64, days_ago: f64| Dispatch {
        id,
        published: iso_string(now - days_ago * 86_400_000.0),
        kind: 0,
        message: "x".into(),
    };
    let list = [d(1, 9.0), d(2, 6.0), d(3, 5.0), d(4, 0.5)];
    let ids = |count| latest_dispatches(&list, count, now).iter().map(|x| x.id).collect::<Vec<_>>();
    assert_eq!(ids("3"), [4, 3, 2]);
    assert_eq!(ids("72h"), [4]);
}
