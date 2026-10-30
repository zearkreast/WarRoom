mod common;

use common::*;
use warroom_core::api::PlanetEvent;
use warroom_core::api::{Faction, Region};
use warroom_core::cities::{CityState, bar_marks, cities_of, city_stars, size_label};

fn region(id: i64, name: &str, size: &str) -> Region {
    Region { id, name: name.into(), max_health: 400000.0, size: size.into(), ..Default::default() }
}

#[test]
fn marks_size_classes_with_one_to_four_stars() {
    let stars: Vec<usize> =
        ["Settlement", "Town", "City", "MegaCity", "Megafactory"].iter().map(|s| city_stars(s)).collect();
    assert_eq!(stars, [1, 2, 3, 4, 0]);
    assert_eq!(size_label("MegaCity"), "Mega City");
}

#[test]
fn tells_super_earths_capitals_apart_by_what_the_invasion_left_of_them() {
    let mut p = planets().into_iter().find(|p| p.index == 0).unwrap();
    let names = [
        "EAGLEOPOLIS",
        "ADMINISTRATIVE CENTER 02",
        "REMEMBRANCE",
        "YORK SUPREME",
        "PORT MERCY",
        "PROSPERITY CITY",
        "EQUALITY-ON-SEA",
    ];
    p.regions = names.iter().enumerate().map(|(i, n)| region(i as i64, n, "MegaCity")).collect();
    let got: Vec<(String, CityState, Option<String>)> =
        cities_of(&p).into_iter().map(|c| (c.name, c.state, c.formerly)).collect();
    use CityState::*;
    let want = [
        ("EAGLEOPOLIS", Destroyed, None),
        ("ADMINISTRATIVE CENTER 02", Destroyed, None),
        ("GUN", Rebuilt, Some("REMEMBRANCE")),
        ("NEW YORK SUPREME", Rebuilt, Some("YORK SUPREME")),
        ("PORT MERCY", Destroyed, None),
        ("PROSPERITY CITY", Intact, None),
        ("EQUALITY-ON-SEA", Intact, None),
    ];
    let want: Vec<(String, CityState, Option<String>)> =
        want.iter().map(|(n, s, f)| (n.to_string(), *s, f.map(String::from))).collect();
    assert_eq!(got, want);
}

#[test]
fn counts_an_enemy_held_citys_liberation_down_from_its_health() {
    let mut p = planets().into_iter().find(|p| p.current_owner != Faction::Humans && p.index != 0).unwrap();
    p.event = None;
    p.regions = vec![
        Region { health: Some(100000.0), is_available: true, players: 40.0, ..region(0, "OPEN", "City") },
        Region { health: Some(200000.0), max_health: 200000.0, ..region(1, "SHUT", "Town") },
        Region { description: "null".into(), ..region(2, "QUIET", "Settlement") },
    ];
    let got: Vec<(CityState, Option<f64>, String)> =
        cities_of(&p).into_iter().map(|c| (c.state, c.percent, c.description)).collect();
    assert_eq!(
        got,
        [
            (CityState::Liberating, Some(75.0), String::new()),
            (CityState::Locked, Some(0.0), String::new()),
            (CityState::Held, None, String::new()),
        ]
    );
}

fn gated(id: i64, factor: f64, open: bool) -> Region {
    Region { health: Some(400000.0), availability_factor: Some(factor), is_available: open, ..region(id, "C", "City") }
}

#[test]
fn every_closed_city_with_a_threshold_says_where_it_opens() {
    let mut p = planets().into_iter().find(|p| p.current_owner != Faction::Humans && p.index != 0).unwrap();
    p.event = None;
    p.max_health = 1_000_000.0;
    p.health = 600_000.0;
    p.regions = vec![
        gated(0, 1.0, true),
        gated(1, 0.75, false),
        gated(2, 0.5, false),
        gated(3, 0.0, true),
        gated(4, 1.0, false),
        Region { health: None, ..gated(5, 0.2, false) },
    ];
    let got: Vec<(Option<f64>, bool, CityState)> =
        cities_of(&p).into_iter().map(|c| (c.opens_at, c.waiting, c.state)).collect();
    assert_eq!(
        got,
        [
            (Some(0.0), false, CityState::Liberating),
            // Past the planet's 40% but still closed: it keeps its threshold rather than a bare Locked.
            (Some(25.0), true, CityState::Locked),
            (Some(50.0), true, CityState::Locked),
            // A special site outside the opening order has no threshold, not 100%.
            (None, false, CityState::Liberating),
            (Some(0.0), false, CityState::Locked),
            (Some(80.0), true, CityState::Held),
        ]
    );
}

#[test]
fn a_defended_planets_cities_stay_super_earths_and_leave_the_bar_unmarked() {
    let mut p = planets().into_iter().find(|p| p.current_owner == Faction::Humans && p.index != 0).unwrap();
    p.event = Some(PlanetEvent { faction: Faction::Automaton, max_health: 1.0, ..Default::default() });
    p.regions = vec![gated(0, 1.0, false), gated(1, 0.5, false)];
    let got: Vec<(CityState, bool, Faction)> =
        cities_of(&p).into_iter().map(|c| (c.state, c.waiting, c.holder)).collect();
    assert_eq!(got, [(CityState::Secured, false, Faction::Humans), (CityState::Secured, false, Faction::Humans)]);
    assert!(bar_marks(&p, 8.0).is_empty());
}

#[test]
fn bar_marks_keep_their_threshold_and_stack_when_close() {
    let mut p = planets().into_iter().find(|p| p.current_owner != Faction::Humans && p.index != 0).unwrap();
    p.event = None;
    p.regions = vec![
        gated(0, 0.6, false),
        gated(1, 1.0, true),
        gated(2, 0.62, false),
        gated(3, 0.3, false),
        gated(4, 0.0, true),
    ];
    let got: Vec<(i64, i64, usize)> =
        bar_marks(&p, 8.0).into_iter().map(|m| (m.city.id, (m.at * 10.0).round() as i64, m.row)).collect();
    assert_eq!(got, [(1, 0, 0), (2, 380, 0), (0, 400, 1), (3, 700, 0)]);
}
