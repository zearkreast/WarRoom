mod common;
use common::*;
use std::collections::HashMap;
use warroom_core::api::{Faction, Planet, Position};
use warroom_core::planets::{is_off_map, is_placeholder, ownership_key};

#[test]
fn fixtures_have_gaps_in_planet_indices() {
    assert!(planets().iter().enumerate().any(|(i, p)| p.index != i as i64));
}

#[test]
fn drops_placeholder_worlds_stacked_on_super_earth() {
    assert!(!is_placeholder(0, Position { x: 0.0, y: 0.0 }));
    assert!(is_placeholder(265, Position { x: 0.0, y: 0.0 }));
    assert!(!is_placeholder(3, Position { x: 0.1, y: 0.0 }));
}

#[test]
fn files_meridia_under_celeste_and_puts_sangis_and_alderidge_cove_back() {
    let planets = fixed_planets();
    let by_name: HashMap<&str, &Planet> = planets.iter().map(|p| (p.name.as_str(), p)).collect();
    assert_eq!(by_name["MERIDIA"].sector, "Celeste");
    assert_eq!((by_name["SANGIS"].index, by_name["SANGIS"].sector.as_str()), (273, "Guang"));
    assert_eq!((by_name["ALDERIDGE COVE"].index, by_name["ALDERIDGE COVE"].sector.as_str()), (99, "Omega"));
}

#[test]
fn only_changes_the_ownership_key_when_an_owner_or_an_attack_changes() {
    let planets = fixed_planets();
    let key = ownership_key(&planets);
    let healed: Vec<Planet> = planets
        .iter()
        .cloned()
        .map(|mut p| {
            p.health = 1.0;
            p
        })
        .collect();
    assert_eq!(ownership_key(&healed), key);
    let mut flipped = planets.clone();
    flipped[3].current_owner = Faction::Automaton;
    assert_ne!(ownership_key(&flipped), key);
}

#[test]
fn drops_calypso_which_the_game_removed_but_the_api_still_lists() {
    let planets = planets();
    let calypso = planets.iter().find(|p| p.name == "CALYPSO").unwrap();
    assert!(is_off_map(calypso.index, calypso.position));
    let fixed = fixed_planets();
    assert!(!is_off_map(fixed[1].index, fixed[1].position));
}
