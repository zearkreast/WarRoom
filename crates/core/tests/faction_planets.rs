mod common;
use common::*;
use warroom_core::api::Faction;
use warroom_core::faction_planets::{
    CommunityPlanets, busiest_planets, community_planets, parse_community_planets, super_earth_planets,
};

fn seq(values: Vec<f64>) -> impl FnMut() -> f64 {
    let mut i = 0;
    move || {
        let v = values[i % values.len()];
        i += 1;
        v
    }
}

#[test]
fn parses_the_community_file() {
    let community = community_planets();
    assert_eq!(community.fixed, ["MALEVELON CREEK", "CALYPSO"]);
    assert!(community.pool.len() >= 4);
    assert_eq!(
        parse_community_planets("# c\n[fixed]\nA # note\n\n[pool]\nb\nc"),
        CommunityPlanets { fixed: vec!["A".into()], pool: vec!["B".into(), "C".into()] }
    );
}

#[test]
fn lists_a_factions_busiest_planets_at_most_six() {
    let planets = planets();
    let list = busiest_planets(&planets, Faction::Automaton, 6);
    assert!(list.len() <= 6);
    let players: Vec<f64> = list
        .iter()
        .map(|l| planets.iter().find(|p| Some(p.index) == l.planet).unwrap().statistics.player_count)
        .collect();
    let mut sorted = players.clone();
    sorted.sort_by(|a, b| b.total_cmp(a));
    assert_eq!(players, sorted);
}

#[test]
fn leads_super_earths_list_with_super_earth_and_mars_then_the_fixed_planets() {
    let held: Vec<_> = planets()
        .into_iter()
        .map(|mut p| {
            p.current_owner = Faction::Humans;
            p
        })
        .collect();
    let community =
        CommunityPlanets { fixed: vec!["FURY".into()], pool: vec!["ESKER".into(), "MORT".into(), "USTOTU".into()] };
    let list = super_earth_planets(&held, &community, seq(vec![0.0]));
    let names: Vec<&str> = list.iter().map(|l| l.name.as_str()).collect();
    assert_eq!(names, ["SUPER EARTH", "MARS", "FURY", "ESKER", "MORT", "USTOTU"]);
    assert_eq!(list[1].lore, Some("Sol system"));
}

#[test]
fn draws_the_last_picks_at_random_and_tops_up_a_lost_fixed_planet_from_the_pool() {
    let held: Vec<_> = planets()
        .into_iter()
        .map(|mut p| {
            p.current_owner = if p.name == "FURY" { Faction::Automaton } else { Faction::Humans };
            p
        })
        .collect();
    let community = CommunityPlanets {
        fixed: vec!["FURY".into(), "MORT".into()],
        pool: vec!["ESKER".into(), "USTOTU".into(), "K".into()],
    };
    let list = super_earth_planets(&held, &community, seq(vec![0.99]));
    let names: Vec<&str> = list.iter().map(|l| l.name.as_str()).collect();
    assert_eq!(names, ["SUPER EARTH", "MARS", "MORT", "K", "USTOTU", "ESKER"]);
}
