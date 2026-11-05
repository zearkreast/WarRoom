mod common;
use common::*;
use std::collections::HashMap;
use warroom_core::api::{Faction, Named, Planet, Position};
use warroom_core::planets::{fix_statics, is_off_map, is_placeholder, ownership_key, split_unofficial};
use warroom_core::snapshot::PlanetStatic;

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

fn with_biome(index: i64, biome: &str, text: &str) -> PlanetStatic {
    PlanetStatic { index, biome: Named { name: biome.into(), description: text.into() }, ..Default::default() }
}

#[test]
fn a_swapped_biome_takes_the_apis_current_text() {
    let list =
        vec![with_biome(99, "Deciduous Autumn Forest", "Autumn woods."), with_biome(273, "Deadlands", "Grey wastes.")];
    let fixed = fix_statics(list);
    assert_eq!((fixed[0].biome.name.as_str(), fixed[0].biome.description.as_str()), ("Deadlands", "Grey wastes."));
    assert_eq!(
        (fixed[1].biome.name.as_str(), fixed[1].biome.description.as_str()),
        ("Deciduous Autumn Forest", "Autumn woods.")
    );
}

#[test]
fn a_biome_the_api_already_has_right_keeps_its_own_text() {
    let fixed = fix_statics(vec![
        with_biome(273, "Deciduous Autumn Forest", "Official woods."),
        with_biome(272, "Deciduous Autumn Forest", "Official woods."),
    ]);
    assert_eq!(fixed[0].biome.description, "Official woods.");
}

#[test]
fn keeps_its_own_copy_when_no_planet_has_the_biome() {
    let fixed = fix_statics(vec![with_biome(273, "Deadlands", "")]);
    assert_eq!(fixed[0].biome.name, "Deciduous Autumn Forest");
    assert!(fixed[0].biome.description.starts_with("Broadleaf woodlands"));
}

#[test]
fn splits_the_unofficial_mark_wherever_and_however_it_is_written() {
    let beach = "Rolling dunes across a beatiful beach world.";
    assert_eq!(split_unofficial(&format!("{beach} (UNOFFICIAL)")), (beach.to_string(), true));
    assert_eq!(split_unofficial(&format!("[Unofficial] {beach}")), (beach.to_string(), true));
    assert_eq!(split_unofficial(&format!("{beach} ( unofficial )")), (beach.to_string(), true));
    assert_eq!(split_unofficial("(UNOFFICIAL)"), (String::new(), true));
}

#[test]
fn leaves_official_text_alone() {
    let text = "Frequent earthquakes stun players and enemies alike.";
    assert_eq!(split_unofficial(text), (text.to_string(), false));
    assert_eq!(split_unofficial("Unofficial reports say so."), ("Unofficial reports say so.".to_string(), false));
    assert_eq!(split_unofficial(""), (String::new(), false));
}
