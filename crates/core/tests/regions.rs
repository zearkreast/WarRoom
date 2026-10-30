mod common;
use common::*;
use std::collections::HashSet;
use warroom_core::api::{Faction, PlanetEvent};
use warroom_core::regions::{build_regions, chaikin};

#[test]
fn chaikin_doubles_the_points_each_pass() {
    assert_eq!(chaikin(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]], 2).len(), 12);
}

#[test]
fn outlines_enemy_territory_only() {
    let planets = planets();
    let regions = build_regions(&planets);
    let owners: HashSet<Faction> = regions.iter().map(|r| r.owner).collect();
    assert!(!owners.contains(&Faction::Humans));
    assert!(regions.iter().all(|r| r.path.starts_with('M')));
    let enemies: HashSet<Faction> =
        planets.iter().filter(|p| p.current_owner != Faction::Humans).map(|p| p.current_owner).collect();
    assert_eq!(owners, enemies);
}

#[test]
fn a_planet_under_attack_joins_its_attackers_territory_hatched() {
    let mut planets = planets();
    let quiet = build_regions(&planets);
    assert!(quiet.iter().all(|r| r.contested.is_empty()));
    let target =
        planets.iter().position(|p| p.current_owner == Faction::Humans && p.index != 0 && p.position.x != 0.0).unwrap();
    planets[target].event = Some(PlanetEvent { faction: Faction::Automaton, ..Default::default() });
    let index = planets[target].index;
    let regions = build_regions(&planets);
    let bots: Vec<_> = regions.iter().filter(|r| r.owner == Faction::Automaton).collect();
    assert!(bots.iter().any(|r| r.planets.contains(&index)));
    assert!(bots.iter().any(|r| r.contested.starts_with('M')));
    assert!(regions.iter().filter(|r| r.owner != Faction::Automaton).all(|r| r.contested.is_empty()));
}
