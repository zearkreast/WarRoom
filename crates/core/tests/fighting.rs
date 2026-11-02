use warroom_core::fighting::{CombatReading, last_day};

const H: f64 = 3_600_000.0;

fn at(hours: f64, won: f64, lost: f64, bugs: f64) -> CombatReading {
    CombatReading {
        at: hours * H,
        missions_won: won,
        missions_lost: lost,
        terminid_kills: bugs,
        ..CombatReading::default()
    }
}

#[test]
fn measures_the_last_day_from_the_oldest_reading_within_it() {
    let readings = [at(0.0, 100.0, 50.0, 1e6), at(10.0, 150.0, 60.0, 2e6), at(30.0, 400.0, 110.0, 5e6)];
    let t = last_day(&readings).unwrap();
    assert_eq!((t.hours, t.won, t.lost, t.kills[0]), (20.0, 250.0, 50.0, 3e6));
    assert_eq!(t.success_rate().map(|r| r.round()), Some(83.0));
}

#[test]
fn waits_for_half_a_day_and_drops_reset_counters() {
    assert_eq!(last_day(&[at(0.0, 1.0, 1.0, 1.0), at(6.0, 2.0, 2.0, 2.0)]), None);
    assert_eq!(last_day(&[at(0.0, 100.0, 1.0, 1.0), at(20.0, 5.0, 2.0, 2.0)]), None);
    assert_eq!(last_day(&[]), None);
}
