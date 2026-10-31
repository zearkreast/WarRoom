mod common;

use common::*;
use warroom_core::api::{Faction, Planet, PlanetEvent, Region};
use warroom_core::cities::{CityProgress, CityState, bar_marks, cities_of, city_liberations, city_stars, size_label};
use warroom_core::progress::Sample;

const MINUTE: f64 = 60_000.0;

fn region(id: i64, name: &str, size: &str) -> Region {
    Region { id, name: name.into(), max_health: 400000.0, size: size.into(), ..Default::default() }
}

/// Readings 15 minutes apart.
fn readings(owner: Faction, defending: bool, percents: &[f64]) -> Vec<Sample> {
    percents
        .iter()
        .enumerate()
        .map(|(i, percent)| Sample {
            planet: 1,
            at: i as f64 * 15.0 * MINUTE,
            owner,
            percent: *percent,
            players: 0.0,
            defending,
        })
        .collect()
}

/// A liberation front of 3,200,000 health, whose two 200,000 towns each deal 9.375 points.
fn twin_towns() -> Planet {
    Planet {
        index: 7,
        max_health: 3_200_000.0,
        current_owner: Faction::Automaton,
        regions: vec![
            Region { max_health: 200_000.0, ..region(0, "NORTH", "Town") },
            Region { max_health: 200_000.0, ..region(1, "SOUTH", "Town") },
        ],
        ..Default::default()
    }
}

/// Varylia 5's defense of 30 Oct: Sequim's 400,000 health against an event of 1,900,000.
fn varylia() -> Planet {
    Planet {
        index: 244,
        max_health: 1_400_000.0,
        event: Some(PlanetEvent { max_health: 1_900_000.0, ..Default::default() }),
        regions: vec![region(0, "SEQUIM", "City")],
        ..Default::default()
    }
}

#[test]
fn names_the_city_behind_a_jump_in_a_defense() {
    let samples = readings(Faction::Humans, true, &[43.8221, 44.2275, 44.6501, 76.6508, 77.0856, 77.4899]);
    let got = city_liberations(&varylia(), &samples, &[]);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].cities, ["SEQUIM"]);
    assert_eq!(got[0].stars, 3);
    assert_eq!(got[0].at, samples[3].at);
    assert_eq!(got[0].percent, 76.6508);
    assert!(got[0].defending);
    assert!((got[0].boost - 31.5789).abs() < 0.001, "{}", got[0].boost);
}

#[test]
fn names_the_city_behind_a_jump_in_a_liberation() {
    let p = Planet {
        index: 158,
        max_health: 3_200_000.0,
        current_owner: Faction::Automaton,
        regions: vec![
            Region { max_health: 200_000.0, ..region(0, "HILJAISUUS", "Town") },
            region(1, "KALASATAMA", "City"),
            Region { max_health: 600_000.0, ..region(2, "YDINKESKUSTA", "MegaCity") },
            Region { max_health: 1_000_000.0, ..region(3, "EVOLUTION PLATS", "Settlement") },
        ],
        ..Default::default()
    };
    let samples = readings(Faction::Automaton, false, &[4.71966, 4.79678, 4.888, 4.95872, 33.156, 33.2442, 33.34]);
    let got = city_liberations(&p, &samples, &[]);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].cities, ["YDINKESKUSTA"]);
    assert_eq!(got[0].stars, 4);
    assert!(!got[0].defending);
}

#[test]
fn counts_a_citys_boost_against_the_front_it_is_on() {
    let defended = &cities_of(&varylia())[0];
    assert!(defended.defense);
    assert!((defended.boost.unwrap() - 31.5789).abs() < 0.001, "{:?}", defended.boost);

    let at_peace = &cities_of(&Planet { event: None, ..varylia() })[0];
    assert!(!at_peace.defense);
    assert!((at_peace.boost.unwrap() - 42.8571).abs() < 0.001, "{:?}", at_peace.boost);
}

#[test]
fn marks_no_step_a_city_does_not_explain() {
    let p = varylia();
    let calm = readings(Faction::Humans, true, &[40.0, 40.4, 40.8, 41.2, 41.6]);
    assert_eq!(city_liberations(&p, &calm, &[]), []);

    let wrong_size = readings(Faction::Humans, true, &[40.0, 40.4, 60.4, 60.8, 61.2]);
    assert_eq!(city_liberations(&p, &wrong_size, &[]), []);

    let mut changed_hands = readings(Faction::Humans, true, &[40.0, 40.4, 72.4, 72.8]);
    changed_hands[2].owner = Faction::Automaton;
    changed_hands[3].owner = Faction::Automaton;
    assert_eq!(city_liberations(&p, &changed_hands, &[]), []);

    let mut far_apart = readings(Faction::Humans, true, &[40.0, 40.4, 72.4, 72.8]);
    far_apart[2].at += 60.0 * MINUTE;
    far_apart[3].at += 60.0 * MINUTE;
    assert_eq!(city_liberations(&p, &far_apart, &[]), []);
}

#[test]
fn reads_nothing_of_a_defense_whose_event_is_gone() {
    let p = Planet { event: None, ..varylia() };
    let samples = readings(Faction::Humans, true, &[44.2275, 44.6501, 76.6508, 77.0856]);
    assert_eq!(city_liberations(&p, &samples, &[]), []);
}

#[test]
fn lists_same_sized_cities_it_cannot_tell_apart() {
    let samples = readings(Faction::Automaton, false, &[10.0, 10.1, 10.2, 19.675, 19.775, 19.875]);
    let got = city_liberations(&twin_towns(), &samples, &[]);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].cities, ["NORTH", "SOUTH"]);
}

#[test]
fn tells_same_sized_cities_apart_by_which_was_nearly_freed() {
    let samples = readings(Faction::Automaton, false, &[10.0, 10.1, 10.2, 19.675, 19.775, 19.875]);
    let jump = samples[3].at;
    let reading = |region, before, percent| CityProgress { region, at: jump - before * MINUTE, percent };
    let fought = [reading(0, 20.0, 12.0), reading(1, 20.0, 94.0), reading(1, 5.0, 99.5), reading(0, 5.0, 13.0)];
    assert_eq!(city_liberations(&twin_towns(), &samples, &fought)[0].cities, ["SOUTH"]);

    // Readings from long before say nothing of who fell now.
    let stale = [reading(1, 120.0, 99.0)];
    assert_eq!(city_liberations(&twin_towns(), &samples, &stale)[0].cities, ["NORTH", "SOUTH"]);
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
