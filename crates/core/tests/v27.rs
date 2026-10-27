mod common;

use common::*;
use warroom_core::api::{Dispatch, Faction, Named, Planet};
use warroom_core::biomes::{biome_slug, planet_look};
use warroom_core::dispatch::latest_dispatches;
use warroom_core::effects::{EffectKind, planet_effects};
use warroom_core::front_activity::{ActivityPoint, ActivitySeries, Reading, average_readings, plot_activity};
use warroom_core::major_order::{TaskView, order_name, task_percent};
use warroom_core::snapshot::{EpisodeLive, EpisodePhase, EpisodeStatus, campaign_record, to_effects_live};
use warroom_core::time::iso_string;

fn on(index: i64) -> Vec<warroom_core::effects::PlanetEffect> {
    let live = to_effects_live(&status_effects(), 0.0);
    planet_effects(&live.planets.iter().find(|p| p.index == index).map(|p| p.ids.clone()).unwrap_or_default())
}

#[test]
fn names_each_sub_faction_once_though_the_game_lists_it_once_per_side() {
    let fury: Vec<(EffectKind, String)> = on(261).into_iter().map(|e| (e.kind, e.name)).collect();
    assert_eq!(
        fury,
        [
            (EffectKind::Subfaction, "THE INCINERATION CORPS".to_string()),
            (EffectKind::Seaf, "Heavy SEAF Presence".to_string())
        ]
    );
}

#[test]
fn files_cities_the_training_ground_and_tcs_towers_as_points_of_interest() {
    let live = to_effects_live(&status_effects(), 0.0);
    let pois: Vec<String> = live
        .planets
        .iter()
        .flat_map(|p| planet_effects(&p.ids).into_iter().filter(|e| e.kind == EffectKind::Poi).map(|e| e.name))
        .collect();
    for name in
        ["NEW HOPE CITY", "NEW ASPIRATION CITY", "NEW YEARNING CITY", "HELLDIVER TRAINING FACILITIES", "PANDORA BASE"]
    {
        assert!(pois.iter().any(|p| p == name), "{name}");
    }
    assert_eq!(pois.iter().filter(|n| *n == "TERMINID CONTROL SYSTEM+").count(), 14);
}

#[test]
fn keeps_the_effects_war_events_grant_for_a_while_with_when_they_end() {
    let at = ms("2026-10-26T23:57:00Z");
    let status: warroom_core::api::WarStatusRaw = serde_json::from_value(serde_json::json!({
        "time": 85_118_800,
        "planetActiveEffects": [],
        "spaceStations": [],
        "globalEvents": [
            { "effectIds": [], "planetIndices": [], "expireTime": 85_331_008 },
            { "effectIds": [1207], "planetIndices": [123, 75], "expireTime": 85_331_977 },
            { "effectIds": [1231], "planetIndices": [75], "expireTime": 85_000_000 },
        ],
    }))
    .unwrap();
    let timed = to_effects_live(&status, at).timed;
    assert_eq!(timed.len(), 1);
    assert_eq!((timed[0].ids.clone(), timed[0].planets.clone()), (vec![1207], vec![75, 123]));
    assert_eq!(timed[0].until, ms("2026-10-29T11:10:00Z"));
    let none = warroom_core::api::WarStatusRaw { global_events: Vec::new(), ..status };
    let live = to_effects_live(&none, at);
    assert!(live.timed.is_empty());
    assert!(serde_json::to_value(&live).unwrap().get("timed").is_none());
}

#[test]
fn leaves_out_unnamed_ids_and_bookkeeping_like_the_campaign_blocker() {
    let live = to_effects_live(&status_effects(), 0.0);
    let zagon = live.planets.iter().find(|p| p.ids.contains(&1318)).unwrap();
    assert!(!planet_effects(&zagon.ids).iter().any(|e| e.name == "CAMPAIGN BLOCKER"));
    assert!(planet_effects(&[1318, 1319]).is_empty());
}

#[test]
fn lists_the_newest_few_whatever_their_age_or_the_last_72_hours() {
    let now = ms("2026-10-24T12:00:00Z");
    let d = |id: i64, days_ago: f64| Dispatch {
        id,
        published: iso_string(now - days_ago * 86_400_000.0),
        kind: 0,
        message: "x".into(),
    };
    let list = [d(1, 9.0), d(2, 6.0), d(3, 5.0), d(4, 0.5)];
    let ids = |count| latest_dispatches(&list, count, now).iter().map(|x| x.id).collect::<Vec<_>>();
    assert_eq!(ids("3"), [4, 3, 2]);
    assert_eq!(ids("72h"), [4]);
}

#[test]
fn shows_what_a_destroyed_planet_has_become_instead_of_its_old_biome() {
    let planet = |name: &str, biome: &str| Planet {
        name: name.into(),
        biome: Named { name: biome.into(), description: String::new() },
        ..Default::default()
    };
    let look = |p: Planet| planet_look(&p, &[]).map(|(_, art)| art);
    assert_eq!(look(planet("MERIDIA", "Supercolony")), Some("meridian-black-hole"));
    assert_eq!(look(planet("PENTA", "Deadlands")), Some("black-hole"));
    assert_eq!(look(planet("MORADESH", "Desert Dunes")), Some("fractured"));
    assert_eq!(look(planet("FURY", "Magma")), None);
    assert_eq!(biome_slug("Magma"), "magma");
}

