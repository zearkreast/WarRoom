mod common;

use common::*;
use warroom_core::api::{AssignmentTask, Dispatch, Faction, Position};
use warroom_core::dispatch::recent_dispatches;
use warroom_core::format::{MarkupSpan, dispatch_title, fmt_duration, parse_markup};
use warroom_core::galaxy::{
    Anomaly, SectorPlanet, anomaly_of, build_polar_sectors, human_control, sector_at, sector_territory, supply_lines,
};
use warroom_core::major_order::{DeadlineOutlook, deadline_outlook, describe_assignment};
use warroom_core::planets::is_placeholder;
use warroom_core::progress::{
    Sample, liberation_percent, projected_at, projected_completion, projected_loss, rate_per_hour,
};
use warroom_core::regions::chaikin;
use warroom_core::time::iso_string;

fn sample(at: f64, percent: f64) -> Sample {
    Sample { planet: 1, at, owner: Faction::Automaton, percent, players: 100.0, defending: false }
}

#[test]
fn fixtures_have_gaps_in_planet_indices() {
    assert!(planets().iter().enumerate().any(|(i, p)| p.index != i as i64));
}

#[test]
fn computes_liberation_from_health() {
    assert!(close(liberation_percent(1_376_398.0, 2_000_000.0), 31.18, 2));
    assert_eq!(liberation_percent(0.0, 0.0), 0.0);
}

#[test]
fn needs_ten_minutes_of_samples_before_reporting_a_rate() {
    assert_eq!(rate_per_hour(&[sample(0.0, 10.0)]), None);
    assert_eq!(rate_per_hour(&[sample(0.0, 10.0), sample(9.0 * 60_000.0, 10.1)]), None);
    assert!(close(rate_per_hour(&[sample(0.0, 10.0), sample(H / 2.0, 11.0)]).unwrap(), 2.0, 2));
}

#[test]
fn fits_a_trend_that_holds_steady_whatever_the_sampling_phase() {
    // Polled every minute, but the game steps progress by 0.07% every 3 minutes (1.4%/h).
    let m = 60_000.0;
    for phase in 0..3 {
        let s: Vec<Sample> =
            (0..11).map(|i| sample(((phase + i) as f64) * m, 31.0 + ((phase + i) / 3) as f64 * 0.07)).collect();
        assert!((rate_per_hour(&s).unwrap() - 1.4).abs() < 0.06);
    }
}

#[test]
fn ignores_samples_from_before_a_capture() {
    let mut later = sample(10.0 * 60_000.0, 0.0);
    later.owner = Faction::Humans;
    assert_eq!(rate_per_hour(&[sample(0.0, 90.0), later]), None);
}

#[test]
fn projects_completion_loss_and_the_percent_at_a_deadline() {
    assert_eq!(projected_completion(50.0, Some(2.0), 0.0), Some(25.0 * H));
    assert_eq!(projected_completion(50.0, Some(-1.0), 0.0), None);
    assert_eq!(projected_loss(20.0, Some(-2.0), 0.0), Some(10.0 * H));
    assert!(close(projected_at(30.0, Some(1.5), 0.0, 10.0 * H).unwrap(), 45.0, 2));
}

#[test]
fn formats_durations() {
    assert_eq!(fmt_duration(47.0 * H + 19.0 * 60_000.0), "47h 19m");
    assert_eq!(fmt_duration(5.0 * 24.0 * H), "5d 0h");
    assert_eq!(fmt_duration(-1.0), "now");
}

#[test]
fn resolves_liberate_tasks_to_planets() {
    let planets = planets();
    let tasks = describe_assignment(&assignments()[0], Some(&planets));
    let names: Vec<_> = tasks.iter().map(|t| t.planet.as_ref().map(|p| p.name.as_str())).collect();
    assert_eq!(names, [Some("FURY"), Some("ESKER"), Some("ZEA RUGOSIA")]);
    assert!(tasks.iter().all(|t| t.is_planet && !t.done));
}

#[test]
fn falls_back_to_a_count_task_for_unknown_types() {
    let planets = planets();
    let mut a = assignments()[0].clone();
    a.tasks = vec![AssignmentTask { kind: 3, values: vec![3.0, 0.0, 500000.0], value_types: vec![1, 0, 3] }];
    a.progress = vec![1234.0];
    let t = &describe_assignment(&a, Some(&planets))[0];
    assert!(!t.is_planet);
    assert_eq!(t.faction, Some(Faction::Automaton));
    assert_eq!((t.current, t.goal, t.done), (1234.0, 500000.0, false));
}

