use warroom_core::api::Faction;
use warroom_core::away::{AWAY_MIN_MS, captures_since, is_away, name_list, order_moves};
use warroom_core::owner_changes::OwnerChange;
use warroom_core::snapshot::OrderSample;

fn change(planet: i64, name: &str, at: f64, to: Faction) -> OwnerChange {
    let from = if to == Faction::Humans { Faction::Automaton } else { Faction::Humans };
    OwnerChange { planet, planet_name: name.into(), from, to, at }
}

#[test]
fn briefs_only_after_a_real_break() {
    assert!(!is_away(0.0, 1e12));
    assert!(!is_away(1000.0, 1000.0 + AWAY_MIN_MS - 1.0));
    assert!(is_away(1000.0, 1000.0 + AWAY_MIN_MS));
}

#[test]
fn counts_each_planet_by_where_it_ended_up() {
    let changes = [
        change(1, "FURY", 5.0, Faction::Humans),
        change(2, "ESKER", 20.0, Faction::Humans),
        change(2, "ESKER", 30.0, Faction::Terminids),
        change(3, "MYRIUM", 25.0, Faction::Humans),
    ];
    let c = captures_since(&changes, 10.0);
    assert_eq!(c.liberated, ["MYRIUM"]);
    assert_eq!(c.lost, ["ESKER"]);
}

#[test]
fn measures_each_objective_from_the_first_reading_since() {
    let samples = [
        OrderSample { id: 7, at: 5.0, progress: vec![10.0, 0.0] },
        OrderSample { id: 7, at: 12.0, progress: vec![40.0, 1.0] },
        OrderSample { id: 7, at: 20.0, progress: vec![60.0, 1.0] },
        OrderSample { id: 8, at: 11.0, progress: vec![0.0] },
    ];
    assert_eq!(order_moves(&samples, 7, &[90.0, 1.0], 10.0), [Some(50.0), Some(0.0)]);
    assert_eq!(order_moves(&samples, 7, &[90.0], 30.0), [None]);
}

#[test]
fn lists_names_in_a_sentence() {
    let n = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert_eq!(name_list(&n(&["A"]), 3), "A");
    assert_eq!(name_list(&n(&["A", "B", "C"]), 3), "A, B and C");
    assert_eq!(name_list(&n(&["A", "B", "C", "D", "E"]), 3), "A, B, C and 2 more");
}