#[test]
fn averages_war_readings_over_each_window_stamped_at_its_middle() {
    let w = 300_000.0;
    let r =
        |at, players, impact, terminids| Reading { at, players, impact, terminids, automatons: 0.0, illuminate: 0.0 };
    let rows =
        average_readings(&[r(0.0, 100.0, 0.01, 10.0), r(60_000.0, 300.0, 0.03, 30.0), r(w + 1.0, 50.0, 0.02, 5.0)], w);
    assert_eq!(rows.len(), 2);
    assert_eq!((rows[0].at, rows[0].players, rows[0].terminids), (w / 2.0, 200.0, 20.0));
    assert!((rows[0].impact - 0.02).abs() < 1e-12);
    assert_eq!(rows[1], r(w + w / 2.0, 50.0, 0.02, 5.0));
}

fn row(online: Option<f64>, impact: Option<f64>) -> ActivityPoint {
    ActivityPoint { at: 0.0, terminids: 10.0, automatons: 20.0, illuminate: 30.0, players: 60.0, online, impact }
}

#[test]
fn draws_every_series_as_recorded_while_all_are_on() {
    let p = plot_activity(&[row(Some(100.0), Some(0.012))], &[], true, 0.01)[0];
    assert_eq!(
        (p.plot_terminids, p.plot_automatons, p.plot_illuminate, p.plot_online, p.plot_impact),
        (10.0, 20.0, 30.0, Some(100.0), Some(0.012))
    );
}

#[test]
fn flattens_a_switched_off_front_so_the_bands_above_settle_onto_the_rest() {
    let p = plot_activity(&[row(Some(100.0), Some(0.012))], &[ActivitySeries::Automatons], true, 0.01)[0];
    assert_eq!((p.plot_terminids, p.plot_automatons, p.plot_illuminate, p.row.automatons), (10.0, 0.0, 30.0, 20.0));
}

#[test]
fn parks_the_online_line_on_the_fronts_still_shown_and_the_multiplier_flat() {
    let off = [ActivitySeries::Online, ActivitySeries::Impact, ActivitySeries::Terminids];
    let p = plot_activity(&[row(Some(100.0), Some(0.012))], &off, true, 0.01)[0];
    assert_eq!((p.plot_online, p.plot_impact), (Some(50.0), Some(0.01)));
}

#[test]
fn keeps_gaps_as_gaps_and_falls_back_to_everyone_on_the_fronts_without_readings() {
    let gap = plot_activity(&[row(None, None)], &[ActivitySeries::Online, ActivitySeries::Impact], true, 0.01)[0];
    assert_eq!((gap.plot_online, gap.plot_impact), (None, None));
    let fronts = plot_activity(&[row(None, None)], &[], false, 0.01)[0];
    assert_eq!(fronts.plot_online, Some(60.0));
}

fn phase(title: &str, status: EpisodeStatus) -> EpisodePhase {
    EpisodePhase { title: title.into(), status, ..Default::default() }
}

fn episode(start: f64, status: EpisodeStatus, phases: Vec<EpisodePhase>) -> EpisodeLive {
    EpisodeLive {
        id: start as i64,
        title: "CAMPAIGN".into(),
        faction: Faction::Humans,
        status,
        start,
        end: Some(start + 1.0),
        phases,
        ..Default::default()
    }
}

#[test]
fn order_name_is_the_running_order_of_the_running_campaign() {
    use EpisodeStatus::*;
    let list = [
        episode(1.0, Won, vec![phase("OLD", Won)]),
        episode(2.0, Active, vec![phase("FIRST", Won), phase("HONOR THE FALLEN", Active)]),
    ];
    assert_eq!(order_name(Some(&list)).as_deref(), Some("HONOR THE FALLEN"));
}

#[test]
fn order_name_is_left_out_when_it_cant_be_told_which_order_is_which() {
    use EpisodeStatus::*;
    assert_eq!(order_name(Some(&[episode(1.0, Active, vec![phase("A", Active), phase("B", Active)])])), None);
    assert_eq!(order_name(None), None);
}

#[test]
fn order_name_is_left_out_while_the_game_hasnt_named_the_order_yet() {
    use EpisodeStatus::*;
    let list = [episode(1.0, Active, vec![phase("HONOR THE FALLEN", Lost), phase("MAJOR ORDER", Active)])];
    assert_eq!(order_name(Some(&list)), None);
}

fn task(current: f64, goal: f64) -> TaskView {
    TaskView {
        is_planet: false,
        label: "Eradicate".into(),
        planet: None,
        faction: None,
        current,
        goal,
        done: current >= goal,
    }
}

#[test]
fn task_progress_is_the_share_of_the_goal_held_to_0_100() {
    assert!((task_percent(&task(21_200_000.0, 300_000_000.0)) - 7.07).abs() < 0.005);
    assert_eq!(task_percent(&task(-2.0, 1.0)), 0.0);
    assert_eq!(task_percent(&task(70.0, 50.0)), 100.0);
}

#[test]
fn task_progress_reads_100_only_once_the_task_is_done() {
    assert_eq!(task_percent(&task(299_900_000.0, 300_000_000.0)), 99.9);
    assert_eq!(task_percent(&task(5.0, 0.0)), 100.0);
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
