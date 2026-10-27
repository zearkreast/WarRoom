mod common;
use common::*;
use warroom_core::api::Faction;
use warroom_core::snapshot::{
    EpisodeStatus, campaign_record, issued_at, players_by_enemy, to_live, to_planet, to_static, to_war_sample,
};
use warroom_core::time::iso_string;

#[test]
fn splits_players_by_enemy_front_counting_defenses_for_the_attacker() {
    let campaigns = campaigns();
    let fronts = players_by_enemy(campaigns.iter().map(|c| &c.planet));
    let total: f64 = campaigns
        .iter()
        .filter(|c| c.planet.event.as_ref().map_or(c.planet.current_owner, |e| e.faction) != Faction::Humans)
        .map(|c| c.planet.statistics.player_count)
        .sum();
    assert_eq!(fronts.terminids + fronts.automaton + fronts.illuminate, total);
    let war = war();
    let row = to_war_sample(&war, campaigns.iter().map(|c| &c.planet), 0.0);
    assert_eq!(row.at, iso_string(0.0));
    assert_eq!(row.players, war.statistics.player_count);
    assert_eq!(row.terminids, fronts.terminids);
}

#[test]
fn round_trips_planet_statistics_through_the_trimmed_snapshot() {
    let planets = planets();
    let fury = planets.iter().find(|p| p.name == "FURY").unwrap();
    let back = to_planet(&to_static(fury), &to_live(fury));
    assert_eq!(back.statistics.automaton_kills, fury.statistics.automaton_kills);
    assert_eq!(back.statistics.mission_time, fury.statistics.mission_time);
    assert!(!back.hazards.iter().any(|h| h.name.to_lowercase() == "none"));
}

#[test]
fn never_dates_an_order_after_its_campaign_ended_or_before_it_began() {
    let campaigns = campaigns_live();
    let ended = campaigns.iter().find(|c| c.status != EpisodeStatus::Active).unwrap();
    let end = ended.end.unwrap();
    assert_eq!(issued_at(ended.start, ended.end, Some(end + 1.0)), None);
    assert_eq!(issued_at(ended.start, ended.end, Some(ended.start - 1.0)), None);
    assert_eq!(issued_at(ended.start, ended.end, Some(ended.start + 1.0)), Some(ended.start + 1.0));
}

#[test]
fn counts_campaigns_and_major_orders_won() {
    use EpisodeStatus::*;
    let mo = |s| phase("MO", s);
    let past =
        [episode(1.0, Won, vec![mo(Won), mo(Lost), mo(Won)]), episode(2.0, Lost, vec![mo(Lost), mo(Won), mo(Lost)])];
    let r = campaign_record(&past);
    assert_eq!((r.won, r.orders, r.orders_won), (1, 6, 3));
}

#[test]
fn streaks_the_victories_since_the_newest_failure_in_any_order() {
    use EpisodeStatus::*;
    let e = |start, status| episode(start, status, vec![]);
    assert_eq!(campaign_record(&[e(3.0, Won), e(1.0, Won), e(4.0, Won), e(2.0, Lost)]).streak, 2);
    assert_eq!(campaign_record(&[e(1.0, Won), e(2.0, Won)]).streak, 2);
    assert_eq!(campaign_record(&[e(1.0, Won), e(2.0, Lost)]).streak, 0);
    assert_eq!(campaign_record(&[]).streak, 0);
}
