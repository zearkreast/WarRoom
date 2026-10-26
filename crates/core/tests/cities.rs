mod common;

use common::*;
use warroom_core::api::{Faction, Region};
use warroom_core::cities::{CityState, cities_of, city_stars, size_label};

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
