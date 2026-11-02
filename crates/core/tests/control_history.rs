use std::collections::HashMap;
use warroom_core::api::Faction;
use warroom_core::control_history::{DAY_MS, control_by_day, front_tally};
use warroom_core::owner_changes::OwnerChange;

fn change(planet: i64, at: f64, from: Faction, to: Faction) -> OwnerChange {
    OwnerChange { planet, planet_name: String::new(), from, to, at }
}

#[test]
fn works_back_from_today_through_each_change() {
    let now = 10.0 * DAY_MS + 3_600_000.0;
    let owners = HashMap::from([(1, Faction::Humans), (2, Faction::Terminids), (3, Faction::Automaton)]);
    let changes = [
        // Planet 1 was liberated today; planet 2 fell two days ago.
        change(1, 10.0 * DAY_MS + 60_000.0, Faction::Automaton, Faction::Humans),
        change(2, 8.0 * DAY_MS + 60_000.0, Faction::Humans, Faction::Terminids),
        change(99, 9.0 * DAY_MS, Faction::Humans, Faction::Illuminate),
    ];
    let days = control_by_day(&owners, &changes, 4, now);
    assert_eq!(days.iter().map(|d| d.at).collect::<Vec<_>>(), [8.0 * DAY_MS, 9.0 * DAY_MS, 10.0 * DAY_MS, now]);
    assert_eq!(
        days.iter().map(|d| d.held).collect::<Vec<_>>(),
        [[1, 0, 2, 0], [0, 1, 2, 0], [0, 1, 2, 0], [1, 1, 1, 0]]
    );
}

#[test]
fn tallies_each_enemy_front() {
    let changes = [
        change(1, 5.0, Faction::Automaton, Faction::Humans),
        change(2, 6.0, Faction::Humans, Faction::Terminids),
        change(3, 7.0, Faction::Terminids, Faction::Humans),
        change(4, 1.0, Faction::Illuminate, Faction::Humans),
    ];
    assert_eq!(front_tally(&changes, 2.0), [(1, 1), (1, 0), (0, 0)]);
}
