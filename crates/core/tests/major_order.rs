use warroom_core::major_order::{
    DeadlineOutlook, TaskOutlook, TaskView, order_task_rates, order_verdict, task_outlook,
};
use warroom_core::snapshot::OrderSample;

const H: f64 = 3_600_000.0;

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
