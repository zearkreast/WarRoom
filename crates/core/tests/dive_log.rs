use warroom_core::api::Faction;
use warroom_core::dive_log::{Dive, DiveOutcome, dive_outcome, group_dives};
use warroom_core::owner_changes::OwnerChange;

fn dive(planet: i64, at: f64, event: Option<i64>) -> Dive {
    Dive { planet, name: format!("P{planet}"), at, event, enemy: Faction::Automaton, major_order: false }
}

fn change(planet: i64, at: f64, from: Faction, to: Faction) -> OwnerChange {
    OwnerChange { planet, planet_name: format!("P{planet}"), from, to, at }
}

#[test]
fn a_liberation_is_won_ongoing_or_given_up() {
    let d = dive(1, 10.0, None);
    let won =
        [change(1, 5.0, Faction::Humans, Faction::Automaton), change(1, 20.0, Faction::Automaton, Faction::Humans)];
    assert_eq!(dive_outcome(&d, &won, None), DiveOutcome::Liberated(20.0));
    assert_eq!(dive_outcome(&d, &won[..1], Some(None)), DiveOutcome::Ongoing);
    assert_eq!(dive_outcome(&d, &won[..1], None), DiveOutcome::Withdrawn);
    assert_eq!(dive_outcome(&d, &[change(2, 20.0, Faction::Automaton, Faction::Humans)], None), DiveOutcome::Withdrawn);
}

#[test]
fn a_defense_is_held_ongoing_or_lost() {
    let d = dive(1, 10.0, Some(77));
    let lost = [change(1, 30.0, Faction::Humans, Faction::Terminids)];
    assert_eq!(dive_outcome(&d, &lost, None), DiveOutcome::Lost(30.0));
    assert_eq!(dive_outcome(&d, &[], Some(Some(77))), DiveOutcome::Ongoing);
    assert_eq!(dive_outcome(&d, &[], None), DiveOutcome::Held);
    assert_eq!(dive_outcome(&d, &[], Some(Some(78))), DiveOutcome::Held);
}

#[test]
fn dives_on_one_campaign_count_together() {
    let won = DiveOutcome::Liberated(50.0);
    let dives = [
        (dive(1, 10.0, None), won),
        (Dive { major_order: true, ..dive(1, 20.0, None) }, won),
        (dive(1, 60.0, None), DiveOutcome::Ongoing),
        (dive(2, 15.0, Some(9)), DiveOutcome::Held),
    ];
    let groups = group_dives(&dives);
    assert_eq!(groups.iter().map(|g| (g.planet, g.count)).collect::<Vec<_>>(), [(1, 1), (1, 2), (2, 1)]);
    assert_eq!((groups[1].first, groups[1].last, groups[1].major_order), (10.0, 20.0, true));
}
