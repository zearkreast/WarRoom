use std::collections::HashSet;
use warroom_core::api::{Faction, Planet};
use warroom_core::sieges::{circumvallation, neighbours};

fn planet(index: i64, owner: Faction, waypoints: &[i64], max_health: f64, regen_pct: f64) -> Planet {
    Planet {
        index,
        name: format!("P{index}"),
        waypoints: waypoints.to_vec(),
        max_health,
        health: max_health,
        disabled: false,
        current_owner: owner,
        regen_per_second: max_health * regen_pct / 360_000.0,
        event: None,
        ..Default::default()
    }
}

fn p(index: i64, owner: Faction, waypoints: &[i64]) -> Planet {
    planet(index, owner, waypoints, 1_000_000.0, 1.0)
}

fn ring(planets: &[Planet], front: i64, fronts: &[i64]) -> Option<warroom_core::sieges::Circumvallation> {
    let f = planets.iter().find(|p| p.index == front).unwrap();
    circumvallation(f, planets, &neighbours(planets), &fronts.iter().copied().collect::<HashSet<_>>())
}

fn indices(list: &[Planet]) -> Vec<i64> {
    let mut v: Vec<i64> = list.iter().map(|p| p.index).collect();
    v.sort_unstable();
    v
}

#[test]
fn finds_the_planet_a_front_would_leave_surrounded_and_says_when_taking_it_directly_is_quicker() {
    // Zea Rugosia (1.7M, 2.94%/h) would ring Haldus (1M, 1.5%/h): Haldus is the cheaper fight.
    let planets = [
        planet(1, Faction::Illuminate, &[2, 3], 1_700_000.0, 2.94),
        planet(2, Faction::Illuminate, &[4], 1_000_000.0, 1.5),
        p(3, Faction::Humans, &[]),
        p(4, Faction::Humans, &[]),
    ];
    let c = ring(&planets, 1, &[1, 2]).unwrap();
    assert_eq!(indices(&c.targets), [2]);
    assert!(!c.worth);
    assert_eq!(indices(&c.direct), [2]);
}

#[test]
fn counts_two_sieges_at_once_and_a_ring_is_worth_closing_when_the_targets_cost_more_together() {
    // RD-4 rings both Rogue 5 and Hesoe Prime.
    let planets = [
        p(1, Faction::Illuminate, &[2, 3, 4]),
        planet(2, Faction::Illuminate, &[5], 1_000_000.0, 1.0),
        planet(3, Faction::Illuminate, &[5], 1_000_000.0, 4.0),
        p(4, Faction::Humans, &[]),
        p(5, Faction::Humans, &[]),
    ];
    let c = ring(&planets, 1, &[1, 2, 3]).unwrap();
    assert_eq!(indices(&c.targets), [2, 3]);
    assert!(c.worth);
}

#[test]
fn needs_a_planet_on_its_own_a_pair_cut_off_together_is_no_siege() {
    let planets = [
        p(1, Faction::Automaton, &[2]),
        p(2, Faction::Automaton, &[3]),
        p(3, Faction::Automaton, &[4]),
        p(4, Faction::Humans, &[]),
    ];
    assert!(ring(&planets, 1, &[1]).is_none());
}

#[test]
fn is_worth_it_when_the_surrounded_planet_has_no_campaign_of_its_own() {
    let planets = [
        planet(1, Faction::Terminids, &[2], 3_000_000.0, 3.33),
        planet(2, Faction::Terminids, &[3], 2_000_000.0, 5.0),
        p(3, Faction::Humans, &[]),
    ];
    let c = ring(&planets, 1, &[1]).unwrap();
    assert!(c.direct.is_empty());
    assert!(c.worth);
}

#[test]
fn only_applies_to_enemy_planets() {
    let planets = [p(1, Faction::Humans, &[2]), p(2, Faction::Automaton, &[])];
    assert!(ring(&planets, 1, &[1]).is_none());
}
