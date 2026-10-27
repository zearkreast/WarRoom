mod common;
use common::*;
use std::collections::HashSet;
use warroom_core::api::Faction;
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
