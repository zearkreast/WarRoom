mod common;
use common::*;
use warroom_core::api::Faction;
use warroom_core::progress::{
    Sample, liberation_percent, projected_at, projected_completion, projected_loss, rate_per_hour,
};

fn sample(at: f64, percent: f64) -> Sample {
    Sample { planet: 1, at, owner: Faction::Automaton, percent, players: 100.0, defending: false }
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