#[test]
fn splits_highlighted_spans() {
    let spans = parse_markup("<i=3>NEW MAJOR ORDER</i>\n\nGo to <i=1>Fury</i>.");
    let span = |text: &str, style: Option<i64>| MarkupSpan { text: text.into(), style };
    assert_eq!(
        spans,
        [span("NEW MAJOR ORDER", Some(3)), span("\n\nGo to ", None), span("Fury", Some(1)), span(".", None)]
    );
    assert_eq!(dispatch_title("<i=3>STRATEGIC ALERT</i>\n\nBody"), "STRATEGIC ALERT");
}

fn sector_planets(planets: &[warroom_core::api::Planet]) -> Vec<SectorPlanet<'_>> {
    planets
        .iter()
        .filter(|p| !is_placeholder(p.index, p.position))
        .map(|p| SectorPlanet { index: p.index, sector: &p.sector, position: p.position })
        .collect()
}

#[test]
fn rebuilds_the_games_ring_sectors_with_every_planet_inside_its_own_sector() {
    let planets = planets();
    let geo = build_polar_sectors(&sector_planets(&planets));
    assert_eq!(geo.rings[0].runs.iter().map(|r| r.sector.as_str()).collect::<Vec<_>>(), ["Sol"]);
    for ring in &geo.rings {
        assert!(ring.runs.iter().all(|r| geo.fills.iter().any(|(s, _)| *s == r.sector)));
    }
    for p in planets.iter().filter(|q| !is_placeholder(q.index, q.position)) {
        assert_eq!(sector_at(&geo, p.position.x, p.position.y), Some(p.sector.as_str()), "{}", p.name);
    }
    for (_, d) in &geo.fills {
        assert!(d.starts_with('M') && d.ends_with('Z') && !d.contains("NaN"));
    }
    // Edges only separate different sectors; the rim counts as one side.
    assert!(geo.edges.iter().all(|e| Some(&e.a) != e.b.as_ref()));
    assert!(geo.edges.iter().any(|e| e.b.is_none()));
}

#[test]
fn gives_a_sector_to_the_enemy_holding_or_attacking_most_of_its_planets() {
    let planets = planets();
    let territory = sector_territory(&planets);
    let held = planets.iter().find(|p| p.current_owner != Faction::Humans).unwrap();
    let of = |s: &str| territory.iter().find(|(n, _)| n == s).map(|(_, t)| *t);
    assert!(of(&held.sector).unwrap().is_some());
    assert_eq!(of("Sol"), Some(None));
}

#[test]
fn dedupes_supply_lines_and_computes_control_share() {
    let planets = planets();
    let lines = supply_lines(&planets);
    let keys: std::collections::HashSet<_> = lines.iter().map(|&(a, b)| (a.min(b), a.max(b))).collect();
    assert_eq!(keys.len(), lines.len());
    assert!(human_control(&planets) > 0.0);
}

#[test]
fn marks_meridia_and_penta_as_anomalies() {
    assert_eq!(anomaly_of("MERIDIA", &[]), Some(Anomaly::Wormhole));
    assert_eq!(anomaly_of("Penta", &[]), Some(Anomaly::Blackhole));
    assert_eq!(anomaly_of("FURY", &[]), None);
}

#[test]
fn chaikin_doubles_the_points_each_pass() {
    assert_eq!(chaikin(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]], 2).len(), 12);
}

#[test]
fn drops_placeholder_worlds_stacked_on_super_earth() {
    assert!(!is_placeholder(0, Position { x: 0.0, y: 0.0 }));
    assert!(is_placeholder(265, Position { x: 0.0, y: 0.0 }));
    assert!(!is_placeholder(3, Position { x: 0.1, y: 0.0 }));
}

#[test]
fn keeps_dispatches_from_the_last_72_hours_newest_first() {
    let now = ms("2026-10-23T12:00:00Z");
    let at = |h: f64, id: i64| Dispatch { id, kind: 0, message: String::new(), published: iso_string(now - h * H) };
    let ids: Vec<i64> = recent_dispatches(&[at(80.0, 1), at(70.0, 2), at(1.0, 3)], now).iter().map(|d| d.id).collect();
    assert_eq!(ids, [3, 2]);
}

#[test]
fn calls_a_target_on_track_behind_done_or_still_measuring() {
    let now = ms("2026-10-23T12:00:00Z");
    assert_eq!(deadline_outlook(40.0, Some(5.0), now, now + 20.0 * H), (DeadlineOutlook::OnTrack, Some(100.0)));
    assert_eq!(deadline_outlook(40.0, Some(1.0), now, now + 20.0 * H), (DeadlineOutlook::Behind, Some(60.0)));
    assert_eq!(deadline_outlook(100.0, None, now, now + H).0, DeadlineOutlook::Done);
    assert_eq!(deadline_outlook(10.0, None, now, now + H), (DeadlineOutlook::Measuring, None));
    assert_eq!(deadline_outlook(10.0, Some(-2.0), now, now + 20.0 * H).1, Some(0.0));
}
