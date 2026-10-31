use warroom_core::drill::*;
use warroom_core::stratagems::{Arrow, Permit, STRATAGEMS, Stratagem, by_id};

fn started(pool: Vec<&'static Stratagem>) -> Drill {
    let mut d = Drill::new(pool, 7);
    assert_eq!(d.tick(5.0), Tick::Go);
    d
}

fn key_current(d: &mut Drill) -> Press {
    let code = d.current().unwrap().code;
    code.iter().map(|a| d.press(*a)).last().unwrap()
}

fn wrong(a: Arrow) -> Arrow {
    if a == Arrow::Up { Arrow::Down } else { Arrow::Up }
}

#[test]
fn waits_for_the_countdown_before_taking_keys() {
    let mut d = Drill::new(classic_pool(), 1);
    assert_eq!(d.press(Arrow::Up), Press::Ignored);
    assert_eq!(d.tick(1.0), Tick::None);
    assert_eq!(d.tick(1.0), Tick::Go);
    assert_eq!(d.phase, Phase::Play);
}

#[test]
fn deals_more_stratagems_each_round_up_to_fifteen() {
    assert_eq!(Drill::new(classic_pool(), 3).queue.len(), 6);
    assert_eq!(stratagems_per_round(4), 9);
    assert_eq!(stratagems_per_round(40), 15);
}

#[test]
fn a_miss_restarts_the_code_and_breaks_the_streak() {
    let mut d = started(vec![by_id("reinforce").unwrap()]);
    let code = d.current().unwrap().code;
    assert_eq!(d.press(code[0]), Press::Hit);
    assert_eq!(d.press(wrong(code[1])), Press::Miss);
    assert_eq!((d.keyed, d.streak, d.misses), (0, 0, 1));
}

#[test]
fn scores_ten_points_an_arrow_times_the_streak_multiplier() {
    let mut d = started(vec![by_id("reinforce").unwrap()]);
    let len = d.current().unwrap().code.len() as u32;
    assert_eq!(key_current(&mut d), Press::Called(10 * len));
    assert_eq!(multiplier(4), 1);
    assert_eq!(multiplier(5), 2);
    assert_eq!(multiplier(99), 4);
}

#[test]
fn clearing_a_round_adds_its_bonuses_and_counts_down_to_the_next() {
    let mut d = started(vec![by_id("resupply").unwrap()]);
    let mut last = Press::Ignored;
    while d.phase == Phase::Play {
        last = key_current(&mut d);
    }
    let Press::Cleared(_, bonus) = last else { panic!("expected the round to clear, got {last:?}") };
    assert_eq!(bonus.clear, 75);
    assert_eq!(bonus.flawless, 100);
    assert!(bonus.time > 0 && bonus.time <= 100);
    d.tick(10.0);
    assert_eq!((d.round, d.queue.len()), (2, 7));
    assert!(matches!(d.phase, Phase::Ready(_)));
}

#[test]
fn ends_when_the_fuse_burns_out_and_burns_faster_in_later_rounds() {
    let mut d = started(classic_pool());
    assert_eq!(d.tick(FUSE - 0.5), Tick::None);
    assert_eq!(d.tick(1.0), Tick::Over);
    assert_eq!(d.press(Arrow::Up), Press::Ignored);
    assert!(burn_rate(5) > burn_rate(1));
    assert_eq!(burn_rate(100), 2.2);
}

#[test]
fn a_completed_code_puts_time_back_on_the_fuse() {
    let mut d = started(vec![by_id("reinforce").unwrap()]);
    d.tick(5.0);
    let before = d.fuse;
    key_current(&mut d);
    assert!(d.fuse > before);
}

#[test]
fn never_deals_the_same_stratagem_twice_running_from_a_larger_pool() {
    let pool = vec![by_id("reinforce").unwrap(), by_id("resupply").unwrap()];
    for seed in 0..50 {
        let d = Drill::new(pool.clone(), seed);
        assert!(d.queue.windows(2).all(|w| !std::ptr::eq(w[0], w[1])), "seed {seed}");
    }
}

#[test]
fn builds_pools_for_each_mode() {
    assert_eq!(classic_pool().len(), STRATAGEMS.len());
    let picked = custom_pool(&["railgun".into(), "gone".into(), "reinforce".into()]);
    assert_eq!(picked.iter().map(|s| s.id).collect::<Vec<_>>(), ["railgun", "reinforce"]);
    let red = permit_pool(&[Permit::Offensive]);
    assert!(!red.is_empty() && red.iter().all(|s| s.permit() == Permit::Offensive));
    assert_eq!(pool_label(Mode::Permits, &[], &[Permit::Offensive, Permit::Defensive]), "Red, Green");
    assert_eq!(pool_label(Mode::Custom, &["railgun".into()], &[]), "Railgun");
}

fn score(mode: Mode, score: u32, at: f64) -> DrillScore {
    DrillScore { mode, score, round: 1, called: 1, at, pool: String::new() }
}

#[test]
fn keeps_the_ten_best_scores_of_each_mode() {
    let mut all = Vec::new();
    for i in 0..12 {
        record(&mut all, score(Mode::Classic, 100 + i, i as f64));
    }
    assert_eq!(record(&mut all, score(Mode::Custom, 5, 99.0)), Some(1));
    assert_eq!(all.iter().filter(|s| s.mode == Mode::Classic).count(), MAX_SCORES);
    assert_eq!(record(&mut all, score(Mode::Classic, 50, 100.0)), None);
    assert_eq!(record(&mut all, score(Mode::Classic, 106, 101.0)), Some(7));
    assert_eq!(best(&all, Mode::Classic).map(|s| s.score), Some(111));
    assert_eq!(record(&mut all, score(Mode::Classic, 0, 102.0)), None);
}
