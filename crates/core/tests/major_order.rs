mod common;
use common::*;
use warroom_core::api::{AssignmentTask, Faction};
use warroom_core::major_order::{
    DeadlineOutlook, TaskOutlook, TaskView, deadline_outlook, describe_assignment, order_name, order_task_rates,
    order_verdict, task_outlook, task_percent,
};
use warroom_core::snapshot::{EpisodeStatus, OrderSample};

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
fn a_defend_task_with_no_planet_is_a_count_not_super_earth() {
    let planets = planets();
    let mut a = assignments()[0].clone();
    a.tasks = vec![AssignmentTask {
        kind: 12,
        values: vec![3.0, 0.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        value_types: vec![1, 2, 3, 4, 6, 5, 8, 9, 11, 12],
    }];
    a.progress = vec![2.0];
    let t = &describe_assignment(&a, Some(&planets))[0];
    assert!(!t.is_planet);
    assert_eq!(t.planet, None);
    assert_eq!((t.current, t.goal, t.done), (2.0, 5.0, false));
}

#[test]
fn a_defend_task_that_names_super_earth_keeps_its_planet() {
    let planets = planets();
    let mut a = assignments()[0].clone();
    a.tasks = vec![AssignmentTask { kind: 12, values: vec![4.0, 1.0, 1.0, 0.0], value_types: vec![1, 3, 11, 12] }];
    a.progress = vec![0.0];
    let t = &describe_assignment(&a, Some(&planets))[0];
    assert!(t.is_planet);
    assert_eq!(t.planet.as_ref().map(|p| p.name.as_str()), Some("SUPER EARTH"));
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
fn measures_each_tasks_count_per_hour_from_the_orders_readings() {
    let mut samples: Vec<OrderSample> =
        (0..3).map(|h| OrderSample { id: 7, at: h as f64 * H, progress: vec![h as f64 * 1000.0, 50.0] }).collect();
    samples.push(OrderSample { id: 8, at: 0.0, progress: vec![999_999.0, 0.0] });
    assert_eq!(order_task_rates(Some(&samples), 7, 2), [Some(1000.0), Some(0.0)]);
}

#[test]
fn waits_for_half_an_hour_of_readings_before_trusting_a_rate() {
    let samples: Vec<OrderSample> =
        [0.0, 10.0, 20.0].iter().map(|m| OrderSample { id: 7, at: m * 60_000.0, progress: vec![*m] }).collect();
    assert_eq!(order_task_rates(Some(&samples), 7, 1), [None]);
    assert_eq!(order_task_rates(None, 7, 2), [None, None]);
}

#[test]
fn wins_when_the_pace_reaches_100_before_the_deadline() {
    let o = task_outlook(50.0, Some(10.0), 0.0, 10.0 * H);
    assert_eq!(
        o,
        TaskOutlook { outlook: DeadlineOutlook::OnTrack, end: Some(100.0), eta: Some(5.0 * H), rate: Some(10.0) }
    );
    let o = task_outlook(50.0, Some(2.0), 0.0, 10.0 * H);
    assert_eq!(o, TaskOutlook { outlook: DeadlineOutlook::Behind, end: Some(70.0), eta: None, rate: Some(2.0) });
    assert_eq!(task_outlook(50.0, None, 0.0, 10.0 * H).outlook, DeadlineOutlook::Measuring);
    assert_eq!(task_outlook(100.0, Some(3.0), 0.0, 10.0 * H).outlook, DeadlineOutlook::Done);
}

#[test]
fn calls_the_order_lost_if_any_objective_falls_short() {
    let t = |label: &str| TaskView {
        is_planet: false,
        label: label.into(),
        planet: None,
        faction: None,
        current: 0.0,
        goal: 1.0,
        done: false,
    };
    let o = |outlook| TaskOutlook { outlook, end: None, eta: None, rate: None };
    use DeadlineOutlook::*;
    let tasks = [t("A"), t("B")];
    assert!(order_verdict(&tasks, &[o(OnTrack), o(Behind)]).unwrap().0.starts_with("Losing: B"));
    assert!(order_verdict(&tasks, &[o(OnTrack), o(Done)]).unwrap().0.starts_with("Winning"));
    assert!(order_verdict(&tasks, &[o(OnTrack), o(Measuring)]).unwrap().0.starts_with("Measuring"));
}
