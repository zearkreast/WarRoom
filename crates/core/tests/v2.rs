mod common;

use common::*;
use std::collections::HashSet;
use warroom_core::api::Faction;
use warroom_core::campaign_filter::{FrontActivity, is_stalled};
use warroom_core::collate::locale_cmp;
use warroom_core::dispatch::{DispatchKind, dispatch_body, dispatch_kind};
use warroom_core::faction_planets::{
    CommunityPlanets, busiest_planets, community_planets, parse_community_planets, super_earth_planets,
};
use warroom_core::format::{sest_date, war_day};
use warroom_core::galaxy::sector_territory;
use warroom_core::history::OwnerChange;
use warroom_core::planet_index::{
    PlanetFilter, SectorSort, control_shares, group_by_sector, matches_filter, matches_search,
};
use warroom_core::regions::build_regions;
use warroom_core::snapshot::{players_by_enemy, to_live, to_planet, to_static, to_war_sample};
use warroom_core::time::{iso_string, utc_ms};
use warroom_core::war_history::{HistoryInput, LogEntry, build_war_history, defense_events, fmt_long_duration};

const D: f64 = 24.0 * H;

fn changes() -> Vec<OwnerChange> {
    let c = |from, to, at| OwnerChange { planet: 1, planet_name: "X".into(), from, to, at };
    vec![
        c(Faction::Automaton, Faction::Humans, 2.0 * D),
        c(Faction::Humans, Faction::Automaton, 55.0 * D),
        // After now: still counted by time order.
        c(Faction::Automaton, Faction::Humans, 108.0 * D),
    ]
}

#[test]
fn splits_the_record_into_owner_segments_and_counts_captures() {
    let changes = changes();
    let h = build_war_history(HistoryInput {
        changes: &changes[..2],
        defenses: &[],
        current_owner: Faction::Automaton,
        record_start: 0.0,
        now: 100.0 * D,
    });
    let owners: Vec<_> = h.segments.iter().map(|s| s.owner).collect();
    assert_eq!(owners, [Faction::Automaton, Faction::Humans, Faction::Automaton]);
    assert_eq!((h.liberated, h.lost), (1, 1));
    assert!((h.held_free_pct - 53.0).abs() < 0.5);
}

#[test]
fn numbers_battles_in_time_order_and_ends_the_log_with_the_record_start() {
    let changes = changes();
    let now = 100.0 * D;
    let defenses = defense_events(&[50.0 * D, 50.0 * D + H, 54.0 * D], &changes, None, now);
    let h = build_war_history(HistoryInput {
        changes: &changes[..2],
        defenses: &defenses,
        current_owner: Faction::Automaton,
        record_start: 0.0,
        now,
    });
    assert!(matches!(h.log.last(), Some(LogEntry::Start { owner: Faction::Automaton, .. })));
    let battles: Vec<usize> = h
        .log
        .iter()
        .filter_map(|e| match e {
            LogEntry::Change { battle, .. } | LogEntry::Defense { battle, .. } => Some(*battle),
            LogEntry::Start { .. } => None,
        })
        .collect();
    let mut sorted = battles.clone();
    sorted.sort_by(|a, b| b.cmp(a));
    assert_eq!(battles, sorted);
    assert!(matches!(h.log[0], LogEntry::Change { to: Faction::Automaton, .. }));
}

#[test]
fn groups_defending_samples_into_events_and_marks_the_lost_one() {
    // The planet falls the moment its last defense ends, just after the last defending sample.
    let events = defense_events(&[10.0 * D, 10.0 * D + H, 54.0 * D, 55.0 * D - 60_000.0], &changes(), None, 100.0 * D);
    assert_eq!(events.len(), 3);
    assert!(events[0].held && events[1].held && !events[2].held);
}

#[test]
fn marks_the_latest_event_ongoing_while_the_planet_is_under_attack() {
    let now = 100.0 * D;
    let events = defense_events(&[now - H], &[], Some(now + 5.0 * H), now);
    assert!(events[0].ongoing);
}

