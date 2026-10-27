mod common;
use common::*;
use warroom_core::api::Faction;
use warroom_core::front_activity::{
    ActivityPoint, ActivitySeries, BUCKET_MS, FrontBucket, FrontRow, Reading, average_readings, front_buckets,
    latest_run, merge_activity, plot_activity,
};
use warroom_core::time::iso_string;

const T0: f64 = 1_000.0 * BUCKET_MS;

fn bucket(at: f64, n: f64) -> FrontBucket {
    FrontBucket { at, terminids: n, automatons: 0.0, illuminate: 0.0, players: n }
}

fn reading(at: f64, split: f64) -> Reading {
    Reading { at, impact: 0.01, players: 5000.0, terminids: split, automatons: 0.0, illuminate: 0.0 }
}

#[test]
fn uses_the_buckets_until_the_first_reading_then_each_readings_own_split() {
    let rows = merge_activity(
        &[bucket(T0, 100.0), bucket(T0 + BUCKET_MS, 200.0), bucket(T0 + 2.0 * BUCKET_MS, 999.0)],
        &[reading(T0 + BUCKET_MS / 2.0, 4000.0)],
        T0 + 2.0 * BUCKET_MS + 60_000.0,
    );
    let got: Vec<(f64, f64, Option<f64>)> = rows.iter().map(|r| (r.at - T0, r.terminids, r.online)).collect();
    assert_eq!(got, [(0.0, 100.0, None), (BUCKET_MS / 2.0, 4000.0, Some(5000.0))]);
}

#[test]
fn draws_from_the_readings_alone_once_they_cover_an_hour() {
    let rows = merge_activity(
        &[bucket(T0, 100.0), bucket(T0 + 8.0 * BUCKET_MS, 200.0)],
        &[reading(T0 + 4.0 * BUCKET_MS, 4000.0), reading(T0 + 8.0 * BUCKET_MS, 4100.0)],
        T0 + 10.0 * BUCKET_MS,
    );
    assert_eq!(rows.iter().map(|r| r.terminids).collect::<Vec<_>>(), [4000.0, 4100.0]);
}

#[test]
fn falls_back_to_the_buckets_for_a_reading_saved_without_a_split() {
    let rows = merge_activity(
        &[bucket(T0, 100.0), bucket(T0 + BUCKET_MS, 200.0)],
        &[reading(T0 + BUCKET_MS / 2.0, 0.0)],
        T0 + 3.0 * BUCKET_MS,
    );
    assert_eq!(rows.last().unwrap().terminids, 150.0);
}

fn front_rows() -> Vec<FrontRow> {
    let at = |q: f64| iso_string(ms("2026-10-23T12:00:00Z") + q * BUCKET_MS);
    let row = |q, owner, planet, players| FrontRow { bucket: at(q), owner, planet, players };
    vec![
        row(0.0, 3, None, 100.0),
        row(4.0, 3, None, 200.0),
        row(4.0, 1, Some(9), 50.0),
        row(5.0, 2, None, 300.0),
        row(5.0, 1, Some(8), 70.0),
    ]
}

#[test]
fn credits_defenses_to_their_attacker_and_drops_ones_it_cant_place() {
    let buckets = front_buckets(&front_rows(), |p| (p == 9).then_some(Faction::Illuminate));
    assert_eq!((buckets[1].automatons, buckets[1].illuminate, buckets[1].players), (200.0, 50.0, 250.0));
    assert_eq!((buckets[2].terminids, buckets[2].players), (300.0, 300.0));
}

#[test]
fn starts_the_chart_after_the_latest_gap_in_the_data() {
    let buckets = front_buckets(&front_rows(), |_| None);
    let run: Vec<f64> = latest_run(&buckets).iter().map(|b| b.at).collect();
    let at = |q: f64| ms("2026-10-23T12:00:00Z") + q * BUCKET_MS;
    assert_eq!(run, [at(4.0), at(5.0)]);
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

#[test]
fn averages_the_fronts_only_over_the_readings_that_split_them() {
    let w = 300_000.0;
    let r = |at, terminids| Reading { at, players: 100.0, impact: 0.01, terminids, automatons: 0.0, illuminate: 0.0 };
    let rows = average_readings(&[r(0.0, 30.0), r(60_000.0, 0.0), r(w, 0.0)], w);
    assert_eq!((rows[0].players, rows[0].terminids), (100.0, 30.0));
    assert_eq!((rows[1].players, rows[1].terminids), (100.0, 0.0));
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
