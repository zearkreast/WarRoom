use warroom_core::watchlist::{FrontAlert, FrontLook, OrderAlert, OrderLook, front_alerts, order_alerts};

const H: f64 = 3_600_000.0;

#[test]
fn a_front_alerts_once_as_victory_nears_and_when_it_starts_losing() {
    let far = FrontLook { at: 0.0, rate: Some(2.0), victory_at: Some(3.0 * H), ..FrontLook::default() };
    let near = FrontLook { at: 2.5 * H, ..far.clone() };
    assert_eq!(front_alerts(&far, &near), [FrontAlert::VictorySoon(3.0 * H)]);
    assert!(front_alerts(&near, &FrontLook { at: 2.6 * H, ..near.clone() }).is_empty());

    let losing = FrontLook { at: 1.0 * H, rate: Some(-0.2), victory_at: None, ..FrontLook::default() };
    assert_eq!(front_alerts(&far, &losing), [FrontAlert::LosingGround(-0.2)]);
    assert!(front_alerts(&losing, &losing).is_empty());
    assert!(front_alerts(&FrontLook::default(), &losing).is_empty());
}

#[test]
fn a_city_alerts_when_it_opens() {
    let before = FrontLook { waiting: vec![7, 8], ..FrontLook::default() };
    let after = FrontLook { waiting: vec![8], open: vec![(7, "Pilen V".into())], ..FrontLook::default() };
    assert_eq!(front_alerts(&before, &after), [FrontAlert::CityOpens("Pilen V".into())]);
    assert!(front_alerts(&after, &after).is_empty());
}

#[test]
fn an_order_alerts_when_it_turns_losing_nears_its_end_or_finishes_an_objective() {
    let start = OrderLook { at: 0.0, ends: 10.0 * H, losing: false, done: vec![false, false] };
    let turned = OrderLook { at: 1.0 * H, losing: true, ..start.clone() };
    assert_eq!(order_alerts(&start, &turned), [OrderAlert::Losing]);

    let six = OrderLook { at: 4.5 * H, ..start.clone() };
    assert_eq!(order_alerts(&start, &six), [OrderAlert::EndsIn(6)]);
    let slept = OrderLook { at: 9.5 * H, ..start.clone() };
    assert_eq!(order_alerts(&start, &slept), [OrderAlert::EndsIn(1)]);
    assert!(order_alerts(&slept, &OrderLook { at: 11.0 * H, ..start.clone() }).is_empty());

    let done = OrderLook { done: vec![false, true], ..start.clone() };
    assert_eq!(order_alerts(&start, &done), [OrderAlert::ObjectiveDone(1)]);
}