#[test]
fn formats_long_durations() {
    assert!(fmt_long_duration(3_551_077_816.0).starts_with("1C 12Y"));
    assert_eq!(fmt_long_duration(12.0 * 3600.0), "12H");
    assert_eq!(fmt_long_duration(0.0), "0H");
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
fn names_a_sector_held_only_when_the_enemy_holds_every_planet_in_it() {
    let planets = planets();
    let territory = sector_territory(&planets);
    for (sector, t) in &territory {
        let Some(t) = t else { continue };
        let all = planets.iter().filter(|p| &p.sector == sector).all(|p| p.current_owner == t.faction);
        assert_eq!(t.held, all);
    }
    assert!(territory.iter().any(|(_, t)| t.is_some_and(|t| !t.held)));
}

fn active() -> HashSet<i64> {
    campaigns().iter().map(|c| c.planet.index).collect()
}

#[test]
fn searches_planet_sector_biome_and_hazard_names() {
    let planets = planets();
    let fury = planets.iter().find(|p| p.name == "FURY").unwrap();
    assert!(matches_search(fury, "fury"));
    assert!(matches_search(fury, &fury.sector.to_lowercase()));
    assert!(matches_search(fury, &format!("{} fury", fury.biome.name)));
    assert!(!matches_search(fury, "fury nonsense"));
}

#[test]
fn filters_by_front_and_owner() {
    let planets = planets();
    let active = active();
    let none = HashSet::new();
    assert_eq!(
        planets.iter().filter(|p| matches_filter(p, PlanetFilter::Active, &active, &none)).count(),
        active.len()
    );
    assert!(planets.iter().all(|p| !matches_filter(p, PlanetFilter::Owner(Faction::Terminids), &active, &none)
        || p.current_owner == Faction::Terminids));
    let pinned = HashSet::from([planets[3].index]);
    let hits: Vec<i64> =
        planets.iter().filter(|p| matches_filter(p, PlanetFilter::Pinned, &active, &pinned)).map(|p| p.index).collect();
    assert_eq!(hits, [planets[3].index]);
}

#[test]
fn groups_by_sector_with_exactly_one_hottest_front_and_fronts_first() {
    let planets = planets();
    let active = active();
    let groups = group_by_sector(&planets, &active, SectorSort::Players);
    assert_eq!(groups.iter().filter(|g| g.hottest).count(), 1);
    assert_eq!(groups.iter().map(|g| g.planets.len()).sum::<usize>(), planets.len());
    for g in &groups {
        let flags: Vec<bool> = g.planets.iter().map(|p| active.contains(&p.index)).collect();
        let mut sorted = flags.clone();
        sorted.sort_by(|a, b| b.cmp(a));
        assert_eq!(flags, sorted);
    }
    for w in groups.windows(2) {
        assert!(w[0].players >= w[1].players);
    }
    let az: Vec<String> = group_by_sector(&planets, &active, SectorSort::Az).iter().map(|g| g.name.clone()).collect();
    let mut sorted = az.clone();
    sorted.sort_by(|a, b| locale_cmp(a, b));
    assert_eq!(az, sorted);
}

#[test]
fn control_shares_add_up_to_the_whole_galaxy() {
    let planets = planets();
    let shares = control_shares(&planets);
    assert_eq!(shares.iter().map(|s| s.planets).sum::<usize>(), planets.len());
    assert!((shares.iter().map(|s| s.pct).sum::<f64>() - 100.0).abs() < 1e-6);
}

#[test]
fn categorises_dispatches_by_headline() {
    assert_eq!(dispatch_kind("<i=3>NEW MAJOR ORDER</i>\nLiberate X."), DispatchKind::Order);
    assert_eq!(dispatch_kind("<i=3>MAJOR ORDER WON</i>\nWell done."), DispatchKind::Victory);
    assert_eq!(dispatch_kind("<i=3>STRATEGIC ALERT</i>\nBots on Y."), DispatchKind::Alert);
    assert_eq!(dispatch_kind("<i=3>CONTROL CENTER ADVISORY</i>\nNote."), DispatchKind::Advisory);
    assert_eq!(dispatch_kind("Something happened."), DispatchKind::News);
}

#[test]
fn drops_the_headline_from_the_body() {
    let body = dispatch_body("<i=3>NEW MAJOR ORDER</i>\nLiberate <i=1>FURY</i>.");
    assert_eq!(body.iter().map(|s| s.text.as_str()).collect::<String>(), "Liberate FURY.");
}

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
fn runs_the_calendar_160_years_ahead_in_utc() {
    assert_eq!(sest_date(utc_ms(2026, 10, 23, 1, 0, 0, 0)), "23/10/2186");
}

#[test]
fn counts_war_days_from_the_start_day() {
    assert_eq!(war_day("2024-02-08T00:00:00Z", utc_ms(2024, 2, 8, 12, 0, 0, 0)), 1);
    assert_eq!(war_day("2024-02-08T00:00:00Z", utc_ms(2024, 2, 10, 12, 0, 0, 0)), 3);
}

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

fn front(players: f64, rate: Option<f64>) -> FrontActivity {
    FrontActivity { players, rate, in_major_order: false, pinned: false }
}

#[test]
fn hides_fronts_under_5k_divers_that_gain_less_than_0_4_per_hour() {
    assert!(is_stalled(&front(2_800.0, None)));
    assert!(is_stalled(&front(41.0, Some(0.002))));
    assert!(is_stalled(&front(4_999.0, Some(0.39))));
    assert!(is_stalled(&front(3_000.0, Some(-1.2))));
}

#[test]
fn shows_fronts_that_are_moving_or_busy_enough_to_matter() {
    assert!(!is_stalled(&front(1_200.0, Some(0.4))));
    assert!(!is_stalled(&front(5_000.0, Some(0.0))));
    assert!(!is_stalled(&front(40_000.0, None)));
}

#[test]
fn always_shows_major_order_targets_and_pinned_planets() {
    assert!(!is_stalled(&FrontActivity { in_major_order: true, ..front(100.0, None) }));
    assert!(!is_stalled(&FrontActivity { pinned: true, ..front(100.0, Some(-2.0)) }));
}
