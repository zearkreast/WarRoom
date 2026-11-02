use warroom_core::most_needed::{FrontNeed, NeedReason, front_need, per_thousand, typical_per_thousand};

const H: f64 = 3_600_000.0;

fn front(rate: Option<f64>, players: f64) -> FrontNeed {
    FrontNeed { percent: 40.0, rate, regen: 1.0, players, ..FrontNeed::default() }
}

#[test]
fn measures_each_front_against_the_middle_one() {
    assert_eq!(per_thousand(&front(Some(1.0), 2000.0)), Some(1.0));
    assert_eq!(per_thousand(&front(None, 2000.0)), None);
    assert_eq!(per_thousand(&front(Some(1.0), 0.0)), None);
    let fronts = [front(Some(1.0), 2000.0), front(Some(3.0), 1000.0), front(Some(0.0), 1000.0), front(None, 10.0)];
    assert_eq!(typical_per_thousand(&fronts), Some(1.0));
    assert_eq!(typical_per_thousand(&[front(None, 10.0)]), None);
}

#[test]
fn ranks_targets_losses_and_missed_deadlines_first() {
    let quiet = front_need(&front(Some(1.0), 2000.0), 0.0, Some(1.0));
    assert!(quiet.reasons.is_empty());

    let losing = front_need(&front(Some(-0.5), 2000.0), 0.0, Some(1.0));
    assert_eq!(losing.reasons, [NeedReason::LosingGround]);

    let target = FrontNeed { in_major_order: true, deadline: Some(10.0 * H), ..front(Some(1.0), 2000.0) };
    let need = front_need(&target, 0.0, Some(1.0));
    assert_eq!(need.reasons, [NeedReason::MajorOrder, NeedReason::MissesDeadline]);
    assert!(need.score > losing.score && losing.score > quiet.score);

    let on_time = FrontNeed { deadline: Some(100.0 * H), ..target };
    assert_eq!(front_need(&on_time, 0.0, Some(1.0)).reasons, [NeedReason::MajorOrder]);

    let thin = front_need(&front(Some(2.0), 1000.0), 0.0, Some(1.0));
    assert_eq!(thin.reasons, [NeedReason::EachDiverCounts]);
}

#[test]
fn a_front_held_at_zero_is_neither_losing_ground_nor_measured_per_helldiver() {
    let held = FrontNeed { percent: 0.0, ..front(Some(0.0), 100.0) };
    assert_eq!(per_thousand(&held), None);
    assert!(front_need(&held, 0.0, Some(1.0)).reasons.is_empty());
}

#[test]
fn a_defense_that_falls_outranks_a_liberation_losing_ground() {
    let falls = FrontNeed { regen: 0.0, deadline: Some(10.0 * H), ..front(Some(1.0), 2000.0) };
    let losing = front(Some(-0.5), 2000.0);
    assert!(front_need(&falls, 0.0, Some(1.0)).score > front_need(&losing, 0.0, Some(1.0)).score);
}
