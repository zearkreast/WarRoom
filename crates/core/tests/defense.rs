mod common;

use common::H;
use std::collections::HashMap;
use warroom_core::api::{Faction, Planet, PlanetEvent};
use warroom_core::defense::{
    GambitOutlook, attackers, divers_short, enemy_advance, event_span, gambit, gambits_from, needed_rate,
};
use warroom_core::time::{iso_string, parse_ms};

const START: &str = "2026-10-29T12:00:00Z";
const END: &str = "2026-10-31T12:00:00Z";

fn event(faction: Faction, defended: f64) -> PlanetEvent {
    PlanetEvent {
        faction,
        health: (1.0 - defended / 100.0) * 1_000_000.0,
        max_health: 1_000_000.0,
        start_time: START.into(),
        end_time: END.into(),
        ..Default::default()
    }
}

fn planet(index: i64, owner: Faction, liberated: f64, attacking: &[i64]) -> Planet {
    Planet {
        index,
        name: format!("P{index}"),
        current_owner: owner,
        max_health: 1_000_000.0,
        health: (1.0 - liberated / 100.0) * 1_000_000.0,
        attacking: attacking.to_vec(),
        ..Default::default()
    }
}

fn defended(index: i64, faction: Faction, percent: f64) -> Planet {
    Planet { event: Some(event(faction, percent)), ..planet(index, Faction::Humans, 0.0, &[]) }
}

#[test]
fn the_enemy_advances_with_the_events_clock() {
    let e = event(Faction::Automaton, 10.0);
    let start = parse_ms(START);
    assert_eq!(enemy_advance(&e, start - H), Some(0.0));
    assert_eq!(enemy_advance(&e, start + 12.0 * H), Some(25.0));
    assert_eq!(enemy_advance(&e, start + 60.0 * H), Some(100.0));
    assert_eq!(event_span(&e), Some((start, start + 48.0 * H)));
    let broken = PlanetEvent { end_time: String::new(), ..e };
    assert_eq!(enemy_advance(&broken, start), None);
    assert_eq!(event_span(&broken), None);
}

#[test]
fn finds_the_planets_an_attack_comes_from() {
    let target = defended(1, Faction::Automaton, 20.0);
    let planets = vec![
        target.clone(),
        planet(2, Faction::Automaton, 0.0, &[1]),
        planet(3, Faction::Terminids, 0.0, &[1]),
        planet(4, Faction::Humans, 100.0, &[1]),
        planet(5, Faction::Automaton, 0.0, &[9]),
    ];
    let got: Vec<i64> = attackers(&target, &planets).iter().map(|p| p.index).collect();
    assert_eq!(got, [2], "the attacking faction's planets, not another enemy's or Super Earth's");
    let quiet = planet(1, Faction::Humans, 0.0, &[]);
    let got: Vec<i64> = attackers(&quiet, &planets).iter().map(|p| p.index).collect();
    assert_eq!(got, [2, 3]);
}

#[test]
fn works_out_the_pace_and_the_helldivers_a_defense_needs() {
    let now = parse_ms(START);
    assert_eq!(needed_rate(40.0, now, now + 20.0 * H), Some(3.0));
    assert_eq!(needed_rate(100.0, now, now + 20.0 * H), Some(0.0));
    assert_eq!(needed_rate(40.0, now, now - H), None);
    // 1.5%/h from 3000 Helldivers is 0.0005%/h each; 1.5%/h more needs 3000 more.
    assert_eq!(divers_short(Some(1.5), 3.0, 3000.0), Some(3000.0));
    assert_eq!(divers_short(Some(4.0), 3.0, 3000.0), Some(0.0));
    assert_eq!(divers_short(Some(-1.0), 3.0, 3000.0), None);
    assert_eq!(divers_short(None, 3.0, 3000.0), None);
    assert_eq!(divers_short(Some(1.5), 3.0, 0.0), None);
}

fn rates(list: &[(i64, Option<f64>)]) -> HashMap<i64, Option<f64>> {
    list.iter().copied().collect()
}

#[test]
fn a_gambit_needs_every_attacker_to_be_a_front() {
    let now = parse_ms(START) + 12.0 * H;
    let planets = vec![defended(1, Faction::Automaton, 20.0), planet(2, Faction::Automaton, 50.0, &[1])];
    assert!(gambit(&planets[0], &planets, &rates(&[(1, Some(1.0))]), now).is_none());
    assert!(gambit(&planets[1], &planets, &rates(&[(1, Some(1.0)), (2, Some(1.0))]), now).is_none());
    let g = gambit(&planets[0], &planets, &rates(&[(1, Some(1.0)), (2, Some(5.0))]), now).unwrap();
    assert_eq!(g.attackers.len(), 1);
    assert_eq!(g.attackers[0].percent, 50.0);
    assert_eq!(g.done_at, Some(now + 10.0 * H));
    assert_eq!(g.deadline, parse_ms(END));
}

#[test]
fn judges_the_gambit_against_the_deadline_and_against_holding() {
    let now = parse_ms(START) + 12.0 * H;
    let planets = vec![defended(1, Faction::Automaton, 20.0), planet(2, Faction::Automaton, 50.0, &[1])];
    let outlook = |def: Option<f64>, att: Option<f64>| {
        gambit(&planets[0], &planets, &rates(&[(1, def), (2, att)]), now).unwrap().outlook
    };
    // 36 h left. The attacker falls in 10 h; the defense alone takes 80 h.
    assert_eq!(outlook(Some(1.0), Some(5.0)), GambitOutlook::Ahead);
    // The defense alone takes 4 h.
    assert_eq!(outlook(Some(20.0), Some(5.0)), GambitOutlook::Slower);
    // The attacker takes 50 h.
    assert_eq!(outlook(Some(1.0), Some(1.0)), GambitOutlook::OutOfReach);
    assert_eq!(outlook(Some(1.0), Some(-0.5)), GambitOutlook::OutOfReach);
    assert_eq!(outlook(Some(1.0), None), GambitOutlook::Measuring);
}

#[test]
fn a_front_attacking_a_defense_lists_its_gambit() {
    let now = parse_ms(START) + 12.0 * H;
    let planets = vec![
        defended(1, Faction::Automaton, 20.0),
        planet(2, Faction::Automaton, 50.0, &[1]),
        planet(3, Faction::Automaton, 0.0, &[]),
    ];
    let r = rates(&[(1, Some(1.0)), (2, Some(5.0)), (3, Some(1.0))]);
    let list = gambits_from(&planets[1], &planets, &r, now);
    assert_eq!(list.iter().map(|g| g.defended.index).collect::<Vec<_>>(), [1]);
    assert!(gambits_from(&planets[2], &planets, &r, now).is_empty());
    assert_eq!(iso_string(list[0].deadline), "2026-10-31T12:00:00.000Z");
}
