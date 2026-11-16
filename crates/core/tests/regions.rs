mod common;
use common::*;
use std::collections::HashSet;
use warroom_core::api::{Faction, PlanetEvent};
use warroom_core::galaxy::{GALAXY_SCALE, RINGS, ring_radius};
use warroom_core::regions::{build_attack_regions, build_regions, chaikin};

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
    let quiet = build_attack_regions(&planets);
    assert!(quiet.iter().all(|r| r.contested.is_empty()));
    let target =
        planets.iter().position(|p| p.current_owner == Faction::Humans && p.index != 0 && p.position.x != 0.0).unwrap();
    planets[target].event = Some(PlanetEvent { faction: Faction::Automaton, ..Default::default() });
    let index = planets[target].index;
    assert!(build_regions(&planets).iter().all(|r| !r.planets.contains(&planets[target].index)));
    let regions = build_attack_regions(&planets);
    let bots: Vec<_> = regions.iter().filter(|r| r.owner == Faction::Automaton).collect();
    assert!(bots.iter().any(|r| r.planets.contains(&index)));
    assert!(bots.iter().any(|r| r.contested.iter().any(|c| c.holds(index) && c.path.starts_with('M'))));
    assert!(regions.iter().filter(|r| r.owner != Faction::Automaton).all(|r| r.contested.is_empty()));
}

fn points(path: &str) -> Vec<[f64; 2]> {
    path.split(['M', 'L', 'Z', ' '])
        .filter_map(|xy| xy.split_once(','))
        .map(|(x, y)| [x.parse().unwrap(), y.parse().unwrap()])
        .collect()
}

#[test]
fn keeps_every_outline_inside_the_galaxys_rim() {
    let rim = ring_radius(RINGS - 1) * GALAXY_SCALE;
    let regions = build_attack_regions(&planets());
    let far = regions.iter().flat_map(|r| points(&r.path)).map(|[x, y]| x.hypot(y)).fold(0.0, f64::max);
    assert!(far <= rim + 0.1, "{far} > {rim}");
    // Outer regions still reach out to it rather than stopping at their planets.
    assert!(far > rim - 15.0, "{far}");
}

#[test]
fn draws_a_planet_under_siege_as_ground_of_its_own_hatched_in_super_earth_blue() {
    let mut planets = planets();
    let target = planets.iter().find(|p| p.current_owner != Faction::Humans && !p.waypoints.is_empty()).unwrap().index;
    let around: Vec<i64> = warroom_core::sieges::neighbours(&planets)[&target].clone();
    for p in &mut planets {
        if around.contains(&p.index) {
            p.current_owner = Faction::Humans;
            p.event = None;
        }
    }
    let regions = build_attack_regions(&planets);
    let home = regions.iter().find(|r| r.planets.contains(&target)).unwrap();
    // Cut off from its faction's ground: a region of its own.
    assert_eq!(home.planets, [target]);
    assert_eq!(home.owner, planets.iter().find(|p| p.index == target).unwrap().current_owner);
    let [patch] = home.besieged.as_slice() else { panic!("{:?}", home.besieged) };
    let [spot] = patch.spots.as_slice() else { panic!("{:?}", patch.spots) };
    assert_eq!(spot.planet, target);
    assert!(spot.reach > 0.0);
    assert!(regions.iter().flat_map(|r| &r.besieged).all(|b| b.spots.iter().all(|s| s.planet == target)));
}

#[test]
fn neighbouring_planets_under_the_same_attack_share_one_hatched_patch() {
    let mut planets = planets();
    // Three Super Earth worlds next to each other, each a neighbour of the next.
    let lanes = warroom_core::sieges::neighbours(&planets);
    let human = |i: i64| planets.iter().any(|p| p.index == i && p.current_owner == Faction::Humans && p.index != 0);
    let chain = planets
        .iter()
        .filter(|p| human(p.index))
        .find_map(|a| {
            let b = lanes[&a.index].iter().copied().find(|&b| human(b))?;
            let c = lanes[&b].iter().copied().find(|&c| c != a.index && human(c))?;
            Some([a.index, b, c])
        })
        .unwrap();
    for p in &mut planets {
        if chain.contains(&p.index) {
            p.event = Some(PlanetEvent { faction: Faction::Terminids, ..Default::default() });
        }
    }
    let regions = build_attack_regions(&planets);
    let patches: Vec<_> = regions
        .iter()
        .filter(|r| r.owner == Faction::Terminids)
        .flat_map(|r| &r.contested)
        .filter(|c| chain.iter().any(|&i| c.holds(i)))
        .collect();
    assert_eq!(patches.len(), 1, "{patches:?}");
    assert_eq!(patches[0].spots.len(), 3);
    assert!(patches[0].path.starts_with('M'));
}

#[test]
fn far_apart_planets_under_attack_keep_patches_of_their_own() {
    let mut planets = planets();
    let lanes = warroom_core::sieges::neighbours(&planets);
    let human: Vec<i64> =
        planets.iter().filter(|p| p.current_owner == Faction::Humans && p.index != 0).map(|p| p.index).collect();
    let a = human[0];
    let pos = |i: i64| planets.iter().find(|p| p.index == i).unwrap().position;
    let b = *human
        .iter()
        .filter(|&&b| b != a && !lanes[&a].contains(&b))
        .max_by(|&&x, &&y| {
            let d = |i: i64| (pos(i).x - pos(a).x).hypot(pos(i).y - pos(a).y);
            d(x).total_cmp(&d(y))
        })
        .unwrap();
    for p in &mut planets {
        if p.index == a || p.index == b {
            p.event = Some(PlanetEvent { faction: Faction::Illuminate, ..Default::default() });
        }
    }
    let patches: Vec<_> = build_attack_regions(&planets).into_iter().flat_map(|r| r.contested).collect();
    assert_eq!(patches.len(), 2, "{patches:?}");
    assert!(patches.iter().all(|p| p.spots.len() == 1));
}
