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
