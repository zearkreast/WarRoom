use warroom_core::api::{TacticalAction, TacticalActionCost};
use warroom_core::station_actions::{ActionAlert, ActionChange, action_alerts, action_record, is_charged};

fn action(id: i64, status: i64, current: f64) -> TacticalAction {
    TacticalAction {
        id32: id,
        name: format!("A{id}"),
        status,
        costs: vec![TacticalActionCost {
            target_value: 100.0,
            current_value: current,
            ..TacticalActionCost::default()
        }],
        ..TacticalAction::default()
    }
}

#[test]
fn counts_when_each_action_went_active() {
    let c = |action, at, status| ActionChange { action, at, status };
    let log = [c(1, 10.0, 2), c(1, 20.0, 3), c(1, 30.0, 1), c(1, 40.0, 2), c(2, 50.0, 2)];
    let r = action_record(&log, 1);
    assert_eq!((r.last_active, r.times), (Some(40.0), 2));
    assert_eq!(action_record(&log, 3).last_active, None);
}

#[test]
fn alerts_when_an_action_charges_or_goes_active() {
    assert!(is_charged(&action(1, 1, 100.0)));
    assert!(!is_charged(&action(1, 1, 99.0)));
    assert!(!is_charged(&action(1, 2, 100.0)));
    let before = [action(1, 1, 99.0), action(2, 1, 100.0), action(3, 3, 0.0)];
    let after = [action(1, 1, 100.0), action(2, 2, 100.0), action(3, 3, 0.0), action(4, 2, 0.0)];
    assert_eq!(action_alerts(&before, &after), [ActionAlert::Charged("A1".into()), ActionAlert::Active("A2".into())]);
    assert!(action_alerts(&after, &after).is_empty());
}
