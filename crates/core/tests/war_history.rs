mod common;
use common::*;
use warroom_core::api::Faction;
use warroom_core::owner_changes::OwnerChange;
use warroom_core::war_history::{HistoryInput, LogEntry, build_war_history, defense_events, fmt_long_duration};

const D: f64 = 24.0 * H;

fn changes() -> Vec<OwnerChange> {
    let c = |from, to, at| OwnerChange { planet: 1, planet_name: "X".into(), from, to, at };
    vec![
        c(Faction::Automaton, Faction::Humans, 2.0 * D),
        c(Faction::Humans, Faction::Automaton, 55.0 * D),
        // After now: still counted by time order.
        c(Faction::Automaton, Faction::Humans, 108.0 * D),
    ]
}

#[test]
fn splits_the_record_into_owner_segments_and_counts_captures() {
    let changes = changes();
    let h = build_war_history(HistoryInput {
        changes: &changes[..2],
        defenses: &[],
        current_owner: Faction::Automaton,
        record_start: 0.0,
        now: 100.0 * D,
    });
    let owners: Vec<_> = h.segments.iter().map(|s| s.owner).collect();
    assert_eq!(owners, [Faction::Automaton, Faction::Humans, Faction::Automaton]);
    assert_eq!((h.liberated, h.lost), (1, 1));
    assert!((h.held_free_pct - 53.0).abs() < 0.5);
}

#[test]
fn numbers_battles_in_time_order_and_ends_the_log_with_the_record_start() {
    let changes = changes();
    let now = 100.0 * D;
    let defenses = defense_events(&[50.0 * D, 50.0 * D + H, 54.0 * D], &changes, None, now);
    let h = build_war_history(HistoryInput {
        changes: &changes[..2],
        defenses: &defenses,
        current_owner: Faction::Automaton,
        record_start: 0.0,
        now,
    });
    assert!(matches!(h.log.last(), Some(LogEntry::Start { owner: Faction::Automaton, .. })));
    let battles: Vec<usize> = h
        .log
        .iter()
        .filter_map(|e| match e {
            LogEntry::Change { battle, .. } | LogEntry::Defense { battle, .. } => Some(*battle),
            LogEntry::Start { .. } => None,
        })
        .collect();
    let mut sorted = battles.clone();
    sorted.sort_by(|a, b| b.cmp(a));
    assert_eq!(battles, sorted);
    assert!(matches!(h.log[0], LogEntry::Change { to: Faction::Automaton, .. }));
}

#[test]
fn groups_defending_samples_into_events_and_marks_the_lost_one() {
    // The planet falls the moment its last defense ends, just after the last defending sample.
    let events = defense_events(&[10.0 * D, 10.0 * D + H, 54.0 * D, 55.0 * D - 60_000.0], &changes(), None, 100.0 * D);
    assert_eq!(events.len(), 3);
    assert!(events[0].held && events[1].held && !events[2].held);
}

#[test]
fn marks_the_latest_event_ongoing_while_the_planet_is_under_attack() {
    let now = 100.0 * D;
    let events = defense_events(&[now - H], &[], Some(now + 5.0 * H), now);
    assert!(events[0].ongoing);
}

#[test]
fn formats_long_durations() {
    assert!(fmt_long_duration(3_551_077_816.0).starts_with("1C 12Y"));
    assert_eq!(fmt_long_duration(12.0 * 3600.0), "12H");
    assert_eq!(fmt_long_duration(0.0), "0H");
}
