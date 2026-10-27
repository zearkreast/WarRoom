use warroom_core::campaign_filter::{FrontActivity, is_stalled};

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
