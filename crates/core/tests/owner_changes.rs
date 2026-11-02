use warroom_core::api::Faction;
use warroom_core::owner_changes::{LogQuery, OwnerChange, enemy_of, log_matches};

fn change(name: &str, from: Faction, to: Faction) -> OwnerChange {
    OwnerChange { planet: 1, planet_name: name.into(), from, to, at: 0.0 }
}

#[test]
fn names_the_enemy_on_either_side_of_a_change() {
    assert_eq!(enemy_of(&change("FURY", Faction::Automaton, Faction::Humans)), Faction::Automaton);
    assert_eq!(enemy_of(&change("ESKER", Faction::Humans, Faction::Terminids)), Faction::Terminids);
}

#[test]
fn filters_the_war_log_by_enemy_sector_and_words() {
    let fury = change("FURY", Faction::Automaton, Faction::Humans);
    let all = LogQuery::default();
    assert!(all.is_empty());
    assert!(log_matches(&fury, Some("Xzar"), &all));

    let bots = LogQuery { enemy: Some(Faction::Automaton), ..LogQuery::default() };
    assert!(log_matches(&fury, Some("Xzar"), &bots));
    let bugs = LogQuery { enemy: Some(Faction::Terminids), ..LogQuery::default() };
    assert!(!log_matches(&fury, Some("Xzar"), &bugs));

    let xzar = LogQuery { sector: Some("Xzar".into()), ..LogQuery::default() };
    assert!(log_matches(&fury, Some("Xzar"), &xzar));
    assert!(!log_matches(&fury, Some("Trigon"), &xzar));
    assert!(!log_matches(&fury, None, &xzar));

    let text = |t: &str| LogQuery { text: t.into(), ..LogQuery::default() };
    assert!(log_matches(&fury, Some("Xzar"), &text("fu")));
    assert!(log_matches(&fury, Some("Xzar"), &text("xzar fury")));
    assert!(!log_matches(&fury, Some("Xzar"), &text("esker")));
    assert!(text("  ").is_empty());
}
