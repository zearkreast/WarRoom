use warroom_core::stratagems::{Arrow, CALL_IN, arrow_of, step};

#[test]
fn steps_on_with_the_next_arrow_and_falls_back_on_a_wrong_one() {
    let code = CALL_IN.code;
    assert_eq!(code.iter().fold(0, |at, a| step(at, *a, code)), code.len());
    assert_eq!(step(3, Arrow::Right, code), 0);
    assert_eq!(step(3, Arrow::Up, code), 1);
}

#[test]
fn keeps_the_presses_that_still_start_the_code() {
    assert_eq!(step(2, Arrow::Up, CALL_IN.code), 2);
}

#[test]
fn reads_wasd_and_the_arrow_keys() {
    let keys: Vec<_> = ["w", "A", "s", "d", "ArrowUp", "ArrowLeft"].iter().map(|k| arrow_of(k)).collect();
    use Arrow::*;
    assert_eq!(keys, [Some(Up), Some(Left), Some(Down), Some(Right), Some(Up), Some(Left)]);
    assert_eq!(arrow_of("q"), None);
}

#[test]
fn every_stratagem_has_a_unique_id_and_a_code() {
    use warroom_core::stratagems::STRATAGEMS;
    for (i, s) in STRATAGEMS.iter().enumerate() {
        assert!(!s.code.is_empty(), "{} has no code", s.name);
        assert!(STRATAGEMS[..i].iter().all(|t| t.id != s.id), "{} is listed twice", s.id);
    }
    assert!(STRATAGEMS.iter().any(|s| s.id == CALL_IN.id));
}

#[test]
fn drops_the_designation_from_the_name() {
    use warroom_core::stratagems::by_id;
    let railgun = by_id("railgun").unwrap();
    assert_eq!((railgun.short_name(), railgun.designation()), ("Railgun", Some("RS-422")));
    let eagle = by_id("eagle-500kg-bomb").unwrap();
    assert_eq!((eagle.short_name(), eagle.designation()), ("Eagle 500kg Bomb", None));
    assert_eq!(by_id("guard-dog").unwrap().short_name(), "Guard Dog");
}

#[test]
fn colours_each_type_by_its_permit() {
    use warroom_core::stratagems::{Kind, Permit, by_id};
    assert_eq!(by_id("orbital-laser").unwrap().permit(), Permit::Offensive);
    assert_eq!(by_id("jump-pack").unwrap().permit(), Permit::Supply);
    assert_eq!(by_id("tesla-tower").unwrap().permit(), Permit::Defensive);
    assert_eq!(by_id("reinforce").unwrap().kind, Kind::Ship);
    assert_eq!(by_id("reinforce").unwrap().permit(), Permit::Mission);
}
