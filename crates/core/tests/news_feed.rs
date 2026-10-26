mod common;

use common::ms;
use warroom_core::news_feed::{body_echoes_title, day_label, group_by_day, matches_query, plain_text};

#[test]
fn spots_an_alert_whose_body_only_repeats_its_headline() {
    let alert = "<i=3>TERREK IS UNDER ATTACK BY THE TERMINIDS! ALL HELLDIVERS MUST DEPLOY IMMEDIATELY TO DEFEND IT!</i>\nTerrek is under attack by the Terminids! All Helldivers must deploy immediately to defend it!";
    assert!(body_echoes_title(alert));
    assert!(!body_echoes_title("<i=3>ARSENAL BOOST</i>\nHigh Command has authorized high-volume use."));
    assert!(body_echoes_title("<i=3>BRIEF</i>"));
}

#[test]
fn strips_the_games_markup() {
    assert_eq!(plain_text("Defend <i=1>Malevelon Creek</i> now"), "Defend Malevelon Creek now");
}

#[test]
fn names_days_on_super_earth_standard_time() {
    let now = ms("2026-10-25T18:00:00Z");
    assert_eq!(day_label(ms("2026-10-25T00:10:00Z"), now), "Today");
    assert_eq!(day_label(ms("2026-10-24T23:59:00Z"), now), "Yesterday");
    assert_eq!(day_label(ms("2026-10-22T12:00:00Z"), now), "22/10/2186");
}

#[test]
fn groups_items_by_day_and_keeps_their_order() {
    let now = ms("2026-10-25T18:00:00Z");
    let items =
        vec![(ms("2026-10-25T09:00:00Z"), "a"), (ms("2026-10-25T01:00:00Z"), "b"), (ms("2026-10-23T20:00:00Z"), "c")];
    let groups = group_by_day(items, |i| i.0, now);
    assert_eq!(groups.iter().map(|g| g.label.as_str()).collect::<Vec<_>>(), ["Today", "23/10/2186"]);
    assert_eq!(groups[0].items.iter().map(|i| i.1).collect::<Vec<_>>(), ["a", "b"]);
}

#[test]
fn matches_every_word_of_a_search_in_any_order() {
    assert!(matches_query("The Automaton Incineration Corps on Vindemitarix Prime", "corps vindemitarix"));
    assert!(!matches_query("Arsenal boost", "arsenal terminid"));
    assert!(matches_query("anything", "  "));
}
